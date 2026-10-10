#!/usr/bin/env python3
"""Bundle the executable, Qt plugins/QML and their ELF dependency closure."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
VERSION = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
HOST_RUNTIME = {"libc.so.6", "libm.so.6", "libpthread.so.0", "libdl.so.2", "librt.so.1", "libresolv.so.2", "libutil.so.1", "ld-linux-x86-64.so.2", "ld-linux-aarch64.so.1"}

def output(args):
    return subprocess.check_output(args, text=True)

def dependencies(path):
    report = subprocess.run(["ldd", str(path)], text=True, capture_output=True, check=True).stdout
    if "not found" in report:
        raise RuntimeError(f"Fehlende Bibliothek für {path}:\n{report}")
    return [Path(match) for match in re.findall(r"=> (/\S+)", report)]

def build_bundle(bundle):
    (bundle / "bin").mkdir(parents=True)
    (bundle / "lib").mkdir()
    shutil.copy2(ROOT / "qt/kanata-studio-rust", bundle / "bin/kanata-studio-rust")
    shutil.copy2(ROOT / "target/release/kanata-studio-helper", bundle / "bin/kanata-studio-helper")
    qml = Path(output(["qmake6", "-query", "QT_INSTALL_QML"]).strip())
    plugins = Path(output(["qmake6", "-query", "QT_INSTALL_PLUGINS"]).strip())
    for name in ("QtQuick", "QtQml", "QtCore", "Qt/labs/settings", "Qt/labs/platform"):
        if (qml / name).exists():
            shutil.copytree(qml / name, bundle / "qml" / name, dirs_exist_ok=True)
    for name in ("platforms", "wayland-decoration-client", "wayland-graphics-integration-client", "wayland-shell-integration", "xcbglintegrations", "imageformats", "iconengines", "platformthemes", "styles", "networkinformation", "tls"):
        if (plugins / name).exists():
            shutil.copytree(plugins / name, bundle / "plugins" / name,
                            ignore=lambda directory,names:[n for n in names if n.endswith('.so')
                                and Path(directory).name in ("imageformats", "iconengines", "platformthemes", "styles")
                                and not n.startswith('libq')])
    queue = [path for path in bundle.rglob("*") if path.is_file() and path.open("rb").read(4) == b"\x7fELF"]
    scanned = set()
    versions = set()
    for path in queue:
        resolved = path.resolve()
        if resolved in scanned: continue
        scanned.add(resolved)
        versions.update(re.findall(r"GLIBC_([0-9]+\.[0-9]+)", output(["readelf", "--version-info", str(path)])))
        for dependency in dependencies(path):
            if dependency.name in HOST_RUNTIME: continue
            destination = bundle / "lib" / dependency.name
            if not destination.exists():
                shutil.copy2(dependency, destination)
                queue.append(destination)
    for licenses in Path("/usr/share/licenses").glob("qt6-*"):
        shutil.copytree(licenses, bundle / "licenses" / licenses.name)
    # Include distribution notices for other copied shared libraries when available.
    packages = set()
    for path in (bundle / "lib").iterdir() if shutil.which("pacman") else ():
        found = subprocess.run(["pacman", "-Qqo", f"/usr/lib/{path.name}"], text=True, capture_output=True)
        if found.returncode == 0: packages.update(found.stdout.splitlines())
    for package in packages:
        licenses = Path("/usr/share/licenses") / package
        if licenses.is_dir() and not (bundle / "licenses" / package).exists():
            shutil.copytree(licenses, bundle / "licenses" / package)
    (bundle / "bin/qt.conf").write_text("[Paths]\nPrefix=..\nPlugins=plugins\nQmlImports=qml\nLibraries=lib\n")
    (bundle / "launch.sh").write_text('''#!/bin/sh
set -eu
# Resolve installation symlinks before locating the runtime.
launcher=$(readlink -f -- "$0")
bundle_dir=$(CDPATH= cd -- "$(dirname -- "$launcher")" && pwd)
if [ "${KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH+x}" != x ]; then
  export KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH="${LD_LIBRARY_PATH-}"
fi
export LD_LIBRARY_PATH="$bundle_dir/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export QT_PLUGIN_PATH="$bundle_dir/plugins"
export QML_IMPORT_PATH="$bundle_dir/qml"
export QML2_IMPORT_PATH="$bundle_dir/qml"
export KANATA_STUDIO_LAUNCHER="$bundle_dir/launch.sh"
if [ "$#" -eq 1 ] && [ "$1" = --self-test ]; then
  exec "$bundle_dir/bin/kanata-studio-rust" --self-test "$bundle_dir/share/test.kbd"
fi
exec "$bundle_dir/bin/kanata-studio-rust" "$@"
''')
    (bundle / "launch.sh").chmod(0o755)
    shutil.copy2(ROOT / "tools/install.py", bundle / "install.py")
    (bundle / "install.py").chmod(0o755)
    (bundle / "share").mkdir()
    shutil.copy2(ROOT / "tests/fixtures/wrapped-layout.kbd", bundle / "share/test.kbd")
    minimum = max(versions, key=lambda version:tuple(map(int,version.split('.'))))
    metadata = {"name":"Kanata Studio Rust", "version":VERSION, "architecture":platform.machine(),
        "minimumGlibc":minimum, "qt":output(["qmake6", "-query", "QT_VERSION"]).strip(),
        "bundledLibraries":sorted(path.name for path in (bundle / "lib").iterdir())}
    (bundle / "manifest.json").write_text(json.dumps(metadata, indent=2)+"\n")
    (bundle / "README.txt").write_text(f'''Kanata Studio Rust {VERSION}

Installation: ./install.py
Direkt starten: ./launch.sh
Qt-Selbsttest: QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./launch.sh --self-test

Dieses Paket enthält Qt {metadata['qt']}, QML und die benötigten Bibliotheken.
Rust, Cargo, Node und ein Compiler sind auf dem Zielgerät nicht nötig.
Zielsystem: Linux {metadata['architecture']} mit glibc {minimum} oder neuer,
passenden Grafiktreibern und einer Wayland- oder X11-Sitzung.
Kanata und kanata.service müssen auf dem Zielgerät vorhanden sein, um
Konfigurationen zu prüfen, zu speichern und vorübergehend zu testen.
Python 3 wird nur für das Installationsskript benötigt.

Die grafische Oberfläche und der Qt-Adapter bleiben Qt/C++; Parser,
Bearbeitung, Verlauf, Auswahl, Layout-/Geräte-/Theme-Erkennung und der
Test-/Wiederherstellungsprozess verwenden Rust.

Lizenztexte der Laufzeitbibliotheken liegen unter licenses/.
Qt-Bibliotheken werden dynamisch geladen und können unter lib/ ersetzt werden.
Qt-Quellpakete zu dieser Version: https://download.qt.io/archive/qt/
Weitere Distributionsquellen: https://gitlab.archlinux.org/archlinux/packaging/packages
''')
    return metadata

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--install", action="store_true")
    args = parser.parse_args()
    name = f"kanata-studio-rust-{VERSION}-linux-{platform.machine()}"
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="kanata-rust-package-") as temporary:
        bundle = Path(temporary) / name
        metadata = build_bundle(bundle)
        env = dict(os.environ, QT_QPA_PLATFORM="offscreen", QT_QPA_PLATFORMTHEME="")
        check = subprocess.run([str(bundle / "launch.sh"), "--self-test"], env=env,
                               text=True, capture_output=True, timeout=30)
        if check.returncode:
            raise RuntimeError(f"Paket-Selbsttest fehlgeschlagen:\n{check.stdout}\n{check.stderr}")
        archive = dist / f"{name}.tar.gz"
        with tarfile.open(archive,"w:gz") as tar: tar.add(bundle, arcname=name)
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        (dist / f"{archive.name}.sha256").write_text(f"{digest}  {archive.name}\n")
        print(f"Paket: {archive}\nMindestversion glibc: {metadata['minimumGlibc']}\nPaket-Selbsttest: OK", flush=True)
        if args.install:
            subprocess.run(["python3", str(bundle / "install.py")], check=True)
