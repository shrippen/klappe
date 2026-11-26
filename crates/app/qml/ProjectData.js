.pragma library

// The project overview (`projectJson` of the bridge) parsed, for every page that reads it.
// Pages call `of(controller.projectJson)` in a binding: the binding follows the property, the
// text is parsed once per change, not once per page, and every page gets the same value.

// What a page sees without a project: every list the pages read is there, and empty.
const NONE = Object.freeze({
    scenes: [], plans: [], plan: "", characters: [], places: [], locations: [],
    numberings: [], numbering: ""
})

let lastJson = ""
let lastValue = NONE

function of(json) {
    if (!json) {
        return NONE
    }
    if (json !== lastJson) {
        lastValue = JSON.parse(json)
        lastJson = json
    }
    return lastValue
}
