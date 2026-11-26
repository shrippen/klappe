import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Compares two states of the book (DB-42, DB-44) in two columns: the old state left, the new
// one right, the head of each is its name ("from" a locked version, "to" a locked version or the
// book now). Both columns are one list, so they scroll together and the two halves of a scene
// stand level: lines that went are struck through on the left, lines that came are underlined on
// a tint on the right; a removed scene has a gap on the right, a new one on the left; a scene that
// moved says so in words in both columns. Up and Down, or a click, choose the scene. The session
// does the comparing; this only shows it.
//
//  Von [Drehfassung ▾]          Bis [Aktuelles Buch ▾]
//  ┌────────────────────────────┬────────────────────────────┐
//  │ 1 INT. SAAL  geändert      │ 1 INT. SAAL  geändert      │
//  │ ~~Er geht.~~               │ Sie geht.                  │
//  │ 3 INT. KELLER  gestrichen  │                            │
//  │                            │ 4 INT. DACH  neu           │
//  └────────────────────────────┴────────────────────────────┘
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

    /// The lines of that side: the removed ones of the old state, the added ones of the new.
    function sideLines(scene, side) {
        const change = side === "old" ? "removed" : "added"
        return scene.lines.filter(l => l.change === change)
    }

    onOpened: rows.forceActiveFocus()

    // One half of a scene: its head with what became of it, then the lines of its side.
    component Half: ColumnLayout {
        id: half
        required property var scene
        /// "old" or "new".
        required property string side
        readonly property bool here: dialog.present(scene, side)

        Layout.fillWidth: true
        Layout.alignment: Qt.AlignTop
        spacing: Kirigami.Units.smallSpacing
        Accessible.name: here ? "" : dialog.texts.t("version.absent")

        QQC2.Label {
            objectName: "half.head." + half.side
            Layout.fillWidth: true
            visible: half.here
            wrapMode: Text.WordWrap
            font.bold: true
            color: KanteStyle.strongTextColor
            text: (half.scene.number ? half.scene.number + " " : "") + dialog.heading(half.scene)
                + (half.scene.status !== "same" || half.scene.place === "moved" ? " · " + dialog.status(half.scene) : "")
        }

        Repeater {
            model: half.here ? dialog.sideLines(half.scene, half.side) : []
            delegate: Item {
                id: line
                required property var modelData
                readonly property bool added: modelData.change === "added"
                objectName: "line." + half.side
                Layout.fillWidth: true
                implicitHeight: words.implicitHeight
                Accessible.role: Accessible.StaticText
                Accessible.name: dialog.texts.t(added ? "version.lineAdded" : "version.lineRemoved") + " " + modelData.text

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
                    text: line.modelData.text
                    Accessible.ignored: true
                }
            }
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

                RowLayout {
                    id: pair
                    x: Kirigami.Units.smallSpacing
                    y: Kirigami.Units.smallSpacing
                    width: parent.width - bar.width - Kirigami.Units.smallSpacing * 2
                    spacing: dialog.columnGap
                    uniformCellSizes: true

                    Half { objectName: "half.old"; scene: block.modelData; side: "old" }
                    Half { objectName: "half.new"; scene: block.modelData; side: "new" }
                }
            }
        }
    }
}
