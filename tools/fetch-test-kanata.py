#!/usr/bin/env python3
"""Fetch the pinned upstream Kanata validator for isolated CI checks only."""
import hashlib
import io
from pathlib import Path
import sys
import urllib.request
import zipfile

VERSION = "v1.12.0"
SHA256 = "0bedd91567c5d7c54679061baadc37e4f83fb71750003999bc1d11f2c9754f36"


def main():
    destination = Path(sys.argv[1])
    url = f"https://github.com/jtroo/kanata/releases/download/{VERSION}/linux-binaries-x64.zip"
    with urllib.request.urlopen(url, timeout=60) as response:
        data = response.read()
    if hashlib.sha256(data).hexdigest() != SHA256:
        raise RuntimeError("Upstream Kanata archive checksum mismatch")
    destination.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        binary = destination / "kanata"
        # Deliberately select the build without command execution support.
        binary.write_bytes(archive.read("kanata_linux_x64"))
        binary.chmod(0o755)
    print(f"Test validator: Kanata {VERSION}, {binary}")


if __name__ == "__main__":
    main()
