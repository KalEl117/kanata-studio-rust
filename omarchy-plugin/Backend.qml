import QtQuick
import Quickshell.Io

// Reusable transport for the future in-shell editor. Requests run serially so
// every edit sees the source returned by its predecessor, including Undo/Redo.
Item {
    id: root
    property string executable: decodeURIComponent(Qt.resolvedUrl("bin/kanata-studio-helper").toString().replace(/^file:\/\//, ""))
    property bool ready: false
    property string error: ""
    property string source: ""
    property var view: ({})
    property bool canUndo: false
    property bool canRedo: false
    property int sequence: 0
    property var queue: []
    property var active: null
    signal replied(int requestId, var response)
    signal failed(int requestId, string message)
    signal sessionLost()

    function start() {
        if (process.running) return
        error = ""
        watchdog.restart()
        process.running = true
    }

    function request(payload) {
        if (!ready) {
            failed(-1, error || "Rust-Backend ist nicht bereit")
            return -1
        }
        var id = ++sequence
        var copy = JSON.parse(JSON.stringify(payload))
        copy.id = id
        queue.push(copy)
        pump()
        return id
    }

    function loadSource(text) {
        request({op: "clear", source: text})
        return request({op: "loadView", source: text})
    }

    function pump() {
        if (active !== null || queue.length === 0 || !ready) return
        var payload = queue.shift()
        if (payload.source === undefined) payload.source = source
        active = payload
        watchdog.restart()
        process.write(JSON.stringify(payload) + "\n")
    }

    function loseSession(message) {
        ready = false
        error = message
        watchdog.stop()
        if (active !== null) failed(active.id, message)
        active = null
        var pending = queue
        queue = []
        for (var i = 0; i < pending.length; ++i) failed(pending[i].id, message)
        canUndo = false
        canRedo = false
        sessionLost()
    }

    function receive(line) {
        var response
        try { response = JSON.parse(line) }
        catch (exception) {
            loseSession("Ungültige Backend-Antwort: " + exception)
            process.running = false
            return
        }
        if (active === null || response.id !== active.id) {
            loseSession("Backend-Antwort passt nicht zur Anfrage")
            process.running = false
            return
        }
        watchdog.stop()
        var id = active.id
        var handshake = active.op === "ping"
        active = null
        if (handshake) {
            if (!response.ok || response.protocolVersion !== 1) {
                loseSession("Inkompatibles Rust-Backend")
                process.running = false
                return
            }
            ready = true
        } else if (response.ok) {
            if (response.source !== undefined) source = response.source
            if (response.view) view = response.view
            if (response.canUndo !== undefined) canUndo = response.canUndo
            if (response.canRedo !== undefined) canRedo = response.canRedo
        }
        if (response.ok) {
            error = ""
            replied(id, response)
        } else {
            error = response.error || "Backend-Anfrage fehlgeschlagen"
            failed(id, error)
        }
        pump()
    }

    Process {
        id: process
        command: [root.executable, "--serve"]
        stdinEnabled: true
        stdout: SplitParser { onRead: data => root.receive(data) }
        stderr: SplitParser { onRead: data => console.warn("Kanata Studio: " + data) }
        onStarted: {
            root.active = {id: ++root.sequence, op: "ping"}
            watchdog.restart()
            process.write(JSON.stringify(root.active) + "\n")
        }
        onExited: function(exitCode) {
            root.loseSession(root.error || "Rust-Backend beendet (" + exitCode + ")")
        }
    }
    Timer {
        id: watchdog
        interval: 20000
        onTriggered: {
            root.loseSession("Zeitüberschreitung beim Rust-Backend")
            process.running = false
        }
    }
    Component.onCompleted: start()
}
