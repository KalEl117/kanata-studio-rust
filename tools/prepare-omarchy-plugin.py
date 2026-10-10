#!/usr/bin/env python3
"""Build a self-contained plugin directory without changing the live shell."""
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main():
    cargo = shutil.which("cargo") or str(pathlib.Path.home() / ".cargo/bin/cargo")
    subprocess.run([cargo, "build", "--release", "--locked", "--offline", "--bin", "kanata-studio-helper"], cwd=ROOT, check=True)
    target = ROOT / "dist/omarchy-plugin/zoltan.kanata-studio"
    target.mkdir(parents=True, exist_ok=True)
    for name in ("manifest.json", "BarWidget.qml", "Backend.qml", "MiniKeyboard.qml", "KeyboardModel.js", "KeyEditor.qml", "README.md"):
        shutil.copy2(ROOT / "omarchy-plugin" / name, target / name)
    (target / "bin").mkdir(exist_ok=True)
    shutil.copy2(ROOT / "target/release/kanata-studio-helper", target / "bin/kanata-studio-helper")
    shutil.copy2(ROOT / "tools/install-plugin.py", target / "install.py")
    subprocess.run(["omarchy", "plugin", "validate", str(target)], check=True)
    print(f"Plugin vorbereitet: {target}")


if __name__ == "__main__":
    main()
