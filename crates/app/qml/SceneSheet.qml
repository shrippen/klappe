import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-22: the sheet of the scene on view. Story day, pre-stop, planned shooting time, continuity and
// the note for the production; the figures (those with a cue are fixed, silent ones and extras
// come and go); the elements per category, only the categories that have some; and what the set
// suggests. A field is taken when its editing ends: one undo step per change.
//
//  ┌ 12  INT. SAAL – TAG ────────────────────────┐
//  │ Spieltag [3]  Vorstopp [1:30]  Drehdauer [2:00]│
//  │ FIGUREN   ANNA  BERT  [Fischer ×3 ×]  [Figur hinzufügen ▾]│
//  │ REQUISITE [Brief ×] [Kerze ×]  [ ____ ]        │
//  │ [Kategorie hinzufügen ▾]                       │
//  │ VORSCHLÄGE AUS DEM SET  ┆Vase┆ ✓ ×              │
//  └────────────────────────────────────────────────┘
Flickable {
    id: sheetView

    required property var texts
    required property var controller

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : ({})
    readonly property var sheet: project.sheet || null
    readonly property var scene: (project.scenes || []).find(s => s.id === controller.sceneId) || null
    readonly property var categories: project.element_categories || []
    readonly property var elements: project.elements || []
    /// Categories opened by "Add category" that have no element yet.
    property var extraCategories: []
    /// The categories a row is shown for, in the catalogue's order.
    readonly property var shownCategories: categories.filter(c => extraCategories.includes(c)
        || (sheet !== null && sheet.categories.some(g => g.category === c)))
    readonly property var otherCategories: categories.filter(c => !shownCategories.includes(c))
    /// Project characters that are neither speaking nor present.
    readonly property var otherFigures: !sheet ? [] : (project.characters || []).filter(c =>
        !sheet.speaking.some(f => f.id === c.id) && !sheet.present.some(f => f.id === c.id))

    clip: true
    contentWidth: width
    contentHeight: column.implicitHeight + Kirigami.Units.largeSpacing * 2
    boundsBehavior: Flickable.StopAtBounds
    QQC2.ScrollBar.vertical: QQC2.ScrollBar {}

    onSceneChanged: {
        extraCategories = []
        fill()
    }
    onSheetChanged: fill()

    // The fields show the sheet, except the one being typed in.
    function fill() {
        if (!sheet) {
            return
        }
        const shown = [
            [dayField, sheet.story_day],
            [prestopField, sheet.prestop_text],
            [plannedField, sheet.planned_text],
            [continuityField, sheet.continuity],
            [noteField, sheet.note]
        ]
        for (const [field, text] of shown) {
            if (!field.activeFocus) {
                field.text = text
                field.invalid = false
            }
        }
    }

    // Sends the sheet when a field differs from it; the session reads the times and the field
    // it refuses is marked.
    function commit() {
        if (!sheet || !scene) {
            return
        }
        const same = dayField.text === sheet.story_day && prestopField.text === sheet.prestop_text
            && plannedField.text === sheet.planned_text && continuityField.text === sheet.continuity
            && noteField.text === sheet.note
        if (same) {
            return
        }
        const refused = controller.setSheet(scene.id, dayField.text, prestopField.text, plannedField.text,
                                            continuityField.text, noteField.text)
        prestopField.invalid = refused === "prestop"
        plannedField.invalid = refused === "planned"
    }

    function statusText(status) {
        return texts.t("elementStatus." + status)
    }

    // The names of the category's elements that this scene does not list yet.
    function offered(category) {
        const group = sheet ? sheet.categories.find(g => g.category === category) : null
        const have = group ? group.elements.map(e => e.id) : []
        return elements.filter(e => e.category === category && !have.includes(e.id))
    }

    function add(category, name) {
        if (name.trim() !== "") {
            controller.addElementToScene(scene.id, category, name)
        }
    }

    function nameOfFigure(f) {
        return f.group > 0 ? f.name + " ×" + f.group : f.name
    }

    ColumnLayout {
        id: column
        x: Kirigami.Units.largeSpacing
        y: Kirigami.Units.largeSpacing
        width: Math.min(sheetView.width - Kirigami.Units.largeSpacing * 2, Kirigami.Units.gridUnit * 50)
        spacing: Kirigami.Units.largeSpacing

        QQC2.Label {
            objectName: "noScene"
            visible: !sheetView.sheet
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            color: KanteStyle.mutedTextColor
            text: sheetView.texts.t("sheet.none")
        }

        KanteHeading {
            objectName: "sheetTitle"
            visible: sheetView.sheet !== null && sheetView.scene !== null
            Layout.fillWidth: true
            level: 3
            text: sheetView.scene ? (sheetView.scene.number + "  " + sheetView.scene.heading).trim() : ""
        }

        Flow {
            visible: sheetView.sheet !== null
            Layout.fillWidth: true
            spacing: Kirigami.Units.largeSpacing

            ColumnLayout {
                spacing: 0
                KanteSectionLabel { text: sheetView.texts.t("sheet.storyDay") }
                KanteTextField {
                    id: dayField
                    objectName: "dayField"
                    Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                    Accessible.name: sheetView.texts.t("sheet.storyDay")
                    onEditingFinished: sheetView.commit()
                }
            }
            ColumnLayout {
                spacing: 0
                KanteSectionLabel { text: sheetView.texts.t("sheet.prestop") }
                KanteTextField {
                    id: prestopField
                    objectName: "prestopField"
                    Layout.preferredWidth: Kirigami.Units.gridUnit * 7
                    Accessible.name: sheetView.texts.t("sheet.prestop")
                    onTextEdited: invalid = false
                    onEditingFinished: sheetView.commit()
                }
            }
            ColumnLayout {
                spacing: 0
                KanteSectionLabel { text: sheetView.texts.t("sheet.planned") }
                KanteTextField {
                    id: plannedField
                    objectName: "plannedField"
                    Layout.preferredWidth: Kirigami.Units.gridUnit * 10
                    Accessible.name: sheetView.texts.t("sheet.planned")
                    onTextEdited: invalid = false
                    onEditingFinished: sheetView.commit()
                }
            }
        }

        ColumnLayout {
            visible: sheetView.sheet !== null
            Layout.fillWidth: true
            spacing: 0
            KanteSectionLabel { text: sheetView.texts.t("sheet.continuity") }
            KanteTextField {
                id: continuityField
                objectName: "continuityField"
                Layout.fillWidth: true
                Accessible.name: sheetView.texts.t("sheet.continuity")
                onEditingFinished: sheetView.commit()
            }
        }

        ColumnLayout {
            visible: sheetView.sheet !== null
            Layout.fillWidth: true
            spacing: 0
            KanteSectionLabel { text: sheetView.texts.t("sheet.note") }
            KanteTextField {
                id: noteField
                objectName: "noteField"
                Layout.fillWidth: true
                Accessible.name: sheetView.texts.t("sheet.note")
                onEditingFinished: sheetView.commit()
            }
        }

        // The figures: a cue in the book fixes a chip, the others can leave.
        ColumnLayout {
            visible: sheetView.sheet !== null
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: sheetView.texts.t("sheet.characters"); rule: true; Layout.fillWidth: true }
            Flow {
                Layout.fillWidth: true
                spacing: Kirigami.Units.smallSpacing

                Repeater {
                    model: sheetView.sheet ? sheetView.sheet.speaking : []
                    delegate: KanteChip {
                        required property var modelData
                        objectName: "speaking." + modelData.id
                        text: sheetView.nameOfFigure(modelData)
                        chipColor: KanteStyle.labelColor(modelData.color)
                        interactive: false
                        toolTip: sheetView.texts.t("sheet.speaks")
                    }
                }
                Repeater {
                    model: sheetView.sheet ? sheetView.sheet.present : []
                    delegate: KanteChip {
                        required property var modelData
                        objectName: "present." + modelData.id
                        text: sheetView.nameOfFigure(modelData)
                        chipColor: KanteStyle.labelColor(modelData.color)
                        removable: true
                        interactive: false
                        removeText: sheetView.texts.t("sheet.characterRemove", { name: "%1" })
                        onRemoveRequested: sheetView.controller.removeSceneCharacter(sheetView.scene.id, modelData.id)
                    }
                }
            }
            KanteComboBox {
                id: figureBox
                objectName: "figureBox"
                enabled: sheetView.otherFigures.length > 0
                Layout.preferredWidth: Kirigami.Units.gridUnit * 14
                Accessible.name: sheetView.texts.t("sheet.addCharacter")
                displayText: sheetView.texts.t(enabled ? "sheet.addCharacter" : "sheet.noCharacterLeft")
                model: sheetView.otherFigures.map(f => sheetView.nameOfFigure(f))
                currentIndex: -1
                onActivated: index => {
                    sheetView.controller.addSceneCharacter(sheetView.scene.id, sheetView.otherFigures[index].id)
                    currentIndex = -1
                }
            }
        }

        // The elements: a row for each category that has some.
        ColumnLayout {
            visible: sheetView.sheet !== null
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: sheetView.texts.t("sheet.elements"); rule: true; Layout.fillWidth: true }
            QQC2.Label {
                objectName: "noElements"
                visible: sheetView.shownCategories.length === 0
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                color: KanteStyle.mutedTextColor
                text: sheetView.texts.t("sheet.noElements")
            }

            Repeater {
                model: sheetView.shownCategories
                delegate: ColumnLayout {
                    id: categoryRow
                    required property string modelData
                    readonly property var own: sheetView.sheet.categories.find(g => g.category === modelData)
                    objectName: "category." + modelData
                    Layout.fillWidth: true
                    spacing: Kirigami.Units.smallSpacing

                    KanteSectionLabel { text: sheetView.texts.t("elementCategory." + categoryRow.modelData) }
                    Flow {
                        Layout.fillWidth: true
                        visible: categoryRow.own !== undefined
                        spacing: Kirigami.Units.smallSpacing

                        Repeater {
                            model: categoryRow.own ? categoryRow.own.elements : []
                            delegate: ElementChip {
                                required property var modelData
                                objectName: "element." + modelData.id
                                text: modelData.name
                                status: modelData.status
                                removable: true
                                removeText: sheetView.texts.t("sheet.elementRemove", { name: "%1" })
                                toolTip: sheetView.texts.t("sheet.elementTip", {
                                    category: sheetView.texts.t("elementCategory." + categoryRow.modelData),
                                    status: sheetView.statusText(modelData.status) })
                                onRemoveRequested: sheetView.controller.removeSceneElement(sheetView.scene.id, modelData.id)
                            }
                        }
                    }
                    // Typing narrows the elements of this category the scene lacks; Enter takes the
                    // one under the cursor, or makes the typed one.
                    KanteSearchCombo {
                        objectName: "add." + categoryRow.modelData
                        Layout.preferredWidth: Kirigami.Units.gridUnit * 18
                        keepText: false
                        allowNew: true
                        model: sheetView.offered(categoryRow.modelData).map(e => e.name)
                        placeholderText: sheetView.texts.t("sheet.elementAdd", {
                            category: sheetView.texts.t("elementCategory." + categoryRow.modelData) })
                        newText: sheetView.texts.t("form.add") + " “%1”"
                        onActivated: index => sheetView.add(categoryRow.modelData, model[index])
                        onNewEntered: text => sheetView.add(categoryRow.modelData, text)
                    }
                }
            }

            KanteComboBox {
                objectName: "categoryBox"
                visible: sheetView.otherCategories.length > 0
                Layout.preferredWidth: Kirigami.Units.gridUnit * 14
                Accessible.name: sheetView.texts.t("sheet.addCategory")
                displayText: sheetView.texts.t("sheet.addCategory")
                model: sheetView.otherCategories.map(c => sheetView.texts.t("elementCategory." + c))
                currentIndex: -1
                onActivated: index => {
                    sheetView.extraCategories = sheetView.extraCategories.concat([sheetView.otherCategories[index]])
                    currentIndex = -1
                }
            }
        }

        // The set's objects that stand for an element this sheet does not list. Not the sheet's
        // yet: the chip is dimmed (Kante has no dashed chip), ✓ takes it, × turns it down.
        ColumnLayout {
            visible: sheetView.sheet !== null && sheetView.sheet.suggestions.length > 0
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: sheetView.texts.t("sheet.suggestions"); rule: true; Layout.fillWidth: true }
            Flow {
                Layout.fillWidth: true
                spacing: Kirigami.Units.largeSpacing

                Repeater {
                    model: sheetView.sheet ? sheetView.sheet.suggestions : []
                    delegate: RowLayout {
                        id: suggestion
                        required property var modelData
                        objectName: "suggestion." + modelData.element
                        spacing: Kirigami.Units.smallSpacing

                        ElementChip {
                            objectName: "suggestionChip"
                            opacity: KanteStyle.dimOpacity
                            text: suggestion.modelData.name
                            removable: true
                            interactive: false
                            removeText: sheetView.texts.t("sheet.exclude", { name: "%1" })
                            toolTip: sheetView.texts.t("elementCategory." + suggestion.modelData.category)
                                + " · " + sheetView.texts.t("sheet.suggestionFrom", { object: suggestion.modelData.object })
                            onRemoveRequested: sheetView.controller.excludeSuggestion(sheetView.scene.id, suggestion.modelData.element)
                        }
                        KanteButton {
                            objectName: "take"
                            size: KanteButton.Size.Small
                            text: "✓"
                            Accessible.name: sheetView.texts.t("sheet.take", { name: suggestion.modelData.name })
                            onClicked: sheetView.controller.takeSuggestion(sheetView.scene.id, suggestion.modelData.element)
                        }
                    }
                }
            }
        }
    }
}
