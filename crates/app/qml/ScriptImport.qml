import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import QtQuick.Dialogs
import Kante

// "Load screenplay …": the file dialog for a Fountain file. A project that has a screenplay
// is asked first; the file replaces the text, and nothing is saved by it. The register follows
// the book at once; a report says how many scenes were found, and offers the open questions.
Item {
    id: imp

    required property var texts
    required property var controller
    readonly property alias dialog: ask
    readonly property alias picker: picker
    readonly property alias report: report
    /// The counts of the last import, `{ scenes, created, pending, notInBook }`.
    property var counts: ({ scenes: 0, created: 0, pending: 0, notInBook: 0 })

    // "14 Szenen erkannt · 3 neu · 2 Fragen", with the singular for one scene or question.
    function countsText() {
        const one = (n, single, many) => n === 1 ? texts.t(single) : texts.t(many, { count: n })
        return [one(counts.scenes, "script.reportScenes1", "script.reportScenes"),
                texts.t("script.reportNew", { count: counts.created }),
                one(counts.pending, "script.reportQuestion1", "script.reportQuestions")].join(" · ")
    }

    /// The open questions were asked for.
    signal questionsRequested()

    /// Asks (only for a project with a screenplay), then offers the file dialog.
    function start() {
        if (controller.scriptText.trim() === "") {
            picker.open()
            return
        }
        ask.open()
    }

    /// Imports the file and reports; a refused file is on the status line only.
    function load(path) {
        if (!controller.importScript(path)) {
            return
        }
        counts = JSON.parse(controller.importReport)
        report.open()
    }

    KanteDialog {
        id: ask
        objectName: "importAsk"
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        title: imp.texts.t("script.import")
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: imp.texts.t("script.importReplace")
        rejectText: imp.texts.t("form.cancel")
        onAccepted: picker.open()
        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: imp.texts.t("script.importAsk")
        }
    }

    FileDialog {
        id: picker
        onVisibleChanged: {
            if (!visible && Window.window) {
                Window.window.requestActivate()
            }
        }
        title: imp.texts.t("script.importPick")
        nameFilters: [imp.texts.t("script.filter")]
        onAccepted: imp.load(selectedFile.toString())
    }

    KanteDialog {
        id: report
        objectName: "importReport"
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        title: imp.texts.t("script.reportTitle")
        standardButtons: imp.counts.pending > 0 ? QQC2.Dialog.Ok | QQC2.Dialog.Close : QQC2.Dialog.Close
        acceptText: imp.texts.t("script.reportAsk")
        rejectText: imp.texts.t("book.later")
        onAccepted: imp.questionsRequested()
        ColumnLayout {
            width: parent ? parent.width : implicitWidth
            QQC2.Label {
                objectName: "reportCounts"
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: KanteStyle.textColor
                text: imp.countsText()
            }
            QQC2.Label {
                objectName: "reportGone"
                Layout.fillWidth: true
                visible: imp.counts.notInBook > 0
                wrapMode: Text.WordWrap
                color: KanteStyle.textColor
                text: imp.texts.t("script.reportGone", { count: imp.counts.notInBook })
            }
        }
    }
}
