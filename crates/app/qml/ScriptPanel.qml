import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The scene's script with chips to the marks and shots its lines are linked to (E-14,
// design B). The line the playback is at is marked with a bar, not by colour alone. The
// scene's heading and the names above the dialogue open a card (D-08) with what the scene or
// the character is and where it occurs.
//
// DB-63, to the plan: a chip (Tab to it, Enter) selects its mark or shot; Enter on a line
// selected with the arrow keys does the same for the line's first chip; the button at the
// scene's head asks for the plan. From the plan: `showLine` and `showHead` bring a line or the
// head into view with the focus on it.
Item {
    id: panel

    required property var texts
    required property var controller
    property var lines: []
    // The scene on view: { id, number, heading } or null.
    property var scene: null
    signal chipClicked(string kind, string target, int index, real at)
    // The head's button: show the scene's plan.
    signal planRequested()
    // The card's button: change the scene or character in the project dialog.
    signal editRequested(string kind, string id)
    // The empty column's buttons: load a screenplay, or open the screenplay view.
    signal importRequested()
    signal screenplayRequested()

    // Why the column is empty (D-17): "book" no screenplay at all, "number" none of its scenes
    // carries this scene's number, "" otherwise (the scene is there, without text).
    readonly property string emptyCause: {
        if (lines.length > 0 || scene === null) {
            return ""
        }
        if ((controller.scriptText || "").trim() === "") {
            return "book"
        }
        const outline = controller.outlineJson ? JSON.parse(controller.outlineJson) : []
        const found = outline.some(e => e.kind === "scene" && e.number === scene.number)
        return scene.number !== "" && !found ? "number" : ""
    }

    // The line `index` in view, the arrow keys' current line, with the focus on the list.
    function showLine(index) {
        list.currentIndex = index
        list.positionViewAtIndex(index, ListView.Contain)
        list.forceActiveFocus()
    }

    // The scene's head with the focus on it.
    function showHead() {
        sceneHead.forceActiveFocus()
    }

    // Enter on a line: its first chip's link is shown on the plan.
    function activateLine(index) {
        const chips = index >= 0 && index < lines.length ? lines[index].chips : []
        if (chips.length > 0) {
            chipClicked(chips[0].kind, chips[0].target, chips[0].index, chips[0].at_s)
        }
    }

    RefCard {
        id: card
        texts: panel.texts
        controller: panel.controller
        onEditRequested: (kind, id) => panel.editRequested(kind, id)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            KanteHeading {
                Layout.fillWidth: true
                level: 4
                text: panel.texts.t("script.title")
            }
            KanteSectionLabel { text: panel.texts.t("script.page") }
        }

        RowLayout {
            visible: panel.scene !== null
            spacing: Kirigami.Units.smallSpacing

            QQC2.Label {
                id: sceneHead
                objectName: "sceneHead"
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                font.family: KanteStyle.monoFamily
                font.underline: headHover.hovered
                color: KanteStyle.strongTextColor
                text: panel.scene ? (panel.scene.number + " " + panel.scene.heading).trim() : ""
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: text
                // The focus shows as a frame, like the line list's.
                background: Rectangle {
                    color: "transparent"
                    border.width: sceneHead.activeFocus ? KanteStyle.borderWidth : 0
                    border.color: KanteStyle.focusColor
                }
                HoverHandler { id: headHover; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: card.showAt("scene", panel.scene.id, sceneHead) }
                Keys.onReturnPressed: card.showAt("scene", panel.scene.id, sceneHead)
            }
            KanteToolButton {
                objectName: "toPlan"
                Layout.alignment: Qt.AlignTop
                text: panel.texts.t("script.toPlan")
                onClicked: panel.planRequested()
            }
        }

        QQC2.Label {
            objectName: "emptyReason"
            Layout.fillWidth: true
            visible: panel.lines.length === 0
            wrapMode: Text.Wrap
            color: KanteStyle.mutedTextColor
            text: panel.emptyCause === "book" ? panel.texts.t("script.noBook")
                : panel.emptyCause === "number" ? panel.texts.t("script.noNumber", { number: panel.scene.number })
                : panel.texts.t("script.empty")
        }
        KanteButton {
            objectName: "loadScreenplay"
            visible: panel.emptyCause === "book"
            text: panel.texts.t("script.import")
            onClicked: panel.importRequested()
        }
        KanteButton {
            objectName: "openScreenplay"
            visible: panel.emptyCause === "number"
            text: panel.texts.t("script.open")
            onClicked: panel.screenplayRequested()
        }

        ListView {
            id: list
            objectName: "lineList"
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: Kirigami.Units.smallSpacing
            model: panel.lines
            // Arrow keys step through the lines once the list has the focus (Tab).
            keyNavigationEnabled: true
            activeFocusOnTab: true
            Accessible.role: Accessible.List
            Accessible.name: panel.texts.t("script.title")
            Keys.onReturnPressed: panel.activateLine(currentIndex)

            delegate: Rectangle {
                id: row
                required property var modelData
                width: ListView.view.width
                height: content.implicitHeight + 2 * Kirigami.Units.smallSpacing
                color: modelData.current ? KanteStyle.selectionColor : "transparent"
                // The line the arrow keys are on: a frame, not only a colour.
                border.width: ListView.isCurrentItem && list.activeFocus ? KanteStyle.borderWidth : 0
                border.color: KanteStyle.focusColor
                Accessible.role: Accessible.ListItem
                Accessible.name: lineText.text

                // The current line's bar (shape, not only colour).
                Rectangle {
                    width: Kirigami.Units.smallSpacing
                    height: parent.height
                    color: KanteStyle.focusColor
                    visible: row.modelData.current
                }

                RowLayout {
                    id: content
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.smallSpacing
                    anchors.leftMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing

                    QQC2.Label {
                        id: lineText
                        objectName: row.modelData.kind === "character" ? "cue" : "line"
                        // A cue of a character the project knows opens its card.
                        readonly property bool linked: row.modelData.character !== undefined && row.modelData.character !== ""
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignTop
                        wrapMode: Text.Wrap
                        font.family: KanteStyle.monoFamily
                        font.underline: linked && lineHover.hovered
                        Accessible.role: linked ? Accessible.Button : Accessible.StaticText
                        HoverHandler {
                            id: lineHover
                            enabled: lineText.linked
                            cursorShape: Qt.PointingHandCursor
                        }
                        TapHandler {
                            enabled: lineText.linked
                            onTapped: card.showAt("character", row.modelData.character, lineText)
                        }
                        // Screenplay layout: cues and dialogue indented, as on paper.
                        leftPadding: row.modelData.kind === "character" ? Kirigami.Units.gridUnit * 4
                            : row.modelData.kind === "parenthetical" ? Kirigami.Units.gridUnit * 3
                            : row.modelData.kind === "dialogue" ? Kirigami.Units.gridUnit * 2 : 0
                        color: row.modelData.kind === "character" ? KanteStyle.strongTextColor
                            : row.modelData.kind === "parenthetical" ? KanteStyle.mutedTextColor : KanteStyle.textColor
                        text: row.modelData.kind === "parenthetical" ? "(" + row.modelData.text + ")" : row.modelData.text
                    }

                    ColumnLayout {
                        Layout.alignment: Qt.AlignTop
                        spacing: Kirigami.Units.smallSpacing
                        Repeater {
                            model: row.modelData.chips
                            delegate: KanteChip {
                                objectName: "chip"
                                required property var modelData
                                text: modelData.text
                                chipColor: modelData.label !== "" ? KanteStyle.labelColor(modelData.label) : KanteStyle.textColor
                                onClicked: panel.chipClicked(modelData.kind, modelData.target, modelData.index, modelData.at_s)
                            }
                        }
                    }
                }
            }
        }
    }
}
