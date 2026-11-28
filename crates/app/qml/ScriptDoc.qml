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

    /// How the area shows the source: as the printed screenplay, or as Fountain as it stands.
    enum View { Shown, Source }
    property int view: ScriptDoc.View.Shown
    readonly property string _view: view === ScriptDoc.View.Source ? "source" : "shown"

    /// Where a change of the source comes from: typed in the area, which holds it already, or
    /// made by a part of the editor, which the area still lacks.
    enum Origin { Typed, Replaced }
    /// The end of a text a comparison starts from.
    enum Edge { Start, End }

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
    /// The emphasis for what is typed next, `{ at, bold, italic, underline }`, set at the cursor
    /// (`at`, a position of the area) by a key without a selection; null for none.
    property var _pending: null
    /// The marks of the source view, by the name of the emphasis.
    readonly property var _fountainMarks: ({ bold: "**", italic: "*", underline: "_" })

    /// What a line of the source hides at its start that is no forcing character, and the marks
    /// of emphasis, which a copy writes by their runs.
    readonly property var _forcingOutside: /[^.!@>~]/g
    readonly property var _emphasisMarks: /[*_]/g
    /// The hidden marks that end a line: a dual cue, a centered line.
    readonly property var _lineEnds: ["^", "<"]

    /// Text this long is compared in one go.
    readonly property int _chunk: 512

    Component.onCompleted: {
        area.textChanged.connect(_onArea)
        area.cursorPositionChanged.connect(_cursorMoved)
    }

    /// The source as it stands, also inside the handlers of an edit, where `source` may lag.
    function read() {
        return _source
    }

    /// Takes `text` from outside (an import, another project): no typing.
    function load(text) {
        loading = true
        _pending = null
        _source = text
        _lines = JSON.parse(controller.scriptShownJson(text, _view)).map(_clean)
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
        if (shown.cols) {
            return _t.src[line] + shown.cols[at]
        }
        // The end of a line is behind a return that the shown text does not have (CRLF).
        return at === shown.text.length ? _lineEnd(line) : _t.src[line] + at
    }

    /// Where the shown text of line `line` ends in the source: before the hidden ` ^` or `<`.
    function textEnd(line) {
        _tables()
        const shown = _lines[line]
        return shown.cols ? _t.src[line] + shown.cols[shown.text.length] : _lineEnd(line)
    }

    /// Where line `line` ends in the source, before its line break.
    function _lineEnd(line) {
        return line + 1 < _lines.length ? _t.src[line + 1] - 1 : _source.length
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

    /// The source of the selected text as closed Fountain: the marks of the emphasis the selection
    /// starts or ends in close it, so a copy of `ett` in `**fett**` is `**ett**`, and pasting it
    /// shows no stray mark. Empty without a selection.
    function selectedSource() {
        if (area.selectedText === "") {
            return ""
        }

        _tables()
        const start = area.selectionStart
        const end = area.selectionEnd
        return _leadOf(start) + _marksOf(_runAt(start), true)
            + _source.slice(toSource(start), toSourceEnd(end))
            + _marksOf(_runAt(end - 1), false) + _endOf(end)
    }

    /// The forcing character a selection starting at shown position `pos` takes along when `pos`
    /// is the start of its line: `@` of `@mcClane`, or a copy of the cue is an action.
    function _leadOf(pos) {
        const line = _lineOf(_t.disp, pos)
        const shown = _lines[line]
        if (pos !== _t.disp[line] || !shown.cols) {
            return ""
        }

        return _source.slice(_t.src[line], _t.src[line] + shown.cols[0]).replace(_forcingOutside, "")
    }

    /// The mark that ends the line (` ^`, `<`) a selection ending at shown position `end` takes
    /// along when it ends with the shown text of its line.
    function _endOf(end) {
        const line = _lineOf(_t.disp, end - 1)
        const shown = _lines[line]
        if (end - _t.disp[line] !== shown.text.length || !shown.cols) {
            return ""
        }

        const behind = _source.slice(_t.src[line] + shown.cols[shown.text.length], _lineEnd(line))
            .replace(_emphasisMarks, "")
        return _lineEnds.includes(behind.trim()) ? behind : ""
    }

    /// Takes the selection out of the text, as a deletion typed in the area is.
    function cutSelection() {
        if (area.selectedText !== "") {
            area.remove(area.selectionStart, area.selectionEnd)
        }
    }

    /// The emphasis the shown character at `pos` has, null for none or at the end of a line.
    function _runAt(pos) {
        if (pos < 0) {
            return null
        }

        const line = _lineOf(_t.disp, pos)
        const col = pos - _t.disp[line]
        return _lines[line].runs.find(run => run.from <= col && col < run.to) || null
    }

    /// The marks that open (`opening`) or close the emphasis `run`.
    function _marksOf(run, opening) {
        if (!run) {
            return ""
        }

        const stars = run.bold && run.italic ? "***" : run.bold ? "**" : run.italic ? "*" : ""
        const underline = run.underline ? "_" : ""
        return opening ? underline + stars : stars + underline
    }

    // ---- emphasis ----

    /// Shows the source in `view` (`ScriptDoc.View`): nothing is written, the cursor and the
    /// selection stay where they are in the source.
    function setView(next) {
        if (next === view) {
            return
        }

        const reversed = area.cursorPosition === area.selectionStart
        const from = selectionStart
        const to = selectionEnd
        const selected = area.selectedText !== ""
        const at = cursor

        view = next
        load(_source)
        if (!selected) {
            setCursor(at)
        } else if (reversed) {
            area.select(toDisplay(to), toDisplay(from))
        } else {
            area.select(toDisplay(from), toDisplay(to))
        }
    }

    /// The emphasis of the selection (all of it has it), or at the cursor that of what is typed
    /// next, `{ bold, italic, underline, heading }`; `heading`: bold is not for the writer to set.
    function markState() {
        _tables()
        const selected = area.selectedText !== ""
        const a = _sourceSpot(selected ? selectionStart : cursor)
        const b = selected ? _sourceSpot(selectionEnd) : a
        const state = JSON.parse(controller.scriptEmphasisJson(_source, a.line, a.col, b.line, b.col))
        if (selected || !_pending || _pending.at !== area.cursorPosition) {
            return state
        }

        state.bold = _pending.bold
        state.italic = _pending.italic
        state.underline = _pending.underline
        return state
    }

    /// Toggles `mark` ("bold", "italic", "underline"): for the selection, as one step of the
    /// history; at the cursor for what is typed next (the source view: the marks, in pairs).
    function toggleMark(mark) {
        if (area.selectedText !== "") {
            _toggleSelected(mark)
        } else if (view === ScriptDoc.View.Source) {
            _insertMarks(mark)
        } else {
            _toggleNext(mark)
        }
    }

    function _toggleSelected(mark) {
        _tables()
        const a = _sourceSpot(selectionStart)
        const b = _sourceSpot(selectionEnd)
        const done = controller.scriptToggleJson(_source, a.line, a.col, b.line, b.col, mark, _view)
        if (done === "") {
            return
        }

        // A step of its own: what was typed before and after is none of it.
        _seal()
        _take(JSON.parse(done), ScriptDoc.Origin.Replaced)
        _seal()
    }

    function _seal() {
        if (tools && tools.seal) {
            tools.seal()
        }
    }

    function _toggleNext(mark) {
        const state = markState()
        if (mark === "bold" && state.heading) {
            return
        }

        _pending = { at: area.cursorPosition, bold: state.bold, italic: state.italic,
                     underline: state.underline }
        _pending[mark] = !_pending[mark]
    }

    /// The marks of `mark` in pairs at the cursor, which stands between them.
    function _insertMarks(mark) {
        const at = cursor
        const marks = _fountainMarks[mark]

        replace(at, at, marks + marks)
        setCursor(at + marks.length)
    }

    /// The cursor moved: an emphasis chosen for the next typing is gone if it left its place.
    function _cursorMoved() {
        if (_pending) {
            Qt.callLater(_dropPending)
        }
    }

    function _dropPending() {
        if (_pending && _pending.at !== area.cursorPosition) {
            _pending = null
        }
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
        const done = controller.scriptReplaceJson(_source, a.line, a.col, b.line, b.col, text, _view)
        _take(JSON.parse(done), ScriptDoc.Origin.Replaced)
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
        let start = _same(old, now, 0, ScriptDoc.Edge.Start)
        if (start > 0 && _isHigh(old.charCodeAt(start - 1))) {
            start--
        }
        let end = _same(old, now, start, ScriptDoc.Edge.End)
        if (end > 0 && _isLow(old.charCodeAt(old.length - end))) {
            end--
        }
        const typed = now.slice(start, now.length - end)
        const from = _shownSpot(start)
        const to = _shownSpot(old.length - end)
        _text = now

        // A key before the typing set its emphasis, for this place only.
        const chosen = _pending && _pending.at === start && old.length - end === start ? _pending : null
        _pending = null
        const change = { line: from.line, from: from.col, endLine: to.line, to: to.col,
                         typed: typed, runs: _inherit(from, typed, chosen) }
        const done = controller.scriptEditJson(_source, JSON.stringify(change), _view)
        if (done === "") {
            return
        }
        _take(JSON.parse(done), ScriptDoc.Origin.Typed)
    }

    /// What `typed` has of emphasis: what a key chose for it (`chosen`), else that of the
    /// character before it, at the start of a line of the one behind. Several lines (a paste)
    /// bring none.
    function _inherit(from, typed, chosen) {
        if (typed === "" || typed.indexOf("\n") >= 0) {
            return []
        }

        const at = Math.max(from.col - 1, 0)
        const run = chosen || _lines[from.line].runs.find(r => r.from <= at && at < r.to)
        if (!run || !(run.bold || run.italic || run.underline)) {
            return []
        }
        return [{ from: 0, to: typed.length, bold: run.bold, italic: run.italic,
                  underline: run.underline }]
    }

    /// Makes the answer of the controller the state: the source, the lines, the drawn emphasis,
    /// the area and the history. `origin` says whether the area holds the change already.
    function _take(res, origin) {
        _pending = null
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
        _show(shownStart, origin === ScriptDoc.Origin.Typed ? shown.length : count, shown)
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
        const start = _same(before, after, 0, ScriptDoc.Edge.Start)
        const end = _same(before, after, start, ScriptDoc.Edge.End)

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

        const head = _same(current, want, 0, ScriptDoc.Edge.Start)
        const tail = _same(current, want, head, ScriptDoc.Edge.End)
        const from = start + head
        const to = start + current.length - tail
        const put = want.slice(head, want.length - tail)
        // Where a position goes: one before the change stays, one behind it moves by what the
        // change adds, one inside has no place (-1).
        const delta = put.length - (to - from)
        const where = pos => pos <= from ? pos : pos >= to ? pos + delta : -1
        const cursor = area.cursorPosition
        const selected = area.selectionStart !== area.selectionEnd
        const reversed = cursor === area.selectionStart
        const first = where(area.selectionStart)
        const last = where(area.selectionEnd)
        let moved = where(cursor)
        if (moved < 0) {
            moved = Math.max(start, start + want.length - (end - cursor))
        }

        _busy = true
        area.remove(from, to)
        area.insert(from, put)
        _text = _text.slice(0, from) + put + _text.slice(to)
        if (selected && first >= 0 && last >= 0) {
            // The selection keeps its direction: the cursor is at the end it was.
            area.select(reversed ? last : first, reversed ? first : last)
        } else {
            area.cursorPosition = moved
        }
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

    /// How far `a` and `b` agree: from `from` on to the first difference (`Edge.Start`; the
    /// position), or from the end back to `from` (`Edge.End`; the number of characters).
    function _same(a, b, from, edge) {
        const max = Math.min(a.length, b.length) - from
        let n = 0
        if (edge === ScriptDoc.Edge.Start) {
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
