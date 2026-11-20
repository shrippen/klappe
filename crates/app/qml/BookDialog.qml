import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The open questions about the book (DATENMODELL §4): a register entry the screenplay could not
// place, then a place name of the scene headings no scripted location is called by (DB-51).
// One question at a time: the scenes of the book it may be, best first, and "not in the book";
// for a location the scripted locations it may be, or a new one. Arrows and Enter choose, a click
// answers; Close (Esc) leaves the questions open. The next question follows; the dialog closes
// after the last.
Item {
    id: root

    required property var texts
    required property var controller
    /// The open questions in the order they are asked: the scenes (`pending`), then the place
    /// names of the headings (`pendingPlaces`), `[{ kind, key, title, ask, answers: [{ number, text }] }]`.
    readonly property var questions: {
        const project = controller.projectJson ? JSON.parse(controller.projectJson) : {}
        const scenes = (project.pending || []).map(q => ({
            kind: "scene", key: q.id, title: texts.t("book.title"),
            ask: texts.t("book.ask", { number: q.number, heading: q.heading }),
            answers: q.answers.map(a => ({ number: a.number, text: a.none ? texts.t("book.none") : a.heading }))
        }))
        const places = (project.pendingPlaces || []).map(q => ({
            kind: "place", key: q.name, title: texts.t("book.placeTitle"),
            ask: texts.t("book.placeAsk", { name: q.name, count: q.headings }),
            answers: q.answers.map(a => ({ number: "", text: placeText(a, q.name) }))
        }))
        return scenes.concat(places)
    }
    readonly property var question: questions.length > 0 ? questions[0] : null
    readonly property alias dialog: dialog

    // "New scripted location HOF", "Spelling of SAAL" or "SAAL is now called HOF".
    function placeText(answer, name) {
        const key = { create: "book.placeNew", rename: "book.placeRename", variant: "book.placeVariant" }[answer.kind]
        return texts.t(key, { name: name, old: answer.name, new: name })
    }

    function open() {
        if (question) {
            dialog.open()
        }
    }

    // The answers as the session lists them; the controller takes the position.
    readonly property var options: question ? question.answers.map((a, i) => ({ index: i, number: a.number, text: a.text })) : []

    function answer(index) {
        if (!question) {
            return
        }
        if (question.kind === "place") {
            controller.resolvePlace(question.key, index)
        } else {
            controller.resolveScene(question.key, index)
        }
        rows.currentIndex = 0
        if (!question) {
            dialog.close()
        }
    }

    KanteDialog {
        id: dialog
        objectName: "dialog"
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Medium
        title: root.question ? root.question.title : root.texts.t("book.title")
        standardButtons: QQC2.Dialog.Close
        rejectText: root.texts.t("book.later")
        onAboutToShow: rows.currentIndex = 0
        onOpened: rows.forceActiveFocus()

        ColumnLayout {
            width: parent ? parent.width : implicitWidth
            spacing: Kirigami.Units.smallSpacing

            QQC2.Label {
                objectName: "ask"
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: KanteStyle.textColor
                text: root.question ? root.question.ask : ""
            }

            ListView {
                id: rows
                objectName: "rows"
                Accessible.name: root.question ? root.question.ask : ""
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(contentHeight, Kirigami.Units.gridUnit * 16)
                clip: true
                model: root.options
                keyNavigationEnabled: true
                Keys.onReturnPressed: root.answer(root.options[currentIndex].index)
                Keys.onEnterPressed: root.answer(root.options[currentIndex].index)
                delegate: KanteListRow {
                    required property var modelData
                    required property int index
                    width: ListView.view.width
                    leadingText: modelData.number
                    leadingWidth: Kirigami.Units.gridUnit * 3
                    text: modelData.text
                    selected: ListView.isCurrentItem
                    onClicked: root.answer(modelData.index)
                }
            }
        }
    }
}
