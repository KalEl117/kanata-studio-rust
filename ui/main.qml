import QtQuick
import "KeyboardModel.js" as Keyboard
import "Translations.js" as Translations
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
    // Reads uiLanguage here so every binding that calls tr() re-evaluates on a language switch.
    function tr(source) { return Translations.translate(source, uiLanguage) }
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
    function buildKeyRows(includeThinkPad) { return Keyboard.baseRows(studio.layoutInfo || {}, includeThinkPad) }
    function effectiveDisplayLayout() { return Keyboard.displayLayout(studio.layoutInfo || {}) }
    readonly property bool usesFullSizeGeometry: Keyboard.isFullSize(studio.layoutInfo || {})
    readonly property bool usesLaptopGeometry: Keyboard.isLaptop(studio.layoutInfo || {})
    property var keyRows: Keyboard.buildRows(studio.layoutInfo || {})
    property var outputKeyRows: buildKeyRows(false)
    property var layoutChoices: ["auto", "thinkpad", "fullsize", "tkl", "de", "us", "gb"]

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
    // defsrc keys the drawn keyboard does not show (e.g. media keys); only these need a list.
    readonly property var hiddenSourceKeys: {
        var drawn = {}
        for (var r = 0; r < keyRows.length; ++r)
            for (var k = 0; k < keyRows[r].length; ++k) {
                var index = keyIndex(keyRows[r][k][0])
                if (index >= 0) drawn[index] = true
            }
        var hidden = []
        for (var i = 0; i < studio.keys.length; ++i)
            if (!drawn[i]) hidden.push(i)
        return hidden
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
    StudioDialog {
        id: addLayerDialog
        objectName: "addLayerDialog"
        title: window.tr("Neue Ebene")
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
    StudioDialog {
        id: configDialog
        objectName: "configDialog"
        title: window.tr("Weitere Konfigurationsblöcke")
        width: 760
        height: 610
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
                StudioCombo {
                    id: configFormBox
                    Layout.fillWidth: true
                    model: studio.configForms
                    textRole: "name"
                    onActivated: configFormText.text = studio.configForms[currentIndex].expression
                }
                FlatButton { text: window.tr("+ Neuer Block"); Layout.fillWidth: false; onClicked: addConfigFormDialog.open() }
            }
            Text { text: window.tr("`defsrc` und `deflayer` werden über die Tastaturansicht bearbeitet. Andere Kanata-Blöcke bleiben als vollständiger Ausdruck editierbar."); textFormat: Text.MarkdownText; color: window.muted; font.pixelSize: 11; wrapMode: Text.WordWrap; Layout.fillWidth: true }
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
    StudioDialog {
        id: addConfigFormDialog
        title: window.tr("Konfigurationsblock hinzufügen")
        width: 620
        height: 440
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
    StudioDialog {
        id: renameLayerDialog
        objectName: "renameLayerDialog"
        title: window.tr("Ebene umbenennen")
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
    StudioDialog {
        id: deleteLayerDialog
        title: window.tr("Ebene löschen")
        footer: DialogActions { targetDialog: deleteLayerDialog; acceptLabel: "Löschen" }
        Label { text: window.tr("Ebene „") + (studio.layers[studio.selectedLayer] || "") + window.tr("“ löschen?"); color: window.fg }
        onAccepted: { if (studio.deleteLayer(studio.selectedLayer)) editorPanel.sync() }
    }
    StudioDialog {
        id: discardDialog
        title: window.tr("Ungespeicherte Änderungen")
        footer: DialogActions { targetDialog: discardDialog; acceptLabel: "Entwurf verwerfen" }
        Label { text: window.tr("Entwurf verwerfen und eine andere Datei öffnen?"); color: window.fg }
        onAccepted: fileDialog.open()
    }
    StudioDialog {
        id: closeDialog
        title: window.tr("Ungespeicherter Entwurf")
        footer: DialogActions { targetDialog: closeDialog; acceptLabel: "Verwerfen" }
        Label { text: window.tr("Änderungen verwerfen und Kanata Studio schließen?"); color: window.fg }
        onAccepted: { window.forceClose = true; window.close() }
    }
    onClosing: function(close) {
        if (!forceClose && (studio.dirty || pendingEdit)) { close.accepted = false; closeDialog.open() }
    }
    Shortcut { sequence: StandardKey.Undo; onActivated: window.undoEdit() }
    Shortcut { sequence: StandardKey.Redo; onActivated: window.redoEdit() }

    component StudioDialog: Dialog {
        id: dialog
        anchors.centerIn: Overlay.overlay
        modal: true
        padding: 16
        standardButtons: Dialog.NoButton
        background: Rectangle { color: window.surface; radius: 7; border.color: Qt.alpha(window.fg, 0.25) }
        header: Rectangle {
            implicitHeight: 46
            color: window.chrome
            radius: 7
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 7; color: parent.color }
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Qt.alpha(window.fg, 0.12) }
            Text { anchors.left: parent.left; anchors.leftMargin: 18; anchors.verticalCenter: parent.verticalCenter; text: dialog.title; color: window.fg; font.pixelSize: 15; font.weight: Font.DemiBold }
        }
    }
    component FlatButton: Button {
        id: button
        property bool emphasized: false
        font.pixelSize: 13
        font.weight: Font.DemiBold
        leftPadding: 14
        rightPadding: 14
        topPadding: 7
        bottomPadding: 7
        opacity: enabled ? 1 : 0.4
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
        implicitHeight: 36
        leftPadding: 10
        rightPadding: 10
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
            text: control.textRole ? modelData[control.textRole] : modelData
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
                StudioCombo {
                    Layout.preferredWidth: 112
                    Layout.preferredHeight: 34
                    model: ["Deutsch", "English"]
                    currentIndex: window.uiLanguage === "en" ? 1 : 0
                    onActivated: window.uiLanguage = currentIndex === 1 ? "en" : "de"
                    ToolTip.visible: hovered
                    ToolTip.text: "Language / Sprache"
                }
                FlatButton { text: "↶"; enabled: studio.canUndo; onClicked: window.undoEdit(); ToolTip.visible: hovered; ToolTip.text: window.tr("Rückgängig") + " · Ctrl+Z" }
                FlatButton { text: "↷"; enabled: studio.canRedo; onClicked: window.redoEdit(); ToolTip.visible: hovered; ToolTip.text: window.tr("Wiederherstellen") + " · Ctrl+Shift+Z" }
                FlatButton { text: window.tr("Datei öffnen"); enabled: !studio.previewActive && !studio.busy; onClicked: { if (!window.commitEditor()) return; studio.dirty ? discardDialog.open() : fileDialog.open() } }
                FlatButton { text: window.tr("Konfiguration"); enabled: !studio.previewActive; onClicked: { if (window.commitEditor()) configDialog.open() } }
                FlatButton { text: window.tr("Prüfen"); enabled: !studio.busy; onClicked: { if (window.commitEditor()) studio.requestCheck() } }
                FlatButton {
                    text: studio.previewActive ? window.tr("Test beenden") + " · " + studio.previewRemaining + " s" : window.tr("30 s testen")
                    enabled: studio.previewActive || (studio.previewReady && !studio.busy)
                    onClicked: {
                        if (studio.previewActive) studio.cancelPreview()
                        else if (window.commitEditor()) studio.requestPreview()
                    }
                }
                FlatButton { text: window.tr("Speichern"); emphasized: true; enabled: !studio.previewActive && !studio.busy; onClicked: { if (window.commitEditor()) studio.requestSave() } }
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
                            leftPadding: 0
                            rightPadding: 0
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
                            leftPadding: 0
                            rightPadding: 0
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
                            model: ["Automatisch", "ThinkPad-Laptop", "100%-Tastatur", "Ohne Nummernblock (TKL)", "Deutsch (ISO)", "Englisch (US, ANSI)", "Englisch (UK, ISO)"].map(function(item) { return window.tr(item) })
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
                                var geometry = Keyboard.displayGeometry(info)
                                var names = {de:window.tr("Deutsch"), us:window.tr("Englisch US"), gb:window.tr("Englisch UK")}
                                var device = info.keyboard ? (info.keyboard.type === "built-in" ? window.tr("Integrierte Tastatur") : window.tr("Externe Tastatur")) : (info.source || window.tr("Tastatur"))
                                return (info.keyboard && info.keyboard.name ? info.keyboard.name : device) + " · " + names[id] + " · " + geometry.toUpperCase() + (window.usesFullSizeGeometry ? " · 100%" : "")
                            }
                            color: window.muted; font.pixelSize: 10; horizontalAlignment: Text.AlignRight; elide: Text.ElideRight
                        }
                    }
                    Text {
                        objectName: "deviceStatus"
                        Layout.fillWidth: true
                        Layout.leftMargin: 28
                        Layout.rightMargin: 28
                        color: window.muted
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        text: {
                            var info = studio.layoutInfo || {}
                            var devices = info.devices || []
                            var lines = devices.map(function(d) {
                                return d.name + " · " + (d.captured === true ? "Kanata" : d.captured === false ? "Ausgenommen" : "Gerätepfad prüfen") + (d.firmware ? " · eigene Firmware" : "")
                            })
                            if (info.available === false) lines.unshift("Keine von Kanata bearbeitete Tastatur angeschlossen.")
                            return lines.join("\n")
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.leftMargin: 28
                        Layout.rightMargin: 28
                        visible: studio.layoutInfo.available !== false
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
                                var gaps = -5
                                if (window.usesLaptopGeometry && r === 0) gaps += 28
                                if (window.usesLaptopGeometry && Keyboard.isIsoEnter(studio.layoutInfo || {}) && r === 3) units += 1.5
                                unit = Math.min(unit, (available - gaps) / units - 5)
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
                                property real targetWidth: index === 3 && window.usesLaptopGeometry && Keyboard.isIsoEnter(studio.layoutInfo || {}) ? width - keyboardColumn.keyUnitWidth * 1.5 - 12 : width
                                property real extraPerKey: laptopStretch ? Math.max(0, (targetWidth - baseWidth) / modelData.length) : 0
                                height: arrowRow ? 32 : window.usesLaptopGeometry && index === 0 ? 41 : window.usesLaptopGeometry && index === 5 ? 29 : 53
                                spacing: 5
                                Item {
                                    visible: parent.arrowRow
                                    // Three gaps: spacer→←, ←→↓, ↓→→; aligns the arrows under PageUp/↑/PageDown.
                                    width: visible ? Math.max(0, parent.width - 3 * parent.bottomArrowWidth - 3 * parent.spacing) : 0
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
                                        property bool isoEnterKey: Keyboard.isIsoEnter(studio.layoutInfo || {}) && modelData[0] === "Enter"
                                        property bool spacer: modelData[0] === ""
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
                                        height: modelData[3] === 2 ? 113 : isoEnterKey ? 113 : window.usesLaptopGeometry && parent.index === 0 ? 38 : laptopArrowKey ? 32 : window.usesLaptopGeometry && parent.index === 5 ? 53 : parent.height
                                        radius: 5
                                        color: isoEnterKey ? "transparent" : chosen ? Qt.alpha(window.accent, 0.16) : draftChanged ? Qt.alpha(window.changeColor, 0.10) : keyState.kind === "own" ? Qt.alpha(window.accent, 0.08) : Qt.alpha(window.fg, 0.045)
                                        border.color: isoEnterKey ? "transparent" : chosen ? window.accent : draftChanged ? window.changeColor : keyState.kind === "own" ? Qt.alpha(window.accent, 0.55) : Qt.alpha(window.fg, 0.3)
                                        opacity: spacer ? 0 : unavailableFn ? 0.58 : 1
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
                                                    if (parent.unavailableFn || parent.spacer) return
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
                        visible: window.hiddenSourceKeys.length > 0
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
                            Text { text: window.tr("WEITERE QUELLTASTEN"); color: window.muted; font.pixelSize: 11; font.weight: Font.Bold; font.letterSpacing: 1.8 }
                            Text { text: window.tr("Diese Tasten aus defsrc sind auf der Tastatur oben nicht dargestellt."); color: window.muted; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: window.hiddenSourceKeys
                                    delegate: Rectangle {
                                        required property int modelData
                                        width: keyText.implicitWidth + 24; height: 28; radius: 6
                                        color: modelData === studio.selectedKey ? window.accent : Qt.alpha(window.fg, 0.07)
                                        Text { id: keyText; anchors.centerIn: parent; text: window.tr(studio.displayKeyName(studio.keys[modelData])); color: modelData === studio.selectedKey ? window.bg : window.fg; font.pixelSize: 11 }
                                        MouseArea { anchors.fill: parent; onClicked: window.chooseKey(modelData) }
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
                Rectangle { width: 7; height: 7; radius: 4; color: studio.statusError ? window.red : window.green }
                Text { text: window.tr(studio.status); color: window.fg; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                Text { text: studio.previewActive ? window.tr("TEST") + " · " + studio.previewRemaining + " S" : studio.dirty || window.pendingEdit ? window.tr("UNGESPEICHERTER ENTWURF") : window.tr("GESPEICHERT"); color: studio.previewActive || studio.dirty || window.pendingEdit ? window.accent : window.muted; font.pixelSize: 10; font.weight: Font.Bold; font.letterSpacing: 1 }
            }
        }
    }
}
