//! Self-updates: check GitHub Releases for a newer Raygun, download it and apply it.
//!
//! Nothing here reinstalls from scratch: on Windows `update.apply` runs the new
//! MSI silently in place (Raygun UpgradeCode, settings and library untouched).
//! Downloads are checked against the release asset digest. Errors are values —
//! offline machines simply report that no update could be checked.
//!
//! Native only: the web build has no updater (the host serves a new bundle).

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use lightcraft_fetch::http::{Limits, Url, get};

/// The repository updates come from.
pub const REPO: &str = "kao-offline/raygun";
#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub bytes: u64,
    pub sha256: Option<String>,
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
                sha256: a.get("digest").and_then(serde_json::Value::as_str).and_then(|s| s.strip_prefix("sha256:")).map(str::to_string),
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
/// macOS, the AppImage on Linux. Unsupported platforms return None.
pub fn install_asset(rel: &Release) -> Option<&Asset> {
    install_asset_for(rel, std::env::consts::OS, std::env::consts::ARCH)
}

fn install_asset_for<'a>(rel: &'a Release, os: &str, arch: &str) -> Option<&'a Asset> {
    let (platform, architecture, extensions): (&str, &str, &[&str]) = match (os, arch) {
        ("windows", "x86_64") => ("windows", "x64", &[".msi"]),
        ("windows", "x86") => ("windows", "x86", &[".msi"]),
        ("windows", "aarch64") => ("windows", "arm64", &[".msi"]),
        ("macos", "x86_64" | "aarch64") => ("macos", "universal", &[".dmg"]),
        ("linux", "x86_64") => ("linux", "x86_64", &[".appimage", ".deb", ".tar.gz"]),
        ("linux", "aarch64") => ("linux", "aarch64", &[".appimage", ".deb", ".tar.gz"]),
        _ => return None,
    };
    let prefix = format!("raygun-{}-{platform}-{architecture}", rel.tag.strip_prefix('v').unwrap_or(&rel.tag));
    let pick = |want: &[&str]| {
        rel.assets.iter().find(|a| {
            let n = a.name.to_ascii_lowercase();
            want.iter().any(|extension| n == format!("{prefix}{extension}"))
        })
    };
    extensions.iter().find_map(|w| pick(&[w]))
}

/// Folder downloaded updates wait in (`<config>/updates`).
pub fn updates_dir() -> Option<PathBuf> {
    crate::config::config_dir().map(|d| d.join("updates"))
}

fn read_record(path: &std::path::Path) -> Result<serde_json::Value, String> {
    use std::io::Read;
    let mut text = String::new();
    std::fs::File::open(path).map_err(|e| e.to_string())?.take(4097).read_to_string(&mut text).map_err(|e| e.to_string())?;
    if text.len() > 4096 {
        return Err("update record is too large".into());
    }
    serde_json::from_str(&text).map_err(|e| format!("invalid update record: {e}"))
}

/// Download `asset` into the updates folder (resumes a partial file).
pub fn download(asset: &Asset, progress: &mut dyn FnMut(u64, u64)) -> Result<PathBuf, String> {
    use lightcraft_fetch::{FileSpec, Options, download_url};
    let dir = updates_dir().ok_or_else(|| "updates: no config folder on this machine".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("updates: cannot use {dir:?}: {e}"))?;
    let cancel = AtomicBool::new(false);
    let hash = asset.sha256.as_deref().ok_or_else(|| "updates: release asset has no SHA-256 digest".to_string())?;
    let spec = FileSpec { name: &asset.name, size: (asset.bytes > 0).then_some(asset.bytes), sha256: Some(hash), max: 2 * 1024 * 1024 * 1024 };
    let opts = Options::default();
    download_url(&spec, &asset.url, &dir, &opts, &cancel, &mut |p| progress(p.done, p.total)).map_err(|e| format!("download failed: {e}"))?;
    let receipt = serde_json::json!({"name": asset.name, "sha256": hash});
    std::fs::write(dir.join("downloaded.json"), receipt.to_string()).map_err(|e| format!("updates: could not record download: {e}"))?;
    Ok(dir.join(&asset.name))
}

/// Locate exactly the verified download, never an unrelated or stale MSI in the folder.
#[cfg(target_os = "windows")]
pub fn downloaded_installer() -> Result<PathBuf, String> {
    let dir = updates_dir().ok_or("no updates folder on this machine")?;
    let receipt = read_record(&dir.join("downloaded.json")).map_err(|e| format!("run update.download again: {e}"))?;
    installer_from_receipt(&dir, &receipt)
}

#[cfg(any(target_os = "windows", test))]
fn installer_from_receipt(dir: &std::path::Path, receipt: &serde_json::Value) -> Result<PathBuf, String> {
    let name = receipt.get("name").and_then(serde_json::Value::as_str).ok_or("missing installer name")?;
    if !valid_installer_name(name) {
        return Err("invalid installer name".into());
    }
    let hash = receipt.get("sha256").and_then(serde_json::Value::as_str).ok_or("missing installer checksum")?;
    let path = dir.join(name);
    if lightcraft_fetch::sha256_file(&path)? != hash {
        return Err("installer checksum mismatch: download the update again".into());
    }
    Ok(path)
}

#[cfg(any(target_os = "windows", test))]
fn valid_installer_name(name: &str) -> bool {
    name.starts_with("raygun-") && name.ends_with(".msi") && !name.contains(['/', '\\', ':']) && !name.contains("..")
}

/// Wait for the calling process to save and exit, then upgrade and restart at normal integrity.
#[cfg(target_os = "windows")]
pub fn schedule_install(library: Option<&std::path::Path>) -> Result<PathBuf, String> {
    use std::os::windows::process::CommandExt;
    let msi = downloaded_installer()?;
    let dir = updates_dir().ok_or("no updates folder")?;
    let script = dir.join("install.ps1");
    let quote = |p: &std::path::Path| format!("'{}'", p.to_string_lossy().replace('\'', "''"));
    let log = dir.join("install.log");
    let status = dir.join("install-status.txt");
    let restart = match library {
        Some(path) => format!("Start-Process -FilePath $updatedExe -ArgumentList ('--library \"' + {} + '\"')", quote(path)),
        None => "Start-Process -FilePath $updatedExe".to_string(),
    };
    let body = format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ Wait-Process -Id {pid} -Timeout 120 }}
    $msi = {msi}
    $log = {log}
    $arguments = '/i "' + $msi + '" /qn /norestart MSIRESTARTMANAGERCONTROL=Disable /L*v "' + $log + '"'
    $process = Start-Process -FilePath 'msiexec.exe' -Verb RunAs -ArgumentList $arguments -Wait -PassThru
    $code = $process.ExitCode
    Set-Content -LiteralPath {status} -Value $code
    if ($code -eq 0 -or $code -eq 3010) {{
        $folder = (Get-ItemProperty -LiteralPath 'HKLM:\Software\Raygun' -Name InstallFolder).InstallFolder
        $updatedExe = Join-Path $folder 'lightcraft.exe'
        {restart}
    }}
}} catch {{ Set-Content -LiteralPath {status} -Value $_.Exception.Message }}
"#,
        pid = std::process::id(),
        msi = quote(&msi),
        log = quote(&log),
        status = quote(&status)
    );
    std::fs::write(&script, body).map_err(|e| e.to_string())?;
    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&script)
        .creation_flags(0x0800_0000)
        .spawn()
        .map_err(|e| format!("could not schedule installer: {e}"))?;
    Ok(log)
}

/// Last-check stamp (`<config>/update-check.json`): at most one automatic
/// check per week; manual checks always run.
pub fn due_for_auto_check() -> bool {
    let stamp = crate::config::config_dir().map(|d| d.join("update-check.json"));
    let last = stamp.as_ref().and_then(|p| read_record(p).ok());
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
    fn oversized_update_records_are_rejected() {
        let path = std::env::temp_dir().join(format!("raygun-update-record-{}.json", std::process::id()));
        std::fs::write(&path, format!("{{\"padding\":\"{}\"}}", "x".repeat(5000))).unwrap();
        assert!(read_record(&path).unwrap_err().contains("too large"));
        std::fs::write(&path, "{\"at\":123}").unwrap();
        assert_eq!(read_record(&path).unwrap()["at"], 123);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_installer_paths() {
        assert!(valid_installer_name("raygun-0.2.1-windows-x64.msi"));
        for name in ["../raygun-test.msi", "C:\\raygun-test.msi", "raygun-test/evil.msi", "other.msi", "raygun-test.msi.exe"] {
            assert!(!valid_installer_name(name), "{name}");
        }
    }

    #[test]
    fn modified_download_is_never_applied() {
        let dir = std::env::temp_dir().join(format!("raygun-receipt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("raygun-test.msi");
        std::fs::write(&path, b"verified download").unwrap();
        let receipt = serde_json::json!({"name": "raygun-test.msi", "sha256": lightcraft_fetch::sha256_file(&path).unwrap()});
        assert_eq!(installer_from_receipt(&dir, &receipt).unwrap(), path);
        std::fs::write(&path, b"corrupted download").unwrap();
        assert!(installer_from_receipt(&dir, &receipt).unwrap_err().contains("checksum mismatch"));
        assert!(installer_from_receipt(&dir, &serde_json::json!({"name": "../raygun-test.msi", "sha256": "invalid"})).is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

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
                Asset { name: "raygun-0.2.0-windows-x64.msi".into(), url: "u1".into(), bytes: 1, sha256: None },
                Asset { name: "raygun-0.2.0-windows-x64-portable.zip".into(), url: "u2".into(), bytes: 1, sha256: None },
                Asset { name: "raygun-0.2.0-macos-universal.dmg".into(), url: "u3".into(), bytes: 1, sha256: None },
                Asset { name: "raygun-0.2.0-linux-x86_64.AppImage".into(), url: "u4".into(), bytes: 1, sha256: None },
            ],
        };
        let got = install_asset_for(&rel, "windows", "x86_64").map(|a| a.name.as_str()).unwrap_or_default();
        assert_eq!(got, "raygun-0.2.0-windows-x64.msi");
        assert!(install_asset_for(&rel, "windows", "aarch64").is_none());
        assert!(install_asset_for(&rel, "windows", "x86").is_none());
        assert_eq!(install_asset_for(&rel, "linux", "x86_64").map(|a| a.name.as_str()), Some("raygun-0.2.0-linux-x86_64.AppImage"));
        assert!(install_asset_for(&rel, "freebsd", "x86_64").is_none());
        let mut rel = rel;
        rel.assets.insert(0, Asset { name: "raygun-0.2.0-windows-arm64.msi".into(), url: "arm".into(), bytes: 1, sha256: None });
        assert_eq!(install_asset_for(&rel, "windows", "x86_64").map(|a| a.url.as_str()), Some("u1"));
        assert_eq!(install_asset_for(&rel, "windows", "aarch64").map(|a| a.url.as_str()), Some("arm"));
    }
}
