//! Immutable export snapshot shared with a dedicated browser worker.
use lightcraft_engine::Session;
use lightcraft_engine::catalog::{Catalog, PhotoId};
use lightcraft_engine::export::{Destination, ExportOptions, PreparedExport};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExportWire {
    pub catalog: Catalog,
    pub ids: Vec<PhotoId>,
    pub options: ExportOptions,
    pub dir: String,
    pub exact: Option<String>,
}

impl ExportWire {
    pub fn new(session: &Session, ids: &[PhotoId], options: ExportOptions, to: Destination) -> Result<Self, String> {
        if ids.is_empty() || ids.iter().any(|id| session.catalog.photo(*id).is_none()) {
            return Err("no photo selected or a selected photo was removed".into());
        }
        Ok(Self { catalog: session.catalog.export_snapshot(ids), ids: ids.to_vec(), options, dir: to.dir, exact: to.exact })
    }

    pub fn prepare(&self, session: &mut Session) -> Result<Vec<PreparedExport>, String> {
        lightcraft_engine::export::prepare_batch(session, &self.ids, &self.options)
    }

    pub fn destination(&self) -> Destination {
        Destination { dir: self.dir.clone(), exact: self.exact.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Originals;
    use serde_json::json;

    #[test]
    fn snapshot_exports_identical_pixels_names_and_metadata_without_other_photos() {
        let originals = Originals::default();
        let mut session = Session::new();
        originals.install(&mut session);
        let mut paths = Vec::new();
        for (width, name) in [(16, "one.png"), (20, "two.png")] {
            let bytes = crate::store::png_bytes(width, 12);
            paths.push(originals.added(name, name, bytes.into()));
        }
        session.execute("library.import", &json!({"paths": paths})).unwrap();
        let id = session.catalog.photos().find(|p| p.file_name == "one.png").unwrap().id;
        session.execute("library.select", &json!({"ids": [id]})).unwrap();
        session.execute("develop.set", &json!({"control": "light.exposure", "value": 1.0})).unwrap();
        let options = ExportOptions::default();
        let expected = lightcraft_engine::export::prepare_export(&mut session, id, &options, 1).unwrap().run().unwrap();
        let wire = ExportWire::new(&session, &[id], options, Destination::default()).unwrap();
        let wire: ExportWire = serde_json::from_slice(&serde_json::to_vec(&wire).unwrap()).unwrap();
        assert_eq!(wire.catalog.photos().count(), 1);
        let mut worker = Session::new();
        originals.install(&mut worker);
        worker.catalog = wire.catalog.clone();
        let actual = wire.prepare(&mut worker).unwrap().pop().unwrap().run().unwrap();
        assert_eq!(actual.file_name, expected.file_name);
        assert_eq!(actual.bytes, expected.bytes);
        assert_eq!((actual.width, actual.height), (16, 12));
        assert_eq!(session.catalog.photos().count(), 2);
    }
}
