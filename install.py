#!/usr/bin/env python3
"""Download, verify and install a public GitHub release for the current user."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.parse
import urllib.request

REPOSITORY = "KalEl117/kanata-studio-rust"


def run(command):
    subprocess.run(command, check=True)


def preflight():
    if sys.version_info < (3, 10):
        raise SystemExit("Bitte Python 3.10 oder neuer installieren.")
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "aarch64"):
        raise SystemExit("Benötigt Linux x86_64 oder aarch64. Anleitung zum Bauen: README auf GitHub.")
    if platform.libc_ver()[0] != "glibc":
        raise SystemExit("Die Download-Pakete benötigen glibc. Auf musl-Systemen bitte aus dem Quellcode bauen.")


def open_url(url):
    request = urllib.request.Request(url, headers={"User-Agent": "kanata-studio-installer"})
    return urllib.request.urlopen(request, timeout=60)


def release_metadata(release):
    endpoint = "latest" if release == "latest" else "tags/" + urllib.parse.quote(release, safe="")
    with open_url(f"https://api.github.com/repos/{REPOSITORY}/releases/{endpoint}") as response:
        return json.load(response)


def download_asset(url, destination):
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https" or parsed.netloc != "github.com" or not parsed.path.startswith(f"/{REPOSITORY}/releases/download/"):
        raise RuntimeError("Ungültige GitHub-Download-Adresse.")
    with open_url(url) as response, destination.open("wb") as stream:
        shutil.copyfileobj(response, stream)


def extract_bundle(path, directory, root):
    # Only regular files/directories inside the expected package root.
    # This works on Python versions without tarfile extraction filters, too.
    with tarfile.open(path) as archive:
        members = archive.getmembers()
        for member in members:
            parts = PurePosixPath(member.name).parts
            if (not parts or parts[0] != root or ".." in parts
                    or not (member.isdir() or member.isfile())):
                raise RuntimeError(f"Unsicherer Archiveintrag: {member.name}")
        for member in members:
            target = directory / member.name
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with archive.extractfile(member) as source, target.open("wb") as destination:
                    shutil.copyfileobj(source, destination)
                target.chmod(member.mode & 0o777)


def download(directory, plugin, release="latest"):
    metadata = release_metadata(release)
    assets = {asset["name"]: asset["browser_download_url"] for asset in metadata["assets"]}
    architecture = platform.machine()
    pattern = rf"kanata-studio-rust-([0-9][A-Za-z0-9.+-]*)-linux-{architecture}\.tar\.gz"
    candidates = [name for name in assets if re.fullmatch(pattern, name)]
    if len(candidates) != 1:
        raise RuntimeError(f"Release {metadata['tag_name']} enthält kein eindeutiges App-Paket für {architecture}.")
    app_name = candidates[0]
    version = re.fullmatch(pattern, app_name).group(1)
    names = [app_name]
    if plugin:
        names.append(f"kanata-studio-plugin-{version}-linux-{architecture}.tar.gz")
    for name in names:
        if name not in assets or name + ".sha256" not in assets:
            raise RuntimeError(f"Release unvollständig: {name} oder Prüfsumme fehlt.")
    print(f"Lade {metadata['tag_name']}: App" + (" und Omarchy-Plugin" if plugin else "") + " …", flush=True)
    for name in names:
        download_asset(assets[name], directory / name)
        download_asset(assets[name + ".sha256"], directory / (name + ".sha256"))
    # Verify every archive before extracting or installing anything.
    for name in names:
        checksum = (directory / (name + ".sha256")).read_text().split()
        if len(checksum) != 2 or checksum[1] != name or not re.fullmatch(r"[a-fA-F0-9]{64}", checksum[0]):
            raise RuntimeError(f"Ungültige Prüfsummendatei: {name}")
        digest = hashlib.sha256()
        with (directory / name).open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        if digest.hexdigest() != checksum[0].lower():
            raise RuntimeError(f"Prüfsumme stimmt nicht: {name}. Es wurde nichts installiert.")
    print("Prüfsummen stimmen.", flush=True)
    app_root = app_name.removesuffix(".tar.gz")
    for name in names:
        extract_bundle(directory / name, directory, app_root if name == app_name else "zoltan.kanata-studio")
    app = directory / app_root
    manifest = json.loads((app / "manifest.json").read_text())
    libc, installed = platform.libc_ver()
    minimum = manifest["minimumGlibc"]
    if manifest["architecture"] != architecture:
        raise RuntimeError("Das heruntergeladene Paket passt nicht zur Architektur dieses Geräts.")
    if libc != "glibc" or tuple(map(int, installed.split("."))) < tuple(map(int, minimum.split("."))):
        raise RuntimeError(f"Dieses Release benötigt glibc {minimum}+; vorhanden: {libc} {installed}. Bitte aus dem Quellcode bauen.")
    return app, directory / "zoltan.kanata-studio" if plugin else None


def main():
    parser = argparse.ArgumentParser(description="Kanata Studio aus einem öffentlichen GitHub-Release installieren.")
    parser.add_argument("--release", default="latest", help="Release-Tag, standardmäßig neuestes stabiles Release")
    parser.add_argument("--prefix", type=Path, help="Installationspräfix für die App (Standard: ~/.local)")
    parser.add_argument("--app-only", action="store_true", help="Nur die App installieren")
    parser.add_argument("--no-restart", action="store_true", help="Shell-Neustart später selbst ausführen")
    parser.add_argument("--check-only", action="store_true", help="Downloads prüfen, ohne etwas zu installieren")
    args = parser.parse_args()
    preflight()
    plugin_enabled = bool(shutil.which("omarchy")) and not args.app_only
    with tempfile.TemporaryDirectory(prefix="kanata-studio-install-") as folder:
        app, plugin = download(Path(folder), plugin_enabled, args.release)
        if plugin:
            run(["omarchy", "plugin", "validate", str(plugin)])
        if args.check_only:
            print("Download, Prüfsummen und Pakete erfolgreich geprüft. Nichts installiert.")
            return
        command = [sys.executable, str(app / "install.py")]
        if args.prefix:
            command += ["--prefix", str(args.prefix.expanduser().resolve())]
        run(command)
        if plugin:
            run([sys.executable, str(plugin / "install.py"), "--enable"])
            if not args.no_restart:
                run(["omarchy", "restart", "shell"])
        prefix = args.prefix.expanduser().resolve() if args.prefix else Path.home() / ".local"
        print(f"Fertig. Start: {prefix / 'bin/kanata-studio-rust'} oder über den App-Launcher.")
        if plugin and args.no_restart:
            print("Zum Laden des Plugins: omarchy restart shell")
        if not (shutil.which("kanata") or (Path.home() / ".local/bin/kanata").is_file()):
            print("Für Prüfen und Speichern wird zusätzlich Kanata mit kanata.service benötigt.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.CalledProcessError, tarfile.TarError) as error:
        sys.exit(f"Installation fehlgeschlagen: {error}")
