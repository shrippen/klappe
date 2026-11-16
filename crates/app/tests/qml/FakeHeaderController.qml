import QtQuick

// The controller as AppHeader sees it: what the header reads and the calls it makes, recorded
// in `calls` (tst_appheader.qml).
QtObject {
    property bool loaded: true
    property string title: "Wartburg"
    property string sceneId: "s12"
    property string status: ""
    property bool dirty: false
    property bool canUndo: true
    property bool canRedo: true
    property bool hasBackground: false
    property bool viewFree: false
    property int freeRevision: 0
    property int pictureRevision: 0
    // Not a property: reading one would bind the header to it, as the real call does not.
    readonly property var state: ({ look: "" })
    property int missingHeights: 0
    property string selectedKey: ""
    property string choicesJson: JSON.stringify({ object: [{ kind: "table", custom: false, group: "furniture" }, { kind: "sofa", custom: false, group: "furniture" }] })
    // The bound note of the selection, for `describe`.
    property bool noteBound: false
    function describe(key) { return JSON.stringify({ kind: "note", bound: noteBound }) }
    function changeSelected(what, arg) { return call("changeSelected", [what, arg]) }
    function labelSelected() { return call("labelSelected", []) }
    property var calls: []
    property var scenes: [
        { id: "s12", number: "12", heading: "INT. SÄNGERSAAL – TAG" },
        { id: "s13", number: "13", heading: "EXT. HOF – NACHT" }
    ]
    property var plans: [
        { id: "plan-a", name: "Plan A", shots: 5 },
        { id: "plan-b", name: "Plan B", shots: 0 }
    ]
    property string plan: "plan-a"
    // What the project view lists of the exports (services/views.rs `ExportRow`).
    readonly property var formats: [
        { id: "pdf", paper: true, extension: "pdf" },
        { id: "png", paper: true, extension: "png" },
        { id: "svg", paper: true, extension: "svg" },
        { id: "csv", paper: false, extension: "csv" },
        { id: "ods", paper: false, extension: "ods" },
        { id: "xlsx", paper: false, extension: "xlsx" },
        { id: "frames", paper: true, extension: null },
        { id: "view3d", paper: false, extension: "png" },
        { id: "camera", paper: false, extension: "png" }
    ]
    property string projectJson: JSON.stringify({ scenes: scenes, plans: plans, plan: plan, formats: formats })

    function call(name, args) {
        calls.push([name].concat(args))
        return true
    }
    function undo() { call("undo", []) }
    function redo() { call("redo", []) }
    function save() { return call("save", []) }
    function removeBackground() { call("removeBackground", []) }
    function exportFile(extension) { return "file:///work/Wartburg." + extension }
    function exportTo(kind, order, paper, path) { return call("exportTo", [kind, order, paper, path]) }
    function selectScene(id) { return call("selectScene", [id]) }
    function selectPlan(id) { return call("selectPlan", [id]) }
    function clearStatus() {}
    function stepMark(direction) { call("stepMark", [direction]) }
    function toggleView() { call("toggleView", []); viewFree = !viewFree }
    property bool refuseCamera: false
    function freeLook(command) {
        if (command !== "") {
            call("freeLook", [command])
            if (command === "camera" && refuseCamera) {
                return state.look
            }
            const on = state.look.split(" ").filter(w => w !== "")
            state.look = on.includes(command) ? on.filter(w => w !== command).join(" ") : on.concat([command]).join(" ")
            freeRevision++
        }
        return state.look
    }
    function nextMissingHeight() { return call("nextMissingHeight", []) }
}
