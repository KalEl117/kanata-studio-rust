import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui
import qs.Commons

BarWidget {
    id: root
    moduleName: "zoltan.kanata-studio"
    property string serviceState: "unknown"
    property bool manualRefresh: false
    property string feedback: ""
    property bool opened: false
    property var overview: ({})
    property string overviewError: ""
    property bool overviewLoading: false
    property int overviewRequest: -1
    property int previewLayer: 0
    property var document: ({})
    property bool editVisible: false
    property string requestOperation: ""
    readonly property string backendStatus: backend.ready ? "Bereit" : (backend.error ? "Nicht verbunden" : "Verbindet …")
    readonly property string studioExecutable: setting("studioExecutable", Quickshell.env("HOME") + "/.local/bin/kanata-studio-rust")
    readonly property string statusText: serviceState === "active" ? "Kanata aktiv" : "Kanata: " + serviceState
    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    Backend {
        id: backend
        onReadyChanged: if (ready && root.opened) root.loadOverview()
        onReplied: function(requestId, response) {
            if (requestId !== root.overviewRequest) return
            root.overviewLoading = false
            root.document = response.document
            root.overview = response.document.overview
            root.previewLayer = response.document.selectedLayer
            root.overviewError = ""
            if (root.requestOperation === "pluginSave") root.feedback = "Gespeichert und Kanata neu geladen"
            else if (root.requestOperation === "pluginDiscard") root.feedback = "Entwurf verworfen"
            else if (root.requestOperation.indexOf("pluginSet") === 0) root.feedback = "Im Entwurf übernommen · noch nicht gespeichert"
        }
        onFailed: function(requestId, message) {
            if (requestId !== root.overviewRequest) return
            root.overviewLoading = false
            root.overviewError = message
            console.warn("Kanata-Plugin: " + message)
        }
    }

    function loadOverview() {
        if (editVisible && keyEditor.pending) return
        perform({op:"pluginOpen",path:setting("configPath", "")})
    }
    function perform(payload) {
        if (!backend.ready || overviewLoading) return
        overviewLoading = true
        if (payload.op === "pluginOpen" || payload.op === "pluginDiscard") {
            payload.stateDirectory = setting("draftDirectory", undefined)
            payload.layoutInfo = setting("layoutInfo", undefined)
        }
        requestOperation = payload.op
        overviewRequest = backend.request(payload)
    }
    function chooseKey(name) {
        if (overviewLoading) return
        if (editVisible && keyEditor.pending) { overviewError = "Bitte die Eingabe zuerst übernehmen oder zurücksetzen"; return }
        editVisible = true
        perform({op:"pluginSelect",name:name})
    }
    function chooseLayer(index) {
        if (editVisible && keyEditor.pending) { overviewError = "Bitte die Eingabe zuerst übernehmen oder zurücksetzen"; return }
        perform({op:"pluginLayer",layer:index})
    }

    function refresh(manual) {
        if (manual === true) manualRefresh = true
        if (!statusProcess.running) statusProcess.running = true
    }
    function open() {
        opened = true
        refresh()
        loadOverview()
        if (root.bar) root.bar.hideTooltip(button)
    }
    function close() { opened = false }
    function toggle() { opened ? close() : open() }
    function updateStatus() {
        refresh(true)
        if (!backend.ready) backend.start()
        loadOverview()
    }
    function launchStudio() {
        close()
        Quickshell.execDetached([studioExecutable])
    }

    BarIconButton {
        id: button
        objectName: "kanataStatusButton"
        anchors.fill: parent
        bar: root.bar
        text: "󰌌"
        active: root.opened
        activeColor: Color.accent
        dimmed: root.serviceState !== "active"
        tooltipText: (root.feedback ? root.feedback + "\n" : "") + root.statusText
            + "\nLinksklick: Status-Popup · Rechtsklick: Aktualisieren"
            + (backend.error ? "\n" + backend.error : "")
        onPressed: function(mouseButton) {
            if (mouseButton === Qt.RightButton) {
                root.updateStatus()
            } else if (mouseButton === Qt.LeftButton) root.toggle()
        }
    }
    PopupCard {
        id: popup
        anchorItem: button
        bar: root.bar
        owner: root
        open: root.opened
        contentWidth: fittedContentWidth(Style.space(root.editVisible ? 1120 : 880))
        contentHeight: fittedContentHeight(content.implicitHeight)

        Flickable {
            anchors.fill: parent
            clip: true
            contentWidth: width
            contentHeight: content.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
        Column {
            id: content
            width: parent.width
            spacing: Style.space(12)
            Keys.onEscapePressed: root.close()

            Text {
                text: "Kanata Studio"
                color: Color.popups.text
                font.family: Style.font.family
                font.pixelSize: Style.font.subtitle
                font.bold: true
            }
            PanelSeparator { width: parent.width; foreground: Color.popups.text }
            Text {
                objectName: "kanataServiceStatus"
                width: parent.width
                textFormat: Text.PlainText
                text: root.statusText
                color: Color.popups.text
                font.family: Style.font.family
                font.pixelSize: Style.font.body
                wrapMode: Text.WordWrap
            }
            Text {
                width: parent.width
                textFormat: Text.PlainText
                text: "Studio-Backend: " + root.backendStatus
                color: backend.ready ? Color.popups.text : Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.body
                wrapMode: Text.WordWrap
            }
            Text {
                width: parent.width
                visible: backend.error !== ""
                textFormat: Text.PlainText
                text: backend.error
                color: Color.urgent
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                wrapMode: Text.WordWrap
            }
            Text {
                width: parent.width
                visible: root.feedback !== ""
                textFormat: Text.PlainText
                text: root.feedback
                color: Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                wrapMode: Text.WordWrap
            }
            PanelSeparator { width: parent.width; foreground: Color.popups.text }
            Text {
                width: parent.width
                text: root.document.dirty ? "Ungespeicherter Entwurf · geänderte Tasten sind gold markiert" : "Gespeicherte Belegung · Taste anklicken zum Bearbeiten"
                color: Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                wrapMode: Text.WordWrap
            }
            Text {
                width: parent.width
                visible: root.overviewLoading || root.overviewError !== ""
                textFormat: Text.PlainText
                text: root.overviewError || "Belegung wird geladen …"
                color: root.overviewError ? Color.urgent : Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                wrapMode: Text.WordWrap
            }
            Flow {
                width: parent.width
                spacing: Style.space(6)
                Repeater {
                    objectName: "kanataLayerChoices"
                    model: root.overview.views ? root.overview.views[0].layers : []
                    Button {
                        required property string modelData
                        required property int index
                        objectName: "kanataLayer" + index
                        text: modelData
                        selected: root.previewLayer === index
                        enabled: !root.overviewLoading && (!root.editVisible || !keyEditor.pending)
                        foreground: Color.popups.text
                        bordered: true
                        focusable: true
                        onClicked: root.chooseLayer(index)
                    }
                }
            }
            Row {
                width: parent.width
                spacing: Style.space(14)
                visible: root.overview.views !== undefined
                MiniKeyboard {
                    width: root.editVisible ? (parent.width-parent.spacing)*0.64 : parent.width
                    overview: root.overview
                    selectedLayer: root.previewLayer
                    selectedPhysicalKey: root.editVisible ? root.document.selectedPhysicalKey || "" : ""
                    onKeyClicked: function(name) { root.chooseKey(name) }
                }
                KeyEditor {
                    id: keyEditor
                    width: (parent.width-parent.spacing)*0.36
                    visible: root.editVisible
                    document: root.document
                    busy: root.overviewLoading
                    onApplyRequested: function(payload) { root.perform(payload) }
                }
            }
            Flow {
                width: parent.width
                spacing: Style.space(8)
                Button { text:"Rückgängig";foreground:Color.popups.text;bordered:true;enabled:!!root.document.canUndo && !root.overviewLoading && !keyEditor.pending;onClicked:root.perform({op:"pluginUndo"}) }
                Button { text:"Wiederherstellen";foreground:Color.popups.text;bordered:true;enabled:!!root.document.canRedo && !root.overviewLoading && !keyEditor.pending;onClicked:root.perform({op:"pluginRedo"}) }
                Button { objectName:"kanataSaveDraft";text:"Speichern";foreground:Color.popups.text;bordered:true;enabled:!!root.document.dirty && !root.overviewLoading && !keyEditor.pending;onClicked:root.perform({op:"pluginSave"}) }
                Button { text:"Verwerfen";foreground:Color.popups.text;bordered:true;enabled:!root.overviewLoading;onClicked:root.perform({op:"pluginDiscard",path:root.setting("configPath", "")}) }
                Button { text:"Bearbeitung schließen";visible:root.editVisible;foreground:Color.popups.text;bordered:true;enabled:!keyEditor.pending;onClicked:root.editVisible=false }
            }
            Button {
                objectName: "kanataRefreshButton"
                width: parent.width
                text: root.manualRefresh ? "Aktualisiert …" : "Aktualisieren"
                foreground: Color.popups.text
                bordered: true
                focusable: true
                enabled: !root.manualRefresh
                onClicked: root.updateStatus()
            }
            Button {
                objectName: "kanataStudioLaunchButton"
                width: parent.width
                text: "Studio öffnen"
                foreground: Color.popups.text
                bordered: true
                focusable: true
                onClicked: root.launchStudio()
            }
        }
        }
    }
    Process {
        id: statusProcess
        command: ["systemctl", "--user", "show", "kanata.service", "--property=ActiveState", "--value"]
        stdout: StdioCollector { id: statusOutput }
        onExited: function(exitCode) {
            statusTimeout.stop()
            root.serviceState = exitCode === 0 ? (statusOutput.text.trim() || "unknown") : "unavailable"
            if (root.manualRefresh) {
                root.manualRefresh = false
                root.feedback = exitCode === 0 ? "Status aktualisiert" : "Statusabfrage fehlgeschlagen"
                feedbackTimeout.restart()
                if (root.bar && button.tooltipHovered && !root.opened) root.bar.showTooltip(button, button.tooltipText)
            }
        }
        onStarted: statusTimeout.restart()
    }
    Timer {
        id: feedbackTimeout
        interval: 4000
        onTriggered: {
            root.feedback = ""
            if (root.bar && button.tooltipHovered && !root.opened) root.bar.showTooltip(button, button.tooltipText)
        }
    }
    Timer {
        id: statusTimeout
        interval: 3000
        onTriggered: {
            root.serviceState = "unavailable"
            statusProcess.running = false
        }
    }
    Timer { interval: 5000; running: true; repeat: true; onTriggered: root.refresh() }
    Component.onCompleted: refresh()
}
