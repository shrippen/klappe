import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A mark is selected (LH-22, LH-63), a section of the selection card (D-04): the pose of a
// figure's mark, the pause at it, and the pace of the way that leads to it. A figure's mark
// also says how the figure comes to it and whether it leaves after it (LH-26); a camera's
// mark says its lens height and tilt (LH-34, positive tilt looks up), the rig that brings
// the camera to it and the lens it states (LH-33; empty: none, between two marks with a lens
// the focal length runs with time). The pace is shown as the plan has it (a figure's mark:
// in its field, where it can be typed in m/s); the two buttons time the way for walking or
// running and move the later marks along. No geometry here: the controller has the numbers
// from the session. Pose, pause and pace (a camera's mark: lens
// height, rig and lens) are the common fields; arrival, departure and tilt wait behind
// "More".
ColumnLayout {
    id: card

    required property var texts
    required property var controller

    /// The rest of the fields: arrival and departure, the camera's tilt.
    property bool expanded: false
    /// The button for "More"; the selection card has one for all its sections.
    property bool showMore: true
    /// The plan's pace as the field shows it, so an unchanged field sends nothing.
    property string shownPace: ""

    readonly property var choices: controller.choicesJson ? JSON.parse(controller.choicesJson) : ({})
    readonly property var poses: choices.pose || []
    readonly property var arrivals: choices.arrival || []
    readonly property var departures: choices.departure || []
    readonly property var rigs: choices.rig || []
    readonly property var info: {
        // The plan changes with every edit; the key alone would not tell the new values.
        card.controller.planJson
        card.controller.selectedKey
        const d = JSON.parse(card.controller.markInfo())
        return d.kind ? d : null
    }
    readonly property bool isFigure: info !== null && info.kind === "figure"
    readonly property bool isCamera: info !== null && info.kind === "camera"
    readonly property bool shown: info !== null
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: holdField.activeFocus || heightField.activeFocus
        || tiltField.activeFocus || lensField.activeFocus || paceField.activeFocus

    visible: shown
    spacing: Kirigami.Units.smallSpacing

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
        // The plan's pace, also while the field has the focus: what it holds after Enter.
        card.shownPace = info.pace === null ? "" : texts.num(info.pace, 1)
        if (!paceField.activeFocus) {
            paceField.text = card.shownPace
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
        rigChoice.currentIndex = Math.max(0, rigs.indexOf(info.rig))
        if (!lensField.activeFocus) {
            lensField.text = info.lens === null ? "" : texts.num(info.lens, 1)
        }
    }

    function setAim() {
        const ok = controller.setCameraAim(heightField.text, tiltField.text)
        heightField.invalid = !ok
        tiltField.invalid = !ok
    }

    function setLens() {
        lensField.invalid = !controller.setMarkLens(lensField.text)
    }

    function setHold() {
        holdField.invalid = !controller.setMarkHold(holdField.text)
    }

    function setPace(name) {
        controller.setMarkPace(name)
    }

    // The field shows the pace rounded (1,75 as "1,8"): sent unchanged, it would move the
    // later marks.
    function setSpeed() {
        if (paceField.text === card.shownPace) {
            return
        }
        paceField.invalid = !controller.setMarkPace(paceField.text)
    }

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

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

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
    }

    RowLayout {
        spacing: Kirigami.Units.smallSpacing
        visible: card.isFigure || (card.info !== null && card.info.hasWay)

        // The field's name and unit: its placeholder hides once a pace fills it.
        QQC2.Label {
            objectName: "paceLabel"
            visible: card.isFigure
            text: card.texts.t("mark.paceField")
        }
        QQC2.Label {
            objectName: "paceValue"
            visible: card.isCamera && card.info.hasWay
            text: card.texts.t("mark.pace") + ": " + (card.info && card.info.pace !== null
                  ? card.texts.t("mark.paceValue", { value: card.texts.num(card.info.pace, 1) })
                  : card.texts.t("mark.paceNone"))
        }
        KanteTextField {
            id: paceField
            objectName: "paceField"
            visible: card.isFigure
            enabled: card.info !== null && card.info.hasWay
            Layout.preferredWidth: Kirigami.Units.gridUnit * 5
            Accessible.name: card.texts.t("mark.paceField")
            inputMethodHints: Qt.ImhFormattedNumbersOnly
            onTextChanged: invalid = false
            onAccepted: card.setSpeed()
        }
        KanteButton {
            objectName: "paceApply"
            visible: card.isFigure
            enabled: card.info !== null && card.info.hasWay
            text: card.texts.t("form.apply")
            onClicked: card.setSpeed()
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

    // A camera's mark: its lens height, the rig that brings it here and the lens it states.
    RowLayout {
        id: aimRow
        objectName: "aimRow"
        visible: card.isCamera
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
        KanteButton {
            objectName: "aimApply"
            text: card.texts.t("form.apply")
            onClicked: card.setAim()
        }
    }

    RowLayout {
        id: gearRow
        objectName: "gearRow"
        visible: card.isCamera
        spacing: Kirigami.Units.smallSpacing

        KanteSectionLabel { text: card.texts.t("mark.rig") }
        KanteComboBox {
            id: rigChoice
            objectName: "rigChoice"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 9
            model: card.rigs.map(r => card.texts.t("rig." + r))
            onActivated: index => {
                if (!card.controller.setMarkRig(card.rigs[index])) {
                    card.fill()
                }
            }
            QQC2.ToolTip.text: card.texts.t("mark.rigTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
    }

    RowLayout {
        objectName: "lensRow"
        visible: card.isCamera
        spacing: Kirigami.Units.smallSpacing

        KanteTextField {
            id: lensField
            objectName: "lensField"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 6
            Accessible.name: card.texts.t("mark.lens")
            placeholderText: card.texts.t("mark.lens")
            inputMethodHints: Qt.ImhFormattedNumbersOnly
            onTextChanged: invalid = false
            onAccepted: card.setLens()
            QQC2.ToolTip.text: card.texts.t("mark.lensTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
        KanteButton {
            objectName: "lensApply"
            text: card.texts.t("form.apply")
            onClicked: card.setLens()
        }
    }

    KanteButton {
        objectName: "markMore"
        visible: card.showMore
        emphasis: KanteButton.Emphasis.Quiet
        size: KanteButton.Size.Small
        text: card.texts.t(card.expanded ? "card.less" : "card.more")
        onClicked: card.expanded = !card.expanded
    }

    // A figure's mark: how it comes to the mark and whether it leaves after it.
    ColumnLayout {
        id: cueRow
        objectName: "cueRow"
        visible: card.expanded && card.isFigure
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
        objectName: "tiltRow"
        visible: card.expanded && card.isCamera
        spacing: Kirigami.Units.smallSpacing

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
            objectName: "tiltApply"
            text: card.texts.t("form.apply")
            onClicked: card.setAim()
        }
    }
}
