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
    property string undoStep: ""
    property string redoStep: ""
    property bool hasBackground: false
    property bool viewFree: false
    property int freeRevision: 0
    property int pictureRevision: 0
    // Not a property: reading one would bind the header to it, as the real call does not.
    readonly property var state: ({ look: "" })
    property int missingHeights: 0
    property string choicesJson: JSON.stringify({ object: [{ kind: "table", custom: false, group: "furniture" }, { kind: "sofa", custom: false, group: "furniture" }] })
    // The bound note of the selection, for `describe`.
    property bool noteBound: false
    function describe(key) { return JSON.stringify({ kind: "note", bound: noteBound }) }
    function changeSelected(what, arg) { return call("changeSelected", [what, arg]) }
    // What the bridge offers a label for: an object, a figure or a mark, not the grip of one.
    function menuInfo() { return JSON.stringify({ label: /^(object|figure|mark):/.test(selectedKey) }) }
    function labelSelected() { return call("labelSelected", []) }
    property var calls: []
    // The script line the selection is linked to (DB-63).
    property int lineNow: -1
    function scriptLine() { return lineNow }
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
    // The files a report can be written as (services/views.rs `Overview.report_formats`).
    readonly property var reportFormats: ["pdf", "csv", "ods", "xlsx"]
    // What the project view says of the screenplay's setup (services/views.rs `ScriptRow`).
    readonly property var papers: [{ id: "a4", label: "A4" }, { id: "a3", label: "A3" }, { id: "letter", label: "Letter" }]
    readonly property var script: ({ language: "de", words: "local", paper: "letter", contd_after_action: false,
                                     scene_numbering: "locked", first_scene: 1 })
    property string projectJson: JSON.stringify({ scenes: scenes, plans: plans, plan: plan, formats: formats, report_formats: reportFormats,
                                                  papers: papers, script_papers: ["a4", "letter"], script: script, script_versions: 1 })

    function call(name, args) {
        calls.push([name].concat(args))
        return true
    }
    function undo() { call("undo", []) }
    function redo() { call("redo", []) }
    function save() { return call("save", []) }
    function removeBackground() { call("removeBackground", []) }
    function exportFile(extension) { return "file:///work/Wartburg." + extension }
    function scriptFile() { return "file:///work/Wartburg.pdf" }
    function exportScript(paper, numbers, header, revisions, path) { return call("exportScript", [paper, numbers, header, revisions, path]) }
    function reportFile(report) { return "file:///work/" + report + ".pdf" }
    function exportReport(report, format, path) { return call("exportReport", [report, format, path]) }
    function exportTo(kind, order, paper, path) { return call("exportTo", [kind, order, paper, path]) }
    function selectScene(id) { return call("selectScene", [id]) }
    function selectPlan(id) { return call("selectPlan", [id]) }
    function clearStatus() {}
    function stepMark(direction) { call("stepMark", [direction]) }
    property string selectedKey: ""
    property string selectedShot: ""
    function stepView(step) { call("stepView", [step]) }
    function nudgeTime(what, delta) { return call("nudgeTime", [what, delta]) }
    property string curve: ""
    function canArm(what) { return what !== "axisTo" || selectedKey.indexOf("figure") === 0 }
    function arm(what) { return call("arm", [what]) }
    function wayCurve() { return curve }
    function setWayCurve(name) { curve = name; return call("setWayCurve", [name]) }
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
