import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import QtQuick.Shapes
import Qt.labs.settings

ApplicationWindow {
    id: window
    visible: true
    width: 1440
    height: 850
    minimumWidth: 1050
    minimumHeight: 680
    title: "Kanata Studio · Qt Quick" + (studio.dirty || pendingEdit ? "  •" : "")
    color: studio.theme.background
    Settings {
        id: preferences
        property string language: "de"
        property bool layersVisible: true
        property bool editorVisible: true
    }
    property bool layersVisible: preferences.layersVisible
    onLayersVisibleChanged: preferences.layersVisible = layersVisible
    property bool editorVisible: preferences.editorVisible
    onEditorVisibleChanged: preferences.editorVisible = editorVisible
    property string uiLanguage: preferences.language
    onUiLanguageChanged: preferences.language = uiLanguage
    readonly property var translations: ({
        "Taste bearbeiten":"Edit key", "Tastenbearbeitung ausblenden":"Hide key editor", "Tastenbearbeitung einblenden":"Show key editor",
        "Ebenen ausblenden":"Hide layers", "Ebenen einblenden":"Show layers", "Ausgabe":"Output", "Ausgabe wählen":"Choose output", "Aktuelle Ausgabe":"Current output", "Weitere Aktionen…":"More actions…", "Kanata-Ausdruck bearbeiten":"Edit Kanata expression", "Erweiterte Einstellungen":"Advanced settings", "Durchlassen":"Pass through", "Tastenkombination":"Key combination", "Manuelle Eingabe":"Manual input",
        "Tastaturbelegung":"Keyboard layout", "Datei öffnen":"Open file", "Konfiguration":"Configuration", "Prüfen":"Check", "30 s testen":"Test for 30 s", "Test beenden":"End test", "Speichern":"Save", "Rückgängig":"Undo", "Wiederherstellen":"Redo", "Verwerfen":"Discard",
        "EBENEN":"LAYERS", "KONFIGURATION":"CONFIGURATION", "Neue Ebene":"New layer", "Umbenennen":"Rename", "Löschen":"Delete", "Aktive Ebene":"Active layer", "Automatisch":"Automatic", "ThinkPad-Laptop":"ThinkPad laptop", "Deutsch (ISO)":"German (ISO)", "Englisch (US, ANSI)":"English (US, ANSI)", "Englisch (UK, ISO)":"English (UK, ISO)",
        "Deutsch":"German", "Englisch US":"English US", "Englisch UK":"English UK", "Integrierte Tastatur":"Built-in keyboard", "Externe Tastatur":"External keyboard", "Tastatur":"Keyboard", "Eingabe":"Enter", "○ Normale Funktion   ● Eigene Belegung   ↳ Von Basis übernommen. Klicke eine Taste zum Bearbeiten.":"○ Normal function   ● Custom action   ↳ Inherited from base. Click a key to edit it.", "ALLE QUELLTASTEN":"ALL SOURCE KEYS", "Quelltasten":"source keys", "Ebenen":"layers",
        "Abbrechen":"Cancel", "Anlegen":"Create", "Übernehmen":"Apply", "Hinzufügen":"Add", "Schließen":"Close", "Weitere Konfigurationsblöcke":"Additional configuration blocks", "Konfigurationsblock hinzufügen":"Add configuration block", "Physische Taste hinzufügen":"Add physical key", "Ebene umbenennen":"Rename layer", "Ebene löschen":"Delete layer", "Ungespeicherte Änderungen":"Unsaved changes", "Ungespeicherter Entwurf":"Unsaved draft", "Ebenenname":"Layer name", "z. B. Navigation":"e.g. Navigation", "Leer (Durchlassen)":"Empty (transparent)",
        "Lege zuerst einen Konfigurationsblock an":"Add a configuration block first", "+ Neuer Block":"+ New block", "`defsrc` und `deflayer` werden über die Tastaturansicht bearbeitet. Andere Kanata-Blöcke bleiben als vollständiger Ausdruck editierbar.":"Edit `defsrc` and `deflayer` in the keyboard view. Other Kanata blocks remain editable as full expressions.", "Ebene „":"Delete layer ‘", "“ löschen?":"’?", " wird in defsrc und allen Ebenen angelegt.\nIn der gewählten Ebene startet sie mit Durchlassen (_).":" will be added to defsrc and every layer.\nIt starts as transparent (_) in the selected layer.", "Entwurf verwerfen und eine andere Datei öffnen?":"Discard the draft and open another file?", "Änderungen verwerfen und Kanata Studio schließen?":"Discard changes and close Kanata Studio?",
        "TASTE BEARBEITEN":"EDIT KEY", "Keine Taste ausgewählt":"No key selected", "Ebene: ":"Layer: ", "Aktion":"Action", "Tap Hold":"Tap Hold", "Makro":"Macro", "Kanata Aktion oder vollständiger Ausdruck":"Kanata action or full expression", "Aktion grafisch auswählen…":"Choose action visually…", "Tastenkombination zuweisen…":"Assign key combination…", "Aktion übernehmen":"Apply action", "Durchlassen  ( _ )":"Transparent  ( _ )", "BEISPIELE":"EXAMPLES", "Eingebettetes Tap Hold: die umgebende Aktion bleibt erhalten.":"Embedded Tap Hold: the surrounding action is preserved.", "Tipp- und Halteaktion getrennt bearbeiten.":"Edit tap and hold actions separately.", "VARIANTE":"VARIANT", "ZEITEN IN MS":"TIMING IN MS", "Wiederholung":"Repeat", "Halten":"Hold", "Wiederholung                  Haltezeit":"Repeat                  Hold time", "BEIM TIPPEN":"ON TAP", "BEIM HALTEN":"ON HOLD", "Mehrere Modifier":"Multiple modifiers", "Ebene":"Layer", "Andere Aktion":"Other action", "Strg":"Ctrl", "Strg rechts":"Right Ctrl", "Shift rechts":"Right Shift", "Super rechts":"Right Super", "Tap Hold übernehmen":"Apply Tap Hold", "Schritte laufen von oben nach unten. Eine Zahl ist eine Pause in Millisekunden.":"Steps run from top to bottom. A number is a pause in milliseconds.", "ABLAUF":"BEHAVIOR", "SCHRITTE":"STEPS", "TASTE HINZUFÜGEN":"ADD KEY", "Taste aus Tastatur auswählen":"Choose key from keyboard", "PAUSE HINZUFÜGEN":"ADD PAUSE", "Makro übernehmen":"Apply macro", "Tastenausgabe auswählen":"Choose output key", "Tastenkombination zuweisen":"Assign key combination", "Makroschritt auswählen":"Choose macro step", "Zielebene":"Target layer", "Einmaliger Modifier":"One shot modifier", "ZEICHENEBENE":"CHARACTER LAYER", "Tastatur":"Keyboard", "Funktion":"Function", "Navigation":"Navigation", "Numblock":"Numpad", "Sonderzeichen":"Symbols", "Normal":"Normal", "Umschalt":"Shift", "Modifier auswählen und dann die gewünschte Ausgabetaste anklicken.":"Select modifiers, then click the output key.", "Taste anklicken, um sie als nächsten Makroschritt hinzuzufügen.":"Click a key to add it as the next macro step.", "Taste anklicken, um sie als Tippausgabe zu übernehmen.":"Click a key to use it as the tap output.", "Layout: ":"Layout: ", "Enter":"Enter", "Rücktaste":"Backspace", "Entf":"Delete", "Einfg":"Insert", "Pos1":"Home", "Ende":"End", "Bild ↑":"Page up", "Bild ↓":"Page down", "Druck":"Print screen", "Rollen":"Scroll lock", "Pause":"Pause", "Lauter":"Volume up", "Leiser":"Volume down", "Stumm":"Mute", "Wiedergabe":"Play/Pause", "Nächster Titel":"Next track", "Vorheriger Titel":"Previous track", "Leertaste":"Space", "Fn":"Fn", "Alt links":"Left Alt", "Alt rechts":"Right Alt", "AltGr":"AltGr", "Super":"Super", "Ebene wechseln":"Switch layer", "Ebene halten":"Hold layer", "Wort großschreiben":"Caps word", "Zusatztaste einmalig":"One shot modifier", "Letzte Taste wiederholen":"Repeat last key", "Letzte Aktion wiederholen":"Repeat last action", "Tippen oder halten…":"Tap or hold…", "Makro bearbeiten…":"Edit macro…", "Änderungen verwerfen":"Discard changes", "Entwurf verwerfen":"Discard draft",
        "Kanata Prüfung erfolgreich":"Kanata check passed", "Gespeichert und Kanata neu geladen":"Saved and reloaded Kanata", "Entwurf verworfen":"Draft discarded", "Keine Änderungen zum Speichern":"No changes to save", "Konfiguration geladen":"Configuration loaded", "GESPEICHERT":"SAVED", "UNGESPEICHERTER ENTWURF":"UNSAVED DRAFT", "TEST":"TEST",
        "Alle Dateien (*)":"All files (*)", "Kanata Konfiguration (*.kbd)":"Kanata configuration (*.kbd)", "Leer (Durchlassen)":"Empty (transparent)", "Ebene bleibt aktiv, solange du die Taste hältst.":"Keep the layer active while the key is held.", "Wechselt bis zum nächsten Ebenenwechsel.":"Switch until another layer is selected.", "Nutzt die Ebene für die nächste Taste.":"Use the layer for the next key.", "Wendet einen Modifier auf die nächste Taste an.":"Apply a modifier to the next key.", "Schreibt das nächste Wort groß.":"Capitalize the next word.", "Gibt die letzte Taste erneut aus.":"Output the last key again.", "Wiederholt die letzte Tastenkombination.":"Repeat the last key action.", "Öffnet den Tap Hold Editor.":"Open the Tap Hold editor.", "Öffnet den Makro Editor.":"Open the macro editor.", "Ausgabe über Tastatur wählen":"Choose output from keyboard", "z. B. a":"e.g. a", "Eingebettetes Tap Hold: die umgebende Aktion bleibt erhalten.":"Embedded Tap Hold: the surrounding action is preserved.", "EBENEN":"LAYERS", "Ebene einmalig":"One shot layer", "Modifier":"Modifier", "Shift":"Shift", "Ctrl":"Ctrl", "Caps":"Caps", "Bild↑":"PgUp", "Bild↓":"PgDn", "Druck":"Print", "Einfg":"Insert", "Entf":"Delete", "Pos1":"Home", "Ende":"End",
        "Dieser Konfigurationsblock kann hier nicht bearbeitet werden":"This configuration block cannot be edited here", "Konfigurationsblock im Entwurf geändert":"Configuration block changed in draft", "Konfigurationsblock im Entwurf angelegt":"Configuration block added to draft", "Bitte zuerst den laufenden Test beenden":"End the running test first", "Bitte eine Taste auswählen":"Select a key", "Ungültige Tap Hold Zeiten oder Variante":"Invalid Tap Hold timing or variant", "Makro benötigt eine Variante und mindestens einen Schritt":"Macro needs a variant and at least one step", "Makropause muss zwischen 1 und 5000 ms liegen":"Macro pause must be between 1 and 5000 ms", "Keine Quelltasten vorhanden":"No source keys available", "Ungültiger Tastenname":"Invalid key name", "Diese Taste ist bereits in defsrc enthalten":"This key is already in defsrc", "Taste in defsrc und allen Ebenen angelegt":"Added key to defsrc and all layers", "Ungültiger Ebenenname":"Invalid layer name", "Ebene existiert bereits":"Layer already exists", "Die letzte Ebene kann nicht gelöscht werden":"The last layer cannot be deleted", "Ungültige Ebene":"Invalid layer", "Ebene umbenannt":"Layer renamed", "Ebene im Entwurf angelegt":"Layer added to draft", "Ebene im Entwurf gelöscht":"Layer deleted from draft", "Ebene nicht gelöscht":"Layer could not be deleted", "Ebene nicht angelegt":"Layer could not be added", "Ebene nicht umbenannt":"Layer could not be renamed", "Keine Konfiguration geöffnet":"No configuration is open", "Kanata wurde nicht gefunden":"Kanata was not found", "Temporäre Prüfdaten konnten nicht geschrieben werden":"Could not write temporary check data", "Kanata Prüfung konnte nicht beendet werden":"Kanata check did not finish", "Prüfung fehlgeschlagen":"Check failed", "Konfiguration konnte nicht geschrieben werden":"Could not write configuration", "Konfiguration konnte nicht erneut gelesen werden":"Could not reread configuration", "Konfiguration kann nicht gelesen werden":"Could not read configuration", "Konfiguration wurde extern geändert · bitte neu öffnen":"Configuration changed externally · reopen it", "Verknüpfte Konfigurationsdateien werden nicht gespeichert":"Linked configuration files are not saved", "Wiederherstellungsdienst ist nicht bereit":"Recovery service is not ready", "Ein Test läuft bereits":"A test is already running", "Test beendet · gespeicherte Belegung wieder aktiv":"Test ended · saved keymap restored", "Entwurf wird 30 Sekunden getestet · gespeicherte Belegung kehrt automatisch zurück":"Testing draft for 30 seconds · saved keymap will be restored automatically", "Test wird beendet · gespeicherte Belegung wird wiederhergestellt":"Ending test · restoring saved keymap", "Testprozess wurde unterbrochen · Wiederherstellung läuft":"Test process interrupted · recovery is running", "Kanata konnte nicht neu geladen werden · Datei wiederhergestellt":"Kanata could not reload · file restored", "Kanata konnte nicht neu geladen werden · Wiederherstellung fehlgeschlagen":"Kanata could not reload · recovery failed",
        "Feststelltaste":"Caps Lock", "Komma":"Comma", "Punkt":"Period", "Zirkumflex":"Circumflex", "Akzenttaste":"Accent key", "Semikolon":"Semicolon", "Apostroph":"Apostrophe", "Rückstrich":"Backslash", "Kleiner als":"Less than", "Zusatz-Taste":"Extra key", "Pfeil links":"Left arrow", "Pfeil rechts":"Right arrow", "Pfeil hoch":"Up arrow", "Pfeil runter":"Down arrow", "Bild hoch":"Page up", "Bild runter":"Page down", "Numblock Eingabe":"Numpad Enter", "Num Lock":"Num Lock", "Numblock":"Numpad", "Basis":"Base", "Navigation / Zahlen":"Navigation / numbers", "Sonderfunktionen":"Special functions", "Lautlos":"Mute", "Letzte Taste":"Last key", "Letzte Aktion":"Last action", "Tippen / Halten":"Tap / Hold", "Spezialaktion":"Special action", "Einmalige Zusatztaste":"One shot modifier", "Umschalt links":"Left Shift", "Umschalt rechts":"Right Shift", "Strg links":"Left Ctrl", "Super links":"Left Super", "Mehrere Modifier":"Multiple modifiers", "Eingabe":"Enter", "Taste":"Key", "z. B. a, Enter, Digit1":"e.g. a, Enter, Digit1", "z. B. (multi lctl lshft)":"e.g. (multi lctl lshft)", "Kanata Konfiguration (*.kbd)":"Kanata configuration (*.kbd)", "Alle Dateien (*)":"All files (*)",
        "Entwurf geändert · vor dem Speichern prüfen":"Draft changed · check before saving", "Änderung rückgängig gemacht":"Change undone", "Änderung wiederhergestellt":"Change restored", "Ebene nicht gelöscht":"Could not delete layer", "Taste nicht angelegt":"Could not add key", "Konfigurationsblock im Entwurf geändert":"Configuration block changed in draft", "Keine Änderungen zum Speichern":"No changes to save", "Test beendet · gespeicherte Belegung wieder aktiv":"Test ended · saved keymap restored", "Diese Taste kann nicht als Tastenkombination verwendet werden":"This key cannot be used in a key combination", "Diese Ausgabe kann nicht als Makroschritt verwendet werden":"This output cannot be used as a macro step", "Alt und AltGr können nicht gemeinsam gesetzt werden":"Alt and AltGr cannot be combined", "Bitte mindestens einen Modifier auswählen":"Select at least one modifier", "Bitte einen gültigen Tastennamen eingeben":"Enter a valid key name", "Pause muss zwischen 1 und 5000 ms liegen":"Pause must be between 1 and 5000 ms", "Vorlage für Ebene ist ungültig":"Layer template is invalid", "kanata.service konnte nicht gelesen werden":"Could not read kanata.service", "Konfigurationspfad von kanata.service ist unbekannt":"kanata.service configuration path is unknown", "Diese Datei wird von kanata.service nicht verwendet":"This file is not used by kanata.service", "Vorschauverzeichnis kann nicht geschützt werden":"Could not secure preview directory", "Wiederherstellungsdienst kann nicht angelegt werden":"Could not create recovery service", "Wiederherstellungsdienst kann nicht geschrieben werden":"Could not write recovery service", "systemd konnte nicht neu geladen werden":"Could not reload systemd", "Wiederherstellungsdienst konnte nicht aktiviert werden":"Could not enable recovery service", "Unterbrochener Test konnte nicht wiederhergestellt werden":"Could not recover interrupted test", "Ein vorheriger Test muss zuerst wiederhergestellt werden":"A previous test must be recovered first", "Verknüpfte Konfigurationsdateien können nicht getestet werden":"Linked configuration files cannot be tested", "Testdaten konnten nicht geschützt gespeichert werden":"Could not securely save test data", "Testprozess konnte nicht gestartet werden":"Could not start test process", "Es läuft kein Test":"No test is running", "Test konnte nicht beendet werden":"Could not stop test", "Ungültiger Wiederherstellungspfad":"Invalid recovery path", "Datei wurde extern geändert; Wiederherstellung gestoppt":"File changed externally; recovery stopped", "Ursprüngliche Datei konnte nicht wiederhergestellt werden":"Could not restore original file", "Konfiguration wurde vor dem Test geändert":"Configuration changed before test", "Testkonfiguration konnte nicht geschrieben werden":"Could not write test configuration"
    })
    function tr(source) {
        if (uiLanguage !== "en") return source
        var stateTranslations = {"Im Entwurf geändert":"Changed in draft", "Neu im Entwurf":"Added in draft", "Geerbte Ausgabe im Entwurf geändert":"Inherited output changed in draft", "Vorher (gespeichert)":"Before (saved)", "Nachher (Entwurf)":"After (draft)", "Ebene noch nicht gespeichert":"Layer not saved yet", "Im Entwurf geänderte Tasten in dieser Ebene: ":"Keys changed in this layer's draft: ", "Tippen":"Tap", "Halten":"Hold", "Strg":"Ctrl", "Strg rechts":"Right Ctrl", "Shift rechts":"Right Shift", "Super rechts":"Right Super", "Normal":"Normal", "Normale Funktion":"Normal function", "Eigene Belegung":"Custom action", "Von Basis übernommen":"Inherited from base", "Diese Taste verwendet ihre normale Funktion.":"This key uses its normal function.", "Diese Taste verwendet ihre normale Funktion. Erst eine geänderte Belegung wird in den Entwurf aufgenommen.":"This key uses its normal function. Only changing its action adds it to the draft.", "Diese Belegung ist direkt in der ausgewählten Ebene festgelegt.":"This action is assigned directly in the selected layer.", "Durchlassen: folgt der aktiven Basis. Angezeigt wird die Belegung aus ":"Transparent: follows the active base. Showing the action from "}
        if (Object.prototype.hasOwnProperty.call(stateTranslations, source)) return stateTranslations[source]
        if (source === "Umschalt + AltGr") return "Shift + AltGr"
        if (Object.prototype.hasOwnProperty.call(translations, source)) return translations[source]
        var numberMatch = source.match(/^(Numblock|Ziffer) (.+)$/)
        if (numberMatch) return (numberMatch[1] === "Numblock" ? "Numpad " : "Digit ") + numberMatch[2]
        var prefixes = {
            "Datei kann nicht geöffnet werden: ":"Could not open file: ", "Konfiguration nicht lesbar: ":"Configuration is invalid: ",
            "Ungültige Aktion: ":"Invalid action: ", "Aktion nicht übernommen: ":"Could not apply action: ",
            "Ungültige Tap Hold Aktion: ":"Invalid Tap Hold action: ", "Ungültiger Makroschritt: ":"Invalid macro step: ",
            "Ungültige Belegung: ":"Invalid mapping: ", "Prüfung fehlgeschlagen: ":"Check failed: ",
            "Test konnte nicht gestartet werden: ":"Could not start test: ", "Testfehler: ":"Test error: ",
            "Konfigurationsblock nicht übernommen: ":"Could not apply configuration block: ", "Konfigurationsblock nicht angelegt: ":"Could not add configuration block: ",
            "Bitte genau einen gültigen Konfigurationsblock eingeben: ":"Enter exactly one valid configuration block: ", "Konfiguration nicht lesbar: ":"Configuration is invalid: ",
            "Wiederherstellen fehlgeschlagen: ":"Redo failed: ", "Rückgängig fehlgeschlagen: ":"Undo failed: ", "Ungültige Tap Hold Aktion: ":"Invalid Tap Hold action: ",
            "Ungültiger Makroschritt: ":"Invalid macro step: ", "Taste nicht angelegt: ":"Could not add key: ", "Ebene nicht angelegt: ":"Could not add layer: ",
            "Ebene nicht umbenannt: ":"Could not rename layer: ", "Ebene nicht gelöscht: ":"Could not delete layer: ", "30 Sekunden Test nicht verfügbar: ":"30-second test unavailable: "
        }
        for (var prefix in prefixes) if (source.indexOf(prefix) === 0) return prefixes[prefix] + source.slice(prefix.length)
        return source
    }
    palette.window: window.bg
    palette.windowText: window.fg
    palette.button: window.surface
    palette.buttonText: window.fg
    palette.highlight: window.accent
    palette.highlightedText: window.chrome

    property color bg: studio.theme.background
    property color surface: studio.theme.surface
    property color chrome: studio.theme.chrome
    property color fg: studio.theme.foreground
    property color muted: studio.theme.muted
    property color accent: studio.theme.accent
    property color changeColor: studio.theme.yellow || "#e5b46c"
    readonly property int changedKeyCount: studio.keyStates.filter(function(state) { return state.changed }).length
    property color green: studio.theme.green
    property color red: studio.theme.red
    property bool pendingEdit: editorPanel.pendingEdit
    property bool forceClose: false
    function buildKeyRows(includeThinkPad) {
        var rows = [
        [["Escape", "Esc", 1], ["F1", "F1", 1], ["F2", "F2", 1], ["F3", "F3", 1], ["F4", "F4", 1], ["F5", "F5", 1], ["F6", "F6", 1], ["F7", "F7", 1], ["F8", "F8", 1], ["F9", "F9", 1], ["F10", "F10", 1], ["F11", "F11", 1], ["F12", "F12", 1]],
        [["Backquote", "^", 1], ["1", "1", 1], ["2", "2", 1], ["3", "3", 1], ["4", "4", 1], ["5", "5", 1], ["6", "6", 1], ["7", "7", 1], ["8", "8", 1], ["9", "9", 1], ["0", "0", 1], ["Minus", "ß", 1], ["Equal", "´", 1], ["Backspace", "⌫", 1.7]],
        [["Tab", "Tab", 1.4], ["q", "Q", 1], ["w", "W", 1], ["e", "E", 1], ["r", "R", 1], ["t", "T", 1], ["y", "Z", 1], ["u", "U", 1], ["i", "I", 1], ["o", "O", 1], ["p", "P", 1], ["BracketLeft", "Ü", 1], ["BracketRight", "+", 1], ["Enter", "Eingabe", 1.5]],
        [["CapsLock", "Caps", 1.7], ["a", "A", 1], ["s", "S", 1], ["d", "D", 1], ["f", "F", 1], ["g", "G", 1], ["h", "H", 1], ["j", "J", 1], ["k", "K", 1], ["l", "L", 1], ["Semicolon", "Ö", 1], ["Quote", "Ä", 1], ["Backslash", "#", 1]],
        [["ShiftLeft", "Shift", 1.45], ["IntlBackslash", "<", 1], ["z", "Y", 1], ["x", "X", 1], ["c", "C", 1], ["v", "V", 1], ["b", "B", 1], ["n", "N", 1], ["m", "M", 1], ["Comma", ",", 1], ["Period", ".", 1], ["Slash", "-", 1], ["ShiftRight", "Shift", 2.2]],
        [["ControlLeft", "Strg", 1.35], ["MetaLeft", "Super", 1.35], ["AltLeft", "Alt", 1.35], ["Space", "Leertaste", 5.6], ["AltRight", "AltGr", 1.35], ["MetaRight", "Super", 1.35], ["ControlRight", "Strg", 1.35]]
        ]
        var info = studio.layoutInfo || {}
        var choice = info.choice || "auto"
        var layout = ["de", "us", "gb"].indexOf(choice) >= 0 ? choice : (info.id || "de")
        var geometry = choice === "auto" ? (info.geometry || "iso") : choice === "us" ? "ansi" : "iso"
        var labels = layout === "us" ? {"Backquote":"`", "Minus":"-", "Equal":"=", "y":"Y", "z":"Z", "BracketLeft":"[", "BracketRight":"]", "Semicolon":";", "Quote":"'", "Backslash":"\\", "IntlBackslash":"\\", "Slash":"/", "ControlLeft":"Ctrl", "ControlRight":"Ctrl", "AltRight":"Alt"}
            : layout === "gb" ? {"Backquote":"`", "Minus":"-", "Equal":"=", "y":"Y", "z":"Z", "BracketLeft":"[", "BracketRight":"]", "Semicolon":";", "Quote":"'", "Backslash":"#", "IntlBackslash":"\\", "Slash":"/", "ControlLeft":"Ctrl", "ControlRight":"Ctrl"}
            : {"Backquote":"^", "Minus":"ß", "Equal":"´", "y":"Z", "z":"Y", "BracketLeft":"Ü", "BracketRight":"+", "Semicolon":"Ö", "Quote":"Ä", "Backslash":"#", "IntlBackslash":"<", "Slash":"-", "ControlLeft":"Strg", "ControlRight":"Strg"}
        for (var r = 0; r < rows.length; ++r)
            for (var k = 0; k < rows[r].length; ++k)
                if (labels[rows[r][k][0]] !== undefined) rows[r][k][1] = labels[rows[r][k][0]]
        if (geometry === "ansi") {
            rows[2].splice(rows[2].length - 1, 0, ["Backslash", layout === "de" ? "\\" : "\\", 1])
            rows[2][rows[2].length - 1][2] = 1.75
            rows[3] = rows[3].filter(function(key) { return key[0] !== "Backslash" })
            rows[4] = rows[4].filter(function(key) { return key[0] !== "IntlBackslash" })
            rows[4][0][2] = 2.4
        }
        if (includeThinkPad && (choice !== "auto" || (info.keyboard && info.keyboard.type === "built-in"))) {
            rows[0].push(["Home", "Pos1", 1], ["End", "Ende", 1], ["Insert", "Einfg", 1], ["Delete", "Entf", 1])
            rows[5] = [["fn", "Fn", 1], ["ControlLeft", layout === "us" ? "Ctrl" : "Strg", 1.25], ["MetaLeft", "Super", 1], ["AltLeft", "Alt", 1.1], ["Space", "Leertaste", 5.8], ["AltRight", layout === "us" ? "Alt" : "AltGr", 1.15], ["PrintScreen", "Druck", 1], ["ControlRight", layout === "us" ? "Ctrl" : "Strg", 1.1], ["PageUp", "Bild↑", 1], ["ArrowUp", "↑", 1], ["PageDown", "Bild↓", 1]]
            rows.push([["ArrowLeft", "←", 1], ["ArrowDown", "↓", 1], ["ArrowRight", "→", 1]])
        }
        return rows
    }
    property var keyRows: buildKeyRows(true)
    property var outputKeyRows: buildKeyRows(false)
    property var layoutChoices: ["auto", "thinkpad", "de", "us", "gb"]
    readonly property bool usesLaptopGeometry: (studio.layoutInfo.choice || "auto") !== "auto"
        || (studio.layoutInfo.keyboard && studio.layoutInfo.keyboard.type === "built-in")

    function keyIndex(name) {
        var lower = String(name).toLowerCase()
        var aliases = {"min":"minus", "-":"minus", "eql":"equal", "=":"equal", "lbrc":"leftbracket", "[":"leftbracket", "rbrc":"rightbracket", "]":"rightbracket", "scln":"semicolon", ";":"semicolon", "apo":"apostrophe", "apos":"apostrophe", "'":"apostrophe", "comm":"comma", ",":"comma", ".":"dot", "/":"slash", "bksl":"backslash", "\\":"backslash", "102d":"intlbackslash", "lsgt":"intlbackslash", "nubs":"intlbackslash", "<":"intlbackslash", "escape":"esc", "backquote":"grv", "backspace":"bspc", "enter":"ret", "space":"spc", "capslock":"caps", "shiftleft":"lsft", "shiftright":"rsft", "controlleft":"lctl", "controlright":"rctl", "metaleft":"lmeta", "metaright":"rmeta", "altleft":"lalt", "altright":"ralt", "period":"dot", "quote":"apostrophe", "bracketleft":"leftbracket", "bracketright":"rightbracket", "delete":"del", "insert":"ins", "pageup":"pgup", "pagedown":"pgdn", "arrowleft":"lft", "arrowright":"rght", "arrowup":"up", "arrowdown":"down", "printscreen":"ssrq", "sysrq":"ssrq"}
        var target = aliases[lower] || lower
        for (var i = 0; i < studio.keys.length; ++i) {
            var candidate = String(studio.keys[i]).toLowerCase()
            if ((aliases[candidate] || candidate) === target) return i
        }
        return -1
    }
    function actionLineText(text) {
        if (uiLanguage !== "en") return text
        if (text.endsWith(" wechseln")) return text.slice(0, -9) + " switch"
        if (text.endsWith(" halten")) return text.slice(0, -7) + " hold"
        return text.split(" + ").map(function(part) { return window.tr(part) }).join(" + ")
    }
    function stateDescription(state) {
        if (state.lines && state.lines.length) return state.lines.map(function(line) { return window.tr(line.label) + ": " + window.actionLineText(line.text) }).join("\n")
        if (state.output === "Makro" || state.output === "Spezialaktion") return window.tr(state.output) + ": " + (state.expression || "")
        return window.tr(state.output || state.label || "Normale Funktion")
    }
    function keyTooltip(state) {
        var current = window.tr(state.label) + (state.lines && state.lines.length ? "\n" + window.stateDescription(state) : state.kind === "normal" ? "" : " · " + window.tr(state.output || ""))
        if (!state.changed) return current
        var before = window.stateDescription(state.before), after = window.stateDescription(state.after)
        if (before === after) {
            before = state.before.expression || before
            after = state.after.expression || after
        }
        var heading = state.changeKind === "added" ? "Neu im Entwurf" : state.changeKind === "inherited" ? "Geerbte Ausgabe im Entwurf geändert" : "Im Entwurf geändert"
        return window.tr(heading) + "\n\n" + window.tr("Vorher (gespeichert)") + ":\n" + before + "\n\n→ " + window.tr("Nachher (Entwurf)") + ":\n" + after
    }
    function commitEditor() { return editorPanel.commit() }
    function undoEdit() {
        if (!commitEditor() || !studio.undo()) return
        editorPanel.sync()
    }
    function redoEdit() {
        if (!commitEditor() || !studio.redo()) return
        editorPanel.sync()
    }
    function chooseKey(index) {
        if (!commitEditor()) return
        studio.selectKey(index)
        editorPanel.sync()
    }
    function choosePhysicalKey(name) {
        if (!commitEditor() || !studio.selectPhysicalKey(name)) return
        editorPanel.sync()
    }
    function chooseLayer(index) {
        if (!commitEditor()) return
        studio.selectLayer(index)
        editorPanel.sync()
    }

    FileDialog {
        id: fileDialog
        title: window.tr("Datei öffnen")
        nameFilters: [window.tr("Kanata Konfiguration (*.kbd)"), window.tr("Alle Dateien (*)")]
        onAccepted: {
            if (studio.open(selectedFile.toLocalFile())) editorPanel.sync()
        }
    }
    Dialog {
        id: addLayerDialog
        objectName: "addLayerDialog"
        title: window.tr("Neue Ebene")
        anchors.centerIn: Overlay.overlay
        modal: true
        standardButtons: Dialog.NoButton
        background: Rectangle { color: window.surface; radius: 7; border.color: Qt.alpha(window.fg, 0.25) }
        footer: Rectangle {
            implicitHeight: 52
            color: window.surface
            RowLayout {
                anchors.fill: parent; anchors.margins: 8; spacing: 8
                Item { Layout.fillWidth: true }
                FlatButton { Layout.preferredWidth: 110; text: window.tr("Abbrechen"); onClicked: addLayerDialog.close() }
                FlatButton { Layout.preferredWidth: 110; text: window.tr("Anlegen"); emphasized: true; onClicked: addLayerDialog.accept() }
            }
        }
        contentItem: ColumnLayout {
            StudioField {
                id: layerNameField
                Layout.preferredWidth: 300
                placeholderText: window.tr("z. B. Navigation")
                color: window.fg
                selectByMouse: true
            }
            StudioCombo {
                id: layerCopyBox
                Layout.fillWidth: true
                model: [window.tr("Leer (Durchlassen)")].concat(studio.layers)
            }
        }
        onOpened: { layerNameField.text = ""; layerCopyBox.currentIndex = 0; layerNameField.forceActiveFocus() }
        onAccepted: { if (studio.addLayer(layerNameField.text, layerCopyBox.currentIndex - 1)) editorPanel.sync() }
    }
    Dialog {
        id: configDialog
        objectName: "configDialog"
        title: window.tr("Weitere Konfigurationsblöcke")
        anchors.centerIn: Overlay.overlay
        modal: true
        width: 760
        height: 610
        standardButtons: Dialog.NoButton
        background: Rectangle { color: window.surface; radius: 7; border.color: Qt.alpha(window.fg, 0.25) }
        footer: Rectangle {
            implicitHeight: 54
            color: window.surface
            RowLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8
                Item { Layout.fillWidth: true }
                FlatButton { Layout.preferredWidth: 120; text: window.tr("Abbrechen"); onClicked: configDialog.close() }
                FlatButton {
                    Layout.preferredWidth: 120
                    text: window.tr("Übernehmen")
                    emphasized: true
                    onClicked: {
                        if (configFormBox.currentIndex < 0) { studio.report(window.tr("Lege zuerst einen Konfigurationsblock an")); return }
                        if (studio.replaceConfigForm(studio.configForms[configFormBox.currentIndex].index, configFormText.text)) {
                            editorPanel.sync()
                            configDialog.close()
                        }
                    }
                }
            }
        }
        contentItem: ColumnLayout {
            spacing: 9
            RowLayout {
                Layout.fillWidth: true
                ComboBox {
                    id: configFormBox
                    Layout.fillWidth: true
                    model: studio.configForms
                    textRole: "name"
                    onActivated: configFormText.text = studio.configForms[currentIndex].expression
                }
                FlatButton { text: window.tr("+ Neuer Block"); Layout.fillWidth: false; onClicked: addConfigFormDialog.open() }
            }
            Text { text: window.tr("`defsrc` und `deflayer` werden über die Tastaturansicht bearbeitet. Andere Kanata-Blöcke bleiben als vollständiger Ausdruck editierbar."); color: window.muted; font.pixelSize: 11; wrapMode: Text.WordWrap; Layout.fillWidth: true }
            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                TextArea {
                    id: configFormText
                    color: window.fg
                    font.family: "monospace"
                    font.pixelSize: 12
                    wrapMode: TextEdit.NoWrap
                    selectByMouse: true
                    background: Rectangle { color: window.bg; radius: 7; border.color: Qt.alpha(window.fg, 0.15) }
                }
            }
        }
        onOpened: {
            configFormBox.currentIndex = studio.configForms.length ? 0 : -1
            configFormText.text = studio.configForms.length ? studio.configForms[0].expression : ""
        }
    }
    Dialog {
        id: addConfigFormDialog
        title: window.tr("Konfigurationsblock hinzufügen")
        anchors.centerIn: Overlay.overlay
        modal: true
        width: 620
        height: 440
        standardButtons: Dialog.NoButton
        background: Rectangle { color: window.surface; radius: 7; border.color: Qt.alpha(window.fg, 0.25) }
        footer: Rectangle {
            implicitHeight: 54
            color: window.surface
            RowLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8
                Item { Layout.fillWidth: true }
                FlatButton { Layout.preferredWidth: 120; text: window.tr("Abbrechen"); onClicked: addConfigFormDialog.close() }
                FlatButton { Layout.preferredWidth: 120; text: window.tr("Hinzufügen"); emphasized: true; onClicked: addConfigFormDialog.accept() }
            }
        }
        contentItem: TextArea {
            id: newConfigFormText
            color: window.fg
            font.family: "monospace"
            font.pixelSize: 12
            wrapMode: TextEdit.NoWrap
            selectByMouse: true
            text: "(defcfg\n  process-unmapped-keys yes\n)"
            background: Rectangle { color: window.bg; radius: 7; border.color: Qt.alpha(window.fg, 0.15) }
        }
        onAccepted: {
            if (studio.addConfigForm(newConfigFormText.text)) {
                configDialog.open()
                configFormBox.currentIndex = studio.configForms.length - 1
                configFormText.text = studio.configForms[configFormBox.currentIndex].expression
            }
        }
    }
    Dialog {
        id: renameLayerDialog
        objectName: "renameLayerDialog"
        title: window.tr("Ebene umbenennen")
        anchors.centerIn: Overlay.overlay
        modal: true
        standardButtons: Dialog.NoButton
        background: Rectangle { color: window.surface; radius: 7; border.color: Qt.alpha(window.fg, 0.25) }
        footer: DialogActions { targetDialog: renameLayerDialog; acceptLabel: "Übernehmen" }
        contentItem: StudioField {
            id: renameLayerField
            width: 300
            placeholderText: window.tr("Ebenenname")
            color: window.fg
            selectByMouse: true
        }
        onOpened: { renameLayerField.text = studio.layers[studio.selectedLayer] || ""; renameLayerField.selectAll(); renameLayerField.forceActiveFocus() }
        onAccepted: { if (studio.renameLayer(studio.selectedLayer, renameLayerField.text)) editorPanel.sync() }
    }
    Dialog {
        id: deleteLayerDialog
        title: window.tr("Ebene löschen")
        anchors.centerIn: Overlay.overlay
        modal: true
        standardButtons: Dialog.NoButton
        background: Rectangle { color: window.surface; radius: 7; border.color: Qt.alpha(window.fg, 0.25) }
        footer: DialogActions { targetDialog: deleteLayerDialog; acceptLabel: "Löschen" }
        Label { text: window.tr("Ebene „") + (studio.layers[studio.selectedLayer] || "") + window.tr("“ löschen?"); color: window.fg }
        onAccepted: { if (studio.deleteLayer(studio.selectedLayer)) editorPanel.sync() }
    }
    Dialog {
        id: discardDialog
        title: window.tr("Ungespeicherte Änderungen")
        anchors.centerIn: Overlay.overlay
        modal: true
        standardButtons: Dialog.NoButton
        footer: DialogActions { targetDialog: discardDialog; acceptLabel: "Entwurf verwerfen" }
        Label { text: window.tr("Entwurf verwerfen und eine andere Datei öffnen?"); color: window.fg }
        onAccepted: fileDialog.open()
    }
    Dialog {
        id: closeDialog
        title: window.tr("Ungespeicherter Entwurf")
        anchors.centerIn: Overlay.overlay
        modal: true
        standardButtons: Dialog.NoButton
        footer: DialogActions { targetDialog: closeDialog; acceptLabel: "Verwerfen" }
        Label { text: window.tr("Änderungen verwerfen und Kanata Studio schließen?"); color: window.fg }
        onAccepted: { window.forceClose = true; window.close() }
    }
    onClosing: function(close) {
        if (!forceClose && (studio.dirty || pendingEdit)) { close.accepted = false; closeDialog.open() }
    }
    Shortcut { sequence: StandardKey.Undo; onActivated: window.undoEdit() }
    Shortcut { sequence: StandardKey.Redo; onActivated: window.redoEdit() }

    component FlatButton: Button {
        id: button
        property bool emphasized: false
        font.pixelSize: 13
        font.weight: Font.DemiBold
        palette.buttonText: emphasized ? window.bg : window.fg
        background: Rectangle {
            radius: 5
            color: button.emphasized ? window.accent : (button.hovered ? Qt.alpha(window.fg, 0.09) : Qt.alpha(window.fg, 0.045))
            border.color: button.emphasized ? window.accent : Qt.alpha(window.fg, 0.28)
        }
    }
    component DialogActions: Rectangle {
        id: actions
        property var targetDialog
        property string acceptLabel: "Übernehmen"
        property string cancelLabel: "Abbrechen"
        implicitHeight: 54
        color: window.surface
        RowLayout {
            anchors.fill: parent
            anchors.margins: 8
            spacing: 8
            Item { Layout.fillWidth: true }
            FlatButton { Layout.preferredWidth: 120; text: window.tr(actions.cancelLabel); onClicked: actions.targetDialog.reject() }
            FlatButton { Layout.preferredWidth: 120; text: window.tr(actions.acceptLabel); emphasized: true; onClicked: actions.targetDialog.accept() }
        }
    }
    component StudioField: TextField {
        color: window.fg
        font.pixelSize: 12
        selectByMouse: true
        background: Rectangle {
            radius: 5
            color: window.chrome
            border.color: parent.activeFocus ? window.accent : Qt.alpha(window.fg, 0.26)
        }
    }
    component StudioCombo: ComboBox {
        id: control
        implicitHeight: 36
        font.pixelSize: 12
        contentItem: Text {
            leftPadding: 10
            rightPadding: 26
            text: control.displayText
            color: window.fg
            font: control.font
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        indicator: Text {
            x: control.width - width - 10
            y: (control.height - height) / 2
            text: "⌄"
            color: window.muted
            font.pixelSize: 17
        }
        background: Rectangle {
            radius: 5
            color: window.chrome
            border.color: control.activeFocus ? window.accent : Qt.alpha(window.fg, 0.26)
        }
        delegate: ItemDelegate {
            width: control.width
            text: modelData
            font.pixelSize: 11
            palette.buttonText: window.fg
            background: Rectangle { color: highlighted ? Qt.alpha(window.accent, 0.16) : window.surface }
        }
        popup: Popup {
            y: control.height + 2
            width: control.width
            implicitHeight: Math.min(contentItem.implicitHeight + 4, 250)
            padding: 2
            background: Rectangle { color: window.surface; radius: 5; border.color: Qt.alpha(window.fg, 0.25) }
            contentItem: ListView {
                clip: true
                implicitHeight: contentHeight
                model: control.popup.visible ? control.delegateModel : null
                currentIndex: control.highlightedIndex
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 66
            color: window.chrome
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 22
                anchors.rightMargin: 22
                spacing: 8
                ColumnLayout {
                    spacing: 1
                    Text { text: "KANATA / STUDIO"; color: window.muted; font.pixelSize: 10; font.letterSpacing: 1.7; font.weight: Font.Bold }
                    Text { text: window.tr("Tastaturbelegung"); color: window.fg; font.pixelSize: 20; font.weight: Font.DemiBold }
                }
                Item { Layout.fillWidth: true }
                ComboBox {
                    Layout.preferredWidth: 108
                    Layout.preferredHeight: 34
                    model: ["Deutsch", "English"]
                    currentIndex: window.uiLanguage === "en" ? 1 : 0
                    onActivated: window.uiLanguage = currentIndex === 1 ? "en" : "de"
                    ToolTip.visible: hovered
                    ToolTip.text: "Language / Sprache"
                }
                FlatButton { text: "↶"; enabled: studio.canUndo; onClicked: window.undoEdit(); ToolTip.visible: hovered; ToolTip.text: window.tr("Rückgängig") + " · Ctrl+Z" }
                FlatButton { text: "↷"; enabled: studio.canRedo; onClicked: window.redoEdit(); ToolTip.visible: hovered; ToolTip.text: window.tr("Wiederherstellen") + " · Ctrl+Shift+Z" }
                FlatButton { text: window.tr("Datei öffnen"); enabled: !studio.previewActive; onClicked: { if (!window.commitEditor()) return; studio.dirty ? discardDialog.open() : fileDialog.open() } }
                FlatButton { text: window.tr("Konfiguration"); enabled: !studio.previewActive; onClicked: { if (window.commitEditor()) configDialog.open() } }
                FlatButton { text: window.tr("Prüfen"); onClicked: { if (window.commitEditor()) studio.check() } }
                FlatButton {
                    text: studio.previewActive ? window.tr("Test beenden") + " · " + studio.previewRemaining + " s" : window.tr("30 s testen")
                    enabled: studio.previewActive || studio.previewReady
                    onClicked: {
                        if (studio.previewActive) studio.cancelPreview()
                        else if (window.commitEditor()) studio.startPreview()
                    }
                }
                FlatButton { text: window.tr("Speichern"); emphasized: true; enabled: !studio.previewActive; onClicked: { if (window.commitEditor()) studio.save() } }
            }
        }
        Rectangle { Layout.fillWidth: true; Layout.preferredHeight: 1; color: Qt.alpha(window.fg, 0.12) }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            Rectangle {
                id: layerSidebar
                visible: window.layersVisible
                Layout.preferredWidth: 232
                Layout.fillHeight: true
                color: window.chrome
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 20
                    spacing: 12
                    Text { text: window.tr("EBENEN"); color: window.muted; font.pixelSize: 11; font.weight: Font.Bold; font.letterSpacing: 1.8 }
                    Repeater {
                        model: studio.layers
                        delegate: Rectangle {
                            required property int index
                            required property var modelData
                            Layout.fillWidth: true
                            Layout.preferredHeight: 42
                            radius: 5
                            color: studio.selectedLayer === index ? Qt.alpha(window.fg, 0.08) : (layerMouse.containsMouse ? Qt.alpha(window.fg, 0.045) : "transparent")
                            border.color: studio.selectedLayer === index ? Qt.alpha(window.fg, 0.15) : "transparent"
                            Rectangle { anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 3; color: window.accent; visible: studio.selectedLayer === index }
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 12
                                anchors.rightMargin: 12
                                Text { text: String(index + 1).padStart(2, "0"); color: studio.selectedLayer === index ? window.accent : window.muted; font.pixelSize: 11; font.family: "monospace" }
                                Text { text: modelData; color: window.fg; font.pixelSize: 14; font.weight: studio.selectedLayer === index ? Font.DemiBold : Font.Normal; Layout.fillWidth: true; elide: Text.ElideRight }
                            }
                            MouseArea { id: layerMouse; anchors.fill: parent; hoverEnabled: true; onClicked: window.chooseLayer(index) }
                        }
                    }
                    FlatButton { Layout.fillWidth: true; text: "+  " + window.tr("Neue Ebene"); onClicked: addLayerDialog.open() }
                    RowLayout {
                        Layout.fillWidth: true
                        FlatButton { Layout.fillWidth: true; text: window.tr("Umbenennen"); enabled: studio.layers.length > 0; onClicked: renameLayerDialog.open() }
                        FlatButton { Layout.fillWidth: true; text: window.tr("Löschen"); enabled: studio.layers.length > 1; onClicked: deleteLayerDialog.open() }
                    }
                    Item { Layout.fillHeight: true }
                    Rectangle { Layout.fillWidth: true; height: 1; color: Qt.alpha(window.fg, 0.12) }
                    Text { text: window.tr("KONFIGURATION"); color: window.muted; font.pixelSize: 11; font.weight: Font.Bold; font.letterSpacing: 1.8 }
                    Text { text: studio.path; color: window.fg; font.pixelSize: 11; wrapMode: Text.WrapAnywhere; Layout.fillWidth: true }
                    Text { text: studio.keys.length + " " + window.tr("Quelltasten") + "  ·  " + studio.layers.length + " " + window.tr("Ebenen"); color: window.muted; font.pixelSize: 11 }
                }
            }
            Rectangle { visible: window.layersVisible; Layout.preferredWidth: 1; Layout.fillHeight: true; color: Qt.alpha(window.fg, 0.12) }

            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: availableWidth
                ColumnLayout {
                    width: parent.width
                    spacing: 24
                    Item { Layout.preferredHeight: 6 }
                    GridLayout {
                        Layout.fillWidth: true
                        Layout.leftMargin: 28
                        Layout.rightMargin: 28
                        columns: 5
                        columnSpacing: 8
                        rowSpacing: 3
                        FlatButton {
                            objectName: "layersToggle"
                            Layout.row: 1
                            Layout.column: 0
                            Layout.preferredWidth: 28
                            Layout.preferredHeight: 36
                            text: window.layersVisible ? "‹" : "›"
                            accessibleName: window.tr(window.layersVisible ? "Ebenen ausblenden" : "Ebenen einblenden")
                            Accessible.name: accessibleName
                            property string accessibleName
                            onClicked: window.layersVisible = !window.layersVisible
                            ToolTip.visible: hovered
                            ToolTip.text: accessibleName
                        }
                        Text { Layout.row: 0; Layout.column: 1; text: window.tr("Aktive Ebene"); color: window.muted; font.pixelSize: 12 }
                        Text {
                            Layout.row: 1
                            Layout.column: 1
                            Layout.preferredHeight: 36
                            verticalAlignment: Text.AlignVCenter
                            visible: window.layersVisible
                            text: studio.layers[studio.selectedLayer] || "—"
                            color: window.fg; font.pixelSize: 28; font.weight: Font.DemiBold
                        }
                        StudioCombo {
                            objectName: "compactLayerPicker"
                            Layout.row: 1
                            Layout.column: 1
                            Layout.preferredHeight: 36
                            visible: !window.layersVisible
                            Layout.preferredWidth: 150
                            model: studio.layers
                            currentIndex: studio.selectedLayer
                            onActivated: {
                                window.chooseLayer(currentIndex)
                                currentIndex = Qt.binding(function() { return studio.selectedLayer })
                            }
                        }
                        Item { Layout.row: 1; Layout.column: 2; Layout.fillWidth: true }
                        FlatButton {
                            objectName: "editorToggle"
                            Layout.row: 1
                            Layout.column: 3
                            Layout.preferredWidth: 28
                            Layout.preferredHeight: 36
                            text: window.editorVisible ? "›" : "‹"
                            Accessible.name: window.tr(window.editorVisible ? "Tastenbearbeitung ausblenden" : "Tastenbearbeitung einblenden")
                            onClicked: window.editorVisible = !window.editorVisible
                            ToolTip.visible: hovered
                            ToolTip.text: Accessible.name
                        }
                        Text { Layout.row: 0; Layout.column: 4; text: "Layout"; color: window.muted; font.pixelSize: 12 }
                        ComboBox {
                            id: layoutPicker
                            objectName: "layoutPicker"
                            Layout.row: 1
                            Layout.column: 4
                            Layout.preferredWidth: 270
                            Layout.maximumWidth: 270
                            Layout.fillWidth: true
                            Layout.preferredHeight: 36
                            font.pixelSize: 12
                            model: ["Automatisch", "ThinkPad-Laptop", "Deutsch (ISO)", "Englisch (US, ANSI)", "Englisch (UK, ISO)"].map(function(item) { return window.tr(item) })
                            currentIndex: Math.max(0, window.layoutChoices.indexOf(studio.layoutInfo.choice || "auto"))
                            onActivated: studio.setLayoutChoice(window.layoutChoices[currentIndex])
                            palette.text: window.fg
                            palette.buttonText: window.fg
                            contentItem: Text {
                                leftPadding: 12; rightPadding: layoutPicker.indicator.width + 8
                                text: layoutPicker.displayText
                                font: layoutPicker.font
                                color: window.fg
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                            }
                            background: Rectangle {
                                radius: 5
                                color: Qt.alpha(window.fg, 0.045)
                                border.color: layoutPicker.activeFocus ? window.accent : Qt.alpha(window.fg, 0.25)
                            }
                            indicator: Text {
                                x: layoutPicker.width - width - 10
                                y: (layoutPicker.height - height) / 2
                                text: "⌄"; color: window.muted; font.pixelSize: 16
                            }
                            delegate: ItemDelegate {
                                width: layoutPicker.width
                                contentItem: Text { text: modelData; color: window.fg; font: layoutPicker.font; elide: Text.ElideRight; verticalAlignment: Text.AlignVCenter }
                                highlighted: layoutPicker.highlightedIndex === index
                                background: Rectangle { color: parent.highlighted ? Qt.alpha(window.accent, 0.16) : window.surface }
                            }
                            popup: Popup {
                                y: layoutPicker.height + 3
                                width: layoutPicker.width
                                implicitHeight: contentItem.implicitHeight
                                padding: 4
                                contentItem: ListView {
                                    clip: true
                                    implicitHeight: contentHeight
                                    model: layoutPicker.popup.visible ? layoutPicker.delegateModel : null
                                    currentIndex: layoutPicker.highlightedIndex
                                }
                                background: Rectangle { color: window.surface; radius: 5; border.color: Qt.alpha(window.fg, 0.22) }
                            }
                        }
                        Text {
                            Layout.row: 2
                            Layout.column: 4
                            Layout.preferredWidth: 270
                            Layout.maximumWidth: 270
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignRight
                            text: {
                                var info = studio.layoutInfo
                                var id = ["de", "us", "gb"].indexOf(info.choice) >= 0 ? info.choice : (info.id || "de")
                                var geometry = info.choice === "auto" ? (info.geometry || "iso") : info.choice === "us" ? "ansi" : "iso"
                                var names = {de:window.tr("Deutsch"), us:window.tr("Englisch US"), gb:window.tr("Englisch UK")}
                                var device = info.keyboard ? (info.keyboard.type === "built-in" ? window.tr("Integrierte Tastatur") : window.tr("Externe Tastatur")) : (info.source || window.tr("Tastatur"))
                                return device + " · " + names[id] + " · " + geometry.toUpperCase()
                            }
                            color: window.muted; font.pixelSize: 10; horizontalAlignment: Text.AlignRight; elide: Text.ElideRight
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.leftMargin: 28
                        Layout.rightMargin: 28
                        Layout.preferredHeight: keyboardColumn.implicitHeight + 42
                        radius: 7
                        color: window.surface
                        border.color: Qt.alpha(window.fg, 0.2)
                    Column {
                        id: keyboardColumn
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: 20
                        spacing: 7
                        property real keyUnitWidth: {
                            var available = width
                            var unit = 1000
                            for (var r = 0; r < window.keyRows.length; ++r) {
                                var units = 0
                                for (var k = 0; k < window.keyRows[r].length; ++k) units += window.keyRows[r][k][2]
                                var gaps = Math.max(0, window.keyRows[r].length - 1) * 5
                                if (window.usesLaptopGeometry && r === 0) gaps += 28
                                if (window.usesLaptopGeometry && r === 3) units += 1.5
                                unit = Math.min(unit, (available - gaps) / units)
                            }
                            return Math.max(14, unit)
                        }
                        property real laptopBottomExtra: {
                            if (!window.usesLaptopGeometry || window.keyRows.length < 6) return 0
                            var row = window.keyRows[5]
                            var raw = 0
                            for (var k = 0; k < row.length; ++k) raw += keyUnitWidth * row[k][2] + 5 * (row[k][2] - 1)
                            raw += (row.length - 1) * 5
                            return Math.max(0, (width - raw) / row.length)
                        }
                        Repeater {
                            model: window.keyRows
                            delegate: Row {
                                required property var modelData
                                required property int index
                                width: keyboardColumn.width
                                property bool arrowRow: window.usesLaptopGeometry && index === window.keyRows.length - 1
                                property bool laptopStretch: window.usesLaptopGeometry && [1, 2, 3, 4, 5].indexOf(index) >= 0
                                property real baseWidth: modelData.reduce(function(total, key) { return total + keyboardColumn.keyUnitWidth * key[2] + 5 * (key[2] - 1) }, 0) + Math.max(0, modelData.length - 1) * spacing
                                property real bottomArrowWidth: keyboardColumn.keyUnitWidth + keyboardColumn.laptopBottomExtra
                                property real targetWidth: index === 3 && window.usesLaptopGeometry ? width - keyboardColumn.keyUnitWidth * 1.5 - 12 : width
                                property real extraPerKey: laptopStretch ? Math.max(0, (targetWidth - baseWidth) / modelData.length) : 0
                                height: arrowRow ? 32 : window.usesLaptopGeometry && index === 0 ? 41 : window.usesLaptopGeometry && index === 5 ? 29 : 53
                                spacing: 5
                                Item {
                                    visible: parent.arrowRow
                                    width: visible ? Math.max(0, parent.width - 3 * parent.bottomArrowWidth - 10) : 0
                                    height: 1
                                }
                                Repeater {
                                    model: parent.modelData
                                    delegate: Rectangle {
                                        id: keyTile
                                        objectName: "keyTile_" + modelData[0]
                                        required property var modelData
                                        required property int index
                                        property real functionGroupGap: window.usesLaptopGeometry && parent.index === 0 && [1, 5, 9, 13].indexOf(index) >= 0 ? 7 : 0
                                        property bool isoEnterKey: window.usesLaptopGeometry && modelData[0] === "Enter"
                                        property bool laptopArrowKey: window.usesLaptopGeometry && ["PageUp", "ArrowUp", "PageDown"].indexOf(modelData[0]) >= 0
                                        property int keyNumber: window.keyIndex(modelData[0])
                                            property bool configured: keyNumber >= 0
                                            property bool unavailableFn: modelData[0] === "fn" && (!studio.layoutInfo.keyboard || !studio.layoutInfo.keyboard.fnAvailable)
                                            property var keyState: configured ? (studio.keyStates[keyNumber] || {kind: "normal", label: "Normale Funktion"}) : {kind: "normal", label: "Normale Funktion"}
                                            property bool draftChanged: !!keyState.changed
                                            property bool showComparison: false
                                            property var actionLines: keyState.lines || []
                                            property bool chosen: configured ? keyNumber === studio.selectedKey : String(modelData[0]).toLowerCase() === studio.selectedPhysicalKey.toLowerCase()
                                        width: parent.arrowRow ? parent.bottomArrowWidth : modelData[2] * keyboardColumn.keyUnitWidth + 5 * (modelData[2] - 1) + parent.extraPerKey + functionGroupGap
                                        height: isoEnterKey ? 113 : window.usesLaptopGeometry && parent.index === 0 ? 38 : laptopArrowKey ? 32 : window.usesLaptopGeometry && parent.index === 5 ? 53 : parent.height
                                        radius: 5
                                        color: isoEnterKey ? "transparent" : chosen ? Qt.alpha(window.accent, 0.16) : draftChanged ? Qt.alpha(window.changeColor, 0.10) : keyState.kind === "own" ? Qt.alpha(window.accent, 0.08) : Qt.alpha(window.fg, 0.045)
                                        border.color: isoEnterKey ? "transparent" : chosen ? window.accent : draftChanged ? window.changeColor : keyState.kind === "own" ? Qt.alpha(window.accent, 0.55) : Qt.alpha(window.fg, 0.3)
                                        opacity: unavailableFn ? 0.58 : 1
                                        y: isoEnterKey ? 0 : 0
                                        Shape {
                                            id: enterShape
                                            anchors.fill: parent
                                            visible: parent.isoEnterKey
                                            ShapePath {
                                                fillColor: keyTile.chosen ? Qt.alpha(window.accent, 0.16) : keyTile.draftChanged ? Qt.alpha(window.changeColor, 0.10) : keyTile.keyState.kind === "own" ? Qt.alpha(window.accent, 0.08) : Qt.alpha(window.fg, 0.045)
                                                strokeColor: keyTile.chosen ? window.accent : keyTile.draftChanged ? window.changeColor : keyTile.keyState.kind === "own" ? Qt.alpha(window.accent, 0.55) : Qt.alpha(window.fg, 0.3)
                                                strokeWidth: 1
                                                startX: 0; startY: 0
                                                PathLine { x: enterShape.width; y: 0 }
                                                PathLine { x: enterShape.width; y: enterShape.height }
                                                PathLine { x: enterShape.width * 0.24; y: enterShape.height }
                                                PathLine { x: enterShape.width * 0.24; y: enterShape.height * 0.49 }
                                                PathLine { x: 0; y: enterShape.height * 0.49 }
                                                PathLine { x: 0; y: 0 }
                                            }
                                        }
                                            Column {
                                                anchors.horizontalCenter: parent.horizontalCenter
                                                y: parent.isoEnterKey ? 7 : (parent.height - height) / 2
                                                width: parent.width - 6
                                                spacing: 2
                                                Text {
                                                    width: parent.width
                                                    horizontalAlignment: Text.AlignHCenter
                                                    text: window.tr(modelData[0] === "Enter" ? "Eingabe" : modelData[1])
                                                    color: window.fg
                                                    font.pixelSize: Math.max(8, Math.min(13, parent.width / (Math.max(1, text.length) * 0.62)))
                                                    font.weight: Font.DemiBold
                                                    wrapMode: Text.WordWrap
                                                    maximumLineCount: 2
                                                    elide: Text.ElideNone
                                                }
                                                Text {
                                                    width: parent.width
                                                    horizontalAlignment: Text.AlignHCenter
                                                    text: keyState.kind === "normal" ? window.tr("Normal") : (keyState.kind === "inherited" ? "↳ " : "") + window.tr(keyState.output || keyState.label)
                                                    color: chosen ? window.accent : Qt.alpha(window.fg, 0.78)
                                                    font.pixelSize: 8
                                                    wrapMode: Text.WordWrap
                                                    maximumLineCount: 2
                                                    elide: Text.ElideNone
                                                    visible: text !== "" && keyTile.actionLines.length === 0
                                                }
                                                Repeater {
                                                    model: keyTile.actionLines
                                                    delegate: Text {
                                                        required property var modelData
                                                        required property int index
                                                        width: parent.width
                                                        horizontalAlignment: Text.AlignHCenter
                                                        text: (index === 0 && keyTile.keyState.kind === "inherited" ? "↳ " : "") + window.tr(modelData.label) + ": " + window.actionLineText(modelData.text)
                                                        color: keyTile.chosen ? window.accent : Qt.alpha(window.fg, 0.88)
                                                        font.pixelSize: 8
                                                        wrapMode: Text.WordWrap
                                                        maximumLineCount: index === 0 || keyTile.height < 50 ? 1 : 2
                                                        elide: Text.ElideRight
                                                    }
                                                }
                                            }
                                            Rectangle {
                                                anchors.top: parent.top; anchors.right: parent.right
                                                anchors.margins: 3
                                                width: 5; height: 5; radius: 2.5
                                                color: window.changeColor
                                                visible: keyTile.draftChanged
                                            }
                                            ToolTip.visible: keyMouse.containsMouse || showComparison
                                            ToolTip.text: window.keyTooltip(keyState)
                                            ToolTip.delay: 500
                                            MouseArea {
                                                id: keyMouse
                                                anchors.fill: parent
                                                hoverEnabled: true
                                                onClicked: {
                                                    if (parent.unavailableFn) return
                                                    if (parent.configured) window.chooseKey(parent.keyNumber)
                                                    else window.choosePhysicalKey(parent.modelData[0])
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Text { Layout.leftMargin: 30; text: window.tr("○ Normale Funktion   ● Eigene Belegung   ↳ Von Basis übernommen. Klicke eine Taste zum Bearbeiten."); color: window.muted; font.pixelSize: 12 }
                    Text {
                        Layout.leftMargin: 30
                        text: "● " + window.tr("Im Entwurf geänderte Tasten in dieser Ebene: ") + window.changedKeyCount
                        visible: window.changedKeyCount > 0
                        color: window.changeColor; font.pixelSize: 12
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.leftMargin: 28
                        Layout.rightMargin: 28
                        Layout.preferredHeight: extras.implicitHeight + 40
                        radius: 7
                        color: window.surface
                        ColumnLayout {
                            id: extras
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: 20
                            spacing: 12
                            Text { text: window.tr("ALLE QUELLTASTEN"); color: window.muted; font.pixelSize: 11; font.weight: Font.Bold; font.letterSpacing: 1.8 }
                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: studio.keys
                                    delegate: Rectangle {
                                        required property int index
                                        required property var modelData
                                        width: keyText.implicitWidth + 24; height: 28; radius: 6
                                        color: index === studio.selectedKey ? window.accent : Qt.alpha(window.fg, 0.07)
                                        Text { id: keyText; anchors.centerIn: parent; text: window.tr(studio.displayKeyName(modelData)); color: index === studio.selectedKey ? window.bg : window.fg; font.pixelSize: 11 }
                                        MouseArea { anchors.fill: parent; onClicked: window.chooseKey(index) }
                                    }
                                }
                            }
                        }
                    }
                    Item { Layout.preferredHeight: 20 }
                }
            }
            Rectangle { visible: window.editorVisible; Layout.preferredWidth: 1; Layout.fillHeight: true; color: Qt.alpha(window.fg, 0.12) }

            ActionPanel {
                id: editorPanel
                objectName: "editorPanel"
                visible: window.editorVisible
                Layout.preferredWidth: 370
                Layout.fillHeight: true
                backend: studio
                outputKeyRows: window.outputKeyRows
                backgroundColor: window.bg
                surfaceColor: window.surface
                foregroundColor: window.fg
                mutedColor: window.muted
                accentColor: window.accent
                translateText: window.tr
            }
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 44
            color: window.chrome
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 20
                anchors.rightMargin: 20
                Rectangle { width: 7; height: 7; radius: 4; color: studio.status.toLowerCase().includes("fehl") || studio.status.toLowerCase().includes("ungültig") ? window.red : window.green }
                Text { text: window.tr(studio.status); color: window.fg; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                Text { text: studio.previewActive ? window.tr("TEST") + " · " + studio.previewRemaining + " S" : studio.dirty || window.pendingEdit ? window.tr("UNGESPEICHERTER ENTWURF") : window.tr("GESPEICHERT"); color: studio.previewActive || studio.dirty || window.pendingEdit ? window.accent : window.muted; font.pixelSize: 10; font.weight: Font.Bold; font.letterSpacing: 1 }
            }
        }
    }
}
