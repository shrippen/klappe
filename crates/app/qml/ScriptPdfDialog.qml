import QtQuick
import QtQuick.Dialogs
import Kante

// DB-70: asks how the screenplay is output as PDF, then where. The paper starts from the
// project's and is kept there by the export; scene numbers and header line are asked each
// time. Cancelling either step writes nothing. The folder is kept for the session.
//
//   request() → Papier A4 | Letter, Szenennummern, Kopfzeile → file dialog
//             → controller.exportScript("a4", true, false, file)
FormDialog {
    id: dialog

    /// The papers offered, `{ id, label }` each: the sizes of the project view that the
    /// screenplay can be paged for.
    property var papers: []
    /// The project's paper for the screenplay, as an id; the dialog starts from it.
    property string projectPaper: ""
    /// What is chosen.
    property string paper: "a4"
    property bool numbers: true
    property bool headerLine: false
    /// Mark the revisions (DB-41); asked only while a version is locked, else never sent.
    property bool revisions: true
    /// How many versions are locked; the switch shows from one.
    property int versions: 0
    /// The folder of the last output, as a `file://` URL; empty before the first one.
    property string folder: ""

    title: texts.t("script.pdfTitle")
    acceptText: texts.t("export.next")

    function request() {
        if (projectPaper !== "") {
            paper = projectPaper
        }
        open()
    }

    // The file dialog, offered the project's title in the folder of the last output.
    function pick() {
        files.title = texts.t("script.pdfPick")
        files.nameFilters = [texts.t("script.pdfFilter")]
        files.defaultSuffix = "pdf"
        files.currentFile = suggestion()
        files.open()
    }

    function suggestion() {
        const url = controller.scriptFile()
        if (url === "" || folder === "") {
            return url
        }
        return folder + url.substring(url.lastIndexOf("/"))
    }

    // The file was chosen: remembers the folder and writes.
    function save(url) {
        dialog.folder = url.substring(0, url.lastIndexOf("/"))
        return controller.exportScript(paper, numbers ? "shown" : "hidden", headerLine ? "title" : "off",
            revisions && versions > 0 ? "on" : "off", url)
    }

    submit: function () {
        pick()
        return true
    }

    KanteSectionLabel { text: dialog.texts.t("script.paper") }
    KanteSegmented {
        objectName: "paper"
        model: dialog.papers.map(p => p.label)
        currentIndex: dialog.papers.findIndex(p => p.id === dialog.paper)
        onActivated: index => {
            dialog.paper = dialog.papers[index].id
            // The segment control assigns its index and so drops the binding; the next
            // opening would show the paper chosen last time, not the project's.
            currentIndex = Qt.binding(() => dialog.papers.findIndex(p => p.id === dialog.paper))
        }
    }
    KanteSwitch {
        objectName: "numbers"
        text: dialog.texts.t("script.sceneNumbers")
        checked: dialog.numbers
        onToggled: dialog.numbers = checked
    }
    KanteSwitch {
        objectName: "header"
        text: dialog.texts.t("script.header")
        checked: dialog.headerLine
        onToggled: dialog.headerLine = checked
    }
    KanteSwitch {
        objectName: "revisions"
        visible: dialog.versions > 0
        text: dialog.texts.t("script.revisions")
        checked: dialog.revisions
        onToggled: dialog.revisions = checked
    }

    // The file dialog is a window of its own; without a window manager nobody hands the keys
    // back when it closes, and the shortcuts stay dead (Q1-19).
    readonly property var hostWindow: dialog.parent ? dialog.parent.Window.window : null

    FileDialog {
        id: files
        objectName: "files"
        parentWindow: dialog.hostWindow
        fileMode: FileDialog.SaveFile
        onVisibleChanged: {
            if (!visible && dialog.hostWindow) {
                dialog.hostWindow.requestActivate()
            }
        }
        onAccepted: dialog.save(selectedFile.toString())
    }
}
