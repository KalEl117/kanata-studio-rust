#!/usr/bin/env python3
"""Bundle the Windows executable with windeployqt, the MSVC runtime, notices and Qt sources."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import urllib.request
import zipfile
from license_materials import copy_project_notices

ROOT = Path(__file__).resolve().parents[1]
VERSION = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
# Deployed Qt DLLs and the Qt submodule whose source covers them (plugins and QML included).
QT_MODULES = {
    "Qt6Core": "qtbase", "Qt6Gui": "qtbase", "Qt6Network": "qtbase", "Qt6OpenGL": "qtbase",
    "Qt6Concurrent": "qtbase", "Qt6Widgets": "qtbase",
    "Qt6Qml": "qtdeclarative", "Qt6QmlMeta": "qtdeclarative", "Qt6QmlModels": "qtdeclarative",
    "Qt6QmlWorkerScript": "qtdeclarative", "Qt6QmlCore": "qtdeclarative", "Qt6QmlLocalStorage": "qtdeclarative",
    "Qt6Quick": "qtdeclarative", "Qt6QuickControls2": "qtdeclarative", "Qt6QuickTemplates2": "qtdeclarative",
    "Qt6QuickLayouts": "qtdeclarative", "Qt6QuickShapes": "qtdeclarative", "Qt6QuickDialogs2": "qtdeclarative",
    "Qt6QuickDialogs2QuickImpl": "qtdeclarative", "Qt6QuickDialogs2Utils": "qtdeclarative",
    "Qt6QuickEffects": "qtdeclarative", "Qt6LabsSettings": "qtdeclarative", "Qt6LabsFolderListModel": "qtdeclarative",
    "Qt6LabsQmlModels": "qtdeclarative", "Qt6LabsPlatform": "qtdeclarative",
    "Qt6Svg": "qtsvg", "Qt6ShaderTools": "qtshadertools",
}


def output(args):
    return subprocess.check_output(args, text=True).strip()


def qt_tool(*names):
    for name in names:
        if found := shutil.which(name):
            return found
    raise RuntimeError(f"{names[0]} nicht gefunden; Qt-bin muss im PATH liegen")


def main_executable():
    for candidate in (ROOT / "qt/kanata-studio-rust.exe", ROOT / "qt/release/kanata-studio-rust.exe"):
        if candidate.is_file():
            return candidate
    raise RuntimeError("qt/kanata-studio-rust.exe fehlt; zuerst bauen (siehe README)")


def copy_msvc_runtime(bundle):
    """App-local MSVC runtime, so no Visual C++ Redistributable has to be installed."""
    redist = os.environ.get("VCToolsRedistDir")
    if not redist:
        raise RuntimeError("VCToolsRedistDir fehlt; in einer MSVC-Entwicklerumgebung ausführen")
    crt = next(Path(redist, "x64").glob("Microsoft.VC*.CRT"), None)
    if crt is None:
        raise RuntimeError("MSVC-Laufzeit nicht gefunden unter " + redist)
    copied = []
    for dll in sorted(crt.glob("*.dll")):
        shutil.copy2(dll, bundle / dll.name)
        copied.append(dll.name)
    return copied


def qt_sources(bundle, qt_version):
    """Exact upstream source archives of every deployed Qt module, verified against Qt's MD5 list."""
    modules = set()
    for dll in bundle.glob("Qt6*.dll"):
        module = QT_MODULES.get(dll.stem)
        if module is None:
            raise RuntimeError("Unreviewed Qt module: " + dll.name)
        modules.add(module)
    series = ".".join(qt_version.split(".")[:2])
    base = f"https://download.qt.io/archive/qt/{series}/{qt_version}/submodules/"
    with urllib.request.urlopen(base + "md5sums.txt", timeout=60) as response:
        sums = dict(reversed(line.split()) for line in response.read().decode().splitlines() if line.strip())
    target = bundle / "sources/qt"
    target.mkdir(parents=True)
    inventory = {}
    for module in sorted(modules):
        name = f"{module}-everywhere-src-{qt_version}.zip"
        path = target / name
        with urllib.request.urlopen(base + name, timeout=600) as response, path.open("wb") as file:
            shutil.copyfileobj(response, file)
        data = path.read_bytes()
        if hashlib.md5(data).hexdigest() != sums.get(name):
            raise RuntimeError("Prüfsumme der Qt-Quelle stimmt nicht: " + name)
        inventory[name] = {"url": base + name, "sha256": hashlib.sha256(data).hexdigest()}
    return inventory


def build_bundle(bundle, development=False):
    bundle.mkdir(parents=True)
    exe = bundle / "kanata-studio-rust.exe"
    shutil.copy2(main_executable(), exe)
    shutil.copy2(ROOT / "target/release/kanata-studio-helper.exe", bundle / "kanata-studio-helper.exe")
    subprocess.run([qt_tool("windeployqt6", "windeployqt"), "--release", "--qmldir", str(ROOT / "target/ui"),
                    "--no-translations", "--no-compiler-runtime", str(exe)], check=True)
    runtime = copy_msvc_runtime(bundle)
    copy_project_notices(bundle)
    qt_version = output([qt_tool("qmake6", "qmake"), "-query", "QT_VERSION"])
    sources = {} if development else qt_sources(bundle, qt_version)
    (bundle / "share").mkdir()
    shutil.copy2(ROOT / "tests/fixtures/wrapped-layout.kbd", bundle / "share/test.kbd")
    metadata = {"name": "Kanata Studio Rust", "version": VERSION, "platform": "windows",
                "architecture": "x86_64", "license": "MIT", "redistributable": not development,
                "sourceCommit": output(["git", "-c", f"safe.directory={ROOT}", "-C", str(ROOT), "rev-parse", "HEAD"]),
                "qt": qt_version, "msvcRuntime": runtime, "qtSources": sources,
                "qtLibraries": sorted(p.name for p in bundle.glob("Qt6*.dll"))}
    (bundle / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    (bundle / "README.txt").write_text(f'''Kanata Studio Rust {VERSION} für Windows

Starten: kanata-studio-rust.exe (keine Installation nötig; der Ordner kann an
einen beliebigen Ort verschoben werden, danach den Kanata-Autostart in der App
neu einrichten).

Kanata selbst ist nicht enthalten: https://github.com/jtroo/kanata/releases
Kanata Studio sucht kanata.exe neben kanata-studio-rust.exe, in
%LOCALAPPDATA%\\kanata und im PATH. Die Konfiguration liegt standardmäßig unter
%APPDATA%\\kanata\\kanata.kbd.

„Kanata einrichten …“ legt einen Autostart-Eintrag für den aktuellen Benutzer an
(HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run, Wert „Kanata“) und
startet Kanata ohne Konsolenfenster. Administratorrechte sind nicht nötig. Nach
Speichern und Testen startet Kanata Studio Kanata neu. Ein als Administrator
gestartetes Kanata kann es nicht beenden.

Dieses Paket enthält Qt {qt_version}, QML und die MSVC-Laufzeit.

Project license: MIT (LICENSE). This application uses Qt under LGPLv3.
See THIRD_PARTY_NOTICES.md and licenses/ for copyright and license texts.
Qt DLLs, plugins and qml/ may be modified/replaced in this folder. Reverse
engineering to debug such modifications is permitted.
Qt sources of this exact version: sources/qt/ (release bundles).
See LICENSING.md for rebuild/replacement instructions.
Distribution status: {'PRIVATE DEVELOPMENT BUILD - DO NOT REDISTRIBUTE' if development else 'includes Qt sources and notices'}
''', encoding="utf-8")
    return metadata


def ui_check(bundle):
    env = dict(os.environ, QT_QPA_PLATFORM="offscreen", QT_QUICK_BACKEND="software")
    check = subprocess.run([str(bundle / "kanata-studio-rust.exe"), "--ui-check", str(bundle / "share/test.kbd")],
                           env=env, text=True, capture_output=True, timeout=60)
    if check.returncode:
        raise RuntimeError(f"Paket-UI-Prüfung fehlgeschlagen:\n{check.stdout}\n{check.stderr}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--development", action="store_true",
                        help="Private local test only; omits Qt sources, not for redistribution")
    args = parser.parse_args()
    name = f"kanata-studio-rust-{VERSION}-windows-x86_64"
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="kanata-rust-package-") as temporary:
        bundle = Path(temporary) / name
        metadata = build_bundle(bundle, development=args.development)
        ui_check(bundle)
        archive = dist / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as zipped:
            for path in sorted(bundle.rglob("*")):
                zipped.write(path, Path(name) / path.relative_to(bundle))
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        (dist / f"{archive.name}.sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
        print(f"Paket: {archive}\nQt: {metadata['qt']}\nPaket-UI-Prüfung: OK", flush=True)
