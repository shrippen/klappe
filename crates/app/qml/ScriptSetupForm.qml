import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The screenplay's setup in the project dialog (DN-05, DB-31, DB-33): the language, the words
// at a page break (only for German), the paper, "(CONT'D) after action" and who numbers the
// scenes with the first number. Each change goes to the controller at once, as a whole setup;
// the paper and the rest are no undo step (as the plan's paper), the numbering is written to
// the project file at once.
//
//   Sprache  [Deutsch|Englisch]      Papier [A4|Letter]
//   Begriffe am Seitenumbruch  [deutsch|international]
//   [ ] (FORTS.) auch nach Handlung
//   Szenennummern [automatisch|gesperrt]  Erste Nummer [ 1 ]
ColumnLayout {
    id: form

    required property var texts
    required property var controller
    /// `Overview.script`: the names of the setup as they stand.
    required property var script
    /// The papers the screenplay can be paged for, `{ id, label }` each.
    property var papers: []

    readonly property var languages: ["de", "en"]
    readonly property var wordSets: ["local", "international"]
    readonly property var numberings: ["automatic", "locked"]
    /// `script.language` is the effective one (the project's, else the window's); the words are
    /// a choice for German only.
    readonly property bool german: script.language === "de"
    readonly property bool automatic: script.scene_numbering === "automatic"

    // The setup with `key` set to `value`, sent whole; true if it went through.
    function change(key, value) {
        const next = Object.assign({}, script)
        next[key] = value
        return controller.setScriptSetup(next.language, next.words, next.paper,
            next.contd_after_action, next.scene_numbering, next.first_scene)
    }

    spacing: Kirigami.Units.smallSpacing

    KanteSectionLabel { text: form.texts.t("project.script") }

    RowLayout {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        ColumnLayout {
            spacing: 0
            KanteSectionLabel { text: form.texts.t("project.scriptLanguage") }
            KanteSegmented {
                id: languageBox
                objectName: "languageBox"
                model: [form.texts.t("project.languageDe"), form.texts.t("project.languageEn")]
                currentIndex: form.languages.indexOf(form.script.language)
                onActivated: index => {
                    form.change("language", form.languages[index])
                    currentIndex = Qt.binding(() => form.languages.indexOf(form.script.language))
                }
            }
        }
        ColumnLayout {
            spacing: 0
            KanteSectionLabel { text: form.texts.t("project.scriptPaper") }
            KanteSegmented {
                id: paperBox
                objectName: "paperBox"
                model: form.papers.map(p => p.label)
                currentIndex: form.papers.findIndex(p => p.id === form.script.paper)
                onActivated: index => {
                    form.change("paper", form.papers[index].id)
                    currentIndex = Qt.binding(() => form.papers.findIndex(p => p.id === form.script.paper))
                }
            }
        }
    }

    KanteSectionLabel { text: form.texts.t("project.scriptWords") }
    KanteSegmented {
        id: wordsBox
        objectName: "wordsBox"
        enabled: form.german
        model: [form.texts.t("project.wordsLocal"), form.texts.t("project.wordsInternational")]
        currentIndex: form.wordSets.indexOf(form.script.words)
        onActivated: index => {
            form.change("words", form.wordSets[index])
            currentIndex = Qt.binding(() => form.wordSets.indexOf(form.script.words))
        }
    }

    KanteSwitch {
        id: contdSwitch
        objectName: "contdSwitch"
        text: form.texts.t("project.contdAfterAction")
        checked: form.script.contd_after_action
        onToggled: {
            form.change("contd_after_action", checked)
            checked = Qt.binding(() => form.script.contd_after_action)
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        ColumnLayout {
            spacing: 0
            KanteSectionLabel { text: form.texts.t("project.sceneNumbering") }
            KanteSegmented {
                id: numberingSegments
                objectName: "sceneNumberingBox"
                model: [form.texts.t("project.sceneAutomatic"), form.texts.t("project.sceneLocked")]
                currentIndex: form.numberings.indexOf(form.script.scene_numbering)
                onActivated: index => {
                    form.change("scene_numbering", form.numberings[index])
                    currentIndex = Qt.binding(() => form.numberings.indexOf(form.script.scene_numbering))
                }
            }
        }
        ColumnLayout {
            spacing: 0
            KanteSectionLabel { text: form.texts.t("project.firstScene") }
            KanteSpinBox {
                id: firstScene
                objectName: "firstScene"
                enabled: form.automatic
                from: 1
                to: 9999
                editable: true
                value: form.script.first_scene
                Accessible.name: form.texts.t("project.firstScene")
                onValueModified: {
                    form.change("first_scene", value)
                    value = Qt.binding(() => form.script.first_scene)
                }
            }
        }
    }
    QQC2.Label {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.texts.t("project.sceneNumberingHint")
    }
}
