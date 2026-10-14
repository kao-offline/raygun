//! Sensor-pixel regressions against an independent translation of the pinned Rawler decoder.
//! CC0 inputs from raw.pixls.us are kept in gitignored corpus/raw, never shipped.
use lightcraft_raw::{RawData, RawFormat, decode, probe_info};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn check(name: &str, input: &str, samples_hash: &str, size: (usize, usize)) {
    let root = std::env::var_os("LIGHTCRAFT_CORPUS").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus"));
    let path = root.join("raw").join(name);
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("skip: {} absent", path.display());
        return;
    };
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), input);
    let raw = decode(&bytes).unwrap();
    assert_eq!(raw.format, RawFormat::Orf);
    raw.validate().unwrap();
    assert_eq!((raw.width, raw.height), size);
    assert_eq!(raw.info(), probe_info(&bytes).unwrap());
    let RawData::U16(samples) = &raw.data else { panic!("expected integer sensor samples") };
    let mut hash = Sha256::new();
    for sample in samples {
        hash.update(sample.to_le_bytes());
    }
    assert_eq!(format!("{:x}", hash.finalize()), samples_hash);
    assert!(raw.wb_multipliers.is_some());
    assert!(raw.white_at(0) > raw.black.mean());
}

#[test]
fn olympus_em1_compressed_sensor_is_exact() {
    // https://raw.pixls.us/getfile.php/1051
    check(
        "orf-olympus-em1.orf",
        "19ba17e778ae802d4039f0e88bb1cd68825780788b8058c479067e32b9a63eda",
        "f66855309d3a6b19598952649c382dfbaaf92eea1581142d8a693f1988b17433",
        (4640, 3472),
    );
}

#[test]
fn olympus_em10iii_compressed_sensor_is_exact() {
    // https://raw.pixls.us/getfile.php/1787
    check(
        "orf-olympus-em10iii.orf",
        "11d90cbb564ad2c660c7a8d5d81d99c2c018aa27fdec45ed6c461fa5d24f942b",
        "431e3081ea9bc354f93e809c413b6a0232ae8ffd13073fbb3235a787680f150a",
        (4640, 3472),
    );
}
