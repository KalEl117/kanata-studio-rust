# Kanata Studio Rust

Grafischer Editor für vorhandene Kanata-Konfigurationen mit Rust-Kern,
Qt-Quick-App und einem Omarchy-Plugin. Die Mini-Tastatur im Bar-Popup unterstützt
Tastenbearbeitung, Tippen/Halten, Kanata-Code, Rückgängig und Speichern.
Der vollständige Editor bietet zusätzlich Makros und einen 30-Sekunden-Test.

## Auf einem zweiten Omarchy-Gerät installieren

### Fertige Pakete aus GitHub Releases

Das private Repository erfordert eine GitHub-Anmeldung mit Zugriffsrecht.
Lade unter **Releases** das App-Archiv und bei Bedarf das Plugin-Archiv für
Linux x86_64 herunter. Beide Archive haben eine separate `.sha256`-Datei.
Die hier erzeugten Pakete benötigen **glibc 2.44 oder neuer**; bei älteren
Systemen oder anderer Architektur bitte direkt auf dem Zielgerät bauen.

Im Download-Verzeichnis:

```sh
sha256sum -c kanata-studio-rust-0.2.0-linux-x86_64.tar.gz.sha256
tar -xzf kanata-studio-rust-0.2.0-linux-x86_64.tar.gz
python3 kanata-studio-rust-0.2.0-linux-x86_64/install.py
~/.local/bin/kanata-studio-rust
```

Optional das Omarchy-Plugin installieren:

```sh
sha256sum -c kanata-studio-plugin-0.2.0-linux-x86_64.tar.gz.sha256
tar -xzf kanata-studio-plugin-0.2.0-linux-x86_64.tar.gz
python3 zoltan.kanata-studio/install.py --enable
omarchy restart shell
```

Die App und das Plugin installieren weder Kanata noch einen Tastaturdienst und
übertragen keine persönliche Tastaturbelegung. Für Prüfen, Speichern und Testen
muss Kanata vorhanden sein. `kanata.service` muss als systemd-Benutzerdienst
bereits eingerichtet sein und die geöffnete Konfiguration verwenden.
Das Plugin öffnet standardmäßig `~/.config/kanata/kanata.kbd`, ersatzweise
`~/.config/kanata/config.kbd`. Details: [Plugin-Anleitung](omarchy-plugin/README.md).

### Aus dem Quellcode bauen

Das Repository ist eigenständig; die frühere Electron- oder Qt-Projektkopie
wird nicht benötigt. Benötigt werden Git, Python 3.11+, Rust 1.88+, Cargo,
ein C++-Compiler, make, qmake6 und Qt 6 Quick/Quick Controls 2/Concurrent.
Auf Arch/Omarchy heißen die Qt-Pakete `qt6-base`, `qt6-declarative` und
`qt6-wayland`; die Compiler-Werkzeuge gehören zu `base-devel`.

Nach dem Klonen des privaten Repositorys:

```sh
cd kanata-studio-rust
cargo fetch --locked
./build.sh
./run.sh
./package.sh --install
```

Für das Plugin zusätzlich:

```sh
python3 tools/prepare-omarchy-plugin.py
python3 tools/install-plugin.py --source dist/omarchy-plugin/zoltan.kanata-studio --enable
omarchy restart shell
```

## Installierte App und Paket

Auf diesem Rechner ist **Kanata Studio Rust** als eigener Launcher installiert. Der Terminal-Aufruf lautet `~/.local/bin/kanata-studio-rust`. Der Launcher startet die installierte ausführbare Datei direkt und baut nichts.

Das übertragbare Paket liegt unter `dist/kanata-studio-rust-0.2.0-linux-x86_64.tar.gz`, mit einer SHA-256-Prüfsumme daneben. Es enthält Qt 6.11.2, QML-Module, Plattform-Plugins und die benötigten Bibliotheken. Auf einem kompatiblen Zielgerät sind Rust, Cargo, Node und ein Compiler zum Starten nicht erforderlich.

Zielsystem: Linux x86_64 mit **glibc 2.44 oder neuer**, passenden Grafiktreibern und Wayland oder X11. Eine ältere Linux-Distribution benötigt einen Build auf einer älteren Basis. Ein zweiter Laptop wurde nicht geprüft; Paket und installierte Kopie wurden hier nach dem Entpacken in einem anderen Verzeichnis getestet. Kanata und `kanata.service` werden nicht mitinstalliert; sie sind für Prüfen, Speichern und den Testmodus erforderlich.

Zum Übertragen das Archiv entpacken und im entpackten Ordner `./install.py` ausführen. Python 3 wird nur für den Installer benötigt. Alternativ `./launch.sh` direkt starten. Die Installation verwendet `~/.local/share/kanata-studio-rust`, `~/.local/share/applications/kanata-studio-rust.desktop` und `~/.local/bin/kanata-studio-rust`. Ein anderer Installationspfad ist mit `./install.py --prefix /gewünschter/pfad` möglich.

Qt sucht seine Module und Plugins im Paket über `qt.conf`; das Verfahren beschreibt die [Qt-Dokumentation](https://doc.qt.io/qt-6/qt-conf.html). Die Bibliotheken werden dynamisch geladen, Lizenztexte liegen im Paket unter `licenses/`. Native Hilfsprogramme wie Kanata, Hyprland und systemctl verwenden weiterhin ihre ursprüngliche Bibliotheksumgebung. Die Wiederherstellung startet den Paket-Launcher, damit sie auch nach einem Neustart die mitgelieferte Qt-Laufzeit findet.

Das Paket enthält auch das Wayland-Plugin `xdg-shell`, damit die App unter Hyprland nativ startet und die Bildschirmskalierung übernimmt. `~/.local/bin/kanata-studio-rust --display-check` zeigt das verwendete Fensterprotokoll und die Skalierung je Bildschirm, ohne eine Konfiguration zu öffnen oder einen Dienst einzurichten. Qt 6 übernimmt die Skalierung automatisch; ein zusätzliches `QT_SCALE_FACTOR` multipliziert diese nochmals ([Qt High DPI](https://doc.qt.io/qt-6/highdpi.html)).

## Entwicklung starten und bauen

Voraussetzungen zum Bauen: Rust 1.88 oder neuer, Cargo, Qt 6 Quick/Quick Controls 2/Concurrent, `qmake6`, `make`, Python 3 und ein C++-Compiler. Auf einem neuen Rechner zuerst `cargo fetch --locked` ausführen.

```sh
cd kanata-studio-rust
./run.sh
```

Optional eine `.kbd`-Datei angeben. Ohne Argument wird `~/.config/kanata/kanata.kbd` oder `config.kbd` geöffnet. Es wird keine neue Konfiguration erzeugt. `run.sh` startet die vorhandene ausführbare Datei direkt; es baut nur beim ersten Start oder nach Änderungen an Quellcode, Build-Dateien oder QML. `./build.sh` erzwingt die Build-Prüfung. Die Abhängigkeiten werden dabei aus `Cargo.lock` offline verwendet.

```sh
./package.sh           # Laufzeitpaket und Prüfsumme erzeugen
./package.sh --install # zusätzlich für den aktuellen Benutzer installieren
```

Das Paketwerkzeug benötigt Linux, Python 3.11 oder neuer, `ldd`, `readelf` und die vorhandene Qt-Installation. Auf diesem Arch/Omarchy-System werden verfügbare Distributions-Lizenztexte zusätzlich über die Paketdatenbank übernommen.

Die Oberfläche liegt eigenständig unter `ui/main.qml` und `ui/ActionPanel.qml` und wird beim Bauen übernommen. Kleine Anpassungen binden die Tastenbeschriftungen an das vorbereitete Rust-Modell; die ursprünglichen QML-Dateien werden dabei nicht verändert.

## Bedienung und Datenmodell

Jede verfügbare Taste lässt sich direkt per Klick auswählen, ohne einen Entwurf oder Verlaufseintrag zu erzeugen. Erst eine geänderte Ausgabe, ein Tap Hold oder ein Makro fügt eine bisher fehlende Taste zur gemeinsamen Tastenliste und allen Ebenen hinzu. Hinzufügen und erste Belegung bilden einen einzigen Schritt für Rückgängig/Wiederherstellen; die Auswahl bleibt dabei erhalten. Die anderen Ebenen starten mit Durchlassen (`_`). Erst Testen oder Speichern ändert die aktive Konfiguration.

Die Tastatur und das Bearbeitungspanel unterscheiden normale Funktion, eigene Belegung und von der Basis übernommene Belegung. Normale Tasten sind genauso gut lesbar und auswählbar wie konfigurierte Tasten. Bei Durchlassen zeigt der Editor die Belegung der ersten Basis als Vorschau und benennt deren Herkunft; zur Laufzeit folgt die Ausgabe der tatsächlich aktiven Basis.

Tasten mit Änderungen gegenüber der gespeicherten Belegung erhalten im Layout eine goldene Kontur und einen Punkt. Beim Darüberfahren zeigt „Vorher (gespeichert) → Nachher (Entwurf)“ die alte und neue Ausgabe, einschließlich Tipp-/Halteaktionen. Neue Tasten und neue Ebenen werden kenntlich gemacht. Änderungen an einer übernommenen Basisbelegung sind auch auf durchlassenden Ebenen sichtbar. Rückgängig und Speichern aktualisieren die Markierungen; Kommentare und reine Formatierung erzeugen keine Tastenmarkierung.

Tippen/Halten-Belegungen zeigen direkt auf jeder Taste zwei konkrete Zeilen, etwa „Tippen: Esc“ und „Halten: special wechseln“. Temporär gehaltene Ebenen werden als „special halten“ unterschieden. Eingebettete Aktionen und die Basisvorschau erhalten dieselben Beschriftungen. Ein Hinweis beim Darüberfahren zeigt längere Beschriftungen vollständig.

Die Ausgabeauswahl verwendet gültige, groß-/kleinschreibungsabhängige Kanata-Tastennamen auch für Sonderzeichen und Kombinationen mit Umschalt/AltGr. Auf deutschem Layout bedeutet `Slash` die physische Minus-Taste und `Minus` die ß-Taste.

Rust verwaltet die ausgewählte Taste und Ebene. Das geparste Dokument bleibt bei Auswahländerungen im Speicher. Die Oberfläche liest vorbereitete Aktionen, Beschriftungen und Aktionsdetails; mehrere Zugriffe auf dieselben Felder lösen kein erneutes Parsen aus. Dokument, Auswahl, Aktionen, Farben, Status und Countdown haben getrennte Änderungssignale. Die Tastaturansicht wird dadurch bei einem Statuswechsel oder Countdown-Schritt nicht neu aufgebaut.

Die regelmäßige Layout-/Geräte-/Theme-Abfrage läuft außerhalb des GUI-Threads. Die erste Erkennung erfolgt vor dem Öffnen des Fensters. Deutsch, Englisch US, Englisch UK, ThinkPad sowie automatische Erkennung bleiben verfügbar. Die Layout-Auswahl wird unter `~/.config/kanata-studio-rust/settings.json` gespeichert; eine vorhandene Auswahl aus dem ersten Qt/Rust-Prototyp wird übernommen. Die Sprach- und Panel-Einstellungen der Oberfläche bleiben in Qt.

Kommentare und unberührte Dateibereiche bleiben erhalten. Beim Umbenennen werden direkte Ebenenverweise angepasst. Ebenen mit verbleibenden direkten Verweisen können nicht gelöscht werden. Indirekte Verweise über Variablen oder Alias-Konstruktionen benötigen die abschließende Kanata-Prüfung.

## Speichern und Testmodus

Prüfen verwendet eine temporäre Datei im Konfigurationsverzeichnis und begrenzt die Laufzeit des Kanata-Prozesses. Speichern schreibt atomar mit den bisherigen Dateirechten. Externe Änderungen werden erkannt; schlägt das Dienstneuladen fehl, wird die ursprüngliche Datei wiederhergestellt.

Der 30-Sekunden-Test läuft in einem eigenen Prozess. Ablauf, Abbruch und Wiederherstellung nach einer Unterbrechung liegen in Rust. `kanata-studio-rust-recover.service` verwendet `~/.local/state/kanata-studio-rust/preview`, läuft vor `kanata.service` und startet den Launcher mit `--recover-preview`. Die Diensteinrichtung wird nur bei Änderungen oder einer vorherigen unvollständigen Einrichtung wiederholt. Auf unterbrochene Tests wird bei jedem normalen Start geprüft.

Nach einer Wiederherstellung beim App-Start lädt der Editor die wiederhergestellte Datei neu. Extern veränderte Dateien werden nicht überschrieben; die Wiederherstellungsdaten bleiben zur Klärung erhalten. Während eines Tests oder beim Speichern sollte keine andere Studio-Version dieselbe Datei bearbeiten.

## Prüfungen

```sh
~/.cargo/bin/cargo test --locked --offline
~/.cargo/bin/cargo clippy --locked --offline --all-targets -- -D warnings
~/.cargo/bin/cargo fmt --check
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --self-test
python3 tools/check-recovery.py
python3 tools/check-package.py
```

Die Rust-Tests decken Parser, Bearbeitung, Auswahl, Erkennung, Einstellungen, atomisches Speichern, Dienstfehler und Wiederherstellung ab. Der Qt-Selbsttest verwendet standardmäßig `tests/fixtures/wrapped-layout.kbd` und temporäre Kopien mit einem simulierten Dienst. `check-recovery.py` aktiviert eine temporäre Konfiguration, beendet den echten Testprozess mit SIGKILL und prüft die normale Wiederherstellung beim nächsten App-Start, den korrekten Editorinhalt und den Schutz externer Änderungen. Die Tests verändern weder die aktive Tastaturkonfiguration noch den echten Kanata-Dienst.

Derselbe Wiederherstellungstest kann gegen ein entpacktes oder installiertes Paket laufen: `python3 tools/check-recovery.py /pfad/zum/launch.sh`. Der Installer und der Paket-Launcher werden zusätzlich nach einer Installation in ein temporäres Verzeichnis geprüft.

## Messungen

```sh
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --benchmark-ui tests/fixtures/wrapped-layout.kbd
KANATA_STUDIO_STARTUP_PROFILE=1 ./run.sh
```

Der Bedienungstest prüft zusätzlich den QML-Ablauf für die Auswahl einer fehlenden Taste, ihre erste Belegung und vollständiges Rückgängig/Wiederherstellen. Er misst 60 Tastenauswahlen, Ebenenwechsel und Makroänderungen mit Undo einschließlich der Aktualisierung des grafischen Editors. Er verwirft alle Entwürfe und schreibt keine Konfiguration. Gemessen wird die Bearbeitung der Aufrufe und der unmittelbaren Qt/QML-Aktualisierungen; die Zeiten sind keine vollständige Messung der Eingabe-bis-Bildschirm-Latenz. Ergebnisse für die Testkonfiguration und die aktive Konfiguration liegen unter `reports/interaction.json`.

Das Startprofil zeigt die Zeiten bis zur Qt-Initialisierung, geladenen Konfiguration, vorbereiteten Wiederherstellung, geladenen Oberfläche und zum ersten Bild. Die Zeiten beginnen im Programm; eine erforderliche Kompilierung ist nicht enthalten.

Ein Vorschaubild ohne Diensteinrichtung erzeugt:

```sh
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= ./run.sh --screenshot /tmp/kanata-studio-rust.png
```
