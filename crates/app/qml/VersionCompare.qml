import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Compares two states of the book (DB-42, DB-44): "from" a locked version, "to" a locked version
// or the book now. On the left the scenes of the comparison with what became of each, as a
// word (new, removed, changed, moved), on the right the lines of the chosen scene: what is new
// is underlined on a tint, what is removed is struck through. Up and Down in the list, or a
// click, show the next scene. The session does the comparing; this only shows it.
//
//  Von [Drehfassung ▾]  Bis [Jetzt ▾]
//  ┌ Szenen ─────────────┬──────────────────────────┐
//  │ 1 INT. SAAL  neu    │ INT. SAAL - TAG          │
//  │ 2 EXT. HOF  geändert│ ~~Er geht.~~             │
//  │ 3 INT. KELLER       │ Sie geht.                │
//  └─────────────────────┴──────────────────────────┘
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
    readonly property var scene: rows.currentIndex >= 0 && rows.currentIndex < scenes.length
        ? scenes[rows.currentIndex] : null
    readonly property alias rows: rows

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

    onOpened: rows.forceActiveFocus()

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

        RowLayout {
            Layout.fillWidth: true
            visible: dialog.versions.length > 0
            spacing: Kirigami.Units.largeSpacing

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

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: Kirigami.Units.gridUnit * 22
            visible: dialog.versions.length > 0
            spacing: Kirigami.Units.smallSpacing

            ListView {
                id: rows
                objectName: "compareRows"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 18
                Layout.fillHeight: true
                clip: true
                model: dialog.scenes
                keyNavigationEnabled: true
                highlightFollowsCurrentItem: true
                Accessible.name: dialog.texts.t("version.scenes")
                QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
                delegate: KanteListRow {
                    required property var modelData
                    required property int index
                    objectName: "compare." + index
                    width: ListView.view.width
                    density: KanteListRow.Density.Compact
                    opacity: modelData.status === "same" && modelData.place === "kept" ? KanteStyle.dimOpacity : 1
                    leadingText: modelData.number ?? ""
                    leadingWidth: Kirigami.Units.gridUnit * 2
                    text: dialog.heading(modelData)
                    meta: dialog.status(modelData)
                    selected: ListView.isCurrentItem
                    Accessible.description: dialog.texts.t("version.sceneRow", {
                        number: modelData.number ?? "", heading: dialog.heading(modelData), status: dialog.status(modelData) })
                    onClicked: ListView.view.currentIndex = index
                }
            }

            Kirigami.Separator { Layout.fillHeight: true }

            QQC2.ScrollView {
                id: textView
                objectName: "compareView"
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                Accessible.name: dialog.texts.t("version.sceneText")

                ColumnLayout {
                    objectName: "compareText"
                    // The whole column, not the width the text would have on one line.
                    width: textView.availableWidth
                    spacing: Kirigami.Units.smallSpacing

                    QQC2.Label {
                        objectName: "compareHeading"
                        Layout.fillWidth: true
                        visible: dialog.scene !== null
                        wrapMode: Text.WordWrap
                        font.bold: true
                        color: KanteStyle.strongTextColor
                        text: dialog.scene ? dialog.heading(dialog.scene) : ""
                    }
                    QQC2.Label {
                        objectName: "compareNoChange"
                        Layout.fillWidth: true
                        visible: dialog.scene !== null && dialog.scene.lines.length === 0
                        wrapMode: Text.WordWrap
                        color: KanteStyle.mutedTextColor
                        text: dialog.texts.t("version.noChange")
                    }

                    Repeater {
                        model: dialog.scene ? dialog.scene.lines : []
                        delegate: Item {
                            id: line
                            required property var modelData
                            readonly property bool added: modelData.change === "added"
                            objectName: "line." + (added ? "added" : "removed")
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
            }
        }
    }
}
