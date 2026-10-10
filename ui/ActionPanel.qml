import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    property var backend
    property var translateText
    property color backgroundColor
    property color surfaceColor
    property color foregroundColor
    property color mutedColor
    property color accentColor
    property var outputKeyRows: []
    // Positions in standard key units, matching the Cherry T2 prototype photo.
    // Main block: 15 units; navigation: 3; number pad: 4; cluster gaps: 0.5.
    property var fullsizeGeometry: {
        var result = []
        function add(key, x, y, w, h, symbols, isoEnter) {
            result.push({key: key, x: x, y: y, w: w || 1, h: h || 1,
                         symbols: !!symbols, isoEnter: !!isoEnter})
        }
        add(["Escape", "Esc"], 0, 0)
        for (var f = 1; f <= 12; ++f)
            add(["F" + f, "F" + f], 2 + (f - 1) + Math.floor((f - 1) / 4) * 0.5, 0)
        add(["PrintScreen", "Druck"], 15.5, 0)
        add(["ScrollLock", "Rollen"], 16.5, 0)
        add(["Pause", "Pause"], 17.5, 0)

        var rows = outputKeyRows.slice(1).map(function(row) {
            return row.map(function(key) { return key.slice() })
        })
        var ansi = rows.length > 1 && rows[1].some(function(key) { return key[0] === "Backslash" })
        for (var r = 0; r < rows.length; ++r) {
            var row = rows[r], x = 0
            if (r === 4) {
                row.splice(6, 0, ["ContextMenu", "Menü", 1.25])
                row.forEach(function(key) { key[2] = key[0] === "Space" ? 6.25 : 1.25 })
            }
            if (ansi && r === 1) row = row.filter(function(key) { return key[0] !== "Enter" })
            if (ansi && r === 2) row.push(["Enter", "Eingabe", 2.25])
            for (var k = 0; k < row.length; ++k) {
                var key = row[k], id = key[0], w = 1
                if (id === "Backspace") w = 2
                else if (id === "Tab") w = 1.5
                else if (id === "CapsLock") w = 1.75
                else if (id === "ShiftLeft") w = ansi ? 2.25 : 1.25
                else if (id === "ShiftRight") w = 2.75
                else if (id === "Enter") w = ansi ? 2.25 : 1.5
                else if (ansi && r === 1 && id === "Backslash") w = 1.5
                else if (r === 4) w = key[2]
                var symbols = /^[a-z0-9]$/.test(id) || ["Backquote", "Minus", "Equal", "BracketLeft", "BracketRight", "Semicolon", "Quote", "Backslash", "IntlBackslash", "Comma", "Period", "Slash"].indexOf(id) >= 0
                add(key, x, r + 2, w, id === "Enter" && !ansi ? 2 : 1, symbols, id === "Enter" && !ansi)
                x += w
            }
        }
        var nav = [["Insert", "Einfg"], ["Home", "Pos 1"], ["PageUp", "Bild ↑"],
                   ["Delete", "Entf"], ["End", "Ende"], ["PageDown", "Bild ↓"]]
        for (var n = 0; n < nav.length; ++n) add(nav[n], 15.5 + n % 3, 2 + Math.floor(n / 3))
        add(["ArrowUp", "↑"], 16.5, 5)
        add(["ArrowLeft", "←"], 15.5, 6)
        add(["ArrowDown", "↓"], 16.5, 6)
        add(["ArrowRight", "→"], 17.5, 6)
        var pad = [["nlck","Num",0,0], ["kp/","/",1,0], ["kp*","×",2,0], ["kp-","−",3,0],
                   ["kp7","7",0,1], ["kp8","8",1,1], ["kp9","9",2,1], ["kp+","+",3,1,1,2],
                   ["kp4","4",0,2], ["kp5","5",1,2], ["kp6","6",2,2],
                   ["kp1","1",0,3], ["kp2","2",1,3], ["kp3","3",2,3], ["kprt","↵",3,3,1,2],
                   ["kp0","0",0,4,2], ["kp.",activeLayout() === "de" ? "," : ".",2,4]]
        pad.forEach(function(key) { add([key[0], key[1]], 19 + key[2], 2 + key[3], key[4], key[5]) })
        return result
    }
    property int view: 0
    property bool actionAdvanced: false
    property bool tapAdvanced: false
    property bool macroAdvanced: false
    property bool formDirty: false
    property bool pendingEdit: formDirty || rawArea.text.trim() !== backend.action
    property string pickerTarget: "tap"
    property string pickerPage: "keyboard"
    property string pickerLevel: "normal"
    color: backgroundColor
    function tr(source) { return translateText ? translateText(source) : source }
    function modifierLabel(token) {
        var names = {lctl:"Strg links", lalt:"Alt links", lshft:"Umschalt links", lmeta:"Super links", ralt:"AltGr", rctl:"Strg rechts", rshift:"Shift rechts", rmeta:"Super rechts"}
        return tr(names[token] || token)
    }

    readonly property var macroVariants: [
        "macro", "macro-release-cancel", "macro-cancel-on-press",
        "macro-release-cancel-and-cancel-on-press", "macro-repeat",
        "macro-repeat-release-cancel", "macro-repeat-cancel-on-press",
        "macro-repeat-release-cancel-and-cancel-on-press"
    ]
    readonly property var tapVariants: ["tap-hold", "tap-hold-press", "tap-hold-release"]
    readonly property var modifiers: ["lctl", "lalt", "lshft", "lmeta", "ralt", "rctl", "rshift", "rmeta"]
    readonly property var actionChoices: [
        {id:"layer-hold", label:"Ebene halten", detail:"Ebene bleibt aktiv, solange du die Taste hältst.", group:"Ebenen"},
        {id:"layer-switch", label:"Dauerhaft wechseln", detail:"Wechselt bis zum nächsten Ebenenwechsel.", group:"Ebenen"},
        {id:"layer-once", label:"Ebene einmalig", detail:"Nutzt die Ebene für die nächste Taste.", group:"Ebenen"},
        {id:"modifier-once", label:"Zusatztaste einmalig", detail:"Wendet einen Modifier auf die nächste Taste an.", group:"Tastenverhalten"},
        {id:"caps-word", label:"Wort großschreiben", detail:"Schreibt das nächste Wort groß.", group:"Tastenverhalten"},
        {id:"repeat-key", label:"Letzte Taste wiederholen", detail:"Gibt die letzte Taste erneut aus.", group:"Tastenverhalten"},
        {id:"repeat-action", label:"Letzte Aktion wiederholen", detail:"Wiederholt die letzte Tastenkombination.", group:"Tastenverhalten"},
        {id:"tap-hold", label:"Tippen oder halten…", detail:"Öffnet den Tap Hold Editor.", group:"Editoren"},
        {id:"macro", label:"Makro bearbeiten…", detail:"Öffnet den Makro Editor.", group:"Editoren"}
    ]
    property int actionTargetLayer: 0
    property int actionModifier: 0
    readonly property var symbolSets: ({
        de: [["!","lshft","1"],["\"","lshft","2"],["§","lshft","3"],["$","lshft","4"],["%","lshft","5"],["&","lshft","6"],["/","lshft","7"],["(","lshft","8"],[")","lshft","9"],["=","lshft","0"],["?","lshft","minus"],["*","lshft","rightbracket"],["'","lshft","backslash"],[";","lshft","comma"],[":","lshft","dot"],["_","lshft","slash"],["<","","intlbackslash"],[">","lshft","intlbackslash"],["#","","backslash"],["+","","rightbracket"],["-","","slash"],["ä","","apostrophe"],["ö","","semicolon"],["ü","","leftbracket"],["ß","","minus"],["Ä","lshft","apostrophe"],["Ö","lshft","semicolon"],["Ü","lshft","leftbracket"],["@","ralt","q"],["€","ralt","e"],["{","ralt","7"],["[","ralt","8"],["]","ralt","9"],["}","ralt","0"],["\\","ralt","minus"],["|","ralt","intlbackslash"],["~","ralt","rightbracket"]],
        us: [["!","lshft","1"],["@","lshft","2"],["#","lshft","3"],["$","lshft","4"],["%","lshft","5"],["^","lshft","6"],["&","lshft","7"],["*","lshft","8"],["(","lshft","9"],[")","lshft","0"],["_","lshft","minus"],["+","lshft","equal"],["{","lshft","leftbracket"],["}","lshft","rightbracket"],["|","lshft","backslash"],[":","lshft","semicolon"],["\"","lshft","apostrophe"],["<","lshft","comma"],[">","lshft","dot"],["?","lshft","slash"],["~","lshft","grv"],["-","","minus"],["=","","equal"],["[","","leftbracket"],["]","","rightbracket"],["\\","","backslash"],[";","","semicolon"],["'","","apostrophe"],[",","","comma"],[".","","dot"],["/","","slash"],["`","","grv"]],
        gb: [["!","lshft","1"],["\"","lshft","2"],["£","lshft","3"],["$","lshft","4"],["%","lshft","5"],["^","lshft","6"],["&","lshft","7"],["*","lshft","8"],["(","lshft","9"],[")","lshft","0"],["_","lshft","minus"],["+","lshft","equal"],["{","lshft","leftbracket"],["}","lshft","rightbracket"],["@","lshft","apostrophe"],["~","lshft","backslash"],["|","lshft","intlbackslash"],["?","lshft","slash"],["#","","backslash"],["\\","","intlbackslash"],["€","ralt","4"],[":","lshft","semicolon"],["<","lshft","comma"],[">","lshft","dot"]]
    })
    readonly property var t2AltGrSymbols: [
        ["×","grv"],["’","1"],["²","2"],["³","3"],["—","4"],["¡","5"],["¿","6"],["˙","equal"],
        ["ˇ","w"],["¨","r"],["¯","t"],["˝","z"],["˘","u"],["~","i"],["˚","o"],["̉","p"],["̛","leftbracket"],
        ["⌀","d"],["′","f"],["″","g"],["ẞ","h"],["¸","j"],["̦","k"],["̨","l"],["̣","semicolon"],["̵","apostrophe"],
        ["›","y"],["»","x"],["©","c"],["«","v"],["‹","b"],["–","n"],["µ","m"],["ʻ","comma"],["‌","dot"],["­","slash"]
    ]
    readonly property var t2ShiftAltGrSymbols: [
        ["Œ","e"],["ı","i"],["Ø","o"],["Þ","p"],["ſ","leftbracket"],["Æ","a"],["Ð","d"],["Ł","l"],
        ["Ə","apostrophe"],["…","comma"],["·","dot"],["‑","slash"]
    ]

    function activeLayout() {
        var choice = backend.layoutInfo.choice || "auto"
        return ["de", "us", "gb"].indexOf(choice) >= 0 ? choice : (backend.layoutInfo.id || "de")
    }
    function keyToken(id) {
        var names = {"Escape":"esc", "Backquote":"grv", "Backspace":"bspc", "Tab":"tab", "Enter":"ret", "CapsLock":"caps", "ShiftLeft":"lsft", "ShiftRight":"rsft", "ControlLeft":"lctl", "ControlRight":"rctl", "MetaLeft":"lmeta", "MetaRight":"rmeta", "AltLeft":"lalt", "AltRight":"ralt", "Space":"spc", "ContextMenu":"menu", "BracketLeft":"leftbracket", "BracketRight":"rightbracket", "Semicolon":"semicolon", "Quote":"apostrophe", "Backslash":"backslash", "IntlBackslash":"intlbackslash", "Comma":"comma", "Period":"dot", "Slash":"slash", "Minus":"minus", "Equal":"equal", "PageUp":"pgup", "PageDown":"pgdn", "ArrowLeft":"lft", "ArrowRight":"rght", "ArrowUp":"up", "ArrowDown":"down", "Insert":"ins", "Delete":"del", "Home":"home", "End":"end", "PrintScreen":"ssrq", "ScrollLock":"slck"}
        var position = names[id] || String(id).toLowerCase()
        var kanataNames = {minus:"Minus", equal:"Equal", leftbracket:"BracketLeft", rightbracket:"BracketRight", semicolon:"Semicolon", apostrophe:"Quote", backslash:"Backslash", intlbackslash:"IntlBackslash", comma:"Comma", dot:"Period", slash:"Slash"}
        return kanataNames[position] || position
    }
    function symbolActionFor(token, level) {
        var actionToken = keyToken(token)
        var lookupNames = {bracketleft:"leftbracket", bracketright:"rightbracket", quote:"apostrophe", period:"dot"}
        token = String(token).toLowerCase()
        token = lookupNames[token] || token
        if (level === "normal") return {label: token, action: actionToken}
        if (level === "shift" && activeLayout() === "de" && token === "grv") return {label: "°", action: "(multi lshft grv)"}
        if (level === "shift" && activeLayout() === "de" && token === "equal") return {label: "`", action: "(multi lshft Equal)"}
        if (level === "shift-altgr") {
            if (activeLayout() !== "de") return {label: "", action: "", unavailable: true}
            for (var t = 0; t < t2ShiftAltGrSymbols.length; ++t)
                if (t2ShiftAltGrSymbols[t][1] === token) return {label: t2ShiftAltGrSymbols[t][0], action: "(multi lshft ralt " + actionToken + ")"}
            return {label: "", action: "", unavailable: true}
        }
        var modifier = level === "shift" ? "lshft" : "ralt"
        if (level === "shift" && /^[a-z]$/.test(token)) return {label: token.toUpperCase(), action: "(multi lshft " + actionToken + ")"}
        if (level === "altgr" && activeLayout() === "de") {
            for (var d = 0; d < t2AltGrSymbols.length; ++d)
                if (t2AltGrSymbols[d][1] === token) return {label: t2AltGrSymbols[d][0], action: "(multi ralt " + actionToken + ")"}
        }
        var options = symbolSets[activeLayout()] || symbolSets.de
        for (var i = 0; i < options.length; ++i) {
            if (options[i][1] === modifier && options[i][2] === token)
                return {label: options[i][0], action: "(multi " + modifier + " " + actionToken + ")"}
        }
        return {label: "", action: "", unavailable: true}
    }
    function keyLegend(key, level) {
        var token = keyToken(key[0])
        if (level === "normal") return /^[a-z]$/.test(key[0]) ? key[0] : tr(key[0] === "Enter" ? "Eing." : key[1])
        return symbolActionFor(token, level).label
    }
    function pickerLegend(key, level) {
        var label
        if (level === "normal") {
            label = key[1]
            if (/^[ÜÖÄ]$/.test(label)) label = label.toLowerCase()
        } else {
            if (level === "shift" && /^[a-z]$/.test(key[0])) return ""
            label = keyLegend(key, level)
        }
        // Combining accents need a base glyph to stay inside their own label.
        if (/^[\u0300-\u036f]+$/.test(label)) return "\u25cc" + label
        if (label === "\u200c") return "ZWNJ"
        if (label === "\u00ad") return "SHY"
        return label
    }
    function openKeyPicker(target) {
        pickerTarget = target
        pickerPage = "fullsize"
        pickerLevel = "normal"
        chordCtrl.checked = false
        chordAlt.checked = false
        chordShift.checked = false
        chordMeta.checked = false
        chordAltGr.checked = false
        keyPicker.open()
    }
    function pickedOutput(action) {
        var key = action.trim(), modifiers = []
        var combo = key.match(/^\(multi ((?:(?:lctl|lalt|lshft|lmeta|ralt) )+)([^\s()]+)\)$/)
        if (combo) { modifiers = combo[1].trim().split(/\s+/); key = combo[2] }
        if (chordCtrl.checked) modifiers.push("lctl")
        if (chordAlt.checked) modifiers.push("lalt")
        if (chordShift.checked) modifiers.push("lshft")
        if (chordMeta.checked) modifiers.push("lmeta")
        if (chordAltGr.checked) modifiers.push("ralt")
        if (modifiers.indexOf("lalt") >= 0 && modifiers.indexOf("ralt") >= 0) {
            backend.report(root.tr("Alt und AltGr können nicht gemeinsam gesetzt werden"))
            return ""
        }
        var ordered = ["lctl", "lalt", "lshft", "lmeta", "ralt"].filter(function(value) { return modifiers.indexOf(value) >= 0 })
        return ordered.length ? "(multi " + ordered.join(" ") + " " + key + ")" : action
    }
    function addPickedKey(action) {
        action = pickedOutput(action)
        if (!action) return
        if (pickerTarget === "chord") {
            var chordKey = action.trim(), chordModifiers = []
            var outputChord = chordKey.match(/^\(multi ((?:(?:lctl|lalt|lshft|lmeta|ralt) )+)([^\s()]+)\)$/)
            if (outputChord) {
                if (outputChord[1].indexOf("lctl") >= 0) chordModifiers.push("C")
                if (outputChord[1].indexOf("lalt") >= 0) chordModifiers.push("A")
                if (outputChord[1].indexOf("lmeta") >= 0) chordModifiers.push("M")
                if (outputChord[1].indexOf("lshft") >= 0) chordModifiers.push("S")
                if (outputChord[1].indexOf("ralt") >= 0) chordModifiers.push("AG")
                chordKey = outputChord[2]
            }
            if (!/^[^\s()]+$/.test(chordKey)) { backend.report(root.tr("Diese Taste kann nicht als Tastenkombination verwendet werden")); return }
            if (chordCtrl.checked) chordModifiers.push("C")
            if (chordAlt.checked) chordModifiers.push("A")
            if (chordShift.checked) chordModifiers.push("S")
            if (chordMeta.checked) chordModifiers.push("M")
            if (chordAltGr.checked) chordModifiers.push("AG")
            if (chordModifiers.indexOf("A") >= 0 && chordModifiers.indexOf("AG") >= 0) { backend.report(root.tr("Alt und AltGr können nicht gemeinsam gesetzt werden")); return }
            var chordPrefix = ["C", "A", "S", "M", "AG"].filter(function(value) { return chordModifiers.indexOf(value) >= 0 }).map(function(value) { return value + "-" }).join("")
            if (backend.setAction(chordPrefix + chordKey)) {
                sync()
                keyPicker.close()
            }
            return
        }
        if (pickerTarget === "output") {
            if (backend.setAction(action)) { sync(); keyPicker.close() }
            return
        }
        if (pickerTarget === "tap") {
            tapField.text = action
            formDirty = true
            keyPicker.close()
            return
        }
        var key = action.trim(), prefixes = []
        var combo = key.match(/^\(multi ((?:(?:lctl|lalt|lshft|lmeta|ralt) )+)([^\s()]+)\)$/)
        if (combo) {
            if (combo[1].indexOf("lctl") >= 0) prefixes.push("C")
            if (combo[1].indexOf("lalt") >= 0) prefixes.push("A")
            if (combo[1].indexOf("lmeta") >= 0) prefixes.push("M")
            if (combo[1].indexOf("lshft") >= 0) prefixes.push("S")
            if (combo[1].indexOf("ralt") >= 0) prefixes.push("AG")
            key = combo[2]
        }
        if (!/^[^\s()]+$/.test(key)) { backend.report(root.tr("Diese Ausgabe kann nicht als Makroschritt verwendet werden")); return }
        if (/^\d$/.test(key)) key = "Digit" + key
        if (ctrlMod.checked) prefixes.push("C")
        if (altMod.checked) prefixes.push("A")
        if (shiftMod.checked) prefixes.push("S")
        if (metaMod.checked) prefixes.push("M")
        if (altGrMod.checked) prefixes.push("AG")
        prefixes = prefixes.filter(function(value, index) { return prefixes.indexOf(value) === index })
        if (prefixes.indexOf("A") >= 0 && prefixes.indexOf("AG") >= 0) { backend.report(root.tr("Alt und AltGr können nicht gemeinsam gesetzt werden")); return }
        var order = ["C", "A", "S", "M", "AG"]
        var prefix = order.filter(function(value) { return prefixes.indexOf(value) >= 0 }).map(function(value) { return value + "-" }).join("")
        macroSteps.append({"step": prefix + key})
        macroKey.text = ""
        formDirty = true
        keyPicker.close()
    }
    function chooseAction(id) {
        var layer = backend.layers[actionTargetLayer] || ""
        var modifier = modifiers[actionModifier] || "lshft"
        var action = ""
        if (id === "layer-hold") action = "(layer-while-held " + layer + ")"
        else if (id === "layer-switch") action = "(layer-switch " + layer + ")"
        else if (id === "layer-once") action = "(one-shot 1000 (layer-while-held " + layer + "))"
        else if (id === "modifier-once") action = "(one-shot 1000 " + modifier + ")"
        else if (id === "caps-word") action = "(caps-word-toggle 2000)"
        else if (id === "repeat-key") action = "rpt"
        else if (id === "repeat-action") action = "rpt-any"
        else if (id === "tap-hold") {
            if (!switchView(1)) return
            tapField.text = backend.action === "_" ? keyToken(backend.selectedPhysicalKey) : backend.action
            formDirty = true
            actionChooser.close()
            return
        }
        else if (id === "macro") {
            if (!switchView(2)) return
            var initial = backend.action === "_" ? keyToken(backend.selectedPhysicalKey) : backend.action
            macroSteps.clear(); macroSteps.append({"step": initial}); formDirty = true
            actionChooser.close()
            return
        }
        if (action && backend.setAction(action)) { sync(); actionChooser.close() }
    }
    function showPickedSymbol(option) {
        return option[1] ? {label: option[0], action: "(multi " + option[1] + " " + keyToken(option[2]) + ")"} : {label: option[0], action: keyToken(option[2])}
    }

    function populate() {
        var details = backend.actionDetails
        rawArea.text = backend.action
        tapHead.currentIndex = Math.max(0, tapVariants.indexOf(details.head || "tap-hold"))
        repeatField.text = String(details.kind === "tap-hold" ? details.repeat : 150)
        holdField.text = String(details.kind === "tap-hold" ? details.holdMs : 200)
        tapField.text = details.kind === "tap-hold" ? details.tap : (details.kind === "simple" ? details.value : "spc")
        holdMode.currentIndex = details.holdKind === "modifiers" ? 1 : details.holdKind === "layer" ? 2 : details.holdKind === "action" ? 3 : 0
        modifierBox.currentIndex = Math.max(0, modifiers.indexOf(details.holdValue || "lctl"))
        layerBox.currentIndex = Math.max(0, backend.layers.indexOf(details.holdValue || ""))
        customHold.text = details.holdKind === "action" ? details.holdValue : "(multi lctl lshft)"
        var selectedModifiers = details.holdModifiers || []
        holdCtrl.checked = selectedModifiers.indexOf("lctl") >= 0
        holdAlt.checked = selectedModifiers.indexOf("lalt") >= 0
        holdShift.checked = selectedModifiers.indexOf("lshft") >= 0
        holdSuper.checked = selectedModifiers.indexOf("lmeta") >= 0
        holdAltGr.checked = selectedModifiers.indexOf("ralt") >= 0
        holdRCtrl.checked = selectedModifiers.indexOf("rctl") >= 0
        holdRShift.checked = selectedModifiers.indexOf("rshift") >= 0
        holdRSuper.checked = selectedModifiers.indexOf("rmeta") >= 0
        macroVariant.currentIndex = Math.max(0, macroVariants.indexOf(details.kind === "macro" ? details.head : "macro"))
        macroSteps.clear()
        if (details.kind === "macro") {
            for (var i = 0; i < details.steps.length; ++i) macroSteps.append({"step": details.steps[i]})
        } else macroSteps.append({"step": "spc"})
        formDirty = false
    }

    function sync() {
        populate()
        var kind = backend.actionDetails.kind
        view = kind === "tap-hold" ? 1 : kind === "macro" ? 2 : 0
    }

    function commit() {
        if (backend.selectedPhysicalKey.length === 0) return true
        var ok = true
        if (view === 1 && formDirty) {
            var hold = modifiers[modifierBox.currentIndex] || "lctl"
            if (holdMode.currentIndex === 1) {
                var selected = []
                if (holdCtrl.checked) selected.push("lctl")
                if (holdAlt.checked) selected.push("lalt")
                if (holdShift.checked) selected.push("lshft")
                if (holdSuper.checked) selected.push("lmeta")
                if (holdAltGr.checked) selected.push("ralt")
                if (holdRCtrl.checked) selected.push("rctl")
                if (holdRShift.checked) selected.push("rshift")
                if (holdRSuper.checked) selected.push("rmeta")
                if (selected.length === 0) { backend.report(root.tr("Bitte mindestens einen Modifier auswählen")); return false }
                hold = "(multi " + selected.join(" ") + ")"
            } else if (holdMode.currentIndex === 2) hold = "(layer-while-held " + layerBox.currentText + ")"
            else if (holdMode.currentIndex === 3) hold = customHold.text.trim()
            ok = backend.setTapHold(tapHead.currentText, Number(repeatField.text), Number(holdField.text), tapField.text, hold)
        } else if (view === 2 && formDirty) {
            var steps = []
            for (var i = 0; i < macroSteps.count; ++i) steps.push(macroSteps.get(i).step)
            ok = backend.setMacro(macroVariant.currentText, steps)
        } else if (view === 0 && rawArea.text.trim() !== backend.action) {
            ok = backend.setAction(rawArea.text)
        }
        if (ok) populate()
        return ok
    }

    function switchView(index) {
        if (!commit()) return
        view = index
        populate()
        return true
    }

    component PanelButton: Button {
        id: control
        property bool emphasized: false
        Layout.fillWidth: true
        implicitHeight: 34
        font.pixelSize: 12
        font.weight: Font.DemiBold
        palette.buttonText: emphasized ? root.backgroundColor : root.foregroundColor
        background: Rectangle {
            radius: 5
            color: control.emphasized ? root.accentColor : (control.hovered ? Qt.alpha(root.foregroundColor, 0.09) : Qt.alpha(root.foregroundColor, 0.045))
            border.color: control.emphasized ? root.accentColor : Qt.alpha(root.foregroundColor, 0.26)
        }
    }
    component PanelField: TextField {
        color: root.foregroundColor
        font.pixelSize: 13
        selectByMouse: true
        background: Rectangle {
            radius: 7
            color: root.backgroundColor
            border.color: parent.activeFocus ? root.accentColor : Qt.alpha(root.foregroundColor, 0.25)
        }
    }
    component Caption: Text {
        color: root.mutedColor
        font.pixelSize: 11
        font.weight: Font.Bold
        font.letterSpacing: 1.2
    }
    component PanelCombo: ComboBox {
        id: control
        implicitHeight: 36
        font.pixelSize: 12
        contentItem: Text {
            leftPadding: 10
            rightPadding: 25
            text: control.displayText
            color: root.foregroundColor
            font: control.font
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        indicator: Text {
            x: control.width - width - 10
            y: (control.height - height) / 2
            text: "⌄"
            color: root.mutedColor
            font.pixelSize: 18
        }
        background: Rectangle {
            radius: 7
            color: root.backgroundColor
            border.color: control.activeFocus ? root.accentColor : Qt.alpha(root.foregroundColor, 0.25)
        }
        delegate: ItemDelegate {
            width: control.width
            text: modelData
            font.pixelSize: 11
            palette.buttonText: root.foregroundColor
            background: Rectangle { color: highlighted ? Qt.alpha(root.accentColor, 0.22) : root.surfaceColor }
        }
        popup: Popup {
            y: control.height
            width: control.width
            implicitHeight: Math.min(contentItem.implicitHeight + 4, 250)
            padding: 2
            background: Rectangle { color: root.surfaceColor; radius: 5; border.color: Qt.alpha(root.foregroundColor, 0.25) }
            contentItem: ListView {
                clip: true
                implicitHeight: contentHeight
                model: control.popup.visible ? control.delegateModel : null
                currentIndex: control.highlightedIndex
                ScrollIndicator.vertical: ScrollIndicator {}
            }
        }
    }
    component ModCheck: CheckBox {
        id: control
        font.pixelSize: 11
        spacing: 5
        palette.windowText: root.foregroundColor
        indicator: Rectangle {
            implicitWidth: 17
            implicitHeight: 17
            x: control.leftPadding
            y: (control.height - height) / 2
            radius: 4
            color: control.checked ? root.accentColor : root.backgroundColor
            border.color: control.checked ? root.accentColor : Qt.alpha(root.foregroundColor, 0.28)
            Text { anchors.centerIn: parent; text: "✓"; visible: control.checked; color: root.backgroundColor; font.pixelSize: 12; font.weight: Font.Bold }
        }
        contentItem: Text {
            leftPadding: control.indicator.width + control.spacing
            text: control.text
            color: root.foregroundColor
            font: control.font
            verticalAlignment: Text.AlignVCenter
        }
    }

    ListModel { id: macroSteps }

    Dialog {
        id: actionChooser
        objectName: "actionChooser"
        title: root.tr("Aktion auswählen")
        anchors.centerIn: Overlay.overlay
        modal: true
        width: 620
        height: 590
        standardButtons: Dialog.NoButton
        background: Rectangle { color: root.backgroundColor; radius: 7; border.color: Qt.alpha(root.foregroundColor, 0.28) }
        header: Rectangle {
            implicitHeight: 48
            color: root.surfaceColor
            Text { anchors.left: parent.left; anchors.leftMargin: 18; anchors.verticalCenter: parent.verticalCenter; text: actionChooser.title; color: root.foregroundColor; font.pixelSize: 15; font.weight: Font.DemiBold }
        }
        contentItem: ColumnLayout {
            spacing: 8
            RowLayout {
                Layout.fillWidth: true
                visible: backend.layers.length > 1
                Text { text: root.tr("Zielebene"); color: root.mutedColor; font.pixelSize: 12 }
                PanelCombo { id: actionLayerBox; Layout.fillWidth: true; model: backend.layers; currentIndex: 0; onActivated: root.actionTargetLayer = currentIndex }
                Text { text: root.tr("Einmaliger Modifier"); color: root.mutedColor; font.pixelSize: 12 }
                PanelCombo { id: actionModifierBox; Layout.preferredWidth: 125; model: root.modifiers; currentIndex: 0; onActivated: root.actionModifier = currentIndex }
            }
            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                ColumnLayout {
                    width: parent.width
                    spacing: 8
                    Repeater {
                        model: root.actionChoices
                        delegate: Rectangle {
                            required property var modelData
                            Layout.fillWidth: true
                            implicitHeight: 62
                            radius: 5
                            color: actionMouse.containsMouse ? Qt.alpha(root.foregroundColor, 0.09) : Qt.alpha(root.foregroundColor, 0.045)
                            border.color: actionMouse.containsMouse ? root.accentColor : Qt.alpha(root.foregroundColor, 0.25)
                            Column {
                                anchors.left: parent.left; anchors.right: parent.right
                                anchors.leftMargin: 13; anchors.rightMargin: 13; anchors.verticalCenter: parent.verticalCenter
                                spacing: 3
                                Text { text: root.tr(modelData.label); color: root.foregroundColor; font.pixelSize: 13; font.weight: Font.DemiBold }
                                Text { text: root.tr(modelData.detail); color: root.mutedColor; font.pixelSize: 11; wrapMode: Text.WordWrap; width: parent.width }
                            }
                            MouseArea {
                                id: actionMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                enabled: (modelData.group !== "Ebenen" || backend.layers.length > 1) && backend.selectedPhysicalKey.length > 0
                                onClicked: root.chooseAction(modelData.id)
                            }
                            opacity: (modelData.group === "Ebenen" && backend.layers.length < 2) || backend.selectedPhysicalKey.length === 0 ? 0.45 : 1
                        }
                    }
                }
            }
            PanelButton { Layout.alignment: Qt.AlignRight; text: root.tr("Schließen"); onClicked: actionChooser.close() }
        }
    }

    Dialog {
        id: keyPicker
        title: pickerTarget === "tap" || pickerTarget === "output" ? root.tr("Tastenausgabe auswählen") : pickerTarget === "chord" ? root.tr("Tastenkombination zuweisen") : root.tr("Makroschritt auswählen")
        anchors.centerIn: Overlay.overlay
        modal: true
        width: Math.min(1200, Overlay.overlay.width - 40)
        height: Math.min(660, Overlay.overlay.height - 40)
        standardButtons: Dialog.NoButton
        background: Rectangle { color: root.backgroundColor; radius: 7; border.color: Qt.alpha(root.foregroundColor, 0.28) }
        header: Rectangle {
            implicitHeight: 44
            color: root.surfaceColor
            Text { anchors.left: parent.left; anchors.leftMargin: 18; anchors.verticalCenter: parent.verticalCenter; text: keyPicker.title; color: root.foregroundColor; font.pixelSize: 14; font.weight: Font.DemiBold }
        }
        footer: Rectangle {
            implicitHeight: 48
            color: root.surfaceColor
            PanelButton { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; width: 130; text: root.tr("Schließen"); onClicked: keyPicker.close() }
        }
        contentItem: ColumnLayout {
            spacing: 10
            RowLayout {
                Layout.fillWidth: true
                Caption { text: root.tr("ZEICHENEBENE") }
                Repeater {
                    model: root.activeLayout() === "de"
                        ? [["normal", root.tr("Normal")], ["shift", root.tr("Umschalt")], ["altgr", "AltGr"], ["shift-altgr", root.tr("Umschalt + AltGr")]]
                        : [["normal", root.tr("Normal")], ["shift", root.tr("Umschalt")], ["altgr", "AltGr"]]
                    delegate: PanelButton {
                        required property var modelData
                        text: modelData[1]
                        emphasized: root.pickerLevel === modelData[0]
                        onClicked: root.pickerLevel = modelData[0]
                    }
                }
                Item { Layout.fillWidth: true }
                Text { text: "Layout: " + root.activeLayout().toUpperCase() + (root.activeLayout() === "de" ? " · T2" : ""); color: root.mutedColor; font.pixelSize: 11 }
            }
            Text {
                Layout.fillWidth: true
                text: pickerTarget === "macro" ? root.tr("Taste anklicken, um sie als nächsten Makroschritt hinzuzufügen.") : root.tr("Modifier auswählen und dann die gewünschte Ausgabetaste anklicken.")
                color: root.mutedColor; font.pixelSize: 12; wrapMode: Text.WordWrap
            }
            Flow {
                Layout.fillWidth: true
                objectName: "pickerModifiers"
                spacing: 8
                ModCheck { id: chordCtrl; text: root.tr("Strg") }
                ModCheck { id: chordAlt; objectName: "pickerAlt"; text: "Alt"; onClicked: if (checked) chordAltGr.checked = false }
                ModCheck { id: chordShift; text: root.tr("Umschalt") }
                ModCheck { id: chordMeta; objectName: "pickerSuper"; text: root.tr("Super") }
                ModCheck { id: chordAltGr; text: "AltGr"; onClicked: if (checked) chordAlt.checked = false }
            }
            Item {
                id: keyboardViewport
                Layout.fillWidth: true
                Layout.fillHeight: true
                Item {
                    id: fullsizeBoard
                    readonly property real unit: Math.min(keyboardViewport.width / 23, keyboardViewport.height / 7)
                    readonly property real gap: unit * 0.045
                    width: unit * 23
                    height: unit * 7
                    anchors.centerIn: parent
                    Repeater {
                        model: root.fullsizeGeometry
                        delegate: Button {
                            id: pickerKey
                            required property var modelData
                            readonly property var key: modelData.key
                            readonly property var output: modelData.symbols
                                ? root.symbolActionFor(root.keyToken(key[0]), root.pickerLevel)
                                : {action: root.keyToken(key[0])}
                            readonly property color keyFill: hovered ? Qt.alpha(root.accentColor, 0.24) : Qt.alpha(root.foregroundColor, enabled ? 0.07 : 0.025)
                            readonly property color keyBorder: hovered ? root.accentColor : Qt.alpha(root.foregroundColor, 0.2)
                            x: modelData.x * fullsizeBoard.unit + fullsizeBoard.gap / 2
                            y: modelData.y * fullsizeBoard.unit + fullsizeBoard.gap / 2
                            width: modelData.w * fullsizeBoard.unit - fullsizeBoard.gap
                            height: modelData.h * fullsizeBoard.unit - fullsizeBoard.gap
                            enabled: !output.unavailable
                            padding: fullsizeBoard.unit * 0.09
                            background: Item {
                                Rectangle {
                                    anchors.fill: parent
                                    visible: !pickerKey.modelData.isoEnter
                                    radius: fullsizeBoard.unit * 0.08
                                    color: pickerKey.keyFill
                                    border.color: pickerKey.keyBorder
                                }
                                Canvas {
                                    id: enterShape
                                    anchors.fill: parent
                                    visible: pickerKey.modelData.isoEnter
                                    onWidthChanged: requestPaint()
                                    onHeightChanged: requestPaint()
                                    Connections {
                                        target: pickerKey
                                        function onKeyFillChanged() { enterShape.requestPaint() }
                                        function onKeyBorderChanged() { enterShape.requestPaint() }
                                    }
                                    onPaint: {
                                        var ctx = getContext("2d")
                                        ctx.clearRect(0, 0, width, height)
                                        ctx.beginPath()
                                        ctx.moveTo(0.5, 0.5)
                                        ctx.lineTo(width - 0.5, 0.5)
                                        ctx.lineTo(width - 0.5, height - 0.5)
                                        ctx.lineTo(fullsizeBoard.unit * 0.25 + 0.5, height - 0.5)
                                        ctx.lineTo(fullsizeBoard.unit * 0.25 + 0.5, fullsizeBoard.unit - fullsizeBoard.gap / 2)
                                        ctx.lineTo(0.5, fullsizeBoard.unit - fullsizeBoard.gap / 2)
                                        ctx.closePath()
                                        ctx.fillStyle = pickerKey.keyFill
                                        ctx.fill()
                                        ctx.strokeStyle = pickerKey.keyBorder
                                        ctx.lineWidth = 1
                                        ctx.stroke()
                                    }
                                }
                            }
                            contentItem: Item {
                                Text {
                                    anchors.centerIn: parent
                                    anchors.horizontalCenterOffset: pickerKey.modelData.isoEnter ? fullsizeBoard.unit * 0.125 : 0
                                    visible: !pickerKey.modelData.symbols
                                    text: pickerKey.modelData.isoEnter ? "↵" : root.tr(pickerKey.key[1])
                                    color: root.foregroundColor
                                    font.pixelSize: Math.min(fullsizeBoard.unit * 0.32, parent.width / Math.max(1, text.length) * 1.65)
                                }
                                Repeater {
                                    model: ["normal", "shift", "altgr", "shift-altgr"]
                                    delegate: Text {
                                        required property int index
                                        required property string modelData
                                        x: (index % 2) * parent.width / 2
                                        y: Math.floor(index / 2) * parent.height / 2
                                        width: parent.width / 2 - fullsizeBoard.unit * 0.025
                                        height: parent.height / 2 - fullsizeBoard.unit * 0.025
                                        visible: pickerKey.modelData.symbols && text.length > 0
                                        text: root.pickerLegend(pickerKey.key, modelData)
                                        horizontalAlignment: index % 2 ? Text.AlignRight : Text.AlignLeft
                                        verticalAlignment: index < 2 ? Text.AlignTop : Text.AlignBottom
                                        color: root.pickerLevel === modelData
                                            || (modelData === "normal" && root.pickerLevel === "shift" && /^[a-z]$/.test(pickerKey.key[0]))
                                            ? (modelData === "normal" && root.pickerLevel === "normal" ? root.foregroundColor : root.accentColor)
                                            : root.mutedColor
                                        font.pixelSize: fullsizeBoard.unit * (modelData === "normal" ? 0.36 : 0.28)
                                        fontSizeMode: Text.Fit
                                        minimumPixelSize: Math.max(1, Math.floor(fullsizeBoard.unit * 0.18))
                                        clip: true
                                    }
                                }
                            }
                            onClicked: root.addPickedKey(output.action)
                        }
                    }
                }
            }
        }
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: 19
        clip: true
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 11
            Caption { text: root.tr("TASTE BEARBEITEN") }
            Text { text: backend.selectedPhysicalKey.length > 0 ? root.tr(backend.displayKeyName(backend.selectedPhysicalKey)) : root.tr("Keine Taste ausgewählt"); color: root.foregroundColor; font.pixelSize: 24; font.weight: Font.DemiBold }
            Text { text: root.tr("Ebene: ") + (backend.layers[backend.selectedLayer] || "—"); color: root.mutedColor; font.pixelSize: 12 }
            Text {
                Layout.fillWidth: true
                visible: backend.selectedPhysicalKey.length > 0
                text: (backend.selectedKeyState.kind === "inherited" ? "↳ " : backend.selectedKeyState.kind === "own" ? "● " : "○ ") + root.tr(backend.selectedKeyState.label || "Normale Funktion")
                color: backend.selectedKeyState.kind === "normal" ? root.foregroundColor : root.accentColor
                font.pixelSize: 12
            }
            Text {
                Layout.fillWidth: true
                visible: backend.selectedPhysicalKey.length > 0
                text: backend.selectedKeyState.kind === "inherited"
                    ? root.tr("Durchlassen: folgt der aktiven Basis. Angezeigt wird die Belegung aus ") + (backend.selectedKeyState.baseLayer || "") + "."
                    : root.tr(backend.selectedKeyState.detail || "")
                color: root.mutedColor; font.pixelSize: 11; wrapMode: Text.WordWrap
            }
            Rectangle { Layout.fillWidth: true; height: 1; color: Qt.alpha(root.foregroundColor, 0.12) }

            RowLayout {
                Layout.fillWidth: true
                spacing: 4
                Repeater {
                    model: [root.tr("Ausgabe"), root.tr("Tippen / Halten"), root.tr("Makro")]
                    delegate: Rectangle {
                        required property int index
                        required property var modelData
                        Layout.fillWidth: true
                        height: 34
                        radius: 5
                        color: root.view === index ? Qt.alpha(root.accentColor, 0.15) : Qt.alpha(root.foregroundColor, 0.045)
                        Text { anchors.centerIn: parent; text: modelData; color: root.view === index ? root.accentColor : root.mutedColor; font.pixelSize: 11; font.weight: Font.DemiBold }
                        MouseArea { anchors.fill: parent; onClicked: root.switchView(index) }
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                visible: root.view === 0
                spacing: 10
                enabled: backend.selectedPhysicalKey.length > 0
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: outputSummary.implicitHeight + 24
                    radius: 5
                    color: Qt.alpha(root.foregroundColor, 0.045)
                    ColumnLayout {
                        id: outputSummary
                        anchors.left: parent.left; anchors.right: parent.right
                        anchors.margins: 12; anchors.verticalCenter: parent.verticalCenter
                        spacing: 4
                        Text { text: root.tr("Aktuelle Ausgabe"); color: root.mutedColor; font.pixelSize: 11 }
                        Text {
                            Layout.fillWidth: true
                            text: backend.selectedPhysicalKey.length > 0 ? root.tr(backend.selectedKeyState.output || "Normale Funktion") : "—"
                            color: root.foregroundColor; font.pixelSize: 16
                            wrapMode: Text.WordWrap
                        }
                    }
                }
                PanelButton { text: root.tr("Ausgabe wählen"); emphasized: true; onClicked: { if (root.commit()) root.openKeyPicker("output") } }
                PanelButton { text: root.tr("Tastenkombination"); onClicked: { if (root.commit()) root.openKeyPicker("chord") } }
                PanelButton { text: root.tr("Weitere Aktionen…"); onClicked: { if (root.commit()) { root.actionTargetLayer = backend.selectedLayer; actionLayerBox.currentIndex = backend.selectedLayer; actionChooser.open() } } }
                PanelButton { text: root.tr("Durchlassen"); onClicked: { rawArea.text = "_"; root.commit() } }
                PanelButton {
                    text: (root.actionAdvanced ? "▾  " : "▸  ") + root.tr("Kanata-Ausdruck bearbeiten")
                    onClicked: root.actionAdvanced = !root.actionAdvanced
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    visible: root.actionAdvanced
                    spacing: 8
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 110
                        radius: 5
                        color: root.backgroundColor
                        border.color: rawArea.activeFocus ? root.accentColor : Qt.alpha(root.foregroundColor, 0.25)
                        ScrollView {
                            anchors.fill: parent
                            anchors.margins: 8
                            TextArea {
                                id: rawArea
                                color: root.foregroundColor
                                font.family: "monospace"
                                font.pixelSize: 12
                                wrapMode: TextEdit.Wrap
                                selectByMouse: true
                                enabled: backend.selectedPhysicalKey.length > 0
                                background: null
                            }
                        }
                    }
                    PanelButton { text: root.tr("Übernehmen"); emphasized: true; onClicked: root.commit() }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                visible: root.view === 1
                spacing: 8
                Text { text: root.tr(backend.actionDetails.wrapped ? "Eingebettetes Tap Hold: die umgebende Aktion bleibt erhalten." : "Tipp- und Halteaktion getrennt bearbeiten."); color: root.mutedColor; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                Caption { text: root.tr("BEIM TIPPEN"); Layout.topMargin: 6 }
                PanelField { id: tapField; Layout.fillWidth: true; placeholderText: root.tr("z. B. a"); onTextEdited: root.formDirty = true }
                PanelButton { text: "▦  " + root.tr("Ausgabe über Tastatur wählen"); onClicked: root.openKeyPicker("tap") }
                Caption { text: root.tr("BEIM HALTEN"); Layout.topMargin: 6 }
                PanelCombo { id: holdMode; Layout.fillWidth: true; model: [root.tr("Modifier"), root.tr("Mehrere Modifier"), root.tr("Ebene"), root.tr("Andere Aktion")]; onActivated: root.formDirty = true }
                PanelCombo { id: modifierBox; Layout.fillWidth: true; visible: holdMode.currentIndex === 0; model: root.modifiers.map(function(token) { return root.modifierLabel(token) }); onActivated: root.formDirty = true }
                Flow {
                    Layout.fillWidth: true
                    visible: holdMode.currentIndex === 1
                    spacing: 3
                    ModCheck { id: holdCtrl; text: root.tr("Strg"); onClicked: root.formDirty = true }
                    ModCheck { id: holdAlt; text: root.tr("Alt"); onClicked: root.formDirty = true }
                    ModCheck { id: holdShift; text: root.tr("Shift"); onClicked: root.formDirty = true }
                    ModCheck { id: holdSuper; text: root.tr("Super"); onClicked: root.formDirty = true }
                    ModCheck { id: holdAltGr; text: root.tr("AltGr"); onClicked: root.formDirty = true }
                    ModCheck { id: holdRCtrl; text: root.tr("Strg rechts"); onClicked: root.formDirty = true }
                    ModCheck { id: holdRShift; text: root.tr("Shift rechts"); onClicked: root.formDirty = true }
                    ModCheck { id: holdRSuper; text: root.tr("Super rechts"); onClicked: root.formDirty = true }
                }
                PanelCombo { id: layerBox; Layout.fillWidth: true; visible: holdMode.currentIndex === 2; model: backend.layers; onActivated: root.formDirty = true }
                PanelField { id: customHold; Layout.fillWidth: true; visible: holdMode.currentIndex === 3; placeholderText: root.tr("z. B. (multi lctl lshft)"); onTextEdited: root.formDirty = true }
                PanelButton {
                    text: (root.tapAdvanced ? "▾  " : "▸  ") + root.tr("Erweiterte Einstellungen")
                    onClicked: root.tapAdvanced = !root.tapAdvanced
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    visible: root.tapAdvanced
                    spacing: 8
                    Caption { text: root.tr("VARIANTE") }
                    PanelCombo { id: tapHead; Layout.fillWidth: true; model: root.tapVariants; onActivated: root.formDirty = true }
                    Caption { text: root.tr("ZEITEN IN MS") }
                    RowLayout {
                        Layout.fillWidth: true
                        ColumnLayout {
                            Layout.fillWidth: true
                            Text { text: root.tr("Wiederholung"); color: root.mutedColor; font.pixelSize: 10 }
                            PanelField {
                                id: repeatField
                                Layout.fillWidth: true
                                validator: IntValidator { bottom: 1; top: 5000 }
                                onTextEdited: root.formDirty = true
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Text { text: root.tr("Halten"); color: root.mutedColor; font.pixelSize: 10 }
                            PanelField {
                                id: holdField
                                Layout.fillWidth: true
                                validator: IntValidator { bottom: 1; top: 5000 }
                                onTextEdited: root.formDirty = true
                            }
                        }
                    }
                }
                PanelButton { text: root.tr("Tap Hold übernehmen"); emphasized: true; Layout.topMargin: 8; onClicked: root.commit() }
            }

            ColumnLayout {
                Layout.fillWidth: true
                visible: root.view === 2
                spacing: 8
                Text { text: root.tr("Schritte laufen von oben nach unten. Eine Zahl ist eine Pause in Millisekunden."); color: root.mutedColor; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                PanelButton {
                    text: (root.macroAdvanced ? "▾  " : "▸  ") + root.tr("Erweiterte Einstellungen")
                    onClicked: root.macroAdvanced = !root.macroAdvanced
                }
                PanelCombo { id: macroVariant; visible: root.macroAdvanced; Layout.fillWidth: true; model: root.macroVariants; onActivated: root.formDirty = true }
                Caption { text: root.tr("SCHRITTE"); Layout.topMargin: 7 }
                Repeater {
                    model: macroSteps
                    delegate: RowLayout {
                        required property int index
                        required property string step
                        Layout.fillWidth: true
                        spacing: 3
                        PanelField { Layout.fillWidth: true; text: step; font.family: "monospace"; onTextEdited: { macroSteps.setProperty(index, "step", text); root.formDirty = true } }
                        PanelButton { Layout.fillWidth: false; text: "↑"; implicitWidth: 25; enabled: index > 0; onClicked: { macroSteps.move(index, index - 1, 1); root.formDirty = true } }
                        PanelButton { Layout.fillWidth: false; text: "↓"; implicitWidth: 25; enabled: index < macroSteps.count - 1; onClicked: { macroSteps.move(index, index + 1, 1); root.formDirty = true } }
                        PanelButton { Layout.fillWidth: false; text: "×"; implicitWidth: 25; onClicked: { macroSteps.remove(index); root.formDirty = true } }
                    }
                }
                Caption { text: root.tr("TASTE HINZUFÜGEN"); Layout.topMargin: 8 }
                PanelField { id: macroKey; visible: root.macroAdvanced; Layout.fillWidth: true; placeholderText: root.tr("z. B. a, Enter, Digit1"); font.family: "monospace" }
                Flow {
                    Layout.fillWidth: true
                    visible: root.macroAdvanced
                    spacing: 3
                    ModCheck { id: ctrlMod; text: root.tr("Strg") }
                    ModCheck { id: altMod; text: root.tr("Alt") }
                    ModCheck { id: shiftMod; text: root.tr("Shift") }
                    ModCheck { id: metaMod; text: root.tr("Super") }
                    ModCheck { id: altGrMod; text: root.tr("AltGr") }
                }
                PanelButton { text: "▦  " + root.tr("Taste aus Tastatur auswählen"); onClicked: root.openKeyPicker("macro") }
                PanelButton {
                    text: "+ " + root.tr("Taste")
                    visible: root.macroAdvanced
                    onClicked: {
                        var key = macroKey.text.trim()
                        if (!/^[A-Za-z0-9_+-][A-Za-z0-9_+*/.-]*$/.test(key) || key === "") { backend.report(root.tr("Bitte einen gültigen Tastennamen eingeben")); return }
                        if (/^[0-9]$/.test(key)) key = "Digit" + key
                        var prefix = (ctrlMod.checked ? "C-" : "") + (altMod.checked ? "A-" : "")
                            + (shiftMod.checked ? "S-" : "") + (metaMod.checked ? "M-" : "") + (altGrMod.checked ? "AG-" : "")
                        if (altMod.checked && altGrMod.checked) { backend.report(root.tr("Alt und AltGr können nicht gemeinsam gesetzt werden")); return }
                        macroSteps.append({"step": prefix + key})
                        macroKey.text = ""
                        root.formDirty = true
                    }
                }
                Caption { text: root.tr("PAUSE HINZUFÜGEN"); Layout.topMargin: 8 }
                RowLayout {
                    Layout.fillWidth: true
                    PanelField { id: macroPause; Layout.fillWidth: true; text: "100"; validator: IntValidator { bottom: 1; top: 5000 } }
                    PanelButton {
                        Layout.fillWidth: false
                        text: "+ ms"
                        implicitWidth: 68
                        onClicked: {
                            var value = Number(macroPause.text)
                            if (value < 1 || value > 5000 || !Number.isInteger(value)) { backend.report(root.tr("Pause muss zwischen 1 und 5000 ms liegen")); return }
                            macroSteps.append({"step": String(value)})
                            root.formDirty = true
                        }
                    }
                }
                PanelButton { text: root.tr("Makro übernehmen"); emphasized: true; Layout.topMargin: 8; onClicked: root.commit() }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: Qt.alpha(root.foregroundColor, 0.12); Layout.topMargin: 12 }
            PanelButton {
                text: root.tr("Entwurf verwerfen")
                visible: backend.dirty || root.pendingEdit
                enabled: backend.dirty || root.pendingEdit
                onClicked: { backend.discard(); root.sync() }
            }
            Item { Layout.preferredHeight: 12 }
        }
    }
    Component.onCompleted: sync()
}
