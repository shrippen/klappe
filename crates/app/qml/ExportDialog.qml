import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import Kante

// LH-80, LH-81, LH-83: asks where an export goes. A plan export (PDF, PNG, SVG, frames) asks
// for the paper (size and orientation) first, then for the file or folder; the shot list asks
// for the file only. Cancelling either writes nothing. The paper starts from the project's and
// is kept there by the export; the folder is kept for the session.
//
//   request("pdf") → paper A4 | A3 | Letter, Hoch | Quer → file dialog
//                  → controller.exportTo("pdf", order, "a4-portrait", file)
//
// A report (DB-61) asks for the file only; the file's extension or the chosen filter sets the
// format:  requestReport("szenen") → file dialog → controller.exportReport("szenen", "csv", file)
// The dialogue of one character (DB-52) asks for the character first:
//   requestReport("dialog") → character → file dialog → controller.exportDialogue(id, "csv", file)
FormDialog {
    id: dialog

    /// The order of the exported shot list: "cut" or "shooting".
    property string order: "cut"
    /// The sizes offered, `{ id, label }` each, the orientations, as ids, and the formats,
    /// `{ id, paper, extension }` each (`paper`: asks for a sheet; `extension`: of the file,
    /// null for a folder); from the project view, so the lists stand once, in the services.
    property var papers: []
    property var orientations: []
    property var formats: []
    /// The project's sheet, as ids; the dialog starts from it.
    property string projectSize: ""
    property string projectOrientation: ""
    /// What is chosen: a size id and an orientation id.
    property string size: "a4"
    property string orientation: "landscape"
    /// Size and orientation as the controller takes them: "a4-portrait".
    readonly property string paper: size + "-" + orientation
    /// The folder of the last export, as a `file://` URL; empty before the first one.
    property string folder: ""
    /// The id of the format being asked for, one of `formats`.
    property string kind: ""
    readonly property var format: formats.find(f => f.id === kind)
    /// The files a report can be written as, as ids (also the extensions); from the project view.
    property var reportFormats: []
    /// The report being asked for, as `szenen`; empty while a plan export is.
    property string report: ""
    /// The characters the dialogue report offers, `{ id, name, replies }` each, most replies
    /// first, and the id of the one chosen.
    property var figures: []
    property string figure: ""
    readonly property bool asksFigure: report === "dialog"

    title: asksFigure ? texts.t("report.pickFigure") : texts.t("export.paperTitle")
    acceptText: texts.t("export.next")

    function requestReport(name) {
        dialog.report = name
        if (!asksFigure) {
            pick()
            return
        }
        figures = JSON.parse(controller.dialogueFigures())
        // The controller has said why there is nobody to choose.
        if (figures.length === 0) {
            return
        }
        figure = figures[0].id
        open()
    }

    function request(kind) {
        dialog.report = ""
        dialog.kind = kind
        if (!format || !format.paper) {
            pick()
            return
        }
        if (projectSize !== "" && projectOrientation !== "") {
            size = projectSize
            orientation = projectOrientation
        }
        open()
    }

    // The file dialog, offered the plan's name in the folder of the last export.
    function pick() {
        if (report !== "") {
            files.title = texts.t("export.pick.report")
            files.nameFilters = reportFormats.map(f => texts.t("export.filter.report." + f))
            files.defaultSuffix = reportFormats[0] || ""
            files.currentFile = suggestion()
            files.open()
            return
        }
        if (format && format.extension === null) {
            folders.title = texts.t("export.pick." + kind)
            folders.currentFolder = suggestion().replace(/\/[^/]*$/, "")
            folders.open()
            return
        }
        files.title = texts.t("export.pick." + kind)
        files.nameFilters = [texts.t("export.filter." + kind)]
        files.defaultSuffix = format ? format.extension : ""
        files.currentFile = suggestion()
        files.open()
    }

    function suggestion() {
        const url = report !== "" ? controller.reportFile(report) : controller.exportFile(kind)
        if (url === "" || folder === "") {
            return url
        }
        return folder + url.substring(url.lastIndexOf("/"))
    }

    // The format of a report file: the extension typed, else the one of the filter chosen.
    function reportFormat(url, filterIndex) {
        const name = url.substring(url.lastIndexOf("/") + 1)
        const typed = name.substring(name.lastIndexOf(".") + 1).toLowerCase()
        if (name.includes(".") && reportFormats.includes(typed)) {
            return typed
        }
        return reportFormats[filterIndex] || reportFormats[0]
    }

    // The file was chosen: remembers the folder and writes.
    function save(url) {
        dialog.folder = url.substring(0, url.lastIndexOf("/"))
        if (asksFigure) {
            return controller.exportDialogue(figure, reportFormat(url, files.selectedNameFilter.index), url)
        }
        if (report !== "") {
            return controller.exportReport(report, reportFormat(url, files.selectedNameFilter.index), url)
        }
        return controller.exportTo(kind, order, paper, url)
    }

    submit: function () {
        pick()
        return true
    }

    KanteSectionLabel { visible: dialog.asksFigure; text: dialog.texts.t("report.figure") }
    KanteComboBox {
        objectName: "figureBox"
        Layout.fillWidth: true
        visible: dialog.asksFigure
        model: dialog.figures.map(f => f.name + " (" + f.replies + ")")
        currentIndex: dialog.figures.findIndex(f => f.id === dialog.figure)
        Accessible.name: dialog.texts.t("report.figure")
        onActivated: index => dialog.figure = dialog.figures[index].id
    }

    KanteSectionLabel { visible: !dialog.asksFigure; text: dialog.texts.t("export.paper") }
    KanteSegmented {
        objectName: "paper"
        visible: !dialog.asksFigure
        model: dialog.papers.map(p => p.label)
        currentIndex: dialog.papers.findIndex(p => p.id === dialog.size)
        onActivated: index => {
            dialog.size = dialog.papers[index].id
            // The segment control assigns its index and so drops the binding; request()
            // would set the project's paper and the display would not follow.
            currentIndex = Qt.binding(() => dialog.papers.findIndex(p => p.id === dialog.size))
        }
    }
    KanteSectionLabel { visible: !dialog.asksFigure; text: dialog.texts.t("export.orientation") }
    KanteSegmented {
        objectName: "orientation"
        visible: !dialog.asksFigure
        model: dialog.orientations.map(o => dialog.texts.t("export." + o))
        currentIndex: dialog.orientations.indexOf(dialog.orientation)
        onActivated: index => {
            dialog.orientation = dialog.orientations[index]
            // As with the paper: keeps the index following the chosen orientation.
            currentIndex = Qt.binding(() => dialog.orientations.indexOf(dialog.orientation))
        }
    }

    // The file dialogs sit in this popup, which has no window while closed; the shot list
    // opens one without the paper form, so they take the window from the popup's parent.
    readonly property var hostWindow: dialog.parent ? dialog.parent.Window.window : null

    // The file dialogs are windows of their own; without a window manager nobody hands the keys
    // back when one closes, and the shortcuts stay dead (Q1-19).
    function reactivate() {
        if (hostWindow) {
            hostWindow.requestActivate()
        }
    }

    // The frames go into a new folder in the chosen one, named as the suggestion.
    FolderDialog {
        id: folders
        objectName: "folders"
        parentWindow: dialog.hostWindow
        onVisibleChanged: if (!visible) dialog.reactivate()
        onAccepted: dialog.save(selectedFolder.toString() + suggestion().substring(suggestion().lastIndexOf("/")))
    }

    FileDialog {
        id: files
        objectName: "files"
        parentWindow: dialog.hostWindow
        fileMode: FileDialog.SaveFile
        onVisibleChanged: if (!visible) dialog.reactivate()
        onAccepted: dialog.save(selectedFile.toString())
    }
}
