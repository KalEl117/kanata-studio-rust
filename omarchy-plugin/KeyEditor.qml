import QtQuick
import qs.Ui
import qs.Commons

Column {
    id: root
    property var document: ({})
    property bool busy: false
    property string mode: "simple"
    property string initial: ""
    readonly property bool pending: signature() !== initial
    signal applyRequested(var payload)
    width: parent ? parent.width : 600
    spacing: Style.space(8)
    objectName: "kanataKeyEditor"

    function signature() { return JSON.stringify([mode, action.text, tap.text, hold.text, repeatTime.text, holdTime.text]) }
    function sync() {
        var details = document.details || {}
        mode = details.kind === "tap-hold" ? "tap-hold" : String(document.expression || "_").indexOf("(") >= 0 ? "code" : "simple"
        action.text = document.expression || "_"
        tap.text = details.tap || "esc"
        hold.text = details.hold || "lctl"
        repeatTime.text = String(details.repeat || 150)
        holdTime.text = String(details.holdMs || 200)
        initial = signature()
    }
    function apply() {
        if (mode === "tap-hold") {
            applyRequested({op:"pluginSetTapHold",head:(document.details || {}).head || "tap-hold",repeatMs:Number(repeatTime.text),holdMs:Number(holdTime.text),tap:tap.text,hold:hold.text})
        } else applyRequested({op:"pluginSetAction",expression:action.text})
    }
    onDocumentChanged: sync()

    Text {
        width: parent.width
        text: "Taste bearbeiten · " + (root.document.selectedPhysicalKey || "")
        color: Color.popups.text
        font.family: Style.font.family
        font.pixelSize: Style.font.subtitle
        font.bold: true
    }
    Flow {
        width: parent.width
        spacing: Style.space(6)
        Button { text:"Einfach"; selected:root.mode === "simple"; foreground:Color.popups.text; bordered:true; onClicked:root.mode="simple" }
        Button { text:"Tippen / Halten"; selected:root.mode === "tap-hold"; foreground:Color.popups.text; bordered:true; onClicked:root.mode="tap-hold" }
        Button { text:"Kanata-Code"; selected:root.mode === "code"; foreground:Color.popups.text; bordered:true; onClicked:root.mode="code" }
    }
    Column {
        width: parent.width
        spacing: Style.space(6)
        visible: root.mode !== "tap-hold"
        Text { text:root.mode === "code" ? "Kanata-Ausdruck" : "Ausgabe (Kanata-Tastenname)"; color:Color.muted; font.family:Style.font.family; font.pixelSize:Style.font.caption }
        TextField { id:action; objectName:"kanataActionInput"; width:parent.width; foreground:Color.popups.text; placeholderText:"esc, spc, lctl oder Kanata-Ausdruck"; onAccepted:root.apply() }
        Flow {
            visible: root.mode === "simple"
            width: parent.width
            spacing: Style.space(5)
            Repeater {
                model: [["Normal","_"],["Esc","esc"],["Enter","ret"],["Leertaste","spc"],["Strg","lctl"],["Alt","lalt"],["Super","lmeta"]]
                Button {
                    required property var modelData
                    text:modelData[0]; foreground:Color.popups.text; bordered:true
                    onClicked:action.text=modelData[1]
                }
            }
        }
    }
    Column {
        width: parent.width
        visible: root.mode === "tap-hold"
        spacing: Style.space(6)
        Text { text:"Tippausgabe"; color:Color.muted; font.family:Style.font.family; font.pixelSize:Style.font.caption }
        TextField { id:tap; objectName:"kanataTapInput"; width:parent.width; foreground:Color.popups.text }
        Text { text:"Halteaktion"; color:Color.muted; font.family:Style.font.family; font.pixelSize:Style.font.caption }
        TextField { id:hold; objectName:"kanataHoldInput"; width:parent.width; foreground:Color.popups.text }
        Row {
            width: parent.width
            spacing: Style.space(8)
            Column {
                width: (parent.width-parent.spacing)/2
                Text { text:"Tippzeit (ms)"; color:Color.muted; font.family:Style.font.family; font.pixelSize:Style.font.caption }
                TextField { id:repeatTime; width:parent.width; foreground:Color.popups.text; validator:IntValidator {bottom:1;top:65535} }
            }
            Column {
                width: (parent.width-parent.spacing)/2
                Text { text:"Haltezeit (ms)"; color:Color.muted; font.family:Style.font.family; font.pixelSize:Style.font.caption }
                TextField { id:holdTime; width:parent.width; foreground:Color.popups.text; validator:IntValidator {bottom:1;top:65535} }
            }
        }
    }
    Row {
        spacing: Style.space(8)
        Button { objectName:"kanataApplyEdit";text:"Im Entwurf übernehmen";foreground:Color.popups.text;bordered:true;enabled:root.pending && !root.busy;onClicked:root.apply() }
        Button { text:"Eingabe zurücksetzen";foreground:Color.popups.text;bordered:true;enabled:root.pending && !root.busy;onClicked:root.sync() }
    }
}
