// Raygun export worker: reuse the compiled module; rendering stays off the UI thread.
self.onmessage = async (e) => {
  self.onmessage = null;
  try {
    const { default: init, export_worker_main } = await import(e.data.moduleUrl || "./lightcraft_web.js");
    await init({ module_or_path: e.data.module });
    export_worker_main(e.data.store);
  } catch (err) {
    setTimeout(() => { throw err; });
  }
};
