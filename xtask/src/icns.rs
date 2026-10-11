//! `cargo xtask icns <out.icns> <in.png>…`: pack square PNG images into a macOS `.icns`.
//!
//! Each PNG is stored as-is in an icon element (`ic07` = 128 px … `ic10` = 1024 px):
//! macOS has read PNG-compressed elements since 10.7, so no `iconutil` is needed and
//! icons regenerate on any OS (see packaging/icons.sh).

use std::path::Path;

/// PNG-compressed icon elements, smallest first (sizes must be square).
const SLOTS: &[(u32, &[u8; 4])] = &[(128, b"ic07"), (256, b"ic08"), (512, b"ic09"), (1024, b"ic10")];

/// An ICNS file holding `pngs` (each square, one of 128/256/512/1024 px).
pub fn pack(pngs: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    if pngs.is_empty() {
        return Err("need at least one PNG".into());
    }
    let mut elements = Vec::new();
    for png in pngs {
        let (w, h) = crate::ico::png_size(png)?;
        if w != h {
            return Err(format!("icon images must be square, got {w}x{h}"));
        }
        let (_, tag) = SLOTS.iter().find(|(s, _)| *s == w).ok_or_else(|| format!("icon PNG must be 128, 256, 512 or 1024 px, got {w}"))?;
        elements.push((tag, png));
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&[0, 0, 0, 0]); // total length, filled in below
    for (tag, png) in &elements {
        out.extend_from_slice(&tag[..]);
        let len = u32::try_from(png.len() + 8).map_err(|_| "icon PNG too large".to_string())?;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(png);
    }
    let total = u32::try_from(out.len()).map_err(|_| "icon file too large".to_string())?;
    out[4..8].copy_from_slice(&total.to_be_bytes());
    Ok(out)
}

pub fn run(args: &[&str]) -> Result<(), String> {
    let [out, inputs @ ..] = args else {
        return Err("usage: cargo xtask icns <out.icns> <in.png>…".into());
    };
    let pngs = inputs.iter().map(|p| std::fs::read(p).map_err(|e| format!("read {p}: {e}"))).collect::<Result<Vec<_>, _>>()?;
    let icns = pack(&pngs)?;
    std::fs::write(Path::new(out), &icns).map_err(|e| format!("write {out}: {e}"))?;
    println!("{out}: {} images, {} bytes", pngs.len(), icns.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_png(size: u32) -> Vec<u8> {
        let mut v = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        v.extend_from_slice(&size.to_be_bytes());
        v.extend_from_slice(&size.to_be_bytes());
        v.extend_from_slice(&[8, 6, 0, 0, 0, 1, 2, 3, 4]);
        v
    }

    #[test]
    fn packs_magic_and_elements() {
        let a = fake_png(128);
        let b = fake_png(1024);
        let icns = pack(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(u32::from_be_bytes(icns[4..8].try_into().unwrap()), icns.len() as u32);
        assert_eq!(&icns[8..12], b"ic07");
        assert_eq!(u32::from_be_bytes(icns[12..16].try_into().unwrap()), a.len() as u32 + 8);
        assert_eq!(&icns[16..16 + a.len()], &a[..]);
        let second = 16 + a.len();
        assert_eq!(&icns[second..second + 4], b"ic10");
        assert_eq!(&icns[second + 8..], &b[..]);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(pack(&[]).is_err());
        assert!(pack(&[fake_png(48)]).is_err());
        assert!(pack(&[fake_png(16)]).is_err());
        assert!(pack(&[b"GIF89a".to_vec()]).is_err());
    }
}
