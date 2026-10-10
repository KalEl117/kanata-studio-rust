#!/usr/bin/env python3
"""Install a built Omarchy plugin for the current user."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument("--enable", action="store_true", help="Add the widget to the right bar section")
    args = parser.parse_args()
    source = args.source.resolve()
    manifest = json.loads((source / "manifest.json").read_text())
    plugin_id = manifest["id"]
    if plugin_id != "zoltan.kanata-studio":
        raise SystemExit("Unbekanntes Plugin-Paket.")
    subprocess.run(["omarchy", "plugin", "validate", str(source)], check=True)
    config = Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
    destination = config / "omarchy/plugins" / plugin_id
    if source != destination.resolve():
        for item in source.rglob("*"):
            if item.is_file():
                target = destination / item.relative_to(source)
                target.parent.mkdir(parents=True, exist_ok=True)
                temporary = target.with_name(target.name + ".next")
                shutil.copy2(item, temporary)
                temporary.replace(target)
    if args.enable:
        subprocess.run(["omarchy", "plugin", "enable", plugin_id, "--section", "right"], check=True)
    print(f"Installiert: {destination}")
    print("Zum Laden: omarchy restart shell")


if __name__ == "__main__":
    main()
