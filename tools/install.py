#!/usr/bin/env python3
"""Install an already-built, relocatable Kanata Studio Rust bundle for one user."""
import argparse
import json
from pathlib import Path
import os
import platform
import shutil
import subprocess
import sys

def version(value):
    return tuple(int(part) for part in value.split("."))

def install(bundle, prefix):
    metadata = json.loads((bundle / "manifest.json").read_text())
    if platform.machine() != metadata["architecture"]:
        raise SystemExit(f"Dieses Paket benötigt {metadata['architecture']}; dieser Rechner verwendet {platform.machine()}.")
    libc_name, libc_version = platform.libc_ver()
    if libc_name != "glibc" or version(libc_version) < version(metadata["minimumGlibc"]):
        raise SystemExit(f"Dieses Paket benötigt glibc {metadata['minimumGlibc']} oder neuer (vorhanden: {libc_name} {libc_version}).")
    pending = Path(os.environ.get("XDG_STATE_HOME", str(Path.home()/".local/state"))) / "kanata-studio-rust/preview/state.json"
    try:
        state = json.loads(pending.read_text())
    except (OSError, ValueError):
        state = {}
    if state.get("phase") in ("pending", "testing"):
        raise SystemExit("Bitte den laufenden Kanata-Test beenden, bevor die App aktualisiert wird.")
    destination = prefix / "share/kanata-studio-rust"
    destination.parent.mkdir(parents=True, exist_ok=True)
    if bundle.resolve() != destination.resolve():
        # Replace the whole directory: files left over from another build (e.g. Qt plugins for a
        # different Qt version) would otherwise be loaded by the new one.
        staged = destination.with_name(".kanata-studio-rust.next")
        previous = destination.with_name(".kanata-studio-rust.previous")
        for leftover in (staged, previous):
            if leftover.exists(): shutil.rmtree(leftover)
        shutil.copytree(bundle, staged)
        if destination.exists(): destination.rename(previous)
        staged.rename(destination)
        if previous.exists(): shutil.rmtree(previous)
    applications = prefix / "share/applications"
    applications.mkdir(parents=True, exist_ok=True)
    # Escape Desktop Entry argument syntax, including literal percent signs.
    path = str(destination / "launch.sh").replace("\\", "\\\\").replace('"', '\\"').replace('`', '\\`').replace('$', '\\$').replace('%', '%%')
    entry = applications / "kanata-studio-rust.desktop"
    entry.write_text(f'''[Desktop Entry]
Type=Application
Name=Kanata Studio Rust
Comment=Grafischer Kanata-Konfigurationseditor
Exec="{path}"
Icon=input-keyboard
Terminal=false
Categories=Settings;HardwareSettings;
StartupNotify=true
''')
    bindir = prefix / "bin"
    bindir.mkdir(parents=True, exist_ok=True)
    launcher = bindir / "kanata-studio-rust"
    replacement = bindir / ".kanata-studio-rust.next"
    if replacement.is_symlink(): replacement.unlink()
    replacement.symlink_to(destination / "launch.sh")
    replacement.replace(launcher)
    if shutil.which("update-desktop-database"):
        subprocess.run(["update-desktop-database", str(applications)], check=False, stdout=subprocess.DEVNULL)
    print(f"Installiert: {destination}\nLauncher: Kanata Studio Rust\nStart: {launcher}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--prefix", type=Path, default=Path.home()/".local")
    args = parser.parse_args()
    install(Path(__file__).resolve().parent, args.prefix.expanduser().resolve())
