#!/usr/bin/env python3
"""Load the host's GLVND graphics drivers with the bundle's libraries first in the search path.

The app runs with LD_LIBRARY_PATH=<bundle>/lib. If a bundled library is older than the copy the
host driver was built against, the driver fails to load and Qt reports "EGL not available".
Offscreen smoke tests never load a driver, so this check does it explicitly.
"""
import json
import os
from pathlib import Path
import subprocess
import sys

VENDOR_DIRS = ("/usr/share/glvnd/egl_vendor.d", "/etc/glvnd/egl_vendor.d")
GLX_VENDORS = ("libGLX_mesa.so.0",)


def drivers():
    found = []
    for folder in VENDOR_DIRS:
        for entry in sorted(Path(folder).glob("*.json")) if Path(folder).is_dir() else []:
            found.append(json.loads(entry.read_text())["ICD"]["library_path"])
    return found + list(GLX_VENDORS)


def missing_host_libraries(bundle, env):
    """Libraries the bundle expects from the system (graphics stack, C++ runtime)."""
    missing = set()
    for path in bundle.rglob("*"):
        if path.is_file() and not path.is_symlink() and path.open("rb").read(4) == b"\x7fELF":
            report = subprocess.run(["ldd", str(path)], env=env, capture_output=True, text=True).stdout
            missing.update(line.split()[0] for line in report.splitlines() if "not found" in line)
    return sorted(missing)


def main():
    bundle = Path(sys.argv[1]).resolve()
    candidates = drivers()
    env = dict(os.environ, LD_LIBRARY_PATH=str(bundle / "lib"))
    missing = missing_host_libraries(bundle, env)
    if missing:
        raise SystemExit("Auf dem System fehlen Bibliotheken: " + ", ".join(missing))
    loaded = failed = 0
    for library in candidates:
        probe = subprocess.run([sys.executable, "-I", "-c", "import ctypes, sys; ctypes.CDLL(sys.argv[1])", library],
                               env=env, capture_output=True, text=True)
        if probe.returncode == 0:
            loaded += 1
            print(f"OK      {library}")
        elif "cannot open shared object file" in probe.stderr and library in GLX_VENDORS:
            print(f"fehlt   {library} (auf diesem System nicht installiert)")
        else:
            failed += 1
            print(f"FEHLER  {library}: {probe.stderr.strip().splitlines()[-1]}")
    if failed or not loaded:
        raise SystemExit("Grafiktreiber des Systems lassen sich mit den gebündelten Bibliotheken nicht laden"
                         if failed else "Kein Grafiktreiber gefunden; bitte Mesa (EGL) installieren")
    print("Grafiktreiber laden mit den gebündelten Bibliotheken.")


if __name__ == "__main__":
    main()
