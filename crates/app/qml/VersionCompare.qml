import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Compares two states of the book (DB-42, DB-44) in two columns: the old state left, the new
// one right, the head of each is its name ("from" a locked version, "to" a locked version or the
// book now). Both columns are one list, so they scroll together and the two halves of a scene
// stand level: each head left is the old one, right the new one. Below it every change (a run
// of removed lines and the added lines that replace them) is a row of its own: lines that went
// are struck through on the left, lines that came are underlined on a tint on the right, and a
// row is as high as the taller side, so the next change starts level again. A removed scene has
// a gap on the right, a new one on the left; a scene that moved says so in words in both
// columns. Up and Down, or a click, choose the scene. The session does the comparing; this only
// shows it.
//
//  Von [Drehfassung ▾]               Bis [Aktuelles Buch ▾]
//  ┌─────────────────────────────────┬─────────────────────────────────┐
//  │ 1 INT. SAAL - NACHT · geändert  │ 1 INT. SAAL - TAG · geändert    │
//  │ ~~Er geht.~~                    │ Sie geht.                       │
//  │                                 │ Sie bleibt.                     │
//  │ ~~Er bleibt.~~                  │ Sie schläft.                    │  (next change, level again)
//  │ 3 INT. KELLER · gestrichen      │                                 │
//  │                                 │ 4 INT. DACH · neu               │
//  └─────────────────────────────────┴─────────────────────────────────┘
KanteDialog {
    id: dialog

    required property var texts
    required property var controller
    /// The locked versions, `{ id, name, … }`.
    property var versions: []

    /// The choices of "from": the versions; of "to": the versions and the book now (id empty).
    readonly property var fromChoices: versions.map(v => ({ id: v.id, text: v.name }))
    readonly property var toChoices: fromChoices.concat([{ id: "", text: texts.t("version.now") }])
    /// What differs, `compareJson`; empty without two states to compare.
    property var scenes: []
    readonly property alias rows: rows
    readonly property int columnGap: Kirigami.Units.largeSpacing

    parent: QQC2.Overlay.overlay
    anchors.centerIn: parent
    modal: true
    size: KanteDialog.Size.Large
    title: texts.t("version.compareTitle")
    standardButtons: QQC2.Dialog.Close
    rejectText: texts.t("version.close")

    /// Opens with `from` (a version's id, empty for the last one) against the book now.
    function show(from) {
        const at = fromChoices.findIndex(c => c.id === from)
        fromBox.currentIndex = at >= 0 ? at : fromChoices.length - 1
        toBox.currentIndex = toChoices.length - 1
        open()
        compare()
    }

    /// The words after "from" and "to" as they stand now.
    function compare() {
        const from = fromChoices[fromBox.currentIndex]
        const to = toChoices[toBox.currentIndex]
        if (!from || !to) {
            scenes = []
            return
        }

        scenes = JSON.parse(controller.compareJson(from.id, to.id))
        // The first scene that changed shows first; all unchanged shows the first.
        const first = scenes.findIndex(s => s.status !== "same" || s.place === "moved")
        rows.currentIndex = scenes.length === 0 ? -1 : Math.max(first, 0)
    }

    // What became of a scene, in words: "changed, moved"; "unchanged" when nothing did.
    function status(scene) {
        const parts = []
        if (scene.status !== "same") {
            parts.push(texts.t("compare." + scene.status))
        }
        if (scene.place === "moved") {
            parts.push(texts.t("compare.moved"))
        }
        return parts.length > 0 ? parts.join(", ") : texts.t("compare.same")
    }

    function heading(scene) {
        return scene.heading || texts.t("version.noHeading")
    }

    /// Whether the scene has a half on that side: a new one has none in the old state, a
    /// removed one none in the new.
    function present(scene, side) {
        return scene.status !== (side === "old" ? "new" : "removed")
    }

    /// The changes of a scene, each `{ removed: [text], added: [text] }`: the lines come in text
    /// order, the removed lines of a change right above the added ones that replace it, so a
    /// removed line after an added one starts the next change.
    function hunks(scene) {
        const out = []
        let at = null
        for (const l of scene.lines) {
            const removed = l.change === "removed"
            if (at === null || (removed && at.added.length > 0)) {
                at = { removed: [], added: [] }
                out.push(at)
            }
            (removed ? at.removed : at.added).push(l.text)
        }
        return out
    }

    /// The head of a scene on that side: the number and the heading of that state, then what
    /// became of it; empty on the side where the scene is not.
    function head(scene, side) {
        if (!present(scene, side)) {
            return ""
        }
        const old = side === "old"
        const number = old ? (scene.oldNumber ?? scene.number) : scene.number
        const heading = old ? (scene.oldHeading ?? scene.heading) : scene.heading
        const moved = scene.status !== "same" || scene.place === "moved"
        return (number ? number + " " : "") + (heading || texts.t("version.noHeading"))
            + (moved ? " · " + status(scene) : "")
    }

    onOpened: rows.forceActiveFocus()

    // One line of a change: struck through when removed, underlined on a tint when added.
    component Change: Item {
        id: line
        required property string text
        required property bool added
        objectName: "line." + (added ? "new" : "old")
        Layout.fillWidth: true
        implicitHeight: words.implicitHeight
        Accessible.role: Accessible.StaticText
        Accessible.name: dialog.texts.t(added ? "version.lineAdded" : "version.lineRemoved") + " " + text

        Rectangle {
            anchors.fill: parent
            visible: line.added
            color: KanteStyle.tintHighlightColor
        }
        QQC2.Label {
            id: words
            width: parent.width
            wrapMode: Text.WordWrap
            font.family: KanteStyle.monoFamily
            font.strikeout: !line.added
            font.underline: line.added
            color: line.added ? KanteStyle.strongTextColor : KanteStyle.mutedTextColor
            text: line.text
            Accessible.ignored: true
        }
    }

    ColumnLayout {
        width: parent ? parent.width : implicitWidth
        spacing: Kirigami.Units.smallSpacing

        QQC2.Label {
            objectName: "noVersions"
            Layout.fillWidth: true
            visible: dialog.versions.length === 0
            wrapMode: Text.WordWrap
            color: KanteStyle.mutedTextColor
            text: dialog.texts.t("versions.none")
        }

        // The head of each column: the state it shows.
        RowLayout {
            Layout.fillWidth: true
            visible: dialog.versions.length > 0
            spacing: dialog.columnGap
            uniformCellSizes: true

            RowLayout {
                Layout.fillWidth: true
                KanteSectionLabel { text: dialog.texts.t("version.from") }
                KanteComboBox {
                    id: fromBox
                    objectName: "fromBox"
                    Layout.fillWidth: true
                    Accessible.name: dialog.texts.t("version.from")
                    textRole: "text"
                    model: dialog.fromChoices
                    onActivated: dialog.compare()
                }
            }
            RowLayout {
                Layout.fillWidth: true
                KanteSectionLabel { text: dialog.texts.t("version.to") }
                KanteComboBox {
                    id: toBox
                    objectName: "toBox"
                    Layout.fillWidth: true
                    Accessible.name: dialog.texts.t("version.to")
                    textRole: "text"
                    model: dialog.toChoices
                    onActivated: dialog.compare()
                }
            }
        }

        ListView {
            id: rows
            objectName: "compareRows"
            Layout.fillWidth: true
            Layout.preferredHeight: Kirigami.Units.gridUnit * 22
            visible: dialog.versions.length > 0
            clip: true
            model: dialog.scenes
            keyNavigationEnabled: true
            spacing: Kirigami.Units.smallSpacing
            Accessible.name: dialog.texts.t("version.scenes")
            QQC2.ScrollBar.vertical: QQC2.ScrollBar { id: bar }

            Kirigami.PlaceholderMessage {
                objectName: "compareNone"
                anchors.centerIn: parent
                width: parent.width - Kirigami.Units.gridUnit * 2
                visible: rows.count === 0
                text: dialog.texts.t("version.noScenes")
            }

            delegate: Item {
                id: block
                required property var modelData
                required property int index
                objectName: "compare." + index
                readonly property string meta: dialog.status(modelData)
                width: ListView.view.width
                implicitHeight: pair.implicitHeight + Kirigami.Units.smallSpacing * 2
                opacity: modelData.status === "same" && modelData.place === "kept" ? KanteStyle.dimOpacity : 1
                Accessible.role: Accessible.ListItem
                Accessible.selected: ListView.isCurrentItem
                Accessible.name: dialog.texts.t("version.sceneRow", {
                    number: modelData.number ?? "", heading: dialog.heading(modelData), status: meta })

                Rectangle {
                    anchors.fill: parent
                    visible: block.ListView.isCurrentItem
                    color: KanteStyle.selectionColor
                }
                TapHandler { onTapped: block.ListView.view.currentIndex = block.index }

                ColumnLayout {
                    id: pair
                    x: Kirigami.Units.smallSpacing
                    y: Kirigami.Units.smallSpacing
                    width: parent.width - bar.width - Kirigami.Units.smallSpacing * 2
                    spacing: Kirigami.Units.smallSpacing

                    // The heads: a side without the scene keeps its cell, empty.
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: dialog.columnGap
                        uniformCellSizes: true

                        Repeater {
                            model: ["old", "new"]
                            delegate: QQC2.Label {
                                required property string modelData
                                objectName: "head." + modelData
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignTop
                                wrapMode: Text.WordWrap
                                font.bold: true
                                color: KanteStyle.strongTextColor
                                text: dialog.head(block.modelData, modelData)
                                Accessible.name: text !== "" ? text : dialog.texts.t("version.absent")
                            }
                        }
                    }

                    // Every change one row, both sides as high as the taller.
                    Repeater {
                        model: dialog.hunks(block.modelData)
                        delegate: RowLayout {
                            required property var modelData
                            Layout.fillWidth: true
                            spacing: dialog.columnGap
                            uniformCellSizes: true

                            ColumnLayout {
                                objectName: "half.old"
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignTop
                                Repeater {
                                    model: modelData.removed
                                    delegate: Change { required property string modelData; text: modelData; added: false }
                                }
                            }
                            ColumnLayout {
                                objectName: "half.new"
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignTop
                                Repeater {
                                    model: modelData.added
                                    delegate: Change { required property string modelData; text: modelData; added: true }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
