import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A mark is selected (LH-22, LH-63): the pose of a figure's mark, the pause at it, and the
// pace of the way that leads to it. A figure's mark also says how the figure comes to it and
// whether it leaves after it (LH-26); a camera's mark says its lens height and tilt (LH-34,
// positive tilt looks up). The pace is shown as the plan has it; the two buttons
// time the way for walking or running and move the later marks along. No geometry here: the
// controller has the numbers from the session.
KanteCard {
    id: card

    required property var texts
    required property var controller

    readonly property var poses: ["standing", "sitting", "lying"]
    readonly property var arrivals: ["walk", "appear"]
    readonly property var departures: ["stay", "leave"]
    readonly property var info: {
        // The plan changes with every edit; the key alone would not tell the new values.
        card.controller.planJson
        card.controller.selectedKey
        const d = JSON.parse(card.controller.markInfo())
        return d.kind ? d : null
    }
    readonly property bool isFigure: info !== null && info.kind === "figure"
    readonly property bool isCamera: info !== null && info.kind === "camera"

    visible: info !== null
    implicitWidth: Math.min(parent ? parent.width - Kirigami.Units.largeSpacing * 2 : Infinity,
                            Math.max(row.implicitWidth, cueRow.implicitWidth, aimRow.implicitWidth)
                            + Kirigami.Units.largeSpacing * 2)
    implicitHeight: row.implicitHeight + Kirigami.Units.largeSpacing * 2
                    + (cueRow.visible ? cueRow.implicitHeight + Kirigami.Units.smallSpacing : 0)
                    + (aimRow.visible ? aimRow.implicitHeight + Kirigami.Units.smallSpacing : 0)

    onInfoChanged: card.fill()

    // The fields show the plan; a field the user is typing in is left alone.
    function fill() {
        if (!info) {
            return
        }
        poseChoice.currentIndex = Math.max(0, poses.indexOf(info.pose))
        arrivalChoice.currentIndex = Math.max(0, arrivals.indexOf(info.arrival))
        departureChoice.currentIndex = Math.max(0, departures.indexOf(info.departure))
        if (!holdField.activeFocus) {
            holdField.text = texts.num(info.hold, 2)
        }
        if (!card.isCamera) {
            return
        }
        if (!heightField.activeFocus) {
            heightField.text = texts.num(info.height, 2)
        }
        if (!tiltField.activeFocus) {
            tiltField.text = texts.num(info.tilt, 1)
        }
    }

    function setAim() {
        const ok = controller.setCameraAim(heightField.text, tiltField.text)
        heightField.invalid = !ok
        tiltField.invalid = !ok
    }

    function setHold() {
        holdField.invalid = !controller.setMarkHold(holdField.text)
    }

    function setPace(name) {
        controller.setMarkPace(name)
    }

    RowLayout {
        id: row
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
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

    RowLayout {
        id: cueRow
        objectName: "cueRow"
        visible: card.isFigure
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: row.bottom
        anchors.topMargin: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteSectionLabel { text: card.texts.t("mark.arrival") }
        KanteSegmented {
            id: arrivalChoice
            objectName: "arrivalChoice"
            model: card.arrivals.map(a => card.texts.t("arrival." + a))
            onActivated: index => {
                if (!card.controller.setMarkArrival(card.arrivals[index])) {
                    card.fill()
                }
            }
        }

        Kirigami.Separator { Layout.fillHeight: true }

        KanteSectionLabel { text: card.texts.t("mark.departure") }
        KanteSegmented {
            id: departureChoice
            objectName: "departureChoice"
            model: card.departures.map(d => card.texts.t("departure." + d))
            onActivated: index => {
                if (!card.controller.setMarkDeparture(card.departures[index])) {
                    card.fill()
                }
            }
        }
    }

    RowLayout {
        id: aimRow
        objectName: "aimRow"
        visible: card.isCamera
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: row.bottom
        anchors.topMargin: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteTextField {
            id: heightField
            objectName: "heightField"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 5
            Accessible.name: card.texts.t("mark.height")
            placeholderText: card.texts.t("mark.height")
            inputMethodHints: Qt.ImhFormattedNumbersOnly
            onTextChanged: invalid = false
            onAccepted: card.setAim()
        }
        KanteTextField {
            id: tiltField
            objectName: "tiltField"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 5
            Accessible.name: card.texts.t("mark.tilt")
            placeholderText: card.texts.t("mark.tilt")
            inputMethodHints: Qt.ImhFormattedNumbersOnly
            onTextChanged: invalid = false
            onAccepted: card.setAim()
            QQC2.ToolTip.text: card.texts.t("mark.tiltTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
        KanteButton {
            objectName: "aimApply"
            text: card.texts.t("form.apply")
            onClicked: card.setAim()
        }
    }
}
