import QtQuick

// The seam of the screenplay editor (D-36): every part of the editor talks to the Fountain text
// (the source) through this, never to the text area. All positions are of the source, UTF-16
// like the text area's. The area's focus, keys, scrolling and drawing stay with the area.
//
//   ScriptFlow, ScriptSearch,        ScriptDoc                       text area
//   TitleCard, ScriptSuggest,   ──▶  source ⇄ shown lines     ──▶    (the shown text:
//   RevisionMarks, the view          (`controller` projects)          no `*`, `_` of emphasis)
//
// The doc holds the source and, per line, what the area shows: its text, the emphasis of it
// and where each shown character stands in the source line. What the user types in the area
// becomes one change of the source (`scriptEditJson`); what a part of the editor replaces in
// the source is shown again (`scriptReplaceJson`). Only the touched lines are written and
// shown again. The emphasis is drawn by the native look (`tools`), not set in the document.
QtObject {
    id: doc

    /// The text area that shows the source.
    required property var area
    /// The controller that projects the source (`scriptShownJson`, `scriptEditJson`,
    /// `scriptReplaceJson`).
    required property var controller
    /// The native tools (ScriptTools): the history of the source and the drawn emphasis.
    property var tools: null

    /// The Fountain text; for bindings. Changes with every edit, typed or loaded.
    readonly property string source: _source
    readonly property int cursor: {
        _rev
        return toSource(area.cursorPosition)
    }
    readonly property int selectionStart: {
        _rev
        return toSource(area.selectionStart)
    }
    /// The end of the selection as the source position behind its last character.
    readonly property int selectionEnd: {
        _rev
        return toSourceEnd(area.selectionEnd)
    }
    readonly property string selectedText: area.selectedText
    /// The text area's document, for the native tools that draw it.
    readonly property var document: area.textDocument
    /// The text has the keyboard focus.
    readonly property bool focused: area.activeFocus
    /// `load` is running: what changes the source then is no typing.
    property bool loading: false

    /// The source changed by typing, a replacement or an undo step (not by `load`).
    signal edited()

    property string _source: ""
    /// Per source line `{ text, runs, cols }` as `scriptShownJson` gives them, cleaned for the area.
    property var _lines: [{ text: "", runs: [], cols: null }]
    /// What the area holds, to see what a change of it is.
    property string _text: ""
    /// Where each line starts, in the source (`src`) and in the shown text (`disp`), and whether
    /// that is up to date. An object, so that reading it in a binding does not write a property.
    property var _t: ({ fresh: false, src: [0], disp: [0, 1] })
    /// For bindings on what the tables map; counts up when lines or area changed.
    property int _rev: 0
    /// The doc itself changes the area: that is no typing.
    property bool _busy: false
    /// Whether a change of the source is a step of the history (not an undo or redo).
    property bool _recording: true

    /// Text this long is compared in one go.
    readonly property int _chunk: 512
    /// The characters of emphasis.
    readonly property string _marks: "*_"

    Component.onCompleted: area.textChanged.connect(_onArea)

    /// The source as it stands, also inside the handlers of an edit, where `source` may lag.
    function read() {
        return _source
    }

    /// Takes `text` from outside (an import, another project): no typing.
    function load(text) {
        loading = true
        _source = text
        _lines = JSON.parse(controller.scriptShownJson(text)).map(_clean)
        // The emphasis first: the text is drawn with it when it is set, once.
        drawRuns()
        area.text = _lines.map(l => l.text).join("\n")
        _text = area.text
        _t.fresh = false
        _rev++
        loading = false
    }

    /// Sets the emphasis of all lines, to be drawn with the text that is set next.
    function drawRuns() {
        if (tools && tools.setRuns) {
            tools.setRuns(0, -1, _lines.map(_flat))
        }
    }

    /// The tools came after the text: draws the emphasis of the lines that stand.
    function redrawRuns() {
        drawRuns()
        if (tools && tools.redraw) {
            tools.redraw()
        }
    }

    // ---- positions ----

    function setCursor(pos) {
        area.cursorPosition = toDisplay(pos)
    }

    function select(from, to) {
        area.select(toDisplay(from), toDisplay(to))
    }

    /// Where the character at `pos` is drawn, in the area's coordinates.
    function rectOf(pos) {
        _rev
        return area.positionToRectangle(toDisplay(pos))
    }

    /// Where line `line` (from 0) of the source starts; -1 when the source has fewer lines.
    function lineStart(line) {
        return lineStarts([line])[0]
    }

    /// Where each of `lines` (ascending, from 0) starts, -1 for a line the source does not have.
    /// One pass over the lines, as the page marks ask on every key.
    function lineStarts(lines) {
        _rev
        _tables()
        return lines.map(line => line < _lines.length ? _t.src[line] : -1)
    }

    /// The position in the shown text of source position `pos`: a position in a hidden mark
    /// is the next one that shows.
    function toDisplay(pos) {
        _tables()
        const line = _lineOf(_t.src, pos)
        const shown = _lines[line]
        const col = pos - _t.src[line]
        const length = shown.text.length
        if (!shown.cols) {
            return _t.disp[line] + Math.min(col, length)
        }

        // The first shown character that stands at or behind `col`.
        let lo = 0
        let hi = length
        while (lo < hi) {
            const mid = (lo + hi) >> 1
            if (shown.cols[mid] >= col) {
                hi = mid
            } else {
                lo = mid + 1
            }
        }
        return _t.disp[line] + lo
    }

    /// The source position of the character shown at `pos`; at the end of a line, where its text
    /// ends in the source.
    function toSource(pos) {
        _tables()
        const line = _lineOf(_t.disp, pos)
        const shown = _lines[line]
        const at = Math.min(pos - _t.disp[line], shown.text.length)
        return _t.src[line] + (shown.cols ? shown.cols[at] : at)
    }

    /// The source position behind the character shown before `pos`: an end of a selection.
    function toSourceEnd(pos) {
        _tables()
        const line = _lineOf(_t.disp, pos)
        const shown = _lines[line]
        const at = Math.min(pos - _t.disp[line], shown.text.length)
        if (at === 0) {
            return _t.src[line]
        }
        return _t.src[line] + (shown.cols ? shown.cols[at - 1] + 1 : at)
    }

    // ---- selection ----

    /// The source of the selected text, with the marks that open or close its emphasis at its
    /// ends: a copy of `fett` in `**fett**` is `**fett**`. Empty without a selection.
    function selectedSource() {
        if (area.selectedText === "") {
            return ""
        }

        const range = _selectedRange()
        return _source.slice(range[0], range[1])
    }

    /// Takes the selection out of the source (cut).
    function cutSelection() {
        if (area.selectedText === "") {
            return
        }

        const range = _selectedRange()
        replace(range[0], range[1], "")
    }

    function _selectedRange() {
        _tables()
        const start = area.selectionStart
        const end = area.selectionEnd
        return [
            _withMarks(toSource(start), start, -1),
            _withMarks(toSourceEnd(end), end, 1)
        ]
    }

    /// `pos` moved over the marks next to it when an emphasised stretch starts (`way` -1) or
    /// ends (1) at the shown position `shownAt`.
    function _withMarks(pos, shownAt, way) {
        const line = _lineOf(_t.disp, shownAt)
        const col = shownAt - _t.disp[line]
        const edge = way < 0 ? "from" : "to"
        if (!_lines[line].runs.some(run => run[edge] === col)) {
            return pos
        }

        const last = line + 1 < _lines.length ? _t.src[line + 1] - 1 : _source.length
        let at = pos
        while (way < 0 ? at > _t.src[line] : at < last) {
            if (_marks.indexOf(_source.charAt(way < 0 ? at - 1 : at)) < 0) {
                break
            }
            at += way
        }
        return at
    }

    // ---- changes of the source ----

    /// Typed-like insertion of `text` at `pos`.
    function insert(pos, text) {
        replace(pos, pos, text)
    }

    /// Puts `text` (Fountain as it stands) in place of `from` to `to` as one step of the history.
    function replace(from, to, text) {
        if (from === to && text === "") {
            return
        }

        _tables()
        const a = _sourceSpot(from)
        const b = _sourceSpot(to)
        const done = controller.scriptReplaceJson(_source, a.line, a.col, b.line, b.col, text)
        _take(JSON.parse(done), false)
    }

    /// Takes back the last step of the history; the source position the cursor goes to, -1 for none.
    function undo() {
        return _step(tools ? tools.undo() : null)
    }

    function redo() {
        return _step(tools ? tools.redo() : null)
    }

    function _step(step) {
        if (!step || step.at === undefined) {
            return -1
        }

        _recording = false
        replace(step.at, step.at + step.removed.length, step.put)
        _recording = true
        return step.at + step.put.length
    }

    /// The area's text changed: the user typed, deleted or pasted. That is one change of the
    /// shown text, from which the source is written.
    function _onArea() {
        const now = area.text
        if (_busy || loading || now === _text) {
            return
        }

        _tables()
        const old = _text
        let start = _same(old, now, 0, false)
        if (start > 0 && _isHigh(old.charCodeAt(start - 1))) {
            start--
        }
        let end = _same(old, now, start, true)
        if (end > 0 && _isLow(old.charCodeAt(old.length - end))) {
            end--
        }
        const typed = now.slice(start, now.length - end)
        const from = _shownSpot(start)
        const to = _shownSpot(old.length - end)
        _text = now

        const change = { line: from.line, from: from.col, endLine: to.line, to: to.col,
                         typed: typed, runs: _inherit(from, typed) }
        const done = controller.scriptEditJson(_source, JSON.stringify(change))
        if (done === "") {
            return
        }
        _take(JSON.parse(done), true)
    }

    /// What `typed` has of emphasis: that of the character before it, at the start of a line of
    /// the one behind. Several lines (a paste) bring none.
    function _inherit(from, typed) {
        if (typed === "" || typed.indexOf("\n") >= 0) {
            return []
        }

        const at = Math.max(from.col - 1, 0)
        const run = _lines[from.line].runs.find(r => r.from <= at && at < r.to)
        if (!run) {
            return []
        }
        return [{ from: 0, to: typed.length, bold: run.bold, italic: run.italic,
                  underline: run.underline }]
    }

    /// Makes the answer of the controller the state: the source, the lines, the drawn emphasis,
    /// the area and the history. `ownText` is whether the area holds the change already (typed).
    function _take(res, ownText) {
        _tables()
        const count = res.to - res.from
        const at = _t.src[res.from]
        const end = res.to < _lines.length ? _t.src[res.to] - 1 : _source.length
        const before = _source.slice(at, end)
        const after = res.source.slice(at, end + res.source.length - _source.length)
        const shownStart = _t.disp[res.from]
        const shown = res.shown.map(_clean)
        const unchanged = res.source === _source

        _source = res.source
        Array.prototype.splice.apply(_lines, [res.from, count].concat(shown))
        _t.fresh = false
        if (tools && tools.setRuns) {
            tools.setRuns(res.from, count, shown.map(_flat))
        }
        _show(shownStart, ownText ? shown.length : count, shown)
        _rev++
        if (unchanged) {
            return
        }

        if (_recording && tools && tools.record) {
            _record(at, before, after)
        }
        edited()
    }

    /// The history gets what changed between `before` and `after`, which stood at `at`.
    function _record(at, before, after) {
        const start = _same(before, after, 0, false)
        const end = _same(before, after, start, true)

        tools.record(at + start, before.slice(start, before.length - end),
                     after.slice(start, after.length - end))
    }

    /// The area's `count` lines from `start` show `shown`: only what differs is replaced, the
    /// cursor keeps its place relative to the end of them.
    function _show(start, count, shown) {
        let end = start
        for (let i = 0; i < count; i++) {
            const next = _text.indexOf("\n", end)
            if (next < 0) {
                end = _text.length
                break
            }
            end = i === count - 1 ? next : next + 1
        }

        const current = _text.slice(start, end)
        const want = shown.map(l => l.text).join("\n")
        if (current === want) {
            return
        }

        const head = _same(current, want, 0, false)
        const tail = _same(current, want, head, true)
        const from = start + head
        const to = start + current.length - tail
        const put = want.slice(head, want.length - tail)
        const cursor = area.cursorPosition
        let moved = cursor
        if (cursor > end) {
            moved = cursor + want.length - current.length
        } else if (cursor >= start) {
            moved = Math.max(start, start + want.length - (end - cursor))
        }

        _busy = true
        area.remove(from, to)
        area.insert(from, put)
        _text = _text.slice(0, from) + put + _text.slice(to)
        area.cursorPosition = moved
        _busy = false
    }

    // ---- tables ----

    /// Where each line starts in the source and in the shown text.
    function _tables() {
        if (_t.fresh) {
            return
        }

        // Read once: every read of the source property is a copy of it.
        const text = _source
        const lines = _lines
        const src = [0]
        for (let at = text.indexOf("\n"); at >= 0; at = text.indexOf("\n", at + 1)) {
            src.push(at + 1)
        }
        const shown = [0]
        let total = 0
        for (let i = 0; i < lines.length; i++) {
            total += lines[i].text.length + 1
            shown.push(total)
        }
        _t.src = src
        _t.disp = shown
        _t.fresh = true
    }

    /// The line `pos` is in, by the starts of the lines.
    function _lineOf(starts, pos) {
        let lo = 0
        let hi = _lines.length - 1
        while (lo < hi) {
            const mid = (lo + hi + 1) >> 1
            if (starts[mid] <= pos) {
                lo = mid
            } else {
                hi = mid - 1
            }
        }
        return lo
    }

    /// The line and column of shown position `pos`.
    function _shownSpot(pos) {
        const line = _lineOf(_t.disp, pos)
        return { line: line, col: pos - _t.disp[line] }
    }

    function _sourceSpot(pos) {
        const line = _lineOf(_t.src, pos)
        return { line: line, col: pos - _t.src[line] }
    }

    // ---- helpers ----

    /// A line as the area can hold it: a no-break space is a space there, the end of a CRLF is
    /// no character.
    function _clean(line) {
        let text = line.text
        if (text.indexOf(" ") >= 0) {
            text = text.replace(/ /g, " ")
        }
        if (text.endsWith("\r")) {
            text = text.slice(0, -1)
            if (line.cols) {
                line.cols.splice(line.cols.length - 2, 1)
            }
        }
        return { text: text, runs: line.runs, cols: line.cols }
    }

    /// The runs of a line as the native look takes them: three numbers per stretch.
    function _flat(line) {
        const out = []
        for (const run of line.runs) {
            out.push(run.from, run.to, (run.bold ? 1 : 0) | (run.italic ? 2 : 0) | (run.underline ? 4 : 0))
        }
        return out
    }

    /// How far `a` and `b` agree: from `from` on to the first difference (`atEnd` false; the
    /// position), or from the end back to `from` (`atEnd`; the number of characters).
    function _same(a, b, from, atEnd) {
        const max = Math.min(a.length, b.length) - from
        let n = 0
        if (!atEnd) {
            while (n + _chunk <= max && a.substr(from + n, _chunk) === b.substr(from + n, _chunk)) {
                n += _chunk
            }
            while (n < max && a.charCodeAt(from + n) === b.charCodeAt(from + n)) {
                n++
            }
            return from + n
        }

        while (n + _chunk <= max && a.substr(a.length - n - _chunk, _chunk) === b.substr(b.length - n - _chunk, _chunk)) {
            n += _chunk
        }
        while (n < max && a.charCodeAt(a.length - 1 - n) === b.charCodeAt(b.length - 1 - n)) {
            n++
        }
        return n
    }

    function _isHigh(code) {
        return code >= 0xd800 && code < 0xdc00
    }

    function _isLow(code) {
        return code >= 0xdc00 && code < 0xe000
    }
}
