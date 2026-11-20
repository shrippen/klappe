import QtQuick

// The element flow of the screenplay editor (Lastenheft, Anhang A): Enter at the end of an
// element opens the next one, Tab turns an empty line into another, and headings and cues turn
// to capitals as they are typed (DB-01). The rules are `script/`'s (`enterStep`, `tabStep`,
// `typedCaps`); this only moves the cursor and the text.
//
//   before Enter           after Enter
//   INT. SAAL▌             INT. SAAL
//                          (blank line)
//                          ▌            ← pending: an action line, set as one until the
//                                         cursor leaves it; Tab makes it a cue
QtObject {
    id: flow

    required property var area
    required property var controller
    /// The native tools (ScriptTools), for a change that is one step of the history.
    property var tools: null
    /// The line Enter or Tab made an element, and its kind (a letter of `scriptKinds`); -1 for
    /// none.
    property int pendingLine: -1
    property string pendingKind: ""
    /// The kinds `scriptKinds` gave for the text last, as the editor shows them.
    property string shown: ""
    /// The letter of each kind by its name, as `scriptLooks` gives them.
    readonly property var letters: JSON.parse(controller.scriptLooks()).letters

    /// The lines of the text now; read again each time, as a binding lags behind an edit.
    function rows() {
        return area.text.split("\n")
    }

    /// The pending line changed: the editor shows the lines anew.
    signal pendingMoved()

    function cursorLine() {
        return area.text.slice(0, area.cursorPosition).split("\n").length - 1
    }

    function lineStart(line) {
        const all = rows()
        let at = 0
        for (let i = 0; i < line; i++) {
            at += all[i].length + 1
        }
        return at
    }

    /// `kinds` with the pending line as its element while the reader sees it as plain: an empty
    /// line, or prose on one meant as cue, parenthetical or dialogue.
    function withPending(kinds) {
        if (pendingLine < 0 || pendingLine >= kinds.length) {
            return kinds
        }
        const read = kinds[pendingLine]
        const plain = read === letters.blank || read === letters.action
        const heading = pendingKind === letters.heading && (rows()[pendingLine] || "").trim() !== ""
        if (!plain || heading) {
            return kinds
        }
        return kinds.slice(0, pendingLine) + pendingKind + kinds.slice(pendingLine + 1)
    }

    /// The cursor left the pending line: it is what the reader says.
    function leaveIfMoved() {
        if (pendingLine < 0 || cursorLine() === pendingLine) {
            return false
        }
        pendingLine = -1
        pendingMoved()
        return true
    }

    function setPending(line, kind) {
        pendingLine = line
        pendingKind = kind
        pendingMoved()
    }

    /// Enter; false leaves the plain line break to the text area, also over a selection, which
    /// it replaces.
    function enter() {
        if (area.selectedText !== "") {
            return false
        }
        const all = rows()
        const line = cursorLine()
        const start = lineStart(line)
        const text = all[line]
        const below = line + 1 < all.length ? all[line + 1] : ""
        const step = JSON.parse(controller.enterStep(text, area.cursorPosition - start, shown[line] || letters.action, below))

        switch (step.step) {
        case "retag":
            tab()
            return true
        case "below":
            area.cursorPosition = start + text.length + 1 + below.length
            return true
        case "next":
            openAfter(line, start + text.length, step)
            return true
        }
        return false
    }

    /// The next element after the end of `line`, after a blank line if the gap asks for one.
    function openAfter(line, end, step) {
        const breaks = step.gap === "blank" ? "\n\n" : "\n"
        area.insert(end, breaks)
        area.cursorPosition = end + breaks.length
        setPending(line + breaks.length, step.kind)
    }

    /// Tab on an empty line changes its element; the line takes the text Tab gives it (`()`).
    function tab() {
        const line = cursorLine()
        const text = rows()[line]
        const kind = line === pendingLine ? pendingKind : (shown[line] || letters.action)
        const found = controller.tabStep(text, kind)
        if (found === "") {
            return
        }

        const step = JSON.parse(found)
        const start = lineStart(line)
        replace(start, start + text.length, step.text)
        area.cursorPosition = start + step.col
        setPending(line, step.kind)
    }

    /// The cursor line in capitals if it is a heading or cue; true when it changed the text.
    function capitalize() {
        const line = cursorLine()
        // A pending heading takes capitals once it is one (`INT.`), not while it may be prose.
        const pending = line === pendingLine && pendingKind !== letters.heading ? pendingKind : ""
        const caps = controller.typedCaps(area.text, line, pending)
        if (caps === "") {
            return false
        }

        const at = area.cursorPosition
        const start = lineStart(line)
        replace(start, start + rows()[line].length, caps)
        area.cursorPosition = at
        return true
    }

    function replace(from, to, text) {
        if (tools && tools.replace) {
            tools.replace(from, to, text)
            return
        }
        area.remove(from, to)
        area.insert(from, text)
    }
}
