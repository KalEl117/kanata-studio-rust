// Shared by the desktop editor and the Omarchy popup. Never modifies devices.
function displayLayout(info) {
    return ["de", "us", "gb"].indexOf(info.choice) >= 0 ? info.choice : (info.id || "de")
}
function displayGeometry(info) {
    return info.choice === "us" ? "ansi" : ["de", "gb"].indexOf(info.choice) >= 0 ? "iso" : (info.geometry || "iso")
}
function isFullSize(info) {
    return info.choice === "fullsize" || (["thinkpad", "tkl"].indexOf(info.choice) < 0 && !!(info.keyboard && info.keyboard.fullSize))
}
function isLaptop(info) {
    return info.choice === "thinkpad" || (!isFullSize(info) && info.choice !== "tkl" && !!(info.keyboard && info.keyboard.type === "built-in"))
}
function isIsoEnter(info) { return displayGeometry(info) === "iso" }
function buildRows(info) {
    info = info || {}
    if (info.available === false) return []
    var rows = baseRows(info, true)
    if (isLaptop(info)) return rows
    var full = isFullSize(info)
    var side = [
        [["PrintScreen","Druck",1],["ScrollLock","Rollen",1],["Pause","Pause",1]],
        [["Insert","Einfg",1],["Home","Pos1",1],["PageUp","Bild↑",1]],
        [["Delete","Entf",1],["End","Ende",1],["PageDown","Bild↓",1]],
        [["","",3]],
        [["","",1],["ArrowUp","↑",1],["","",1]],
        [["ArrowLeft","←",1],["ArrowDown","↓",1],["ArrowRight","→",1]]
    ]
    var num = [
        [["","",4]],
        [["nlck","Num",1],["kp/","/",1],["kp*","×",1],["kp-","−",1]],
        [["kp7","7",1],["kp8","8",1],["kp9","9",1],["kp+","+",1,2]],
        [["kp4","4",1],["kp5","5",1],["kp6","6",1],["","",1]],
        [["kp1","1",1],["kp2","2",1],["kp3","3",1],["kprt","↵",1,2]],
        [["kp0","0",2],["kp.",displayLayout(info) === "de" ? "," : ".",1],["","",1]]
    ]
    rows[5] = [["ControlLeft","Strg",1.25],["MetaLeft","Super",1.25],["AltLeft","Alt",1.25],["Space","Leertaste",6.25],["AltRight","AltGr",1.25],["MetaRight","Super",1.25],["ContextMenu","Menü",1.25],["ControlRight","Strg",1.25]]
    for (var r = 0; r < rows.length; ++r) {
        var units = rows[r].reduce(function(total, key) { return total + key[2] }, 0)
        rows[r].push(["","",Math.max(0, 15 - units) + 0.5])
        rows[r] = rows[r].concat(side[r])
        if (full) rows[r] = rows[r].concat([["","",0.5]], num[r])
    }
    return rows
}
    function baseRows(info, includeThinkPad) {
        var rows = [
        [["Escape", "Esc", 1], ["F1", "F1", 1], ["F2", "F2", 1], ["F3", "F3", 1], ["F4", "F4", 1], ["F5", "F5", 1], ["F6", "F6", 1], ["F7", "F7", 1], ["F8", "F8", 1], ["F9", "F9", 1], ["F10", "F10", 1], ["F11", "F11", 1], ["F12", "F12", 1]],
        [["Backquote", "^", 1], ["1", "1", 1], ["2", "2", 1], ["3", "3", 1], ["4", "4", 1], ["5", "5", 1], ["6", "6", 1], ["7", "7", 1], ["8", "8", 1], ["9", "9", 1], ["0", "0", 1], ["Minus", "ß", 1], ["Equal", "´", 1], ["Backspace", "⌫", 2]],
        [["Tab", "Tab", 1.5], ["q", "Q", 1], ["w", "W", 1], ["e", "E", 1], ["r", "R", 1], ["t", "T", 1], ["y", "Z", 1], ["u", "U", 1], ["i", "I", 1], ["o", "O", 1], ["p", "P", 1], ["BracketLeft", "Ü", 1], ["BracketRight", "+", 1], ["Enter", "Eingabe", 1.5]],
        [["CapsLock", "Caps", 1.75], ["a", "A", 1], ["s", "S", 1], ["d", "D", 1], ["f", "F", 1], ["g", "G", 1], ["h", "H", 1], ["j", "J", 1], ["k", "K", 1], ["l", "L", 1], ["Semicolon", "Ö", 1], ["Quote", "Ä", 1], ["Backslash", "#", 1]],
        [["ShiftLeft", "Shift", 1.25], ["IntlBackslash", "<", 1], ["z", "Y", 1], ["x", "X", 1], ["c", "C", 1], ["v", "V", 1], ["b", "B", 1], ["n", "N", 1], ["m", "M", 1], ["Comma", ",", 1], ["Period", ".", 1], ["Slash", "-", 1], ["ShiftRight", "Shift", 2.75]],
        [["ControlLeft", "Strg", 1.35], ["MetaLeft", "Super", 1.35], ["AltLeft", "Alt", 1.35], ["Space", "Leertaste", 5.6], ["AltRight", "AltGr", 1.35], ["MetaRight", "Super", 1.35], ["ControlRight", "Strg", 1.35]]
        ]
        info = info || {}
        var choice = info.choice || "auto"
        var layout = ["de", "us", "gb"].indexOf(choice) >= 0 ? choice : (info.id || "de")
        var geometry = displayGeometry(info)
        var labels = layout === "us" ? {"Backquote":"`", "Minus":"-", "Equal":"=", "y":"Y", "z":"Z", "BracketLeft":"[", "BracketRight":"]", "Semicolon":";", "Quote":"'", "Backslash":"\\", "IntlBackslash":"\\", "Slash":"/", "ControlLeft":"Ctrl", "ControlRight":"Ctrl", "AltRight":"Alt"}
            : layout === "gb" ? {"Backquote":"`", "Minus":"-", "Equal":"=", "y":"Y", "z":"Z", "BracketLeft":"[", "BracketRight":"]", "Semicolon":";", "Quote":"'", "Backslash":"#", "IntlBackslash":"\\", "Slash":"/", "ControlLeft":"Ctrl", "ControlRight":"Ctrl"}
            : {"Backquote":"^", "Minus":"ß", "Equal":"´", "y":"Z", "z":"Y", "BracketLeft":"Ü", "BracketRight":"+", "Semicolon":"Ö", "Quote":"Ä", "Backslash":"#", "IntlBackslash":"<", "Slash":"-", "ControlLeft":"Strg", "ControlRight":"Strg"}
        for (var r = 0; r < rows.length; ++r)
            for (var k = 0; k < rows[r].length; ++k)
                if (labels[rows[r][k][0]] !== undefined) rows[r][k][1] = labels[rows[r][k][0]]
        if (geometry === "ansi") {
            rows[2][rows[2].length - 1] = ["Backslash", labels.Backslash || "\\", 1.5]
            rows[3] = rows[3].filter(function(key) { return key[0] !== "Backslash" })
            rows[3].push(["Enter", "Eingabe", 2.25])
            rows[4] = rows[4].filter(function(key) { return key[0] !== "IntlBackslash" })
            rows[4][0][2] = 2.25
        }
        if (includeThinkPad && isLaptop(info)) {
            rows[0].push(["Home", "Pos1", 1], ["End", "Ende", 1], ["Insert", "Einfg", 1], ["Delete", "Entf", 1])
            rows[5] = [["fn", "Fn", 1], ["ControlLeft", layout === "us" ? "Ctrl" : "Strg", 1.25], ["MetaLeft", "Super", 1], ["AltLeft", "Alt", 1.1], ["Space", "Leertaste", 5.8], ["AltRight", layout === "us" ? "Alt" : "AltGr", 1.15], ["PrintScreen", "Druck", 1], ["ControlRight", layout === "us" ? "Ctrl" : "Strg", 1.1], ["PageUp", "Bild↑", 1], ["ArrowUp", "↑", 1], ["PageDown", "Bild↓", 1]]
            rows.push([["ArrowLeft", "←", 1], ["ArrowDown", "↓", 1], ["ArrowRight", "→", 1]])
        }
        return rows
    }

function normalize(name) {
        var aliases = {"min":"minus", "-":"minus", "eql":"equal", "=":"equal", "lbrc":"leftbracket", "[":"leftbracket", "rbrc":"rightbracket", "]":"rightbracket", "scln":"semicolon", ";":"semicolon", "apo":"apostrophe", "apos":"apostrophe", "'":"apostrophe", "comm":"comma", ",":"comma", ".":"dot", "/":"slash", "bksl":"backslash", "\\":"backslash", "102d":"intlbackslash", "lsgt":"intlbackslash", "nubs":"intlbackslash", "<":"intlbackslash", "escape":"esc", "backquote":"grv", "backspace":"bspc", "enter":"ret", "space":"spc", "capslock":"caps", "shiftleft":"lsft", "shiftright":"rsft", "controlleft":"lctl", "controlright":"rctl", "metaleft":"lmeta", "metaright":"rmeta", "altleft":"lalt", "altright":"ralt", "period":"dot", "quote":"apostrophe", "bracketleft":"leftbracket", "bracketright":"rightbracket", "delete":"del", "insert":"ins", "pageup":"pgup", "pagedown":"pgdn", "arrowleft":"lft", "arrowright":"rght", "arrowup":"up", "arrowdown":"down", "printscreen":"ssrq", "sysrq":"ssrq", "scrolllock":"slck", "contextmenu":"menu"}
    var lower = String(name).toLowerCase()
    return aliases[lower] || lower
}
