# Kanata Studio Rust

Eigener Projektcode: [MIT](LICENSE). Die Desktop-App verwendet Qt unter LGPLv3;
Fremdkomponenten behalten ihre jeweiligen Lizenzen. Siehe
[Lizenzhinweise](THIRD_PARTY_NOTICES.md) und [Weitergabe von Binärpaketen](LICENSING.md).

[English](README.md)

Grafischer Editor für vorhandene Kanata-Konfigurationen mit Rust-Kern,
Qt-Quick-App und einem Omarchy-Plugin. Die Mini-Tastatur im Bar-Popup unterstützt
Tastenbearbeitung, Tippen/Halten, Kanata-Code, Rückgängig und Speichern.
Der vollständige Editor bietet zusätzlich Makros und einen 30-Sekunden-Test.

## Unter Linux installieren

Die eigenständige App benötigt weder Omarchy noch Hyprland. Sie kann unter
Wayland oder X11 laufen; Qt und QML werden im Download-Paket mitgeliefert.
Installiert wird für den aktuellen Benutzer ohne `sudo`.

### Einfacher Download-Installer

Voraussetzungen: Linux mit glibc, Python 3.10+ und ein veröffentlichtes Paket
für die eigene Architektur. GitHub-Konto, GitHub CLI, Rust und Compiler sind
für die Installation nicht erforderlich.

```sh
curl -fL https://raw.githubusercontent.com/KalEl117/kanata-studio-rust/main/install.py -o install.py
python3 install.py
```

Alternativ `install.py` im Browser aus diesem Repository herunterladen und mit
Python starten. Das Skript lädt das neueste stabile GitHub-Release, prüft alle
SHA-256-Prüfsummen, entpackt die Pakete und legt einen App-Launcher an.
Erneutes Ausführen aktualisiert die Installation. Ohne Omarchy wird nur die
App installiert; auf Omarchy zusätzlich das passende Plugin mit Aktivierung
und Shell-Neustart.

**Ab v0.2.1:** Das x86_64-Paket benötigt **glibc 2.36+**. Das ältere
Release v0.2.0 benötigte noch glibc 2.44. Für aarch64 kann der Installer
passende Assets auswählen, sobald sie veröffentlicht sind; der Build-Workflow
baut zunächst nur x86_64.

Optionen:

```sh
python3 install.py --check-only          # nur herunterladen und prüfen
python3 install.py --app-only            # auch auf Omarchy nur die App
python3 install.py --release v0.2.1       # bestimmtes Release auswählen
python3 install.py --prefix ~/.local     # anderes App-Installationspräfix
python3 install.py --no-restart          # Omarchy-Shell später neu starten
```

Das Paket bestimmt seine Mindest-glibc-Version in `manifest.json`. Ein
unpassendes Paket wird vor der Installation mit Angabe der benötigten und
vorhandenen Version abgelehnt. Alpine/musl benötigt einen eigenen Build.

Standardpfade sind `~/.local/share/kanata-studio-rust`,
`~/.local/share/applications/kanata-studio-rust.desktop` und
`~/.local/bin/kanata-studio-rust`. Falls `~/.local/bin` nicht im `PATH` steht,
kann die App über den App-Launcher oder den vollständigen Pfad starten.

### Was zusätzlich für Kanata nötig ist

Die App und das Plugin installieren weder Kanata noch einen Tastaturdienst
und übertragen keine persönliche Tastaturbelegung. Zum Prüfen, Speichern und
Live-Testen benötigt die App Kanata sowie einen eingerichteten
**systemd-Benutzerdienst `kanata.service`**, der die geöffnete Konfiguration
verwendet. Ein ausschließlich systemweiter Kanata-Dienst genügt dem aktuellen
Dienstmodell nicht. Ein automatischer Kanata-/Berechtigungsassistent ist noch
nicht enthalten.

Das Omarchy-Plugin öffnet standardmäßig `~/.config/kanata/kanata.kbd`, ersatzweise
`~/.config/kanata/config.kbd`. Details: [Plugin-Anleitung](omarchy-plugin/README.md).

### Archiv ohne Download-Installer verwenden

App-Archiv und zugehörige `.sha256`-Datei unter
[Releases](https://github.com/KalEl117/kanata-studio-rust/releases) herunterladen.
Im Download-Verzeichnis, hier am Beispiel von v0.2.1:

```sh
sha256sum -c kanata-studio-rust-0.2.1-linux-x86_64.tar.gz.sha256
tar -xzf kanata-studio-rust-0.2.1-linux-x86_64.tar.gz
python3 kanata-studio-rust-0.2.1-linux-x86_64/install.py
~/.local/bin/kanata-studio-rust
```

Alternativ lässt sich `launch.sh` direkt aus dem entpackten Ordner starten.
Python wird nur für die Installation benötigt. Das Paket enthält Qt,
QML-Module, Plattform-Plugins und die benötigten Laufzeitbibliotheken.
Grafiktreiber und eine Wayland- oder X11-Sitzung stellt das Zielsystem bereit.

## Pakete für andere Distributionen bauen

[Portable Linux bundle](.github/workflows/linux-bundle.yml) baut die App
auf Debian 12 mit Qt 6.4 und prüft, dass die gemessene Mindestversion von glibc
höchstens 2.36 beträgt. Diese Basis soll unter anderem Debian 12+, Ubuntu
24.04+/entsprechende Mint-Versionen, Fedora und Arch erreichen. Ubuntu 22.04
mit glibc 2.35 liegt außerhalb dieser Build-Basis.

Der Workflow läuft für Pull Requests, `main`, Versionstags und manuell. Er
prüft das entpackte Paket, eine Installation in einem temporären Verzeichnis
und die Wiederherstellung. Zusätzliche Container-Starttests unter Debian 12,
Ubuntu 24.04 und Fedora verwenden das fertige Paket ohne systemweit
installiertes Qt. Das prüft keinen echten Wayland-/X11-Desktop und ersetzt
keinen Grafiktreibertest auf den Zielsystemen.

Nach einem erfolgreichen Lauf liegen App- und Plugin-Archiv, Prüfsummen und Download-Installer
im Actions-Artefakt `linux-x86_64-glibc-2.36`. Der Workflow veröffentlicht kein
Release automatisch. Für ein Release müssen diese Dateien mit der passenden
Version aus `Cargo.toml` als Release-Assets hochgeladen werden. Auf Omarchy
benötigt der automatische Installer außerdem das Plugin-Archiv derselben
Version samt Prüfsumme; andernfalls `--app-only` verwenden. Die CI baut das
Plugin mit `--skip-validation`, weil dort Omarchy fehlt; auf dem Zielgerät
wird es vor der Installation weiterhin durch Omarchy validiert. Die bisherigen
lokalen Omarchy-Builds sind weiterhin möglich; private Pakete benötigen
`--development` und behalten die glibc-Grenze des Build-Rechners.

Qt sucht Module und Plugins über `qt.conf`; siehe
[Qt-Dokumentation](https://doc.qt.io/qt-6/qt-conf.html). Die Bibliotheken werden
dynamisch geladen. Neue Release-Builds enthalten die Distributions-Lizenztexte
unter `licenses/` und die passenden Quellpakete unter `sources/debian/`.
Die veröffentlichten Archive v0.2.0/v0.2.1 sind älter als dieser Mechanismus;
siehe [LICENSING.md](LICENSING.md). Native Hilfsprogramme wie Kanata und systemctl
verwenden weiterhin ihre ursprüngliche Bibliotheksumgebung.

`~/.local/bin/kanata-studio-rust --display-check` zeigt Fensterprotokoll und
Bildschirmskalierung. Qt 6 übernimmt die Skalierung automatisch; ein zusätzliches
`QT_SCALE_FACTOR` multipliziert sie nochmals ([Qt High DPI](https://doc.qt.io/qt-6/highdpi.html)).

### Aus dem Quellcode bauen

Benötigt werden Git, Python 3.11+, Rust 1.88+, Cargo, ein C++-Compiler,
make, qmake6 und Qt 6.4+ mit Quick/Quick Controls 2/Concurrent.

Auf Debian 12 / Ubuntu 24.04:

```sh
sudo apt install git python3 build-essential qt6-base-dev qt6-declarative-dev qt6-wayland \
  qml6-module-qtquick qml6-module-qtquick-controls qml6-module-qtquick-templates \
  qml6-module-qtquick-window qml6-module-qtquick-layouts qml6-module-qtquick-dialogs \
  qml6-module-qtquick-shapes qml6-module-qtqml qml6-module-qtqml-models \
  qml6-module-qtqml-workerscript qml6-module-qt-labs-settings qml6-module-qt-labs-platform
```

Auf Arch/Omarchy:

```sh
sudo pacman -S --needed git python base-devel rust qt6-base qt6-declarative qt6-wayland
```

Rust 1.88+ muss zusätzlich vorhanden sein, etwa über eine vorhandene rustup-
Installation. Nach dem Klonen:

```sh
git clone https://github.com/KalEl117/kanata-studio-rust.git
cd kanata-studio-rust
cargo fetch --locked
./build.sh
./run.sh
./package.sh --development --install # Private lokale Installation
```

Für das Omarchy-Plugin zusätzlich:

```sh
python3 tools/prepare-omarchy-plugin.py
python3 tools/install-plugin.py --source dist/omarchy-plugin/zoltan.kanata-studio --enable
omarchy restart shell
```

## Entwicklung starten und bauen

Voraussetzungen zum Bauen: Rust 1.88 oder neuer, Cargo, Qt 6 Quick/Quick Controls 2/Concurrent, `qmake6`, `make`, Python 3 und ein C++-Compiler. Auf einem neuen Rechner zuerst `cargo fetch --locked` ausführen.

```sh
cd kanata-studio-rust
./run.sh
```

Optional eine `.kbd`-Datei angeben. Ohne Argument wird `~/.config/kanata/kanata.kbd` oder `config.kbd` geöffnet. Es wird keine neue Konfiguration erzeugt. `run.sh` startet die vorhandene ausführbare Datei direkt; es baut nur beim ersten Start oder nach Änderungen an Quellcode, Build-Dateien oder QML. `./build.sh` erzwingt die Build-Prüfung. Die Abhängigkeiten werden dabei aus `Cargo.lock` offline verwendet.

```sh
./package.sh                         # Debian mit deb-src: Release-Paket samt Quellen
./package.sh --development --install # Privates lokales Paket, nicht weitergeben
```

Das Paketwerkzeug benötigt Linux, Python 3.11 oder neuer, `ldd`, `readelf` und die vorhandene Qt-Installation. Release-Pakete benötigen Debian mit passenden `deb-src`-Paketquellen und Netzwerkzugriff für die exakten Bibliotheksquellen. Diese liegen im Archiv und erhöhen Downloadgröße sowie Speicherbedarf. Details zum Neubau und Bibliotheksaustausch stehen in [LICENSING.md](LICENSING.md).

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
