#!/usr/bin/env python3
"""Validate a relocated archive and an installed copy without changing user config."""
import hashlib
import importlib.util
import json
import os
import platform
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib

root = Path(__file__).resolve().parents[1]
version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
archive = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / f"dist/kanata-studio-rust-{version}-linux-{platform.machine()}.tar.gz"
checksum = Path(str(archive)+".sha256").read_text().split()[0]
assert hashlib.sha256(archive.read_bytes()).hexdigest() == checksum
env = dict(os.environ, QT_QPA_PLATFORM="offscreen", QT_QPA_PLATFORMTHEME="")
spec = importlib.util.spec_from_file_location("release_installer", root / "install.py")
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)
with tempfile.TemporaryDirectory(prefix="kanata-rust-package-check-") as folder:
    relocated = Path(folder)
    installer.extract_bundle(archive, relocated, archive.name.removesuffix(".tar.gz"))
    bundle = next(relocated.iterdir())
    assert (bundle / "plugins/wayland-shell-integration/libxdg-shell.so").is_file(), "Wayland xdg-shell plugin missing"
    metadata = json.loads((bundle / "manifest.json").read_text())
    assert metadata.get('license') == 'MIT'
    assert metadata.get('redistributable') is True, 'Development bundles cannot pass release checks'
    for name in ('LICENSE', 'THIRD_PARTY_NOTICES.md', 'LICENSING.md',
                 'licenses/RUST-DEPENDENCIES.txt', 'licenses/LGPL-3.0.txt',
                 'licenses/GPL-3.0.txt', 'licenses/rust-toolchain/COPYRIGHT.html'):
        assert (bundle / name).is_file(), f'Missing license material: {name}'
    inventory = json.loads((bundle / 'licenses/NATIVE-PACKAGES.json').read_text())
    native_files = {str(path.relative_to(bundle)) for directory in ('lib', 'qml', 'plugins')
                    for path in (bundle / directory).rglob('*') if path.is_file()}
    assert native_files == set(inventory['files']), 'Native file inventory is incomplete'
    assert inventory['sourceFiles'], 'Missing corresponding native sources'
    for filename, digest in inventory['sourceFiles'].items():
        assert hashlib.sha256((bundle / filename).read_bytes()).hexdigest() == digest, filename
    for package in inventory['packages'].values():
        assert (bundle / package['notice']).is_file(), package
        assert list((bundle / 'sources/debian' / package['source']).glob('*.dsc')), package
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
