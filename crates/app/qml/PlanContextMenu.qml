import QtQuick
import QtQuick.Controls as QQC2
import QtQml
import QtQuick.Dialogs
import Kante

// The context menu of the plan (LH-100): right click, the Menu key or Shift+F10 on an element
// (it is selected first) or on empty ground. The entries are the commands the keys of
// PlanArea already run, each with its key beside it; the menu only offers what the selection
// can take. A locked or hidden layer cannot be picked, so empty ground offers to unlock or show
// it again.
//
//  element                     empty ground
//  ┌────────────────────┐      ┌────────────────────┐
//  │ Delete         Del │      │ Undo        Ctrl+Z │
//  │ Duplicate   Ctrl+D │      │ Redo  Ctrl+Shift+Z │
//  │ Turn right       R │      │ Fit plan    Ctrl+0 │
//  │ Straighten way   G │      │ Unlock layer Notes │
//  ├────────────────────┤      └────────────────────┘
//  │ Lock layer Figures │
//  │ Hide layer Figures │
//  │ Clear selection Esc│
//  └────────────────────┘
QQC2.Menu {
    id: menu

    required property var texts
    required property var controller
    /// The angle R and Shift+R turn by.
    required property real turnStepDeg

    signal fitRequested()

    /// What the controller says the selection takes (`menuInfo`): the layer and the commands;
    /// asked when the menu opens.
    property var info: ({})
    onAboutToShow: info = JSON.parse(controller.menuInfo())
    readonly property int picked: controller.selectedCount
    readonly property bool onElement: picked > 0
    readonly property string layer: info.layer || ""
    readonly property string layerName: layer === "" ? "" : texts.t("layer." + layer)

    // Fade (LH-15): the level of the layer, and the one a click brings.
    readonly property var _fades: controller.fadesJson ? JSON.parse(controller.fadesJson) : ({})
    readonly property var _fadeSteps: controller.choicesJson ? (JSON.parse(controller.choicesJson).fade || []) : []
    readonly property string fade: _fades[layer] || "100"
    readonly property string nextFade: _fadeSteps.length > 0 ? _fadeSteps[(_fadeSteps.indexOf(fade) + 1) % _fadeSteps.length] : fade

    readonly property var _shown: controller.layersJson ? JSON.parse(controller.layersJson) : ({})
    readonly property var _locks: controller.locksJson ? JSON.parse(controller.locksJson) : ({})
    /// The layers empty ground offers to bring back: locked ones, then hidden ones.
    readonly property var _lapsed: {
        if (onElement) {
            return []
        }
        const names = Object.keys(_locks).concat(Object.keys(_shown))
            .filter((n, i, all) => all.indexOf(n) === i)
        return names.filter(n => _locks[n] === true).map(n => ({ layer: n, unlock: true }))
            .concat(names.filter(n => _shown[n] === false).map(n => ({ layer: n, unlock: false })))
    }

    /// "Ctrl+Shift+Z" as the keyboard of the language names it.
    function keyName(sequence) {
        return sequence.replace("Ctrl", texts.t("key.ctrl")).replace("Shift", texts.t("key.shift"))
            .replace("Delete", texts.t("key.delete"))
    }

    KantePopupSkin { popup: menu }

    KanteMenuItem {
        objectName: "ctxDelete"
        visible: menu.info.remove === true
        text: menu.texts.t("ctx.delete")
        shortcutText: menu.keyName("Delete")
        destructive: true
        onTriggered: menu.controller.removeSelected()
    }
    KanteMenuItem {
        objectName: "ctxDuplicate"
        visible: menu.info.duplicate === true
        text: menu.texts.t("ctx.duplicate")
        shortcutText: menu.keyName("Ctrl+D")
        onTriggered: menu.controller.duplicateSelected()
    }
    KanteMenuItem {
        objectName: "ctxTurnRight"
        visible: menu.info.turn === true
        text: menu.texts.t("ctx.turnRight")
        shortcutText: "R"
        onTriggered: menu.controller.turnSelected(menu.turnStepDeg)
    }
    KanteMenuItem {
        objectName: "ctxTurnLeft"
        visible: menu.info.turn === true
        text: menu.texts.t("ctx.turnLeft")
        shortcutText: menu.keyName("Shift+R")
        onTriggered: menu.controller.turnSelected(-menu.turnStepDeg)
    }
    KanteMenuItem {
        objectName: "ctxStraighten"
        visible: menu.info.straighten === true
        text: menu.texts.t("ctx.straighten")
        shortcutText: "G"
        onTriggered: menu.controller.straightenSelected()
    }
    KanteMenuItem {
        objectName: "ctxEditNote"
        visible: menu.info.note === true
        text: menu.texts.t("ctx.editNote")
        shortcutText: "F2"
        onTriggered: menu.controller.editSelectedNote()
    }
    KanteMenuItem {
        objectName: "ctxAddShot"
        visible: menu.info.shot === true
        text: menu.texts.t("ctx.addShot")
        onTriggered: menu.controller.addShot()
    }
    KanteMenuItem {
        objectName: "ctxShotStill"
        visible: menu.info.still === true
        text: menu.texts.t("shot.still")
        onTriggered: menu.controller.shotStill(menu.controller.selectedShot)
    }

    // The commands of an object and of a label (LH-114 to LH-118): the keys are those of the
    // palette; the change of kind is a submenu with the library, as the card's list has it.
    KanteMenuItem {
        objectName: "ctxMirror"
        visible: menu.info.mirror === true
        text: menu.texts.t("object.mirror")
        shortcutText: menu.keyName("Shift+M")
        onTriggered: menu.controller.changeSelected("mirror", "")
    }
    KanteMenuItem {
        objectName: "ctxResetSize"
        visible: menu.info.resetSize === true
        text: menu.texts.t("object.resetSize")
        shortcutText: menu.keyName("Shift+G")
        onTriggered: menu.controller.changeSelected("resetSize", "")
    }
    KanteMenuItem {
        objectName: "ctxLabel"
        visible: menu.info.label === true
        text: menu.texts.t("object.label")
        shortcutText: menu.keyName("Shift+L")
        onTriggered: menu.controller.labelSelected()
    }
    KanteMenuItem {
        objectName: "ctxUnbind"
        visible: menu.info.unbind === true
        text: menu.texts.t("note.unbind")
        shortcutText: menu.keyName("Shift+U")
        onTriggered: menu.controller.changeSelected("unbind", "")
    }
    // The kinds of the library, built in first, then the project's own (as the palette lists them).
    readonly property var kinds: {
        if (!controller.choicesJson) {
            return []
        }
        return (JSON.parse(controller.choicesJson).object || []).map(o => ({
            kind: o.kind,
            text: o.custom ? (o.label || o.kind) : texts.t("objectKind." + o.kind)
        }))
    }
    // The submenu is there while the selection takes a new kind; it goes in before the arm commands.
    Instantiator {
        model: menu.info.kind === true ? 1 : 0
        delegate: QQC2.Menu {
            id: kindMenu
            objectName: "ctxKind"
            title: menu.texts.t("ctx.kind")
            KantePopupSkin { popup: kindMenu }
            Instantiator {
                model: menu.kinds
                delegate: KanteMenuItem {
                    required property var modelData
                    objectName: "ctxKind." + modelData.kind
                    text: modelData.text
                    // `menu` is the item's own menu here: the controller is the plan menu's.
                    onTriggered: controller.changeSelected("kind", modelData.kind)
                }
                onObjectAdded: (index, object) => kindMenu.addItem(object)
                onObjectRemoved: (index, object) => kindMenu.removeItem(object)
            }
        }
        onObjectAdded: (index, object) => menu.insertMenu(menu.indexOfEntry("ctxNextMark"), object)
        onObjectRemoved: (index, object) => menu.removeMenu(object)
    }
    function indexOfEntry(name) {
        for (let i = 0; i < count; i++) {
            if (itemAt(i).objectName === name) {
                return i
            }
        }
        return count
    }

    // The commands that arm the next click (LH-104, LH-105, LH-109) and set a way's curve (LH-108).
    KanteMenuItem {
        objectName: "ctxNextMark"
        visible: menu.info.nextMark === true
        text: menu.texts.t("arm.nextMark")
        shortcutText: menu.keyName("Shift+F")
        onTriggered: menu.controller.arm("nextMark")
    }
    KanteMenuItem {
        objectName: "ctxMarkBefore"
        visible: menu.info.markBefore === true
        text: menu.texts.t("arm.markBefore")
        shortcutText: menu.keyName("Shift+Z")
        onTriggered: menu.controller.arm("markBefore")
    }
    KanteMenuItem {
        objectName: "ctxAxisTo"
        visible: menu.info.axisTo === true
        text: menu.texts.t("arm.axisTo")
        shortcutText: menu.keyName("Shift+A")
        onTriggered: menu.controller.arm("axisTo")
    }
    KanteMenuItem {
        objectName: "ctxWaySmooth"
        visible: menu.info.way === "straight"
        text: menu.texts.t("way.smooth")
        onTriggered: menu.controller.setWayCurve("smooth")
    }
    KanteMenuItem {
        objectName: "ctxWayStraight"
        visible: menu.info.way === "smooth"
        text: menu.texts.t("way.straight")
        onTriggered: menu.controller.setWayCurve("straight")
    }

    // The shot of the selected camera (LH-74, LH-76, LH-110): the commands of the shot panel.
    KanteMenuItem {
        objectName: "ctxVersionShot"
        visible: menu.info.versionShot === true
        text: menu.texts.t("shot.version")
        onTriggered: menu.controller.versionShot(menu.controller.selectedShot)
    }
    KanteMenuItem {
        objectName: "ctxShortName"
        visible: menu.info.shortName === true
        text: menu.texts.t("ctx.shortName")
        onTriggered: menu.askShortName()
    }
    KanteMenuItem {
        objectName: "ctxShotImage"
        visible: menu.info.shotImage === true
        text: menu.texts.t("shot.imagePick")
        onTriggered: menu.askShotImage()
    }
    KanteMenuItem {
        objectName: "ctxShotImageRemove"
        visible: menu.info.shotImageRemove === true
        text: menu.texts.t("shot.imageRemove")
        onTriggered: menu.controller.setShotImage(menu.controller.selectedShot, "")
    }

    // Made on first use: a dialog that exists takes the keyboard focus of the plan's fields.
    property string shotId: ""
    property var imageDialog: null
    property var nameDialog: null
    Component {
        id: imageDialogComponent
        FileDialog {
            title: menu.texts.t("shot.imagePickTitle")
            nameFilters: [menu.texts.t("background.filter")]
            onAccepted: menu.controller.setShotImage(menu.shotId, selectedFile.toString())
        }
    }
    Component {
        id: nameDialogComponent
        KanteDialog {
            id: dialog
            property alias text: field.text
            parent: QQC2.Overlay.overlay
            anchors.centerIn: parent
            modal: true
            size: KanteDialog.Size.Small
            standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
            title: menu.texts.t("field.shortName")
            onOpened: field.forceActiveFocus()
            onAccepted: menu.controller.editShot(menu.shotId, "short_name", field.text)
            KanteTextField {
                id: field
                objectName: "ctxShortNameField"
                width: parent ? parent.width : implicitWidth
                onAccepted: dialog.accept()
            }
        }
    }
    function askShotImage() {
        shotId = controller.selectedShot
        if (!imageDialog) {
            imageDialog = imageDialogComponent.createObject(menu, { objectName: "ctxImageDialog" })
        }
        // After the menu has closed: a dialog opened under an open menu gets no keyboard focus.
        Qt.callLater(() => imageDialog.open())
    }
    function askShortName() {
        shotId = controller.selectedShot
        if (!nameDialog) {
            nameDialog = nameDialogComponent.createObject(menu, { objectName: "ctxNameDialog" })
        }
        nameDialog.text = info.shortNameNow || ""
        nameDialog.open()
    }

    QQC2.MenuSeparator { visible: menu.layer !== "" }
    KanteMenuItem {
        objectName: "ctxLockLayer"
        visible: menu.layer !== ""
        text: menu.texts.t("ctx.lockLayer", { layer: menu.layerName })
        onTriggered: menu.controller.setLock(menu.layer, "locked")
    }
    KanteMenuItem {
        objectName: "ctxFadeLayer"
        visible: menu.layer !== ""
        text: menu.texts.t("ctx.fadeLayer", { layer: menu.layerName, now: menu.fade, next: menu.nextFade })
        onTriggered: menu.controller.cycleFade(menu.layer)
    }
    KanteMenuItem {
        objectName: "ctxHideLayer"
        visible: menu.layer !== ""
        text: menu.texts.t("ctx.hideLayer", { layer: menu.layerName })
        onTriggered: menu.controller.setLayer(menu.layer, "hidden")
    }

    QQC2.MenuSeparator { visible: menu.onElement }
    KanteMenuItem {
        objectName: "ctxDeselect"
        visible: menu.onElement
        text: menu.texts.t("ctx.deselect")
        shortcutText: "Esc"
        onTriggered: menu.controller.clearSelection()
    }

    // Empty ground.
    KanteMenuItem {
        objectName: "ctxUndo"
        visible: !menu.onElement
        enabled: menu.controller.canUndo
        text: menu.texts.t("menu.undo")
        shortcutText: menu.keyName("Ctrl+Z")
        onTriggered: menu.controller.undo()
    }
    KanteMenuItem {
        objectName: "ctxRedo"
        visible: !menu.onElement
        enabled: menu.controller.canRedo
        text: menu.texts.t("menu.redo")
        shortcutText: menu.keyName("Ctrl+Shift+Z")
        onTriggered: menu.controller.redo()
    }
    KanteMenuItem {
        objectName: "ctxClearSketches"
        visible: !menu.onElement
        text: menu.texts.t("sketch.clear")
        onTriggered: menu.controller.clearSketches()
    }
    KanteMenuItem {
        objectName: "ctxFit"
        visible: !menu.onElement
        text: menu.texts.t("menu.zoomFit")
        shortcutText: menu.keyName("Ctrl+0")
        onTriggered: menu.fitRequested()
    }
    Instantiator {
        model: menu._lapsed
        delegate: KanteMenuItem {
            required property var modelData
            objectName: "ctxBack." + modelData.layer + (modelData.unlock ? ".unlock" : ".show")
            text: menu.texts.t(modelData.unlock ? "ctx.unlockLayer" : "ctx.showLayer", { layer: menu.texts.t("layer." + modelData.layer) })
            onTriggered: modelData.unlock ? menu.controller.setLock(modelData.layer, "unlocked") : menu.controller.setLayer(modelData.layer, "shown")
        }
        onObjectAdded: (index, object) => menu.addItem(object)
        onObjectRemoved: (index, object) => menu.removeItem(object)
    }
}
