.pragma library

// Stands in for the controller's projection of the screenplay (`scriptShownJson`,
// `scriptEditJson`, `scriptReplaceJson`): a text without emphasis, where the shown text is the
// source. The rules are `script/`'s, tested there.

function shown(source) {
    return JSON.stringify(source.split("\n").map(text => ({ text: text, runs: [], cols: null })))
}

// `source` with the text from `line`, `from` to `endLine`, `to` replaced by `typed`.
function replace(source, line, from, endLine, to, typed) {
    const old = source.split("\n")
    const at = (n, col) => old.slice(0, n).reduce((sum, l) => sum + l.length + 1, 0) + col
    const next = source.slice(0, at(line, from)) + typed + source.slice(at(endLine, to))
    const lines = next.split("\n")
    const count = lines.length - old.length + endLine - line + 1

    return JSON.stringify({
        source: next, from: line, to: endLine + 1,
        shown: lines.slice(line, line + count).map(text => ({ text: text, runs: [], cols: null }))
    })
}

function edit(source, change) {
    const c = JSON.parse(change)
    return replace(source, c.line, c.from, c.endLine === undefined ? c.line : c.endLine, c.to, c.typed)
}
