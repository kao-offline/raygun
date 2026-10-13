"""Exercise the real packager's static-file allowlist, with synthetic build outputs."""

import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SCRATCH = (ROOT / "target" / "web-package-tests").resolve()
if not SCRATCH.is_relative_to((ROOT / "target").resolve()):
    raise RuntimeError("Packaging test scratch directory escaped target")
SCRATCH.mkdir(parents=True, exist_ok=True)
BASH = "C:/Program Files/Git/bin/bash.exe" if os.name == "nt" else "bash"


with tempfile.TemporaryDirectory(dir=SCRATCH) as temporary:
    work = Path(temporary)
    site = work / "web"
    site.mkdir()
    required = ["index.html", "worker.js", "lightcraft_web.js", "lightcraft_web_bg.wasm", "raygun.svg"]
    for name in required:
        (site / name).write_bytes(b"synthetic static output")
        (site / (name + ".gz")).write_bytes(b"synthetic compressed output")
    (site / "deps").mkdir()
    (site / "deps" / "host-build-script.exe").write_bytes(b"must never ship")
    (site / ".cargo-lock").write_text("must never ship")
    environment = dict(os.environ, CARGO_TARGET_DIR=work.as_posix(), DIST=(work / "dist").as_posix(), RAYGUN_VERSION="0.0.0-test")
    environment.pop("CRAFT_FONTS_DIR", None)
    environment.pop("CRAFT_FONTS_REQUIRED", None)
    command = "bash packaging/web/package.sh --skip-build"
    # Git for Windows has no Info-ZIP. Use Python's real ZIP writer for this copy
    # regression; Linux CI exercises the same script with the production zip tool.
    if os.name == "nt" and not shutil.which("zip"):
        python = shlex.quote(sys.executable.replace("\\", "/"))
        command = f'zip() {{ {python} -m zipfile -c "$2" "$3"; }}; export -f zip; ' + command
    result = subprocess.run([BASH, "-c", command], cwd=ROOT, env=environment, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    archive = work / "dist" / "raygun-web-0.0.0-test.zip"
    with zipfile.ZipFile(archive) as bundle:
        files = {Path(name).name for name in bundle.namelist() if not name.endswith("/")}
        expected = set(required + [name + ".gz" for name in required])
        expected.update(["_headers", ".htaccess", "HOSTING.md", "README.md", "LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "OFL-Inter.txt"])
        assert expected <= files, expected - files
        assert "host-build-script.exe" not in files and ".cargo-lock" not in files
        assert all("/deps/" not in name and "/build/" not in name for name in bundle.namelist())
    # An optional font must never ship without its licence.
    (site / "lightcraft_zh_hans.otf").write_bytes(b"synthetic font output")
    result = subprocess.run([BASH, "-c", command], cwd=ROOT, env=environment, capture_output=True, text=True)
    assert result.returncode != 0 and "Chinese font licence missing" in result.stderr
    (site / "OFL-noto-sans-cjk-sc.txt").write_text("synthetic licence output")
    result = subprocess.run([BASH, "-c", command], cwd=ROOT, env=environment, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    with zipfile.ZipFile(archive) as bundle:
        names = {Path(name).name for name in bundle.namelist()}
        assert {"lightcraft_zh_hans.otf", "OFL-noto-sans-cjk-sc.txt"} <= names

print("ok: web package includes static files and notices, excludes Cargo output, rejects unlicensed font")
