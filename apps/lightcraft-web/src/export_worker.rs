//! One disposable export worker per batch. Cancel terminates it without blocking the UI.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::Ordering;

use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
use lightcraft_engine::Session;
use lightcraft_ui_egui::export_task::{StartWorker, WorkerChannel, WorkerHandle};
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::backend::Backend;
use crate::export_wire::ExportWire;
use crate::store::{Originals, hash_of_path, storage_key};

fn get(o: &JsValue, k: &str) -> JsValue {
    Reflect::get(o, &k.into()).unwrap_or(JsValue::UNDEFINED)
}

fn message(key: &str, value: &JsValue) -> Object {
    let out = Object::new();
    let _ = Reflect::set(&out, &key.into(), value);
    out
}

struct State {
    channel: WorkerChannel,
    ctx: egui::Context,
    files: Vec<Value>,
    finished: bool,
    ready: bool,
}

impl State {
    fn finish(&mut self, result: Result<Vec<Value>, String>) {
        if !self.finished {
            self.finished = true;
            let _ = self.channel.result.send(result);
            self.ctx.request_repaint();
        }
    }
}

struct Handle {
    worker: web_sys::Worker,
    state: Rc<RefCell<State>>,
    started: f64,
    _message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: Closure<dyn FnMut(JsValue)>,
}

impl WorkerHandle for Handle {
    fn poll(&mut self) {
        let mut state = self.state.borrow_mut();
        if state.channel.cancel.load(Ordering::Relaxed) {
            self.worker.terminate();
            let files = state.files.clone();
            state.finish(Ok(files));
        } else if !state.ready && crate::workers::now() - self.started > 30_000.0 {
            self.worker.terminate();
            state.finish(Err("Export worker did not start. Reload the page and try again.".into()));
        }
        if !state.finished {
            state.ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.worker.terminate();
        self.worker.set_onmessage(None);
        self.worker.set_onerror(None);
    }
}

pub fn starter(originals: Originals, backend: Option<Backend>, ctx: egui::Context) -> StartWorker {
    Box::new(move |session, ids, options, to, channel| {
        let request = serde_json::to_string(&ExportWire::new(session, ids, options, to)?).map_err(|e| e.to_string())?;
        let module = get(&js_sys::global(), "lightcraftModule");
        if module.is_undefined() {
            return Err("Export worker unavailable: reload the page.".into());
        }
        let opts = web_sys::WorkerOptions::new();
        opts.set_type(web_sys::WorkerType::Module);
        opts.set_name("raygun-export");
        let worker = web_sys::Worker::new_with_options("./export-worker.js", &opts).map_err(|e| format!("Export worker: {e:?}"))?;
        let state = Rc::new(RefCell::new(State { channel, ctx: ctx.clone(), files: Vec::new(), finished: false, ready: false }));
        let (s, w, originals) = (state.clone(), worker.clone(), originals.clone());
        let onmessage = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            let mut s = s.borrow_mut();
            if s.finished {
                return;
            }
            if get(&data, "ready").is_truthy() {
                s.ready = true;
                if let Err(e) = w.post_message(&message("request", &request.clone().into())) {
                    s.finish(Err(format!("Export worker: {e:?}")));
                }
            } else if let Some(path) = get(&data, "needOriginal").as_string() {
                let out = match originals.get(&path) {
                    Some(bytes) => {
                        let bytes = Uint8Array::from(bytes.as_ref());
                        let out = message("original", &bytes);
                        if let Err(e) = w.post_message_with_transfer(&out, &Array::of1(&bytes.buffer())) {
                            s.finish(Err(format!("Sending original: {e:?}")));
                        }
                        None
                    }
                    None => Some(message("originalError", &format!("{path}: original is not loaded; reopen the photo and try again").into())),
                };
                if let Some(out) = out {
                    if let Err(e) = w.post_message(&out) {
                        s.finish(Err(format!("Sending original error: {e:?}")));
                    }
                }
            } else if let Some(path) = get(&data, "path").as_string() {
                let result = get(&data, "bytes")
                    .dyn_into::<Uint8Array>()
                    .map_err(|_| "Invalid export bytes".to_string())
                    .and_then(|bytes| crate::web::download_array(&path, &bytes));
                match result {
                    Ok(()) if !get(&data, "sidecar").is_truthy() => s.files.push(serde_json::json!({"path": path})),
                    Ok(()) => {}
                    Err(e) => {
                        w.terminate();
                        s.finish(Err(e));
                    }
                }
            } else if let Some(error) = get(&data, "error").as_string() {
                s.finish(Err(error));
            } else if let Some(result) = get(&data, "result").as_string() {
                s.finish(serde_json::from_str(&result).map_err(|e| e.to_string()));
            } else if let Some(current) = get(&data, "current").as_string() {
                let done = get(&data, "done").as_f64().unwrap_or(0.0) as usize;
                if let Ok(mut progress) = s.channel.progress.lock() {
                    *progress = (done, current);
                }
            }
            s.ctx.request_repaint();
        });
        let s = state.clone();
        let onerror = Closure::<dyn FnMut(JsValue)>::new(move |error| {
            let detail = get(&error, "message").as_string().unwrap_or_else(|| format!("{error:?}"));
            s.borrow_mut().finish(Err(format!("Export worker failed: {detail}")));
        });
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        worker.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        let init = message("module", &module);
        let _ = Reflect::set(&init, &"moduleUrl".into(), &get(&js_sys::global(), "lightcraftModuleUrl"));
        let _ = Reflect::set(&init, &"store".into(), &backend.as_ref().map_or("memory", Backend::kind).into());
        if let Err(e) = worker.post_message(&init) {
            worker.terminate();
            worker.set_onmessage(None);
            worker.set_onerror(None);
            return Err(format!("Export worker: {e:?}"));
        }
        Ok(Box::new(Handle { worker, state, started: crate::workers::now(), _message: onmessage, _error: onerror }))
    })
}

thread_local! {
    static ORIGINAL_WAITER: RefCell<Option<Function>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn export_worker_main(store: String) {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    crate::safety::install_worker_panic_hook();
    let scope: web_sys::DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    wasm_bindgen_futures::spawn_local(async move {
        let backend = match store.as_str() {
            "memory" => None,
            _ => Backend::open(store == "idb").await.ok(),
        };
        let s = scope.clone();
        let handler = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            if get(&data, "original").is_undefined() && get(&data, "originalError").is_undefined() {
                if let Some(request) = get(&data, "request").as_string() {
                    let (s, b) = (s.clone(), backend.clone());
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = run(&s, b, request).await;
                        let out = match result {
                            Ok(v) => message("result", &serde_json::to_string(&v).unwrap_or_else(|_| "[]".into()).into()),
                            Err(e) => message("error", &e.into()),
                        };
                        let _ = s.post_message(&out);
                    });
                }
            } else {
                resolve_original(data);
            }
        });
        scope.set_onmessage(Some(handler.as_ref().unchecked_ref()));
        handler.forget();
        let _ = scope.post_message(&message("ready", &true.into()));
    });
}

async fn run(scope: &web_sys::DedicatedWorkerGlobalScope, backend: Option<Backend>, request: String) -> Result<Vec<Value>, String> {
    let wire: ExportWire = serde_json::from_str(&request).map_err(|e| e.to_string())?;
    let originals = Originals::default();
    let mut session = Session::new();
    session.catalog = wire.catalog.clone();
    originals.install(&mut session);
    let items = wire.prepare(&mut session)?;
    let original = wire.options.format == lightcraft_engine::export::ExportFormat::Original;
    let sidecar = Cell::new(false);
    let mut write = |path: &str, bytes: &[u8]| {
        let bytes = Uint8Array::from(bytes);
        let out = message("path", &path.into());
        let _ = Reflect::set(&out, &"bytes".into(), &bytes);
        let _ = Reflect::set(&out, &"sidecar".into(), &sidecar.get().into());
        scope.post_message_with_transfer(&out, &Array::of1(&bytes.buffer())).map_err(|e| format!("Export result: {e:?}"))?;
        sidecar.set(original && !sidecar.get());
        Ok(())
    };
    let mut progress = |done: usize, current: &str| {
        sidecar.set(false);
        let out = message("current", &current.into());
        let _ = Reflect::set(&out, &"done".into(), &(done as u32).into());
        scope.post_message(&out).is_ok()
    };
    let mut prepare = |item: &lightcraft_engine::export::PreparedExport| {
        let path = item.source_path().map(str::to_string);
        let (scope, backend, originals) = (scope.clone(), backend.clone(), originals.clone());
        async move {
            let Some(path) = path else { return Ok(()) };
            if originals.contains(&path) {
                return Ok(());
            }
            let hash = hash_of_path(&path).ok_or_else(|| format!("{path}: not a browser original"))?;
            let bytes = match &backend {
                Some(b) => match b.read(&storage_key(hash)).await? {
                    Some(bytes) => bytes,
                    None => read_from_page(&scope, &path).await?,
                },
                None => read_from_page(&scope, &path).await?,
            };
            originals.insert(hash, bytes.into());
            Ok(())
        }
    };
    lightcraft_engine::export::run_batch_async(items, &wire.options, &wire.destination(), &mut write, &|_| false, &mut progress, &mut prepare).await
}

pub(crate) fn resolve_original(data: JsValue) {
    ORIGINAL_WAITER.with(|waiter| {
        if let Some(resolve) = waiter.borrow_mut().take() {
            let _ = resolve.call1(&JsValue::NULL, &data);
        }
    });
}

pub(crate) async fn read_from_page(scope: &web_sys::DedicatedWorkerGlobalScope, path: &str) -> Result<Vec<u8>, String> {
    let promise = Promise::new(&mut |resolve, _| ORIGINAL_WAITER.with(|w| *w.borrow_mut() = Some(resolve)));
    scope.post_message(&message("needOriginal", &path.into())).map_err(|e| format!("Reading original: {e:?}"))?;
    let result = JsFuture::from(promise).await.map_err(|e| format!("Reading original: {e:?}"))?;
    if let Some(error) = get(&result, "originalError").as_string() {
        return Err(error);
    }
    get(&result, "original").dyn_into::<Uint8Array>().map(|bytes| bytes.to_vec()).map_err(|_| "Invalid original bytes".into())
}
