import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The card of a scene or a character, opened from the script (D-08): its properties and where
// it occurs. The numbers come from the controller's `usesOf`; the button hands the entry on to
// the project dialog, where it is changed.
KantePopover {
    id: card
    objectName: "refCard"

    required property var texts
    required property var controller

    // "scene" or "character".
    property string kind: ""
    property string refId: ""
    // `usesOf`: a scene's card, or the list of scenes a character plays in.
    property var uses: null

    readonly property var project: controller.projectJson
        ? JSON.parse(controller.projectJson)
        : { scenes: [], characters: [] }
    readonly property var scene: kind === "scene" ? project.scenes.find(s => s.id === refId) || null : null
    readonly property var character: kind === "character" ? project.characters.find(c => c.id === refId) || null : null

    // `uses` by what it is: a scene's card (or null), a character's scenes (or empty).
    readonly property var sceneUses: kind === "scene" && uses ? uses : null
    readonly property var characterUses: kind === "character" && Array.isArray(uses) ? uses : []

    signal editRequested(string kind, string id)

    function showAt(kind, id, item) {
        card.kind = kind
        card.refId = id
        const raw = card.controller.usesOf(kind, id)
        card.uses = raw ? JSON.parse(raw) : null
        card.openAt(item)
    }

    // "1 Replik" or "3 Repliken".
    function countText(one, many, count) {
        return count === 1 ? texts.t(one) : texts.t(many, { count: count })
    }

    ColumnLayout {
        Layout.preferredWidth: Kirigami.Units.gridUnit * 20
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            spacing: Kirigami.Units.smallSpacing

            KanteSwatch {
                visible: card.character !== null
                swatchColor: card.character ? KanteStyle.labelColor(card.character.color) : KanteStyle.focusColor
                size: KanteSwatch.Size.Small
            }
            KanteHeading {
                objectName: "cardTitle"
                Layout.fillWidth: true
                level: 4
                wrapMode: Text.Wrap
                text: card.scene ? (card.scene.number + " " + card.scene.heading).trim()
                    : card.character ? card.character.name : ""
            }
        }

        // A scene: its place and time, what is planned there, who plays in it.
        ColumnLayout {
            objectName: "sceneFacts"
            Layout.fillWidth: true
            visible: card.scene !== null && card.sceneUses !== null
            spacing: Kirigami.Units.smallSpacing

            Fact {
                label: card.texts.t("scene.location")
                value: card.scene ? card.scene.locationName : ""
            }
            Fact {
                visible: card.sceneUses !== null && card.sceneUses.time !== ""
                label: card.texts.t("card.time")
                value: card.sceneUses ? card.sceneUses.time : ""
            }
            Fact {
                label: card.texts.t("card.plans")
                value: card.sceneUses ? String(card.sceneUses.plans) : ""
            }
            Fact {
                label: card.texts.t("card.shots")
                value: card.sceneUses ? String(card.sceneUses.shots) : ""
            }

            KanteSectionLabel {
                visible: card.sceneUses !== null && card.sceneUses.cast.length > 0
                text: card.texts.t("card.cast")
            }
            Flow {
                Layout.fillWidth: true
                spacing: Kirigami.Units.smallSpacing

                Repeater {
                    model: card.sceneUses ? card.sceneUses.cast : []
                    delegate: KanteChip {
                        required property var modelData
                        interactive: false
                        chipColor: KanteStyle.labelColor(modelData.color)
                        text: modelData.cues > 0
                            ? modelData.name + " · " + card.countText("card.cue", "card.cues", modelData.cues)
                            : modelData.name
                    }
                }
            }
        }

        // A character: its label and group, and the scenes it is in.
        ColumnLayout {
            objectName: "characterFacts"
            Layout.fillWidth: true
            visible: card.character !== null && card.uses !== null
            spacing: Kirigami.Units.smallSpacing

            Fact {
                label: card.texts.t("character.short")
                value: card.character ? card.character.short : ""
            }
            Fact {
                visible: card.character && card.character.group > 0
                label: card.texts.t("character.group")
                value: card.character ? card.texts.t("character.groupOf", { count: card.character.group }) : ""
            }

            KanteSectionLabel { text: card.texts.t("card.uses") }
            QQC2.Label {
                objectName: "noUses"
                Layout.fillWidth: true
                visible: card.characterUses.length === 0
                color: KanteStyle.mutedTextColor
                text: card.texts.t("card.noUses")
            }
            Repeater {
                model: card.characterUses
                delegate: RowLayout {
                    required property var modelData
                    objectName: "use"
                    Layout.fillWidth: true
                    spacing: Kirigami.Units.smallSpacing

                    QQC2.Label {
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                        color: KanteStyle.textColor
                        text: (modelData.number + " " + modelData.heading).trim()
                    }
                    QQC2.Label {
                        color: KanteStyle.mutedTextColor
                        text: [
                            modelData.cues > 0 ? card.countText("card.cue", "card.cues", modelData.cues) : "",
                            modelData.figures > 0 ? card.countText("card.inPlan", "card.inPlans", modelData.figures) : ""
                        ].filter(t => t !== "").join(" · ")
                    }
                }
            }
        }

        KanteButton {
            objectName: "editInProject"
            Layout.alignment: Qt.AlignRight
            text: card.texts.t("card.open")
            onClicked: {
                card.close()
                card.editRequested(card.kind, card.refId)
            }
        }
    }

    // One property of the card: its name above or beside its value.
    component Fact: RowLayout {
        property string label: ""
        property string value: ""
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        KanteSectionLabel {
            Layout.preferredWidth: Kirigami.Units.gridUnit * 7
            text: parent.label
        }
        QQC2.Label {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            color: KanteStyle.textColor
            text: parent.value
        }
    }
}
