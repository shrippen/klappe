import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami
import Kante

// The shots of the plan, as storyboard tiles or as a table (D-05), and under them the selected
// shot: the camera view (LH-36) and its fields, edited in place (LH-71). Every change is one
// undo step; a field left unchanged edits nothing.
//
//  [Neuer Shot] [Kopieren] [Löschen]
//  [Kacheln|Tabelle]        [Schnitt|Drehreihenfolge]
//  ┌ tiles or table ─────────────────┐
//  └─────────────────────────────────┘
//  Shot 12.3                          details of the selected shot
Item {
    id: panel

    required property var texts
    required property var controller
    property var shot: null

    /// What the list shows: "tiles" or "table"; and in which order: "cut" or "shooting" (LH-72).
    property string view: "tiles"
    property string order: "cut"
    /// The table shows the long description as a second line under the selected shot (LH-112).
    property bool descriptionOpen: false
    signal viewPicked(string view)
    signal orderPicked(string order)

    readonly property var views: ["tiles", "table"]
    readonly property var orders: ["cut", "shooting"]

    // The shots in the chosen order, each with its lens as shown.
    readonly property var listed: {
        const json = order === "shooting" ? controller.shotsShootingJson : controller.shotsJson
        return json ? JSON.parse(json).map(s => Object.assign(s, { lens_label: lensLabel(s), image_url: imageUrl(s) })) : []
    }

    // The storyboard picture of a row as a URL (LH-74); empty for none.
    function imageUrl(row) {
        if (!row.image || typeof controller.projectFileUrl !== "function") {
            return ""
        }
        return controller.projectFileUrl(row.image)
    }

    // A new shot opens its short name and description for typing (LH-110); Enter finishes,
    // Esc leaves the shot as it is.
    function newShot() {
        if (controller.addShot()) {
            newShotDialog.open()
        }
    }

    // Cameras of the plan (the timeline's camera tracks), for the camera chooser.
    readonly property var cameras: controller.tracksJson ? JSON.parse(controller.tracksJson).filter(t => t.kind === "camera") : []

    // Figures of the plan, for the follow chooser.
    readonly property var figures: controller.tracksJson ? JSON.parse(controller.tracksJson).filter(t => t.kind === "figure") : []

    // The characters the 180° axis runs between: {a, b, own}, or null (LH-40).
    readonly property var axis: controller.choicesJson ? (JSON.parse(controller.choicesJson).axis || null) : null

    // Counts refused values; the fields' text bindings read it, so they snap back to the model.
    property int refused: 0

    // The shot's own lens as text; empty if it follows its camera.
    readonly property string ownLens: shot && shot.own_lens_mm !== null ? texts.num(shot.own_lens_mm, 1) : ""

    // The camera's marks state a lens: they give the shot's, a span for a zoom (LH-33).
    readonly property bool lensMarked: !!shot && shot.lens_from_marks
    readonly property string markedLens: shot ? lensLabel(shot) : ""

    // The lens of a shot at its start, a span for a zoom.
    function lensLabel(row) {
        if (row.lens_to_mm === null) {
            return texts.t("camera.lensValue", { lens: texts.num(row.lens_mm, 1) })
        }
        return texts.t("shot.lensSpan", { from: texts.num(row.lens_mm, 1), to: texts.num(row.lens_to_mm, 1) })
    }

    // Sends `value` of the selected shot unless it is what the field showed.
    function commit(field, value, shown) {
        if (value === shown) {
            return
        }
        send(shot.id, field, value)
    }

    // Sets one end of the axis to the figure `id`; the other end stays.
    function setAxisEnd(end, id) {
        const pair = axis ? { a: axis.a, b: axis.b } : { a: "", b: "" }
        pair[end] = id
        if (!controller.setAxis(pair.a, pair.b)) {
            refused++
        }
    }

    // Sends one field of shot `id`; a refusal resets the fields.
    function send(id, field, value) {
        if (!controller.editShot(id, field, value)) {
            refused++
        }
    }

    // Couples the shot's camera to the figure `id`, or lets it go with an empty id.
    function follow(id) {
        if (id === shot.follows) {
            return
        }
        if (!controller.setFollow(shot.camera, id)) {
            refused++
        }
    }

    readonly property var sizes: controller.choicesJson ? (JSON.parse(controller.choicesJson).size || []) : []
    readonly property var angles: controller.choicesJson ? (JSON.parse(controller.choicesJson).angle || []) : []
    // The setups the shot can join (LH-77), counted without the shot itself.
    readonly property var setupChoices: shot ? JSON.parse(controller.setupChoices(shot.id)) : []
    readonly property string ownSetup: shot && shot.setup_of ? shot.setup_of : ""

    function setupLabel(c) {
        return texts.t("shot.setupChoice", { setup: c.setup, camera: c.camera_label, lens: texts.num(c.lens_mm, 1), first: c.first_number })
    }

    // The entry of the box: 0 is "automatisch", else the setup the shot follows; none when its
    // target is gone.
    function setupIndex() {
        if (ownSetup === "") {
            return 0
        }
        if (shot.setup_lost) {
            return -1
        }
        return setupChoices.findIndex(c => c.setup === shot.setup) + 1 || -1
    }
    readonly property var movements: controller.choicesJson ? (JSON.parse(controller.choicesJson).movement || []) : []

    // New, copy, version, renumber and delete a shot (LH-70, LH-76). "New" takes the camera selected in the plan, else
    // the selected shot's. Key S is "new"; deleting asks first.
    RowLayout {
        id: bar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteToolButton {
            objectName: "newShotButton"
            text: panel.texts.t("shot.new")
            onClicked: panel.newShot()
        }
        KanteToolButton {
            objectName: "copyShotButton"
            enabled: panel.shot !== null
            text: panel.texts.t("shot.copy")
            onClicked: panel.controller.copyShot(panel.shot.id)
        }
        KanteToolButton {
            objectName: "versionShotButton"
            enabled: panel.shot !== null
            text: panel.texts.t("shot.version")
            onClicked: panel.controller.versionShot(panel.shot.id)
        }
        KanteToolButton {
            objectName: "renumberShotsButton"
            text: panel.texts.t("shot.renumber")
            onClicked: panel.controller.renumberShots()
        }
        KanteToolButton {
            objectName: "deleteShotButton"
            enabled: panel.shot !== null
            text: panel.texts.t("shot.delete")
            onClicked: confirmDelete.open()
        }
        Item { Layout.fillWidth: true }
    }

    Shortcut {
        sequence: "S"
        enabled: panel.visible && panel.enabled && !!panel.controller.loaded
        onActivated: panel.newShot()
    }

    KanteDialog {
        id: newShotDialog
        objectName: "newShotDialog"

        property string shotId: ""

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        title: panel.texts.t("shot.new")
        onAboutToShow: {
            shotId = panel.shot ? panel.shot.id : ""
            newShortName.text = ""
            newDescription.text = ""
        }
        onOpened: newShortName.forceActiveFocus()
        onAccepted: {
            if (shotId === "") {
                return
            }
            if (newShortName.text === "" && newDescription.text === "") {
                return
            }
            // One call: the new shot and its name are two undo steps, not three.
            if (!panel.controller.nameShot(shotId, newShortName.text, newDescription.text)) {
                panel.refused++
            }
        }

        ColumnLayout {
            width: parent ? parent.width : implicitWidth
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: panel.texts.t("field.shortName") }
            KanteTextField {
                id: newShortName
                objectName: "newShortName"
                Layout.fillWidth: true
                onAccepted: newShotDialog.accept()
            }
            KanteSectionLabel { text: panel.texts.t("field.description") }
            KanteTextField {
                id: newDescription
                objectName: "newDescription"
                Layout.fillWidth: true
                onAccepted: newShotDialog.accept()
            }
            QQC2.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: KanteStyle.mutedTextColor
                text: panel.texts.t("shot.newHint")
            }
        }
    }

    FileDialog {
        id: imageDialog
        objectName: "shotImageDialog"
        title: panel.texts.t("shot.imagePickTitle")
        nameFilters: [panel.texts.t("background.filter")]
        onAccepted: {
            if (panel.shot && !panel.controller.setShotImage(panel.shot.id, selectedFile.toString())) {
                panel.refused++
            }
        }
    }

    KanteDialog {
        id: confirmDelete
        objectName: "confirmDelete"

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        destructive: true
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: panel.texts.t("remove.confirm")
        rejectText: panel.texts.t("form.cancel")
        title: panel.texts.t("shot.delete")
        onAccepted: panel.controller.deleteShot(panel.shot.id)

        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: panel.shot ? panel.texts.t("shot.deleteAsk", { number: panel.shot.number }) : ""
        }
    }

    ColumnLayout {
        anchors.top: bar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: Kirigami.Units.smallSpacing
            Layout.rightMargin: Kirigami.Units.smallSpacing
            spacing: Kirigami.Units.smallSpacing

            KanteSegmented {
                objectName: "viewSwitch"
                model: panel.views.map(v => panel.texts.t("shots.view." + v))
                currentIndex: Math.max(0, panel.views.indexOf(panel.view))
                onActivated: index => panel.viewPicked(panel.views[index])
            }
            Item { Layout.fillWidth: true }
            KanteSegmented {
                objectName: "orderSwitch"
                model: panel.orders.map(o => panel.texts.t("export.order." + o))
                currentIndex: Math.max(0, panel.orders.indexOf(panel.order))
                onActivated: index => panel.orderPicked(panel.orders[index])
            }
        }

        // The list takes what the details leave: about two fifths with a shot selected.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: panel.height * 0.4

            ShotTiles {
                objectName: "shotTiles"
                anchors.fill: parent
                visible: panel.view === "tiles"
                texts: panel.texts
                shots: panel.listed
                selectedId: panel.shot ? panel.shot.id : ""
                onPicked: id => panel.controller.selectShot(id)
            }
            ShotTable {
                objectName: "shotTable"
                anchors.fill: parent
                visible: panel.view === "table"
                texts: panel.texts
                shots: panel.listed
                order: panel.order
                descriptionOpen: panel.descriptionOpen
                selectedId: panel.shot ? panel.shot.id : ""
                cameras: panel.cameras
                sizes: panel.sizes
                angles: panel.angles
                movements: panel.movements
                refused: panel.refused
                onPicked: id => panel.controller.selectShot(id)
                onEdited: (id, field, value) => panel.send(id, field, value)
            }
        }

        KanteEmptyState {
            Layout.fillWidth: true
            Layout.margins: Kirigami.Units.largeSpacing
            visible: panel.shot === null
            title: panel.texts.t("toggle.shot")
            text: panel.texts.t("shot.none")
        }

        QQC2.ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: panel.height * 0.6
            Layout.margins: Kirigami.Units.smallSpacing
            visible: panel.shot !== null
            contentWidth: availableWidth

            // D-05: the details of the selected shot on a card.
            background: KanteCard {
                objectName: "shotDetailsCard"
                color: KanteStyle.cardColor
                chamfer: KanteStyle.chamferSmall
            }

            ColumnLayout {
                width: parent.width
                spacing: Kirigami.Units.largeSpacing

                RowLayout {
                    Layout.fillWidth: true
                    Layout.margins: Kirigami.Units.largeSpacing
                    KanteHeading {
                        Layout.fillWidth: true
                        level: 4
                        text: panel.shot ? panel.texts.t("shot.title", { number: panel.shot.number }) : ""
                    }
                    KanteChip {
                        objectName: "cameraChip"
                        visible: panel.shot !== null && !!panel.shot.camera_color
                        interactive: false
                        chipColor: panel.shot && panel.shot.camera_color ? KanteStyle.labelColor(panel.shot.camera_color) : KanteStyle.textColor
                        text: panel.shot ? panel.shot.camera_label : ""
                    }
                    KanteChip {
                        objectName: "axisChip"
                        visible: panel.shot !== null && panel.shot.axis_jump
                        interactive: false
                        chipColor: KanteStyle.warningColor
                        iconName: "dialog-warning"
                        text: panel.texts.t("shot.axisJump")
                        // Names the shots on the other side of the axis (LH-41).
                        toolTip: panel.shot ? panel.texts.t("shot.axisAgainst", { shots: panel.shot.axis_against }) : ""
                    }
                }

                // Of the selected shot's camera.
                CameraView {
                    objectName: "cameraView"
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    texts: panel.texts
                    controller: panel.controller
                }

                GridLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    columns: 2
                    columnSpacing: Kirigami.Units.largeSpacing
                    rowSpacing: Kirigami.Units.smallSpacing

                    KanteSectionLabel { text: panel.texts.t("field.number") }
                    KanteSectionLabel { text: panel.texts.t("field.camera") }
                    KanteTextField {
                        objectName: "numberField"
                        Layout.fillWidth: true
                        text: { panel.refused; return panel.shot ? panel.shot.number : "" }
                        onEditingFinished: panel.commit("number", text, panel.shot.number)
                    }
                    KanteComboBox {
                        objectName: "cameraBox"
                        Layout.fillWidth: true
                        // A version has the camera of its base (LH-76).
                        enabled: !!panel.shot && !panel.shot.is_version
                        model: panel.cameras.map(c => c.name)
                        currentIndex: panel.shot ? panel.cameras.findIndex(c => c.id === panel.shot.camera) : -1
                        onActivated: index => panel.commit("camera", panel.cameras[index].id, panel.shot.camera)
                    }

                    KanteSectionLabel { text: panel.texts.t("field.size") }
                    KanteSectionLabel { text: panel.texts.t("field.movement") }
                    KanteComboBox {
                        Layout.fillWidth: true
                        model: panel.sizes.map(s => panel.texts.t("size." + s))
                        currentIndex: panel.shot ? panel.sizes.indexOf(panel.shot.size) : -1
                        onActivated: index => panel.commit("size", panel.sizes[index], panel.shot.size)
                    }
                    KanteComboBox {
                        objectName: "movementBox"
                        Layout.fillWidth: true
                        model: panel.movements.map(m => panel.texts.t("movement." + m))
                        currentIndex: panel.shot ? panel.movements.indexOf(panel.shot.movement) : -1
                        onActivated: index => panel.commit("movement", panel.movements[index], panel.shot.movement)
                    }

                    KanteSectionLabel { text: panel.texts.t("field.perspective") }
                    KanteSectionLabel { text: panel.texts.t("field.lens") }
                    KanteComboBox {
                        objectName: "angleBox"
                        Layout.fillWidth: true
                        model: panel.angles.map(a => panel.texts.t("angle." + a))
                        currentIndex: panel.shot ? panel.angles.indexOf(panel.shot.angle) : -1
                        onActivated: index => panel.commit("angle", panel.angles[index], panel.shot.angle)
                    }
                    // Empty: the shot follows its camera, whose lens is the hint. With lens marks the
                    // field only shows theirs.
                    KanteTextField {
                        objectName: "lensField"
                        Layout.fillWidth: true
                        readOnly: panel.lensMarked
                        text: { panel.refused; return panel.lensMarked ? panel.markedLens : panel.ownLens }
                        placeholderText: panel.shot ? panel.texts.num(panel.shot.lens_mm, 1) : ""
                        inputMethodHints: Qt.ImhFormattedNumbersOnly
                        onEditingFinished: {
                            if (!readOnly) {
                                panel.commit("lens", text, panel.ownLens)
                            }
                        }
                    }

                    KanteSectionLabel { text: panel.texts.t("field.setup") }
                    Item { }
                    // "automatisch" computes the setup from camera, place and lens; a setup puts the
                    // shot with the first shot of that setup by hand (LH-77).
                    KanteComboBox {
                        objectName: "setupBox"
                        Layout.fillWidth: true
                        model: [panel.texts.t("setup.auto")].concat(panel.setupChoices.map(c => panel.setupLabel(c)))
                        currentIndex: panel.shot ? panel.setupIndex() : -1
                        onActivated: index => panel.commit("setup", index === 0 ? "" : panel.setupChoices[index - 1].first_id, panel.ownSetup)
                    }
                    Item { }
                }

                QQC2.Label {
                    objectName: "setupHint"
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    visible: !!panel.shot && !!panel.shot.setup_lost
                    wrapMode: Text.WordWrap
                    color: KanteStyle.mutedTextColor
                    text: panel.shot ? panel.texts.t("shot.setupGone", { setup: panel.shot.setup }) : ""
                }

                QQC2.Label {
                    objectName: "lensHint"
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    visible: panel.lensMarked
                    wrapMode: Text.WordWrap
                    color: KanteStyle.mutedTextColor
                    text: panel.texts.t("shot.lensFromMarks")
                }

                // What the camera's marks do during a shot without a movement of its own (LH-33);
                // the button takes it over.
                RowLayout {
                    objectName: "movementHintRow"
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    visible: panel.shot !== null && !!panel.shot.movement_hint
                    spacing: Kirigami.Units.smallSpacing

                    QQC2.Label {
                        objectName: "movementHint"
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: KanteStyle.mutedTextColor
                        text: panel.shot && panel.shot.movement_hint
                              ? panel.texts.t("shot.movementHint", { movement: panel.texts.t("movement." + panel.shot.movement_hint) })
                              : ""
                    }
                    KanteButton {
                        objectName: "movementTake"
                        text: panel.texts.t("form.apply")
                        onClicked: panel.commit("movement", panel.shot.movement_hint, panel.shot.movement)
                    }
                }

                // The shot's camera follows a figure, keeping distance and angle (LH-37).
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing
                    KanteSectionLabel { text: panel.texts.t("field.follows") }
                    KanteComboBox {
                        objectName: "followBox"
                        Layout.fillWidth: true
                        // Index 0 is "none"; the figures come after it.
                        model: [panel.texts.t("follows.none")].concat(panel.figures.map(f => f.name))
                        currentIndex: { panel.refused; return panel.shot ? 1 + panel.figures.findIndex(f => f.id === panel.shot.follows) : -1 }
                        onActivated: index => panel.follow(index === 0 ? "" : panel.figures[index - 1].id)
                    }
                }

                // The 180° axis between two figures; the warning above refers to it (LH-40).
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing
                    KanteSectionLabel { text: panel.texts.t("field.axis") }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Kirigami.Units.smallSpacing
                        Repeater {
                            model: ["a", "b"]
                            delegate: KanteComboBox {
                                required property string modelData
                                objectName: "axisBox_" + modelData
                                Layout.fillWidth: true
                                model: panel.figures.map(f => f.name)
                                currentIndex: { panel.refused; return panel.axis ? panel.figures.findIndex(f => f.id === panel.axis[modelData]) : -1 }
                                onActivated: index => panel.setAxisEnd(modelData, panel.figures[index].id)
                            }
                        }
                    }
                    KanteButton {
                        objectName: "axisAuto"
                        enabled: !!panel.axis && panel.axis.own
                        text: panel.texts.t("axis.auto")
                        onClicked: panel.controller.setAxis("", "")
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing
                    KanteSectionLabel { text: panel.texts.t("field.shortName") }
                    KanteTextField {
                        objectName: "shortNameField"
                        Layout.fillWidth: true
                        text: { panel.refused; return panel.shot && panel.shot.short_name ? panel.shot.short_name : "" }
                        onEditingFinished: panel.commit("short_name", text, panel.shot.short_name)
                    }
                }

                // The storyboard picture (LH-74): a file of the project, shown here, in the tile
                // and in the table.
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing
                    KanteSectionLabel { text: panel.texts.t("field.image") }
                    Image {
                        objectName: "shotImage"
                        Layout.fillWidth: true
                        Layout.preferredHeight: visible ? Kirigami.Units.gridUnit * 8 : 0
                        visible: !!panel.shot && !!panel.shot.image_url
                        source: panel.shot && panel.shot.image_url ? panel.shot.image_url : ""
                        fillMode: Image.PreserveAspectFit
                        sourceSize.width: Kirigami.Units.gridUnit * 24
                    }
                    RowLayout {
                        spacing: Kirigami.Units.smallSpacing
                        KanteButton {
                            objectName: "shotImagePick"
                            text: panel.texts.t("shot.imagePick")
                            onClicked: imageDialog.open()
                        }
                        KanteButton {
                            objectName: "shotStill"
                            enabled: !!panel.shot
                            text: panel.texts.t("shot.still")
                            onClicked: {
                                if (!panel.controller.shotStill(panel.shot.id)) {
                                    panel.refused++
                                }
                            }
                        }
                        KanteButton {
                            objectName: "shotImageRemove"
                            enabled: !!panel.shot && !!panel.shot.image
                            text: panel.texts.t("shot.imageRemove")
                            onClicked: {
                                if (!panel.controller.setShotImage(panel.shot.id, "")) {
                                    panel.refused++
                                }
                            }
                        }
                    }
                }

                Repeater {
                    model: ["description", "props", "gear", "crew", "note"]
                    delegate: ColumnLayout {
                        required property string modelData
                        Layout.fillWidth: true
                        Layout.leftMargin: Kirigami.Units.largeSpacing
                        Layout.rightMargin: Kirigami.Units.largeSpacing
                        spacing: Kirigami.Units.smallSpacing
                        KanteSectionLabel { text: panel.texts.t("field." + parent.modelData) }
                        KanteTextField {
                            Layout.fillWidth: true
                            text: { panel.refused; return panel.shot ? panel.shot[parent.modelData] : "" }
                            onEditingFinished: panel.commit(parent.modelData, text, panel.shot[parent.modelData])
                        }
                    }
                }
            }
        }
    }
}
