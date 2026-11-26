import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-13: the list of every scene of the project, the same in each module. A row shows the number,
// the heading and small markers: its length in pages (`1 3/8`), T the scene is in the screenplay, P the number of its plans,
// S the number of its shots. A scene that is not in the screenplay is dimmed and says so. The
// scene on view is highlighted; a click shows it (`controller.selectScene`) and then tells the
// module with `picked`, so the screenplay view can move to its heading. The field above filters
// by number or heading.
//
// With an `outline` (DB-21, the screenplay view) the rows form a tree: a section row has an arrow
// (click, or Left and Right) that folds what is inside it and the total of its scenes' lengths;
// a synopsis is the pale second line of its scene or section; Alt+Up and Alt+Down ask to move the
// scene or the section with its block (`moveRequested`). Scenes without text stay below the tree.
// A filter shows the flat list. What is folded is remembered by level and title, as in the terminal.
//
//  ┌ Szenen filtern ───────────┐
//  │ ▾ Akt I             1 3/8 │
//  │   Es beginnt.             │
//  │ 1   INT. SAAL  T P1 S5    │
//  │ 2   EXT. HOF   T          │
//  │ ▸ Akt II            2     │
//  │ 3  INT. KELLER P2 S7      │  (dimmed, "nicht im Buch")
//  └───────────────────────────┘
ColumnLayout {
    id: list

    required property var texts
    required property var controller
    /// The lengths are of an older text than the one typed: shown pale (DN-02).
    property bool lengthsStale: false
    /// The outline of the text (`scriptOutlineJson`, parsed); empty gives the flat list.
    property var outline: []
    /// What is folded, by `fold` key: level and title of the section.
    property var collapsed: ({})
    /// A row was clicked and its scene is on view: `{ id, number, heading, inBook, plans, shots }`;
    /// for a section row only `{ line }`, the line of its heading.
    signal picked(var scene)
    /// Alt+Up or Alt+Down on the scene or section whose heading is at text line `line`; `way` is
    /// `up` or `down`. The view moves the text, then calls `focusAt`.
    signal moveRequested(int line, string way)

    readonly property var scenes: controller.projectJson ? JSON.parse(controller.projectJson).scenes : []
    readonly property string _needle: filter.text.trim().toLowerCase()
    readonly property var shown: scenes.filter(s => _needle === ""
        || s.number.toLowerCase().includes(_needle) || s.heading.toLowerCase().includes(_needle))

    readonly property bool tree: outline.length > 0 && _needle === ""
    readonly property var items: tree ? treeItems() : shown.map(s => ({ kind: "scene", depth: 0, scene: s }))

    spacing: Kirigami.Units.smallSpacing

    // What a section is remembered folded by: its level and title. Typing above it does not move
    // it; renaming it unfolds it.
    function foldKey(entry) {
        return entry.level + entry.text
    }

    // The row of the project for the scene whose heading is at text line `line`. Without one
    // (the register does not hold it yet) a row of the outline's own.
    function sceneAt(entry, byLine) {
        return byLine[entry.line] ?? { id: "", number: entry.number ?? "", heading: entry.text,
                                       inBook: true, plans: 0, shots: 0, line: entry.line, length: entry.length }
    }

    // The rows of the tree: the outline's sections and scenes, what a folded section holds left
    // out, a synopsis put on the entry above it; the scenes the text does not have below.
    function treeItems() {
        const byLine = {}
        for (const s of scenes) {
            if (typeof s.line === "number") {
                byLine[s.line] = s
            }
        }

        const out = []
        let hiddenTo = -1
        let above = null
        for (const e of outline) {
            if (e.line < hiddenTo) {
                above = null
                continue
            }
            if (e.kind === "synopsis") {
                if (above) {
                    above.synopsis = above.synopsis === "" ? e.text : above.synopsis + " " + e.text
                }
                continue
            }

            const item = { kind: e.kind, depth: e.depth, line: e.line, text: e.text, length: e.length, synopsis: "" }
            if (e.kind === "section") {
                item.key = foldKey(e)
                item.folded = collapsed[item.key] === true
                hiddenTo = item.folded ? e.end : hiddenTo
            } else {
                item.scene = sceneAt(e, byLine)
            }
            out.push(item)
            above = item
        }

        return out.concat(scenes.filter(s => !s.inBook).map(s => ({ kind: "scene", depth: 0, scene: s })))
    }

    // What the row's meta figure holds: "T P1 S5", each part only when it applies.
    function marks(scene) {
        const parts = []
        if (scene.inBook) {
            parts.push(texts.t("scenelist.markText"))
        }
        if (scene.plans > 0) {
            parts.push(texts.t("scenelist.markPlan") + scene.plans)
        }
        if (scene.shots > 0) {
            parts.push(texts.t("scenelist.markShots") + scene.shots)
        }
        return parts.join(" ")
    }

    // What a screen reader says of a row: all the row shows, the second line and the markers too.
    function describe(scene) {
        const text = scene.heading !== "" ? scene.number + " " + scene.heading : scene.number
        const book = texts.t(scene.inBook ? "scenelist.inBook" : "scene.notInBook")

        const all = [text, book, texts.t("scenelist.marks", { plans: scene.plans, shots: scene.shots })]
        if (scene.length) {
            all.push(texts.t("scenelist.length", { length: scene.length }))
        }
        return all.join(", ")
    }

    // What a screen reader says of a section row.
    function describeSection(item) {
        const all = [item.text, texts.t("scenelist.section"),
                     texts.t(item.folded ? "scenelist.closed" : "scenelist.open")]
        if (item.length !== "") {
            all.push(texts.t("scenelist.length", { length: item.length }))
        }
        return all.join(", ")
    }

    // Folds or unfolds the section row `item`; the focus stays on it.
    function setFolded(item, folded) {
        const next = Object.assign({}, collapsed)
        if (folded) {
            next[item.key] = true
        } else {
            delete next[item.key]
        }
        collapsed = next
        focusAt(item.line)
    }

    // Left and Right on a row: Right opens a folded section, Left folds an open one; Left on
    // anything else goes to the section around it, as in a tree.
    function foldKeyed(item, index, way) {
        if (!tree) {
            return
        }
        const section = item.kind === "section"
        if (section && way === "open" && item.folded) {
            setFolded(item, false)
        } else if (section && way === "close" && !item.folded) {
            setFolded(item, true)
        } else if (way === "close") {
            const parent = items.slice(0, index).reverse().find(i => i.kind === "section" && i.depth < item.depth)
            if (parent) {
                focusAt(parent.line)
            }
        }
    }

    // Up and Down: the focus steps; with Alt the scene or section moves in the book (DB-21).
    function arrowKeyed(event, item, index, by) {
        event.accepted = true
        if (!(event.modifiers & Qt.AltModifier)) {
            step(index, by)
            return
        }
        if (tree && typeof item.line === "number") {
            moveRequested(item.line, by < 0 ? "up" : "down")
        }
    }

    // The focus on the row of the heading at text line `line`, after the rows were made anew.
    function focusAt(line) {
        const at = items.findIndex(i => i.line === line)
        if (at < 0) {
            return
        }
        rows.positionViewAtIndex(at, ListView.Contain)
        rows.forceLayout()
        rows.itemAtIndex(at)?.forceActiveFocus()
    }

    // Arrow keys: the focus moves from row to row and the view follows; up from the first row
    // goes back to the filter field.
    function step(from, by) {
        const to = from + by
        if (to < 0) {
            filter.forceActiveFocus()
            return
        }
        if (to >= rows.count) {
            return
        }
        rows.positionViewAtIndex(to, ListView.Contain)
        // The row may not be made yet: a view creates its rows on its next layout.
        rows.forceLayout()
        rows.itemAtIndex(to).forceActiveFocus()
    }

    KanteTextField {
        id: filter
        objectName: "sceneFilter"
        visible: list.scenes.length > 0
        Layout.fillWidth: true
        placeholderText: list.texts.t("scenelist.filter")
        Accessible.name: list.texts.t("scenelist.filter")
        Keys.onDownPressed: list.step(-1, 1)
    }

    QQC2.Label {
        objectName: "noScenes"
        visible: list.scenes.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("screenplay.noScenes")
    }

    QQC2.Label {
        objectName: "noMatch"
        visible: list.scenes.length > 0 && list.shown.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("scenelist.noMatch")
    }

    ListView {
        id: rows
        objectName: "sceneRows"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: list.items
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        delegate: KanteListRow {
            id: row
            required property var modelData
            required property int index
            readonly property bool section: modelData.kind === "section"
            readonly property var scene: modelData.scene ?? ({})
            objectName: section ? "section." + modelData.line : "scene." + scene.id
            width: ListView.view.width
            density: KanteListRow.Density.Compact
            opacity: section || scene.inBook ? 1 : KanteStyle.dimOpacity
            leadingText: section ? "" : scene.number
            leadingWidth: Kirigami.Units.gridUnit * 2
            text: section ? (modelData.text !== "" ? modelData.text : list.texts.t("scenelist.section"))
                          : (scene.heading !== "" ? scene.heading : list.texts.t("scene.notInBook"))
            subtitle: modelData.synopsis ? modelData.synopsis
                      : (!section && !scene.inBook && scene.heading !== "" ? list.texts.t("scene.notInBook") : "")
            meta: section ? "" : list.marks(scene)
            // The indent of the tree, and a section's arrow.
            Row {
                Item {
                    width: modelData.depth * Kirigami.Units.largeSpacing
                    height: 1
                }
                // The terminal's marks: down open, right folded.
                Text {
                    id: arrow
                    objectName: "fold." + modelData.line
                    visible: row.section
                    width: Kirigami.Units.iconSizes.small
                    horizontalAlignment: Text.AlignHCenter
                    text: modelData.folded ? "\u25B8" : "\u25BE"
                    font: KanteStyle.defaultFont
                    color: KanteStyle.mutedTextColor
                    HoverHandler {
                        id: overArrow
                        objectName: "overArrow." + modelData.line
                    }
                }
            }
            // Its length on the page (DB-32), as in the terminal: `1 3/8`; a section's the total of
            // its scenes; pale while out of date.
            trailing: [
                QQC2.Label {
                    objectName: "length." + (row.section ? "section." + modelData.line : row.scene.id)
                    readonly property string figure: row.section ? modelData.length : (row.scene.length ?? "")
                    visible: figure !== ""
                    text: figure
                    color: KanteStyle.mutedTextColor
                    opacity: list.lengthsStale ? KanteStyle.dimOpacity : 1
                }
            ]
            selected: !section && scene.id !== "" && scene.id === list.controller.sceneId
            Accessible.description: section ? list.describeSection(modelData) : list.describe(scene)
            Keys.onDownPressed: event => list.arrowKeyed(event, modelData, index, 1)
            Keys.onUpPressed: event => list.arrowKeyed(event, modelData, index, -1)
            Keys.onLeftPressed: list.foldKeyed(modelData, index, "close")
            Keys.onRightPressed: list.foldKeyed(modelData, index, "open")
            onClicked: {
                // The arrow folds; the rest of a section row shows its heading in the text.
                if (section && overArrow.hovered) {
                    list.setFolded(modelData, !modelData.folded)
                    return
                }
                if (section) {
                    list.picked({ line: modelData.line })
                    return
                }
                if (scene.id !== "") {
                    list.controller.selectScene(scene.id)
                }
                list.picked(scene)
            }
        }
    }
}
