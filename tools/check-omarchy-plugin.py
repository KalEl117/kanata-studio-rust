#!/usr/bin/env python3
"""Headless test of the packaged QML bridge and widget; no live config changes."""
import os
import pathlib
import re
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
PLUGIN = ROOT / "dist/omarchy-plugin/zoltan.kanata-studio"

SMOKE = r'''
import QtQuick
import Quickshell
import QtTest
import "Plugin" as Plugin

ShellRoot {
    id: root
    property string stage: "connect"
    property string original: "(defsrc a b)\n(deflayer base a b)\n"
    property string changed: ""
    property bool widgetLoaded: false
    property bool testsReady: false
    function fail(message) { console.error("PLUGIN_SMOKE_FAILED: " + message); Qt.quit() }
    function finish() {
        if (stage === "done" && widgetLoaded) {
            if (testsReady) return
            testsReady = true
            Qt.callLater(function() {
                try { interactions.test_hover_and_right_click() }
                catch (error) { root.fail(String(error) + "\n" + error.stack) }
                Qt.quit()
            })
        }
    }
    Plugin.Backend {
        id: backend
        onReadyChanged: if (ready && root.stage === "connect") {
            root.stage = "load"
            loadSource(root.original)
        }
        onReplied: function(requestId, response) {
            if (root.stage === "load" && response.view) {
                root.stage = "edit"
                // Queue two dependent edits without waiting: source must be
                // taken when each request is sent, not when it is queued.
                request({op: "setAction", key: 0, expression: "esc"})
                request({op: "setAction", key: 1, expression: "tab"})
            } else if (root.stage === "edit" && response.source) {
                if (!canUndo) { root.fail("missing Undo"); return }
                root.stage = "edit2"
            } else if (root.stage === "edit2" && response.source) {
                if (source.indexOf("base esc tab") < 0) { root.fail("queued source lost"); return }
                root.changed = source
                root.stage = "undo"
                request({op: "undo"})
            } else if (root.stage === "undo") {
                if (source.indexOf("base esc b") < 0 || !canRedo) { root.fail("Undo failed"); return }
                root.stage = "redo"
                request({op: "redo"})
            } else if (root.stage === "redo") {
                if (source !== root.changed) { root.fail("Redo failed"); return }
                root.stage = "invalid"
                request({op: "unknown-operation"})
            } else if (root.stage === "recover") {
                root.stage = "done"
                root.finish()
            }
        }
        onFailed: function(requestId, message) {
            if (root.stage !== "invalid") { root.fail(message); return }
            root.stage = "recover"
            request({op: "ping"})
        }
    }
    QtObject {
        id: fakeBar
        property bool vertical: false
        property int barSize: 36
        property string fontFamily: "Iosevka Nerd Font"
        property color barForeground: "white"
        property color urgent: "red"
        property bool foregroundAnimationEnabled: false
        property string position: "left"
        property var activePopout: null
        function requestPopout(owner) { activePopout = owner }
        function releasePopout(owner) { if (activePopout === owner) activePopout = null }
        property bool validHover: false
        property string tooltip: ""
        function registerClickTarget(target) {}
        function unregisterClickTarget(target) {}
        function showTooltip(target, text) {
            // Same prerequisite as the actual Omarchy bar.
            validHover = target.tooltipHovered === true
            tooltip = validHover ? text : ""
        }
        function hideTooltip(target) { tooltip = "" }
    }
    FloatingWindow {
        id: testWindow
        visible: true
        implicitWidth: 100
        implicitHeight: 100
        Plugin.BarWidget {
            id: widget
            anchors.centerIn: parent
            width: implicitWidth
            height: implicitHeight
            bar: fakeBar
            settings: ({configPath: Quickshell.env("KANATA_PLUGIN_TEST_CONFIG"),draftDirectory:Quickshell.env("KANATA_PLUGIN_TEST_DRAFTS"),layoutInfo:{id:"de",choice:"auto",geometry:"iso",keyboard:{type:"built-in",fnAvailable:false}}})
            onServiceStateChanged: {
                root.widgetLoaded = serviceState !== "unknown"
                root.finish()
            }
        }
        TestCase {
            id: interactions
            name: "KanataWidgetInteractions"
            when: false
            function test_hover_and_right_click() {
                mouseMove(widget, widget.width / 2, widget.height / 2)
                tryCompare(fakeBar, "validHover", true)
                verify(fakeBar.tooltip.indexOf(widget.statusText) >= 0)
                mouseClick(widget, widget.width / 2, widget.height / 2, Qt.RightButton)
                tryVerify(function() { return widget.feedback !== "" })
                verify(fakeBar.tooltip.indexOf(widget.feedback) >= 0)
                mouseMove(widget, -20, -20)
                tryCompare(fakeBar, "tooltip", "")
                mouseClick(widget, widget.width / 2, widget.height / 2, Qt.LeftButton)
                compare(widget.opened, true)
                compare(fakeBar.activePopout, widget)
                tryVerify(function() { return widget.overview.views !== undefined })
                tryCompare(widget, "overviewLoading", false)
                compare(widget.overview.views.length, 2)
                var mini = findChild(widget, "kanataMiniKeyboard")
                verify(mini !== null)
                verify(mini.rows.length >= 6)
                compare(mini.stateFor("CapsLock").kind, "own")
                verify(mini.outputFor("CapsLock").indexOf("Esc") >= 0)
                verify(mini.describe("CapsLock", "Caps").indexOf("Tippen:") >= 0)
                var layerChoices = findChild(widget, "kanataLayerChoices")
                verify(layerChoices !== null)
                tryVerify(function() { return layerChoices.itemAt(1) !== null })
                var layerButton = layerChoices.itemAt(1)
                verify(layerButton !== null)
                layerButton.clicked()
                tryCompare(widget, "overviewLoading", false)
                compare(widget.previewLayer, 1, widget.overviewError)
                compare(mini.stateFor("CapsLock").kind, "inherited")
                verify(mini.describe("CapsLock", "Caps").indexOf("Basis") >= 0)
                layerChoices.itemAt(0).clicked()
                tryCompare(widget, "overviewLoading", false)
                compare(widget.previewLayer, 0)
                var rows = findChild(mini, "kanataKeyboardRows")
                verify(rows !== null)
                var homeTiles = findChild(rows.itemAt(3), "kanataKeyboardTiles")
                var topTiles = findChild(rows.itemAt(2), "kanataKeyboardTiles")
                verify(homeTiles !== null && topTiles !== null)
                compare(homeTiles.itemAt(0).x, 0)
                var enter = topTiles.itemAt(topTiles.count - 1)
                verify(enter.isoEnterKey && enter.height > rows.itemAt(2).height)
                // Real click on a tile opens the editable Tap-Hold form.
                mouseClick(homeTiles.itemAt(0), 10, 10, Qt.LeftButton)
                tryCompare(widget, "overviewLoading", false)
                compare(widget.editVisible, true)
                var editor = findChild(widget, "kanataKeyEditor")
                compare(editor.mode, "tap-hold")
                var tapInput = findChild(editor, "kanataTapInput")
                tapInput.text = "tab"
                verify(editor.pending)
                findChild(editor, "kanataApplyEdit").clicked()
                tryCompare(widget, "overviewLoading", false)
                verify(widget.document.dirty)
                verify(mini.outputFor("CapsLock").indexOf("Tab") >= 0)
                verify(mini.stateFor("CapsLock").changed)
                widget.perform({op:"pluginUndo"})
                tryCompare(widget, "overviewLoading", false)
                verify(!widget.document.dirty)
                widget.perform({op:"pluginRedo"})
                tryCompare(widget, "overviewLoading", false)
                verify(widget.document.dirty)
                widget.perform({op:"pluginDiscard",path:Quickshell.env("KANATA_PLUGIN_TEST_CONFIG")})
                tryCompare(widget, "overviewLoading", false)
                verify(!widget.document.dirty)
                var refreshButton = findChild(widget, "kanataRefreshButton")
                var launchButton = findChild(widget, "kanataStudioLaunchButton")
                verify(refreshButton !== null && launchButton !== null)
                compare(launchButton.text, "Studio öffnen")
                widget.feedback = ""
                refreshButton.clicked()
                tryVerify(function() { return widget.feedback !== "" })
                mouseClick(widget, widget.width / 2, widget.height / 2, Qt.LeftButton)
                compare(widget.opened, false)
                compare(fakeBar.activePopout, null)
                widget.open()
                compare(widget.opened, true)
                widget.close()
                compare(widget.opened, false)
                console.log("PLUGIN_SMOKE_OK")
            }
        }
    }
    Timer { interval: 10000; running: true; onTriggered: root.fail("timeout in " + root.stage) }
}
'''


def main():
    subprocess.run(["omarchy", "plugin", "validate", str(PLUGIN)], check=True)
    with tempfile.TemporaryDirectory(prefix="kanata-plugin-check-") as folder:
        path = pathlib.Path(folder)
        (path / "Plugin").symlink_to(PLUGIN, target_is_directory=True)
        for name in ("Ui", "Commons"):
            (path / name).symlink_to(pathlib.Path("/usr/share/omarchy/shell") / name, target_is_directory=True)
        (path / "shell.qml").write_text(SMOKE)
        fixture = path / "preview.kbd"
        saved = "(defsrc caps a Semicolon)\n(deflayer base (tap-hold 150 200 esc lctl) a b)\n(deflayer nav _ left _)\n"
        fixture.write_text(saved)
        env = dict(os.environ, QT_QPA_PLATFORM="offscreen", QT_QPA_PLATFORMTHEME="", QSG_RHI_BACKEND="software")
        env["KANATA_PLUGIN_TEST_CONFIG"] = str(fixture)
        env["KANATA_PLUGIN_TEST_DRAFTS"] = str(path / "drafts")
        result = subprocess.run(["quickshell", "--no-duplicate", "-p", str(path / "shell.qml")],
                                env=env, capture_output=True, text=True, timeout=15)
        output = result.stdout + result.stderr
        if fixture.read_text() != saved:
            raise SystemExit("Die Übersicht hat die Testkonfiguration verändert")
        if (result.returncode or "PLUGIN_SMOKE_OK" not in output
                or "PLUGIN_SMOKE_FAILED" in output
                or re.search(r"ReferenceError:|TypeError:|Failed to load configuration", output)):
            raise SystemExit(output)
        print("Plugin-Smoke-Test bestanden: Studio-Geometrie, ISO-Enter, Tastenklick, Tap-Hold-Bearbeitung, Undo/Redo und unveränderte aktive Datei.")


if __name__ == "__main__":
    main()
