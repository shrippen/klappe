import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The open questions about the book (DATENMODELL §4): a register entry the screenplay could not
// place. One question at a time: the scenes of the book it may be, best first, and "not in the
// book". Arrows and Enter choose, a click answers; Close (Esc) leaves the questions open. The
// next question follows; the dialog closes after the last.
Item {
    id: root

    required property var texts
    required property var controller
    /// The open questions, `[{ id, number, heading, candidates: [{ number, heading }] }]`.
    readonly property var questions: controller.projectJson ? (JSON.parse(controller.projectJson).pending || []) : []
    readonly property var question: questions.length > 0 ? questions[0] : null
    readonly property alias dialog: dialog

    function open() {
        if (question) {
            dialog.open()
        }
    }

    // The candidates, then the last row; its position is `-1`, which the controller reads as
    // "not in the book".
    readonly property var options: question
        ? question.candidates.map((c, i) => ({ index: i, number: c.number, text: c.heading }))
            .concat([{ index: -1, number: "", text: texts.t("book.none") }])
        : []

    function answer(index) {
        if (!question) {
            return
        }
        controller.resolveScene(question.id, index)
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
        title: root.texts.t("book.title")
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
                text: root.question ? root.texts.t("book.ask", { number: root.question.number, heading: root.question.heading }) : ""
            }

            ListView {
                id: rows
                objectName: "rows"
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
