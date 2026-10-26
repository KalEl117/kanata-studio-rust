#!/usr/bin/env python3
"""Validate a relocated archive and an installed copy without changing user config."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

root = Path(__file__).resolve().parents[1]
archive = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / "dist/kanata-studio-rust-0.2.0-linux-x86_64.tar.gz"
checksum = Path(str(archive)+".sha256").read_text().split()[0]
assert hashlib.sha256(archive.read_bytes()).hexdigest() == checksum
env = dict(os.environ, QT_QPA_PLATFORM="offscreen", QT_QPA_PLATFORMTHEME="")
with tempfile.TemporaryDirectory(prefix="kanata-rust-package-check-") as folder:
    relocated = Path(folder)
    with tarfile.open(archive) as tar:
        tar.extractall(relocated, filter="data")
    bundle = next(relocated.iterdir())
    assert (bundle / "plugins/wayland-shell-integration/libxdg-shell.so").is_file(), "Wayland xdg-shell plugin missing"
    metadata = json.loads((bundle / "manifest.json").read_text())
    print(f"Paket: Qt {metadata['qt']}, glibc >= {metadata['minimumGlibc']}, {len(metadata['bundledLibraries'])} Bibliotheken", flush=True)
    subprocess.run([str(bundle / "launch.sh"), "--self-test"], env=env, check=True, timeout=30)
    screenshot = relocated / "ui.png"
    subprocess.run([str(bundle / "launch.sh"), "--screenshot", str(screenshot), str(bundle / "share/test.kbd")], env=env, check=True, timeout=30)
    assert screenshot.is_file() and screenshot.stat().st_size > 0, "QML interface did not render"
    subprocess.run([sys.executable, str(root / "tools/check-recovery.py"), str(bundle / "launch.sh")], env=env, check=True, timeout=30)
    prefix = relocated / "installation with spaces"
    subprocess.run([sys.executable, str(bundle / "install.py"), "--prefix", str(prefix)], env=env, check=True, timeout=30)
    subprocess.run([str(prefix / "bin/kanata-studio-rust"), "--self-test"], env=env, check=True, timeout=30)
    subprocess.run([sys.executable, str(root / "tools/check-recovery.py"), str(prefix / "bin/kanata-studio-rust")], env=env, check=True, timeout=30)
    print("OK: Prüfsumme, entpacktes Paket, installierte Kopie und Wiederherstellung", flush=True)
