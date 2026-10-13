//! Update checks and downloads run off the UI thread; installation waits for a saved shutdown.

use crate::LightcraftApp;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub(crate) struct State {
        pub busy: bool,
        pub message: String,
        pub release: Option<lightcraft_engine::update::Release>,
        pub downloaded: bool,
    }

    pub(crate) type Shared = Arc<Mutex<State>>;

    pub(super) fn start(ctx: &egui::Context, shared: Shared, download: bool) -> Result<(), String> {
        {
            let mut s = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if s.busy {
                return Ok(());
            }
            s.busy = true;
            s.message = if download { "Downloading update…" } else { "Checking for updates…" }.into();
        }
        let repaint = ctx.clone();
        let worker = shared.clone();
        let result = std::thread::Builder::new().name("raygun-updater".into()).spawn(move || {
            let result = (|| {
                let release = lightcraft_engine::update::available()?;
                if download {
                    let rel = release.as_ref().ok_or("No newer release is available")?;
                    let asset = lightcraft_engine::update::install_asset(rel).ok_or("No installer is available for this platform")?;
                    lightcraft_engine::update::download(asset, &mut |done, total| {
                        worker.lock().unwrap_or_else(std::sync::PoisonError::into_inner).message =
                            format!("Downloading: {} / {} MB", done / 1_000_000, total / 1_000_000);
                        repaint.request_repaint();
                    })?;
                }
                Ok::<_, String>(release)
            })();
            let mut s = worker.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            s.busy = false;
            match result {
                Ok(release) => {
                    s.downloaded = download;
                    s.message = match &release {
                        Some(rel) if download => format!("Raygun {} is ready to install. Your library and edits will be kept.", rel.tag),
                        Some(rel) => format!("Raygun {} is available.", rel.tag),
                        None => "Raygun is up to date.".into(),
                    };
                    s.release = release;
                }
                Err(error) => s.message = error,
            }
            repaint.request_repaint();
        });
        if let Err(error) = result {
            let mut s = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            s.busy = false;
            s.message = format!("Could not start update task: {error}");
            return Err(s.message.clone());
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Shared = native::Shared;

/// A weekly check offers its fresh result here, without interrupting another dialog or task.
#[cfg(not(target_arch = "wasm32"))]
pub fn offer(app: &mut LightcraftApp, release: lightcraft_engine::update::Release) {
    let mut state = app.updates.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.busy {
        return;
    }
    state.message = format!("Raygun {} is available.", release.tag);
    state.release = Some(release);
    state.downloaded = false;
    if app.ui.dialog.is_none() && app.notices.is_empty() && app.quit_prompt.is_none() {
        app.ui.dialog = Some(crate::state::Dialog::Updates);
    }
}

pub fn check(app: &mut LightcraftApp, ctx: &egui::Context) -> Result<serde_json::Value, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::start(ctx, app.updates.clone(), false)?;
        app.ui.dialog = Some(crate::state::Dialog::Updates);
        Ok(serde_json::json!({"checking": true}))
    }
    #[cfg(target_arch = "wasm32")]
    {
        app.toast(ctx, "Web updates arrive with the site — nothing to install");
        Ok(serde_json::json!({"available": false}))
    }
}

pub fn show(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let shared = app.updates.clone();
        let s = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        ui.label(&s.message);
        if s.busy {
            ui.spinner();
        } else if let Some(rel) = &s.release {
            #[cfg(target_os = "windows")]
            if s.downloaded {
                let button = ui.button("Restart and Install Update");
                crate::widgets::register(ui.ctx(), "button:updateInstall", button.rect);
                if button.clicked() && crate::panels::notices::may_close(app) {
                    match app.session.execute("update.apply", &serde_json::json!({})) {
                        Ok(_) => app.ui.quit = true,
                        Err(error) => {
                            let mut state = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                            state.message = error.to_string();
                            state.downloaded = false;
                        }
                    }
                }
            }
            if !s.downloaded && lightcraft_engine::update::install_asset(rel).is_some() {
                let button = ui.button("Download Update");
                crate::widgets::register(ui.ctx(), "button:updateDownload", button.rect);
                if button.clicked() {
                    let _ = app.run("app.downloadUpdate", serde_json::json!({}));
                }
            }
            if ui.button("Open Release Page").clicked() {
                let _ = crate::links::open(app, &format!("{}/releases/tag/{}", crate::links::GITHUB, rel.tag));
            }
            if s.downloaded
                && ui.button("Show Download").clicked()
                && let Some(asset) = lightcraft_engine::update::install_asset(rel)
                && let Some(dir) = lightcraft_engine::update::updates_dir()
                && let Some(reveal) = app.services.reveal.as_mut()
            {
                let _ = reveal(&dir.join(&asset.name).to_string_lossy());
            }
            ui.add(egui::Label::new(&rel.notes).wrap());
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = app;
        ui.label("Web updates arrive with the site — nothing to install");
    }
}

pub fn download(app: &mut LightcraftApp, ctx: &egui::Context) -> Result<serde_json::Value, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::start(ctx, app.updates.clone(), true)?;
        app.ui.dialog = Some(crate::state::Dialog::Updates);
        Ok(serde_json::json!({"downloading": true}))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (app, ctx);
        Err("Web updates arrive with the site — nothing to install".into())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn download_action_is_discoverable_and_dispatches_without_duplicate_workers() {
        assert!(crate::menus::ui_commands().any(|c| c.0 == "app.downloadUpdate"));
        let mut app = LightcraftApp::new(lightcraft_engine::Session::new(), crate::Services::default());
        app.updates.lock().unwrap().busy = true;
        let result = app.run("app.downloadUpdate", serde_json::json!({})).unwrap();
        assert_eq!(result["downloading"], true);
        assert_eq!(app.ui.dialog, Some(crate::state::Dialog::Updates));
        assert!(app.updates.lock().unwrap().busy);
    }

    #[test]
    fn weekly_offer_uses_the_update_dialog_and_keeps_other_dialogs() {
        let mut app = LightcraftApp::new(lightcraft_engine::Session::new(), crate::Services::default());
        let release = lightcraft_engine::update::Release {
            tag: "v0.2.2".into(),
            version: vec![0, 2, 2],
            prerelease: false,
            notes: String::new(),
            assets: vec![],
        };
        offer(&mut app, release.clone());
        assert_eq!(app.ui.dialog, Some(crate::state::Dialog::Updates));
        assert!(app.notices.is_empty());
        app.ui.dialog = Some(crate::state::Dialog::About);
        offer(&mut app, release);
        assert_eq!(app.ui.dialog, Some(crate::state::Dialog::About));
    }

    #[test]
    fn offered_update_has_a_download_button_and_close_keeps_the_library() {
        let app = LightcraftApp::new(lightcraft_engine::Session::new(), crate::Services::default());
        let mut h = crate::headless::Headless::new(app, [1200.0, 800.0], 1.0);
        let shared = h.app.updates.clone();
        {
            let mut s = shared.lock().unwrap();
            s.message = "Raygun v0.2.1 is available".into();
            s.release = Some(lightcraft_engine::update::Release {
                tag: "v0.2.1".into(),
                version: vec![0, 2, 1],
                prerelease: false,
                notes: String::new(),
                assets: vec![lightcraft_engine::update::Asset {
                    name: match std::env::consts::OS {
                        "windows" => "raygun-0.2.1-windows-x64.msi",
                        "macos" => "raygun-0.2.1-macos-universal.dmg",
                        _ => "raygun-0.2.1-linux-x86_64.AppImage",
                    }
                    .into(),
                    url: String::new(),
                    bytes: 1,
                    sha256: None,
                }],
            });
        }
        h.app.ui.dialog = Some(crate::state::Dialog::Updates);
        h.step();
        h.step();
        if matches!(std::env::consts::OS, "windows" | "linux") && std::env::consts::ARCH == "x86_64" || std::env::consts::OS == "macos" {
            assert!(h.app.widgets.iter().any(|(id, _)| id == "button:updateDownload"));
        }
        h.app.ui.dialog = None;
        h.step();
        assert_eq!(h.app.session.catalog.photos().count(), 0);
        assert!(!h.quit_requested());
    }

    #[test]
    fn starting_a_second_task_does_not_duplicate_busy_work() {
        let ctx = egui::Context::default();
        let shared = native::Shared::default();
        shared.lock().unwrap().busy = true;
        assert!(native::start(&ctx, shared.clone(), false).is_ok());
        assert!(shared.lock().unwrap().busy);
    }
}
