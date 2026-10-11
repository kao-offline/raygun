//! Windows only: embed the app icon and version info (VERSIONINFO) into the
//! executable, so Explorer, the taskbar, the Start menu and Alt-Tab show the raygun.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `RAYGUN_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/raygun.ico");
    println!("cargo:rerun-if-env-changed=RAYGUN_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=LIGHTCRAFT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/raygun.ico")
        .set("ProductName", "Raygun")
        .set("FileDescription", "Raygun photo library and raw developer")
        .set("LegalCopyright", "Copyright (c) the Raygun authors. AGPL-3.0-or-later.")
        .set("OriginalFilename", "lightcraft.exe")
        .set("InternalName", "lightcraft");
    if let Err(e) = res.compile() {
        if std::env::var_os("RAYGUN_REQUIRE_WINRES").is_some() || std::env::var_os("LIGHTCRAFT_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=executable built without icon/version resources: {e}");
    }
}
