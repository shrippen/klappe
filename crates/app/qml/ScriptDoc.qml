import QtQuick

// The seam of the screenplay editor (D-36): every part of the editor talks to the Fountain text
// (the source) through this, never to the text area. All positions are of the source, UTF-16
// like the text area's. The area's focus, keys, scrolling and drawing stay with the area.
//
//   ScriptFlow, ScriptSearch,        ScriptDoc            text area
//   TitleCard, ScriptSuggest,   ──▶  source ⇄ display ──▶ (what is drawn)
//   RevisionMarks, the view          (the same text and positions today)
//
// Today the display is the source, so each call goes straight to the area. Once the editor shows
// the script as printed, without Fountain's marks, this is the one place that maps between them.
QtObject {
    id: doc

    /// The text area that shows the source.
    required property var area
    /// The native tools (ScriptTools), for a change that is one step of the history.
    property var tools: null

    /// The Fountain text; for bindings. Changes with every edit, typed or loaded.
    readonly property string source: area.text
    readonly property int cursor: area.cursorPosition
    readonly property int selectionStart: area.selectionStart
    readonly property int selectionEnd: area.selectionEnd
    readonly property string selectedText: area.selectedText
    /// The text area's document, for the native tools that draw and record it.
    readonly property var document: area.textDocument
    /// The text has the keyboard focus.
    readonly property bool focused: area.activeFocus
    /// `load` is running: what changes the source then is no typing.
    property bool loading: false

    /// The source as it stands, also inside the handlers of an edit, where `source` may lag.
    function read() {
        return area.text
    }

    function setCursor(pos) {
        area.cursorPosition = pos
    }

    function select(from, to) {
        area.select(from, to)
    }

    /// Typed-like insertion of `text` at `pos`.
    function insert(pos, text) {
        area.insert(pos, text)
    }

    /// Puts `text` in place of `from` to `to` as one step of the text's history.
    function replace(from, to, text) {
        if (tools && tools.replace) {
            tools.replace(from, to, text)
            return
        }
        area.remove(from, to)
        area.insert(from, text)
    }

    /// Where line `line` (from 0) of the source starts; -1 when the source has fewer lines.
    function lineStart(line) {
        return lineStarts([line])[0]
    }

    /// Where each of `lines` (ascending, from 0) starts, -1 for a line the source does not have.
    /// One pass over the source, as the page marks ask on every key.
    function lineStarts(lines) {
        const text = read()
        const starts = []
        let line = 0
        let at = 0
        for (const wanted of lines) {
            while (at >= 0 && line < wanted) {
                const next = text.indexOf("\n", at)
                at = next < 0 ? -1 : next + 1
                line++
            }
            starts.push(at)
        }
        return starts
    }

    /// Where the character at `pos` is drawn, in the area's coordinates.
    function rectOf(pos) {
        return area.positionToRectangle(pos)
    }

    /// Takes `text` from outside (an import, another project): no typing.
    function load(text) {
        loading = true
        area.text = text
        loading = false
    }
}
