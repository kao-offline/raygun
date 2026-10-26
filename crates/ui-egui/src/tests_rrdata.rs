//! Real headless control path for the desktop/web RapidRAW import dialog.
use crate::{LightcraftApp, Services, headless::Headless, state::Dialog};
use serde_json::json;
use std::time::Duration;

#[test]
fn rapidraw_dialog_preserves_input_on_error_and_captured_selection_on_import() {
    let mut h = Headless::new(LightcraftApp::new(lightcraft_engine::Session::with_demo(), Services::default()), [1400.0, 900.0], 1.0);
    let t = Duration::from_secs(30);
    let ids: Vec<_> = h.app.session.catalog.photos().take(2).map(|p| p.id).collect();
    assert_eq!(ids.len(), 2);
    h.app.session.execute("library.select", &json!({"ids": [ids[0].0], "active": ids[0].0})).unwrap();
    assert_eq!(h.request("ui.menu.invoke", json!({"id": "file.importRrdata"}), t)["ok"], true);
    if let Some(Dialog::ImportRrdata { text, .. }) = &mut h.app.ui.dialog {
        *text = "{".into();
    } else {
        panic!("missing dialog");
    }
    assert_eq!(h.request("ui.dialog.confirm", json!({}), t)["ok"], false);
    assert!(matches!(h.app.ui.dialog, Some(Dialog::ImportRrdata { .. })));
    assert_eq!(h.app.session.develop_of(ids[0]).unwrap().light.exposure, 0.0);
    h.app.session.execute("library.select", &json!({"ids": [ids[1].0], "active": ids[1].0})).unwrap();
    if let Some(Dialog::ImportRrdata { text, .. }) = &mut h.app.ui.dialog {
        *text =
            json!({"adjustments": {"exposure": 1.25, "crop": {"unit": "%", "x": 10, "y": 20, "width": 50, "height": 40}, "masks": []}}).to_string();
    }
    h.step();
    assert!(h.app.widgets.iter().any(|(id, _)| id == "field:rrdata"));
    assert_eq!(h.request("ui.dialog.confirm", json!({}), t)["ok"], true);
    assert!(h.app.ui.dialog.is_none());
    assert_eq!(h.app.session.develop_of(ids[0]).unwrap().light.exposure, 1.25);
    assert_eq!(h.app.session.develop_of(ids[0]).unwrap().crop.geometry.rect.x0, 0.1);
    assert_eq!(h.app.session.develop_of(ids[1]).unwrap().light.exposure, 0.0);
    assert!(h.app.ui.toast.as_ref().unwrap().0.contains("masks"));
    h.app.session.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(h.app.session.develop_of(ids[0]).unwrap().light.exposure, 0.0);
    assert_eq!(h.request("ui.menu.invoke", json!({"id": "file.importRrdata"}), t)["ok"], true);
    assert_eq!(h.request("ui.dialog.cancel", json!({}), t)["ok"], true);
    assert_eq!(h.app.session.develop_of(ids[1]).unwrap().light.exposure, 0.0);
}
