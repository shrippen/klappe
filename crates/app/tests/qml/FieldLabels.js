.pragma library

// The visible label texts under `item`, for tests that every number field is named.
// A text field is not entered: the greyed placeholder inside it is no label, and it
// disappears once the field has a value (Q1-03, Q2-03).
function of(item) {
    const found = []
    const walk = node => {
        for (const child of node.children) {
            if (child.inputMethodHints !== undefined) {
                continue
            }
            if (child.visible && child.text !== undefined && child.text !== "") {
                found.push(child.text)
            }
            walk(child)
        }
    }
    walk(item)
    return found
}
