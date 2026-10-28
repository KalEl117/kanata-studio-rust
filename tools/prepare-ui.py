#!/usr/bin/env python3
"""Reuse the existing QML and bind key labels to Rust's cached model."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
target = root / "target/ui"
target.mkdir(parents=True, exist_ok=True)
changes = {
    "KeyboardModel.js": [],
    "Translations.js": [],
    "main.qml": [("Kanata Studio · Qt Quick", "Kanata Studio · Rust"),
                 ('studio.displayKeyName(modelData)', '(studio.keyNames[modelData] || studio.displayKeyName(modelData))')],
    "ActionPanel.qml": [('backend.displayKeyName(backend.selectedPhysicalKey)', '(backend.keyNames[backend.selectedPhysicalKey] || backend.displayKeyName(backend.selectedPhysicalKey))')],
}
for name, replacements in changes.items():
    source = (root / "ui" / name).read_text()
    for old, new in replacements:
        if source.count(old) != 1:
            raise SystemExit(f"QML-Anbindung muss aktualisiert werden: {name}: {old}")
        source = source.replace(old, new)
    output = target / name
    if not output.exists() or output.read_text() != source:
        output.write_text(source)
