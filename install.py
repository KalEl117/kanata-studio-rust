#!/usr/bin/env python3
"""Download, verify and install the GitHub release for the current user."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile

REPOSITORY = "KalEl117/kanata-studio-rust"
RELEASE = "v0.2.0"
VERSION = "0.2.0"


def run(command):
    subprocess.run(command, check=True)


def preflight():
    if sys.version_info < (3, 12):
        raise SystemExit("Bitte Python 3.12 oder neuer installieren.")
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        raise SystemExit("Dieses Download-Paket benötigt Linux x86_64. Anleitung zum Bauen: README auf GitHub.")
    libc, version = platform.libc_ver()
    if libc != "glibc" or tuple(map(int, version.split("."))) < (2, 44):
        raise SystemExit("Dieses Download-Paket benötigt glibc 2.44 oder neuer. Bitte auf diesem Gerät aus dem Quellcode bauen.")
    if not shutil.which("gh"):
        raise SystemExit("GitHub CLI fehlt. Auf Omarchy: omarchy pkg add github-cli; danach gh auth login --web --git-protocol https")
    auth = subprocess.run(["gh", "auth", "status"], capture_output=True)
    if auth.returncode:
        raise SystemExit("Bitte zuerst anmelden: gh auth login --web --git-protocol https")


def download(directory, plugin):
    names = [f"kanata-studio-rust-{VERSION}-linux-x86_64.tar.gz"]
    if plugin:
        names.append(f"kanata-studio-plugin-{VERSION}-linux-x86_64.tar.gz")
    command = ["gh", "release", "download", RELEASE, "--repo", REPOSITORY, "--dir", str(directory)]
    for name in names:
        command += ["--pattern", name, "--pattern", name + ".sha256"]
    print("Lade App" + (" und Omarchy-Plugin" if plugin else "") + " herunter …", flush=True)
    run(command)
    # Verify every selected archive before extracting or installing anything.
    for name in names:
        checksum = (directory / (name + ".sha256")).read_text().split()
        if len(checksum) != 2 or checksum[1] != name:
            raise RuntimeError(f"Ungültige Prüfsummendatei: {name}")
        with (directory / name).open("rb") as stream:
            actual = hashlib.file_digest(stream, "sha256").hexdigest()
        if actual != checksum[0]:
            raise RuntimeError(f"Prüfsumme stimmt nicht: {name}. Es wurde nichts installiert.")
    print("Prüfsummen stimmen.", flush=True)
    for name in names:
        with tarfile.open(directory / name) as archive:
            archive.extractall(directory, filter="data")
    app = directory / f"kanata-studio-rust-{VERSION}-linux-x86_64"
    metadata = json.loads((app / "manifest.json").read_text())
    libc, version = platform.libc_ver()
    if metadata["architecture"] != platform.machine() or libc != "glibc" or tuple(map(int, version.split("."))) < tuple(map(int, metadata["minimumGlibc"].split("."))):
        raise RuntimeError("Das heruntergeladene Paket passt nicht zu diesem Gerät.")
    return app, directory / "zoltan.kanata-studio" if plugin else None


def main():
    parser = argparse.ArgumentParser(description="Kanata Studio aus dem privaten GitHub-Release installieren.")
    parser.add_argument("--app-only", action="store_true", help="Nur die App installieren")
    parser.add_argument("--no-restart", action="store_true", help="Shell-Neustart später selbst ausführen")
    parser.add_argument("--check-only", action="store_true", help="Downloads prüfen, ohne etwas zu installieren")
    args = parser.parse_args()
    preflight()
    plugin_enabled = bool(shutil.which("omarchy")) and not args.app_only
    with tempfile.TemporaryDirectory(prefix="kanata-studio-install-") as folder:
        app, plugin = download(Path(folder), plugin_enabled)
        if plugin:
            run(["omarchy", "plugin", "validate", str(plugin)])
        if args.check_only:
            print("Download, Prüfsummen und Pakete erfolgreich geprüft. Nichts installiert.")
            return
        run([sys.executable, str(app / "install.py")])
        if plugin:
            run([sys.executable, str(plugin / "install.py"), "--enable"])
            if not args.no_restart:
                run(["omarchy", "restart", "shell"])
        print("Fertig. Starte Kanata Studio Rust über den App-Launcher oder ~/.local/bin/kanata-studio-rust.")
        if plugin and args.no_restart:
            print("Zum Laden des Plugins: omarchy restart shell")
        if not (shutil.which("kanata") or (Path.home() / ".local/bin/kanata").is_file()):
            print("Für Prüfen und Speichern wird zusätzlich Kanata mit kanata.service benötigt.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError, tarfile.TarError) as error:
        sys.exit(f"Installation fehlgeschlagen: {error}")
