#!/usr/bin/env python3
"""Kill the real preview worker and exercise next-start recovery in an isolated tree."""
import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "qt/kanata-studio-rust"
FIXTURE = ROOT / "tests/fixtures/wrapped-layout.kbd"

def run_case(external=False):
    with tempfile.TemporaryDirectory(prefix="kanata-rust-kill-") as folder:
        root = Path(folder)
        preview = root / "preview"
        preview.mkdir(mode=0o700)
        units = root / "units"
        units.mkdir()
        fakebin = root / "bin"
        fakebin.mkdir()
        log = root / "service.log"
        systemctl = fakebin / "systemctl"
        systemctl.write_text('''#!/bin/sh
printf '%s\\n' "$*" >> "$KS_TEST_SERVICE_LOG"
if [ "$2" = enable ]; then
  mkdir -p "$KANATA_STUDIO_UNIT_DIR/default.target.wants"
  ln -sf "../$3" "$KANATA_STUDIO_UNIT_DIR/default.target.wants/$3"
fi
exit 0
''')
        systemctl.chmod(0o700)
        config = root / "test.kbd"
        original = FIXTURE.read_bytes()
        candidate = original.replace(b"(deflayer base", b"(deflayer candidate", 1)
        config.write_bytes(original)
        config.chmod(0o640)
        request = {"configPath": str(config), "original": base64.b64encode(original).decode(),
                   "candidate": base64.b64encode(candidate).decode(), "deadline": int(time.time()*1000)+30000}
        (preview / "request.json").write_text(json.dumps(request))
        (preview / "request.json").chmod(0o600)
        env = dict(os.environ, PATH=f"{fakebin}:/usr/bin:/bin", QT_QPA_PLATFORM="offscreen",
                   QT_QPA_PLATFORMTHEME="", KANATA_STUDIO_QT_PREVIEW_DIR=str(preview),
                   KANATA_STUDIO_UNIT_DIR=str(units), KS_TEST_SERVICE_LOG=str(log))
        # The chosen binary can also be a packaged launcher; it must survive relocation.
        env.pop("KANATA_STUDIO_LAUNCHER", None)
        child = subprocess.Popen([str(BINARY), "--preview-worker"], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic()+5
            while config.read_bytes() != candidate and child.poll() is None and time.monotonic() < deadline:
                time.sleep(0.01)
            assert config.read_bytes() == candidate, "Worker hat den Entwurf nicht aktiviert"
            child.kill()  # SIGKILL; no cleanup handler can run.
            child.wait(timeout=5)
            assert config.read_bytes() == candidate, "Worker wurde vor der Aktivierung beendet"
            assert json.loads((preview / "state.json").read_text())["phase"] == "testing"
        finally:
            if child.poll() is None:
                child.kill()
                child.wait(timeout=5)
        if external:
            config.write_bytes(original+"\n;; externe Änderung\n".encode())
        result = subprocess.run([str(BINARY), "--startup-check", str(config)], env=env,
                                text=True, capture_output=True, timeout=15)
        report = json.loads(result.stdout)
        if external:
            assert result.returncode == 1 and not report["ok"], report
            assert config.read_bytes().endswith(";; externe Änderung\n".encode())
            assert (preview / "request.json").exists()
            assert report["state"]["phase"] == "error"
        else:
            assert result.returncode == 0 and report["ok"], (report, result.stderr)
            assert config.read_bytes() == original
            assert report["source"].encode() == original, "Editor zeigt nach Wiederherstellung falsche Daten"
            assert report["state"]["phase"] == "reverted"
            assert not (preview / "request.json").exists()
            assert config.stat().st_mode & 0o777 == 0o640
            # Read-only `systemctl show` queries (e.g. which file kanata.service runs) may repeat;
            # anything that sets up or changes units must not.
            changes = lambda: [line for line in log.read_text().splitlines() if " show " not in f" {line} "]
            first_calls = changes()
            second = subprocess.run([str(BINARY), "--startup-check", str(config)], env=env,
                                    text=True, capture_output=True, timeout=15)
            assert second.returncode == 0, second.stderr
            assert changes() == first_calls, "Folgestart richtet den Dienst erneut ein"
        return "externe Änderungen erhalten" if external else "Original und Editor nach SIGKILL wiederhergestellt"

for external in (False, True):
    print(f"OK: {run_case(external)}")
