#!/usr/bin/env python3
"""Build the plugin archive and its download checksum."""
import argparse
import hashlib
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import tomllib

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--skip-validation", action="store_true", help="Nur für Paket-Builds ohne Omarchy")
args = parser.parse_args()
command = [sys.executable, str(root / "tools/prepare-omarchy-plugin.py")]
if args.skip_validation:
    command.append("--skip-validation")
subprocess.run(command, check=True)
version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
plugin = root / "dist/omarchy-plugin/zoltan.kanata-studio"
archive = root / "dist" / f"kanata-studio-plugin-{version}-linux-{platform.machine()}.tar.gz"
with tarfile.open(archive, "w:gz") as tar:
    tar.add(plugin, arcname=plugin.name)
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
Path(str(archive) + ".sha256").write_text(f"{digest}  {archive.name}\n")
print(f"Plugin-Paket: {archive}")
