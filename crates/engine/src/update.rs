//! Self-updates: check GitHub Releases for a newer Raygun, download it and apply it.
//!
//! Nothing here reinstalls from scratch: on Windows `update.apply` runs the new
//! MSI silently in place (same UpgradeCode, settings and library untouched).
//! Portable builds download only and reveal the file. Errors are values —
//! offline machines simply report that no update could be checked.
//!
//! Native only: the web build has no updater (the host serves a new bundle).

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use lightcraft_fetch::http::{Limits, Url, get};

/// The repository updates come from.
pub const REPO: &str = "kao-offline/raygun";
/// What counts as "this app" when picking an asset (the installer family).
#[cfg(target_os = "windows")]
pub const ASSET_HINT: &str = "-windows-";
#[cfg(target_os = "macos")]
pub const ASSET_HINT: &str = "-macos-";
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub const ASSET_HINT: &str = "-linux-";

#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub bytes: u64,
}

#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub version: Vec<u64>,
    pub prerelease: bool,
    pub notes: String,
    pub assets: Vec<Asset>,
}

fn api_limits(cancel: &AtomicBool) -> Limits<'_> {
    Limits { connect: Duration::from_secs(10), stall: Duration::from_secs(10), cancel }
}

fn read_json(url: &str) -> Result<serde_json::Value, String> {
    let u = Url::parse(url).map_err(|e| e.to_string())?;
    let cancel = AtomicBool::new(false);
    let mut resp = get(&u, &[("Accept", "application/vnd.github+json".to_string())], &api_limits(&cancel)).map_err(|e| e.to_string())?;
    if resp.status != 200 {
        return Err(format!("update check: server answered {}", resp.status));
    }
    let mut body = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = resp.read(&mut buf, &api_limits(&cancel)).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(buf.get(..n).unwrap_or_default());
        if body.len() > 256 * 1024 {
            return Err("update check: release listing too large".into());
        }
    }
    serde_json::from_slice(&body).map_err(|e| format!("update check: bad release listing ({e})"))
}

/// `v1.2.3` / `1.2.3-rc.1` → ([1,2,3], prerelease). Garbage → None.
fn parse_version(tag: &str) -> Option<(Vec<u64>, bool)> {
    let t = tag.strip_prefix('v').unwrap_or(tag);
    let (core, pre) = match t.split_once('-') {
        Some((c, _)) => (c, true),
        None => (t, false),
    };
    let mut out = Vec::new();
    for part in core.split('.') {
        out.push(part.parse::<u64>().ok()?);
    }
    if out.is_empty() {
        return None;
    }
    Some((out, pre))
}

fn newer_than(latest: &[u64], current: &[u64]) -> bool {
    for (l, c) in latest.iter().chain(std::iter::repeat(&0)).zip(current.iter().chain(std::iter::repeat(&0))).take(4) {
        if l != c {
            return l > c;
        }
    }
    false
}

fn current_version() -> Vec<u64> {
    parse_version(env!("CARGO_PKG_VERSION")).map(|(v, _)| v).unwrap_or_default()
}

/// The newest published (non-draft) release, if the network answers.
pub fn latest() -> Result<Release, String> {
    let v = read_json(&format!("https://api.github.com/repos/{REPO}/releases/latest"))?;
    let tag = v.get("tag_name").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
    let (version, prerelease) = parse_version(&tag).ok_or_else(|| format!("update check: odd tag name {tag:?}"))?;
    let mut assets = Vec::new();
    if let Some(list) = v.get("assets").and_then(serde_json::Value::as_array) {
        for a in list.iter().take(64) {
            let (Some(name), Some(url)) =
                (a.get("name").and_then(serde_json::Value::as_str), a.get("browser_download_url").and_then(serde_json::Value::as_str))
            else {
                continue;
            };
            assets.push(Asset {
                name: name.to_string(),
                url: url.to_string(),
                bytes: a.get("size").and_then(serde_json::Value::as_u64).unwrap_or(0),
            });
        }
    }
    Ok(Release {
        notes: v.get("body").and_then(serde_json::Value::as_str).unwrap_or_default().chars().take(2000).collect(),
        tag,
        version,
        prerelease,
        assets,
    })
}

/// The newest release when it is newer than this build (prereleases are never
/// offered to a stable build).
pub fn available() -> Result<Option<Release>, String> {
    let rel = latest()?;
    if rel.prerelease {
        return Ok(None);
    }
    Ok(newer_than(&rel.version, &current_version()).then_some(rel))
}

/// The asset this OS installs from: the MSI on Windows, the disk image on
/// macOS, the AppImage on Linux. Portable zips are the fallback.
pub fn install_asset(rel: &Release) -> Option<&Asset> {
    let pick = |want: &[&str]| {
        rel.assets.iter().find(|a| {
            let n = a.name.to_ascii_lowercase();
            n.contains(ASSET_HINT) && want.iter().any(|w| n.contains(w)) && !n.ends_with(".zsync")
        })
    };
    #[cfg(target_os = "windows")]
    let order: &[&[&str]] = &[&[".msi"], &["portable.zip"]];
    #[cfg(target_os = "macos")]
    let order: &[&[&str]] = &[&[".dmg"]];
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let order: &[&[&str]] = &[&[".appimage"], &[".deb"], &[".tar.gz"]];
    order.iter().find_map(|&w| pick(w))
}

/// Folder downloaded updates wait in (`<config>/updates`).
pub fn updates_dir() -> Option<PathBuf> {
    crate::config::config_dir().map(|d| d.join("updates"))
}

/// Download `asset` into the updates folder (resumes a partial file).
pub fn download(asset: &Asset, progress: &mut dyn FnMut(u64, u64)) -> Result<PathBuf, String> {
    use lightcraft_fetch::{FileSpec, Options, download_url};
    let dir = updates_dir().ok_or_else(|| "updates: no config folder on this machine".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("updates: cannot use {dir:?}: {e}"))?;
    let cancel = AtomicBool::new(false);
    let spec = FileSpec { name: &asset.name, size: (asset.bytes > 0).then_some(asset.bytes), sha256: None, max: 2 * 1024 * 1024 * 1024 };
    let opts = Options::default();
    download_url(&spec, &asset.url, &dir, &opts, &cancel, &mut |p| progress(p.done, p.total)).map_err(|e| format!("download failed: {e}"))?;
    Ok(dir.join(&asset.name))
}

/// Last-check stamp (`<config>/update-check.json`): at most one automatic
/// check per week; manual checks always run.
pub fn due_for_auto_check() -> bool {
    let stamp = crate::config::config_dir().map(|d| d.join("update-check.json"));
    let last = stamp.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
    let week = 7 * 24 * 3600u64;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    match last.and_then(|v| v.get("at").and_then(serde_json::Value::as_u64)) {
        Some(at) => now.saturating_sub(at) >= week,
        None => true,
    }
}

/// Remember that an automatic check ran now (success or failure: no retry storm).
pub fn stamp_auto_check() {
    if let Some(p) = crate::config::config_dir().map(|d| d.join("update-check.json")) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = std::fs::write(p, format!("{{\"at\":{now}}}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert!(newer_than(&[0, 2, 0], &[0, 1, 0]));
        assert!(!newer_than(&[0, 1, 0], &[0, 1, 0]));
        assert!(!newer_than(&[0, 1, 0], &[0, 2, 0]));
        assert!(newer_than(&[0, 1, 10], &[0, 1, 9]));
        assert!(newer_than(&[0, 2], &[0, 1, 9]));
    }

    #[test]
    fn tags_parse_leniently() {
        assert_eq!(parse_version("v0.2.0"), Some((vec![0, 2, 0], false)));
        assert_eq!(parse_version("0.2.0-rc.1"), Some((vec![0, 2, 0], true)));
        assert_eq!(parse_version("banana"), None);
        assert_eq!(parse_version("v"), None);
    }

    #[test]
    fn picks_the_installer_per_platform() {
        let rel = Release {
            tag: "v0.2.0".into(),
            version: vec![0, 2, 0],
            prerelease: false,
            notes: String::new(),
            assets: vec![
                Asset { name: "raygun-0.2.0-windows-x64.msi".into(), url: "u1".into(), bytes: 1 },
                Asset { name: "raygun-0.2.0-windows-x64-portable.zip".into(), url: "u2".into(), bytes: 1 },
                Asset { name: "raygun-0.2.0-macos-universal.dmg".into(), url: "u3".into(), bytes: 1 },
                Asset { name: "raygun-0.2.0-linux-x86_64.AppImage".into(), url: "u4".into(), bytes: 1 },
            ],
        };
        let got = install_asset(&rel).map(|a| a.name.as_str()).unwrap_or_default();
        #[cfg(target_os = "windows")]
        assert_eq!(got, "raygun-0.2.0-windows-x64.msi");
        #[cfg(target_os = "macos")]
        assert_eq!(got, "raygun-0.2.0-macos-universal.dmg");
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        assert_eq!(got, "raygun-0.2.0-linux-x86_64.AppImage");
    }
}
