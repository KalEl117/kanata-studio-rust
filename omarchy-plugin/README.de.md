# Kanata Studio als Omarchy-Plugin

Der eigene Plugin-Code steht unter MIT (siehe LICENSE im Repository oder im
vorbereiteten Paket). Pakete enthalten außerdem THIRD_PARTY_NOTICES.md und
licenses/ mit Rust-Lizenzhinweisen. Omarchy/Quickshell und Qt stellt das System bereit.

[English](README.md)

Vorbereitete erste Integrationsstufe: ein echtes Omarchy-Bar-Widget zeigt den
Status von `kanata.service`. Linksklick öffnet und schließt ein kompaktes Popup
mit Kanata-/Backend-Status, „Aktualisieren“ und „Studio öffnen“. Erst der
Studio-Button öffnet die installierte Rust-App. Ein Klick außerhalb schließt
das Popup; Position, Farben und Maße folgen der Omarchy-Bar und dem Theme.
Die Tastatur verwendet die Geometrie der Studio-App, einschließlich ISO-Enter,
Funktionstastengruppen und ThinkPad-Pfeiltasten. Ein Tastenklick öffnet den
Bearbeitungsbereich für einfache Ausgaben, Tippen/Halten oder Kanata-Ausdrücke.
Mit „Im Entwurf übernehmen“ wird die Eingabe angewendet, noch nicht die aktive
Konfiguration verändert. Goldene Markierungen zeigen Änderungen. Rückgängig,
Wiederherstellen und Verwerfen stehen im Popup bereit.

„Speichern“ prüft den Entwurf mit Kanata und kontrolliert, dass `kanata.service`
diese Datei verwendet. Es schreibt atomar, lädt den Dienst neu und stellt bei
Dienstfehlern die vorherige Datei wieder her. Externe Dateiänderungen und laufende
Studio-Tastaturtests verhindern das Speichern. Die Ebenenauswahl schaltet keine
aktive Tastaturebene um. Bei Durchlassen zeigt die Vorschau die erste Basis;
zur Laufzeit gilt die jeweils aktive Basis. Nicht dargestellte Quelltasten
erscheinen zusätzlich unter der Tastatur.

Angewendete Entwürfe werden unter `~/.local/state/kanata-studio-plugin/drafts`
mit Dateirechten 0600 gesichert und beim Wiederöffnen nach einem Shell-Neustart
wiederhergestellt. Der Undo-Verlauf gilt für die aktuelle Backend-Sitzung;
nach einem Neustart bleibt der Entwurf, aber kein Undo-Verlauf erhalten. Noch
nicht übernommene Feldtexte bleiben beim Schließen des Popups in derselben
Shell-Sitzung erhalten. Vor einem Tasten-/Ebenenwechsel müssen sie übernommen
oder zurückgesetzt werden. „Studio öffnen“ bleibt für den vollständigen Editor,
Makro-Dialoge und den 30-Sekunden-Test verfügbar.

`Backend.qml` betreibt den Rust-Helper als separaten Prozess, prüft Protokollversion
1 und serialisiert Anfragen. Fehler und Zeitüberschreitungen werden angezeigt.

## Bauen und prüfen

Im Verzeichnis `rust-studio`:

```sh
python3 tools/prepare-omarchy-plugin.py
python3 tools/check-omarchy-plugin.py
```

Das erste Skript baut ausschließlich den Rust-Helper offline und legt das
installierbare Verzeichnis unter `dist/omarchy-plugin/zoltan.kanata-studio` an.
Es benötigt Cargo und den vorhandenen Cache sowie Omarchy für die Validierung.
Die fertige Kopie braucht kein Rust, Cargo, Node oder eigenes Qt-Laufzeitpaket;
die Oberfläche verwendet die vorhandene Omarchy-/Quickshell-Laufzeit. Das Binary
muss für Architektur und glibc des Zielsystems gebaut sein. Die Statusabfrage
braucht systemd; der Launcher die separat installierte Rust-App.
Es wird nichts in die laufende Shell installiert oder aktiviert.

Für eine spätere lokale Installation das fertige Verzeichnis **kopieren** nach
`~/.config/omarchy/plugins/zoltan.kanata-studio` (keine Symlinks), dann:

```sh
omarchy-shell shell rescanPlugins
omarchy plugin enable zoltan.kanata-studio --section right
```

Optionaler Pfad zum Editor im betreffenden `shell.json`-Bar-Eintrag:

```json
{"id":"zoltan.kanata-studio", "studioExecutable":"/absoluter/pfad/kanata-studio-rust"}
```

Für eine andere gespeicherte Konfiguration kann im selben Eintrag `configPath`
auf einen absoluten `.kbd`-Pfad gesetzt werden. Ohne diesen Wert verwendet die
Übersicht `~/.config/kanata/kanata.kbd`, ersatzweise `config.kbd`.

## Backend verwenden

`kanata-studio-helper` ohne Argumente behält die bisherige Einmalanfrage bei.
Mit `--serve` liest es eine JSON-Anfrage pro Zeile und schreibt sofort eine
JSON-Antwort pro Zeile; stdin bleibt geöffnet. `id` wird unverändert übernommen.
Leerzeilen werden ignoriert, ungültige Anfragen liefern `ok:false`.
Alle bestehenden Kernoperationen bleiben verfügbar. `source` muss bei direkten
Aufrufen mitgegeben werden; die QML-Anbindung ergänzt ihn aus der letzten Antwort.
`overview` liest die Datei unter `path` (optional) und liefert unabhängig vom
Editorverlauf alle Ebenenansichten, normalisierte Tastennamen und Layoutdaten.

```json
{"id":1,"op":"ping"}
{"id":2,"op":"loadView","source":"(defsrc a)\n(deflayer base a)\n"}
{"id":3,"op":"setAction","source":"(defsrc a)\n(deflayer base a)\n","expression":"esc"}
```

In QML nach `ready === true`:

```js
backend.loadSource(text) // Verlauf leeren, Dokument laden
backend.request({op: "select", layer: 0, key: 0})
backend.request({op: "setAction", expression: "esc"})
backend.request({op: "undo"})
```

Antworten kommen über `replied(requestId, response)`, Fehler über
`failed(requestId, message)`. `source`, `view`, `canUndo` und `canRedo` spiegeln
erfolgreiche Antworten. Für den Editor muss erst ein gültiges Dokument geladen
werden. Lange Preview-Worker gehören in einen **eigenständigen Prozess**, nicht
in die serielle Editorverbindung. Die `plugin*`-Operationen verwalten einen eigenen
Editorzustand: `pluginOpen`, `pluginSelect`, `pluginLayer`, `pluginSetAction`,
`pluginSetTapHold`, `pluginUndo`, `pluginRedo`, `pluginDiscard`, `pluginSave`.
Antworten liefern `document` mit Ebenenansichten, Auswahl, Aktionen und Entwurfsstatus.
Die Sicherung eines Entwurfs schreibt nur die Zustandsdatei; ausschließlich
`pluginSave` verändert die Kanata-Datei und lädt den Dienst neu.

## Weitere Ausbauschritte

- Visuellen Ausgabepicker und Makro-Dialog aus der Studio-App übernehmen.
- 30-Sekunden-Test mit eigenständigem Preview-Worker und Recovery anbinden.
- Weitere Layouts und größere Multi-Monitor-Sitzungen prüfen.
