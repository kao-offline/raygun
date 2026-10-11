//! Self-update commands: `update.check`, `update.download`, `update.apply`.
//!
//! Native only (the web build is served new, not updated). `update.check` hits
//! the GitHub Releases API; offline machines get an error, not a hang.

use serde_json::{Value, json};

use super::{CommandSpec, always, bad, cmd};

fn rel_json(rel: &crate::update::Release) -> Value {
    json!({
        "tag": rel.tag,
        "prerelease": rel.prerelease,
        "notes": rel.notes,
        "assets": rel.assets.iter().map(|a| json!({"name": a.name, "bytes": a.bytes})).collect::<Vec<_>>(),
    })
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "update.check",
            "Check for Updates",
            [],
            None,
            "{} — newest published release vs this build → {current, latest?, available, url?, notes?}",
            always,
            |_, _| {
                let current = env!("CARGO_PKG_VERSION").to_string();
                match crate::update::available() {
                    Err(e) => Err(bad("update.check", e)),
                    Ok(None) => Ok(json!({"current": current, "available": false})),
                    Ok(Some(rel)) => {
                        let mut out = rel_json(&rel);
                        out["current"] = json!(current);
                        out["latest"] = json!(rel.tag);
                        out["available"] = json!(true);
                        out["url"] = json!(format!("https://github.com/{}/releases/tag/{}", crate::update::REPO, rel.tag));
                        Ok(out)
                    }
                }
            }
        ),
        cmd!(
            query "update.download",
            "Download Update",
            [],
            None,
            "{} — check, then download this OS's installer into the updates folder → {tag, path, bytes}",
            always,
            |_, _| {
                const ID: &str = "update.download";
                let rel = crate::update::available().map_err(|e| bad(ID, e))?.ok_or_else(|| bad(ID, "no newer release published"))?;
                let asset = crate::update::install_asset(&rel).ok_or_else(|| bad(ID, "no installer for this OS in that release"))?;
                let path = crate::update::download(asset, &mut |_, _| {}).map_err(|e| bad(ID, e))?;
                Ok(json!({"tag": rel.tag, "path": path.to_string_lossy(), "bytes": asset.bytes}))
            }
        ),
        cmd!(
            query "update.apply",
            "Install Downloaded Update",
            [],
            None,
            "{} — run the downloaded installer in place (Windows MSI, silent; keeps settings and library) → {applied}",
            always,
            |_, _| {
                const ID: &str = "update.apply";
                #[cfg(target_os = "windows")]
                {
                    let dir = crate::update::updates_dir().ok_or_else(|| bad(ID, "no updates folder on this machine"))?;
                    let msi = std::fs::read_dir(&dir)
                        .map_err(|e| bad(ID, e.to_string()))?
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("msi")))
                        .max_by_key(|p| p.metadata().map(|m| m.modified().ok()).ok().flatten());
                    let Some(msi) = msi else { return Err(bad(ID, "no downloaded installer: run update.download first")) };
                    std::process::Command::new("msiexec")
                        .args(["/i", &msi.to_string_lossy(), "/passive"])
                        .spawn()
                        .map_err(|e| bad(ID, format!("could not start the installer: {e}")))?;
                    Ok(json!({"applied": true}))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err(bad(ID, "automatic install is Windows-only so far: download the disk image or package from the release and open it"));
                }
            }
        ),
    ]
}
