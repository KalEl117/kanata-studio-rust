import QtQuick
import QtQuick.Shapes
import qs.Commons as Commons
import "KeyboardModel.js" as Keyboard

Column {
    id: root
    property var overview: ({})
    property int selectedLayer: 0
    property string hovered: ""
    property string selectedPhysicalKey: ""
    readonly property color changedColor: "#e5b46c"
    signal keyClicked(string name)
    readonly property bool laptop: Keyboard.isLaptop(overview.layoutInfo || {})
    readonly property real factor: 0.85
    function scaled(value) { return Commons.Style.spaceReal(value * factor) }
    function alpha(color, value) { return Qt.rgba(color.r, color.g, color.b, value) }
    readonly property var view: overview.views && overview.views[selectedLayer] ? overview.views[selectedLayer] : ({})
    readonly property var rows: Keyboard.buildRows(overview.layoutInfo || {})
    readonly property real gap: Commons.Style.space(3)
    readonly property var extraKeys: (view.keys || []).filter(function(key, index) {
        var canonical = view.canonicalKeys[index]
        return !rows.some(function(row) { return row.some(function(item) { return Keyboard.normalize(item[0]) === canonical }) })
    })
    spacing: gap
    objectName: "kanataMiniKeyboard"
    onSelectedLayerChanged: hovered = ""
    onOverviewChanged: hovered = ""

    function indexFor(name) { return (view.canonicalKeys || []).indexOf(Keyboard.normalize(name)) }
    function stateFor(name) {
        var index = indexFor(name)
        return index < 0 ? ({kind: "normal"}) : view.keyStates[index]
    }
    function outputFor(name) {
        var state = stateFor(name)
        if (state.kind === "normal") return ""
        if (state.lines && state.lines.length) return state.lines.map(function(line) { return line.text }).join(" / ")
        return state.output || ""
    }
    function describe(name, label) {
        var index = indexFor(name)
        if (index < 0) return label + " · Normale Funktion"
        var state = view.keyStates[index]
        var lines = state.lines && state.lines.length
            ? state.lines.map(function(line) { return line.label + ": " + line.text }).join(" · ")
            : state.output || label
        return label + " · " + state.label + "\n" + lines + "\n" + state.detail
    }

    Rectangle {
        visible: !!root.overview.layoutInfo && root.overview.layoutInfo.available !== false
        width: parent.width
        height: keyboardColumn.implicitHeight + root.scaled(40)
        radius: root.scaled(7)
        color: root.alpha(Commons.Color.popups.text, 0.025)
        border.color: root.alpha(Commons.Color.popups.text, 0.2)
        Column {
            id: keyboardColumn
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: root.scaled(20)
            spacing: root.scaled(7)
            property real keyUnitWidth: {
                var unit = 1000
                for (var r = 0; r < root.rows.length; ++r) {
                    var units = root.rows[r].reduce(function(total, key) { return total + key[2] }, 0)
                    var gaps = -root.scaled(5)
                    if (root.laptop && r === 0) gaps += root.scaled(28)
                    if (root.laptop && Keyboard.isIsoEnter(root.overview.layoutInfo || {}) && r === 3) units += 1.5
                    unit = Math.min(unit, (width - gaps) / units - root.scaled(5))
                }
                return Math.max(root.scaled(14), unit)
            }
            property real bottomExtra: {
                if (!root.laptop || root.rows.length < 6) return 0
                var row = root.rows[5]
                var raw = row.reduce(function(total, key) { return total + keyUnitWidth * key[2] + root.scaled(5) * (key[2] - 1) }, 0)
                raw += (row.length - 1) * root.scaled(5)
                return Math.max(0, (width - raw) / row.length)
            }
            Repeater {
                objectName: "kanataKeyboardRows"
                model: root.rows
                Row {
                    id: keyboardRow
                    required property var modelData
                    required property int index
                    width: keyboardColumn.width
                    property bool arrowRow: root.laptop && index === root.rows.length - 1
                    property bool stretch: root.laptop && [1, 2, 3, 4, 5].indexOf(index) >= 0
                    property real baseWidth: modelData.reduce(function(total, key) { return total + keyboardColumn.keyUnitWidth * key[2] + root.scaled(5) * (key[2] - 1) }, 0) + Math.max(0, modelData.length - 1) * spacing
                    property real arrowWidth: keyboardColumn.keyUnitWidth + keyboardColumn.bottomExtra
                    property real targetWidth: index === 3 && root.laptop && Keyboard.isIsoEnter(root.overview.layoutInfo || {}) ? width - keyboardColumn.keyUnitWidth * 1.5 - root.scaled(12) : width
                    property real extraPerKey: stretch ? Math.max(0, (targetWidth - baseWidth) / modelData.length) : 0
                    height: root.scaled(arrowRow ? 32 : root.laptop && index === 0 ? 41 : root.laptop && index === 5 ? 29 : 53)
                    spacing: root.scaled(5)
                    Item {
                        visible: keyboardRow.arrowRow
                        width: visible ? Math.max(0, keyboardRow.width - 3 * keyboardRow.arrowWidth - root.scaled(10)) : 0
                        height: 1
                    }
                    Repeater {
                        objectName: "kanataKeyboardTiles"
                        model: keyboardRow.modelData
                        Rectangle {
                            id: key
                            required property var modelData
                            required property int index
                            objectName: "keyTile_" + modelData[0]
                            readonly property var keyState: root.stateFor(modelData[0])
                            readonly property bool chosen: Keyboard.normalize(root.selectedPhysicalKey) === Keyboard.normalize(modelData[0])
                            readonly property bool isoEnterKey: modelData[0] === "Enter" && Keyboard.isIsoEnter(root.overview.layoutInfo || {})
                            readonly property bool laptopArrowKey: root.laptop && ["PageUp", "ArrowUp", "PageDown"].indexOf(modelData[0]) >= 0
                            readonly property bool spacer: modelData[0] === ""
                            readonly property bool unavailableFn: modelData[0] === "fn" && !(root.overview.layoutInfo.keyboard && root.overview.layoutInfo.keyboard.fnAvailable)
                            readonly property real groupGap: root.laptop && keyboardRow.index === 0 && [1,5,9,13].indexOf(index) >= 0 ? root.scaled(7) : 0
                            readonly property color tileFill: chosen ? root.alpha(Commons.Color.accent,0.16) : keyState.changed ? root.alpha(root.changedColor,0.10) : keyState.kind === "own" ? root.alpha(Commons.Color.accent,0.08) : root.alpha(Commons.Color.popups.text,0.045)
                            readonly property color tileBorder: chosen ? Commons.Color.accent : keyState.changed ? "#e5b46c" : keyState.kind === "own" ? root.alpha(Commons.Color.accent,0.55) : root.alpha(Commons.Color.popups.text,0.3)
                            width: keyboardRow.arrowRow ? keyboardRow.arrowWidth : modelData[2] * keyboardColumn.keyUnitWidth + root.scaled(5) * (modelData[2]-1) + keyboardRow.extraPerKey + groupGap
                            height: root.scaled((isoEnterKey || modelData[3] === 2) ? 113 : root.laptop && keyboardRow.index === 0 ? 38 : laptopArrowKey ? 32 : root.laptop && keyboardRow.index === 5 ? 53 : 53)
                            radius: root.scaled(5)
                            color: isoEnterKey ? "transparent" : tileFill
                            border.color: isoEnterKey ? "transparent" : tileBorder
                            opacity: spacer ? 0 : unavailableFn ? 0.58 : 1
                            Shape {
                                id: enterShape
                                anchors.fill: parent
                                visible: key.isoEnterKey
                                ShapePath {
                                    fillColor: key.tileFill
                                    strokeColor: key.tileBorder
                                    strokeWidth: 1
                                    startX: 0; startY: 0
                                    PathLine { x: enterShape.width; y: 0 }
                                    PathLine { x: enterShape.width; y: enterShape.height }
                                    PathLine { x: enterShape.width*0.24; y: enterShape.height }
                                    PathLine { x: enterShape.width*0.24; y: enterShape.height*0.49 }
                                    PathLine { x: 0; y: enterShape.height*0.49 }
                                    PathLine { x: 0; y: 0 }
                                }
                            }
                            Column {
                                anchors.horizontalCenter: parent.horizontalCenter
                                y: key.isoEnterKey ? root.scaled(7) : (key.height-height)/2
                                width: parent.width-root.scaled(6)
                                spacing: root.scaled(2)
                                Text {
                                    width: parent.width
                                    textFormat: Text.PlainText
                                    text: key.modelData[0] === "Enter" ? "Eingabe" : key.modelData[1]
                                    color: Commons.Color.popups.text
                                    font.family: Commons.Style.font.family
                                    font.pixelSize: Math.max(root.scaled(8),Math.min(root.scaled(13),parent.width/(Math.max(1,text.length)*0.62)))
                                    font.weight: Font.DemiBold
                                    horizontalAlignment: Text.AlignHCenter
                                    wrapMode: Text.WordWrap
                                    maximumLineCount: 2
                                }
                                Text {
                                    width: parent.width
                                    textFormat: Text.PlainText
                                    text: key.keyState.kind === "normal" ? "Normal" : (key.keyState.kind === "inherited" ? "↳ " : "") + (key.keyState.output || "")
                                    visible: !(key.keyState.lines && key.keyState.lines.length)
                                    color: key.chosen ? Commons.Color.accent : root.alpha(Commons.Color.popups.text,0.78)
                                    font.family: Commons.Style.font.family
                                    font.pixelSize: root.scaled(8)
                                    horizontalAlignment: Text.AlignHCenter
                                    wrapMode: Text.WordWrap
                                    maximumLineCount: 2
                                }
                                Repeater {
                                    model: key.keyState.lines || []
                                    Text {
                                        required property var modelData
                                        required property int index
                                        width: parent.width
                                        textFormat: Text.PlainText
                                        text: (index === 0 && key.keyState.kind === "inherited" ? "↳ " : "") + modelData.label + ": " + modelData.text
                                        color: key.chosen ? Commons.Color.accent : root.alpha(Commons.Color.popups.text,0.88)
                                        font.family: Commons.Style.font.family
                                        font.pixelSize: root.scaled(8)
                                        horizontalAlignment: Text.AlignHCenter
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: index === 0 || key.height < root.scaled(50) ? 1 : 2
                                        elide: Text.ElideRight
                                    }
                                }
                            }
                            Rectangle {
                                anchors.top: parent.top; anchors.right: parent.right; anchors.margins: 3
                                width: 5; height: 5; radius: 2.5
                                visible: !!key.keyState.changed
                                color: "#e5b46c"
                            }
                            MouseArea {
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: key.unavailableFn ? Qt.ArrowCursor : Qt.PointingHandCursor
                                onEntered: root.hovered = root.describe(key.modelData[0],key.modelData[1])
                                onClicked: if (!key.unavailableFn && !key.spacer) root.keyClicked(key.modelData[0])
                            }
                        }
                    }
                }
            }
        }
    }
    Flow {
        width: parent.width
        spacing: root.gap
        visible: root.extraKeys.length > 0
        Repeater {
            model: root.extraKeys
            Rectangle {
                required property string modelData
                width: Math.max(Commons.Style.space(70), extraLabel.implicitWidth + Commons.Style.space(10))
                height: Commons.Style.space(28)
                color: "transparent"
                border.color: Commons.Color.accent
                radius: Commons.Style.space(4)
                Text {
                    id: extraLabel
                    anchors.centerIn: parent
                    textFormat: Text.PlainText
                    text: modelData + " · " + root.outputFor(modelData)
                    color: Commons.Color.popups.text
                    font.family: Commons.Style.font.family
                    font.pixelSize: Commons.Style.space(10)
                }
                MouseArea { anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onEntered: root.hovered = root.describe(parent.modelData,parent.modelData); onClicked: root.keyClicked(parent.modelData) }
            }
        }
    }
    Text {
        width: parent.width
        height: Commons.Style.space(64)
        textFormat: Text.PlainText
        text: root.hovered || "Taste anklicken zum Bearbeiten · Darüberfahren für Details.\nAkzent: eigene Belegung · Grau: von Basis übernommen"
        color: Commons.Color.muted
        font.family: Commons.Style.font.family
        font.pixelSize: Commons.Style.font.caption
        wrapMode: Text.WordWrap
        elide: Text.ElideRight
        maximumLineCount: 4
    }
}
