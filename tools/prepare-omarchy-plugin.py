#!/usr/bin/env python3
"""Build a self-contained plugin directory without changing the live shell."""
import argparse
import pathlib
import shutil
import subprocess
from license_materials import copy_project_notices

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-validation", action="store_true", help="Nur für Paket-Builds ohne Omarchy; Installation validiert weiterhin")
    args = parser.parse_args()
    cargo = shutil.which("cargo") or str(pathlib.Path.home() / ".cargo/bin/cargo")
    subprocess.run([cargo, "build", "--release", "--locked", "--offline", "--bin", "kanata-studio-helper"], cwd=ROOT, check=True)
    target = ROOT / "dist/omarchy-plugin/zoltan.kanata-studio"
    target.mkdir(parents=True, exist_ok=True)
    for name in ("manifest.json", "BarWidget.qml", "Backend.qml", "MiniKeyboard.qml", "KeyEditor.qml", "README.md"):
        shutil.copy2(ROOT / "omarchy-plugin" / name, target / name)
    shutil.copy2(ROOT / "ui/KeyboardModel.js", target / "KeyboardModel.js")
    (target / "bin").mkdir(exist_ok=True)
    shutil.copy2(ROOT / "target/release/kanata-studio-helper", target / "bin/kanata-studio-helper")
    shutil.copy2(ROOT / "tools/install-plugin.py", target / "install.py")
    copy_project_notices(target)
    if not args.skip_validation:
        subprocess.run(["omarchy", "plugin", "validate", str(target)], check=True)
    print(f"Plugin vorbereitet: {target}")


if __name__ == "__main__":
    main()
