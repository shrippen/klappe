import QtQuick

// DB-40 to DB-42: the locked versions of the book and the marks of what changed since the last
// one. Reads the list and the changed lines from the session, and holds the dialogs that lock,
// compare and remove a version. The screenplay view (ScreenplayView) shows the menu and the marks.
// No size: it only holds the dialogs.
Item {
    id: part

    required property var texts
    required property var controller
    /// The text seam (ScriptDoc), for the positions of the changed lines.
    required property var doc
    /// What was typed goes to the session first.
    signal flushRequested()
    /// The pages are laid out anew.
    signal layoutRequested()

    /// The locked versions (DB-40), `{ id, name, date, color, colorName }`, and the colour of the
    /// revision since the last one (`{ color, colorName }`, null before the first version).
    property var versions: []
    property var nextRevision: null
    /// Mark the lines that changed since the last version (DB-41); on as soon as there is one.
    property bool showRevisions: true
    /// Those lines (from 0) of the text as of the last layout; they follow it like the page
    /// breaks (DN-02).
    property var revisionLines: []
    /// The same as text positions `{ from, to }` of the text at that layout.
    property var revisionRanges: []
    readonly property alias lockDialog: lockDialog
    readonly property alias compareDialog: compareDialog
    readonly property alias removeDialog: removeDialog

    /// Reads the locked versions the session holds.
    /// The project changes with every typing that is sent: only a changed list is taken over and
    /// marked anew (DN-02).
    function readVersions() {
        const json = controller.versionsJson()
        if (json === _versionsRead) {
            return
        }
        _versionsRead = json
        const read = JSON.parse(json)
        versions = read.versions
        nextRevision = read.next
        readRevision()
    }
    property string _versionsRead: ""

    /// The lines changed since the last version, as of the text the session holds now.
    function readRevision() {
        const marking = versions.length > 0 && showRevisions
        revisionLines = marking ? JSON.parse(controller.revisionJson()) : []
        // Placed once per layout, like the page breaks' lines: typing moves no mark (DN-02),
        // a mark cost a text position lookup twice, on every key.
        revisionRanges = lineRanges(revisionLines)
    }

    /// The lines of `lines` (ascending, from 0) as text positions `{ from, to }`, `to` the end of
    /// the line; `from` is -1 when the text no longer has the line.
    function lineRanges(lines) {
        const text = doc.read()
        return doc.lineStarts(lines).map(at => {
            if (at < 0) {
                return { from: -1, to: -1 }
            }
            const end = text.indexOf("\n", at)
            return { from: at, to: end < 0 ? text.length : end }
        })
    }

    /// The version's dialogs; what was typed goes to the session first, the lock saves it.
    function askLock() {
        flushRequested()
        lockDialog.show()
    }

    function askCompare(from) {
        flushRequested()
        compareDialog.show(from)
    }

    /// Shows or hides the marks of the revision.
    function setRevisions(shown) {
        flushRequested()
        showRevisions = shown
        readRevision()
    }

    VersionLockDialog {
        id: lockDialog
        objectName: "lockDialog"
        texts: part.texts
        controller: part.controller
        onLocked: part.readVersions()
    }
    VersionRemoveDialog {
        id: removeDialog
        objectName: "removeDialog"
        texts: part.texts
        controller: part.controller
        // The last version kept the page starts: without it the pages are free again.
        onRemoved: {
            part.readVersions()
            layoutRequested()
        }
    }
    VersionCompare {
        id: compareDialog
        objectName: "compareDialog"
        texts: part.texts
        controller: part.controller
        versions: part.versions
    }
}
