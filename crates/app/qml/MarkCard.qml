import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A mark is selected (LH-22, LH-63): the pose of a figure's mark, the pause at it, and the
// pace of the way that leads to it. The pace is shown as the plan has it; the two buttons
// time the way for walking or running and move the later marks along. No geometry here: the
// controller has the numbers from the session.
KanteCard {
    id: card

    required property var texts
    required property var controller

    readonly property var poses: ["standing", "sitting", "lying"]
    readonly property var info: {
        // The plan changes with every edit; the key alone would not tell the new values.
        card.controller.planJson
        card.controller.selectedKey
        const d = JSON.parse(card.controller.markInfo())
        return d.kind ? d : null
    }
    readonly property bool isFigure: info !== null && info.kind === "figure"

    visible: info !== null
    implicitWidth: Math.min(parent ? parent.width - Kirigami.Units.largeSpacing * 2 : Infinity,
                            row.implicitWidth + Kirigami.Units.largeSpacing * 2)
    implicitHeight: row.implicitHeight + Kirigami.Units.largeSpacing * 2

    onInfoChanged: card.fill()

    // The fields show the plan; a field the user is typing in is left alone.
    function fill() {
        if (!info) {
            return
        }
        poseChoice.currentIndex = Math.max(0, poses.indexOf(info.pose))
        if (!holdField.activeFocus) {
            holdField.text = texts.num(info.hold, 2)
        }
    }

    function setHold() {
        holdField.invalid = !controller.setMarkHold(holdField.text)
    }

    function setPace(name) {
        controller.setMarkPace(name)
    }

    RowLayout {
        id: row
        anchors.centerIn: parent
        spacing: Kirigami.Units.smallSpacing

        KanteSectionLabel { text: card.texts.t("mark.title") }
        KanteSegmented {
            id: poseChoice
            objectName: "poseChoice"
            visible: card.isFigure
            model: card.poses.map(p => card.texts.t("pose." + p))
            onActivated: index => {
                if (!card.controller.setMarkPose(card.poses[index])) {
                    card.fill()
                }
            }
        }

        Kirigami.Separator { Layout.fillHeight: true }

        KanteTextField {
            id: holdField
            objectName: "holdField"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 5
            Accessible.name: card.texts.t("mark.hold")
            placeholderText: card.texts.t("mark.hold")
            inputMethodHints: Qt.ImhFormattedNumbersOnly
            onTextChanged: invalid = false
            onAccepted: card.setHold()
        }
        KanteButton {
            objectName: "holdApply"
            text: card.texts.t("form.apply")
            onClicked: card.setHold()
        }

        Kirigami.Separator { Layout.fillHeight: true }

        QQC2.Label {
            objectName: "paceValue"
            visible: card.info !== null && card.info.hasWay
            text: card.texts.t("mark.pace") + ": " + (card.info && card.info.pace !== null
                  ? card.texts.t("mark.paceValue", { value: card.texts.num(card.info.pace, 1) })
                  : card.texts.t("mark.paceNone"))
        }
        KanteButton {
            objectName: "paceWalk"
            visible: card.isFigure
            enabled: card.info !== null && card.info.hasWay
            text: card.texts.t("pace.walk")
            onClicked: card.setPace("walk")
            QQC2.ToolTip.text: card.texts.t("mark.paceTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
        KanteButton {
            objectName: "paceRun"
            visible: card.isFigure
            enabled: card.info !== null && card.info.hasWay
            text: card.texts.t("pace.run")
            onClicked: card.setPace("run")
            QQC2.ToolTip.text: card.texts.t("mark.paceTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
    }
}
