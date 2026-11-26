import QtQuick
import QtTest
import Kante
import "../../qml"

// ScreenplayView, the locked versions (DB-40 to DB-42, DB-44): the "Fassung" menu with its
// list, the dialog that locks, the question before a version is deleted, the marks of the
// revision in the text and the comparison of two states. The session's rules are tested in
// services; the fake answers as the controller words them.
TestCase {
    id: tc
    name: "Versions"
    width: 1000
    height: 700
    when: windowShown
    visible: true

    readonly property string book: "INT. SAAL - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEs regnet.\n"
    readonly property var scenes: [
        { id: "s1", number: "1", heading: "INT. SAAL - TAG", inBook: true, plans: 0, shots: 0, line: 0 },
        { id: "s2", number: "2", heading: "EXT. HOF - NACHT", inBook: true, plans: 0, shots: 0, line: 4 }
    ]
    // What the session returns for the comparison: scene 1 changed, scene 2 moved only,
    // scene 3 gone, scene 4 new.
    readonly property var diff: [
        { number: "1", heading: "INT. SAAL - TAG", oldNumber: "1", oldHeading: "INT. SAAL - NACHT",
          status: "changed", place: "kept",
          headingText: "INT. SAAL - TAG", oldHeadingText: "INT. SAAL - NACHT", statusText: "geändert",
          lines: [{ change: "removed", line: 2, text: "Er kommt an.", group: 0 }, { change: "added", line: 2, text: "Sie kommt an.", group: 0 },
                  { change: "added", line: 3, text: "Sie setzt sich.", group: 0 }, { change: "added", line: 4, text: "Sie wartet.", group: 0 },
                  { change: "removed", line: 7, text: "Er geht.", group: 1 }, { change: "added", line: 9, text: "Sie geht.", group: 1 }] },
        { number: "2", heading: "EXT. HOF - NACHT", status: "same", place: "moved",
          headingText: "EXT. HOF - NACHT", oldHeadingText: "EXT. HOF - NACHT", statusText: "verschoben", lines: [] },
        { number: "3", heading: "INT. KELLER", status: "removed", place: "kept",
          headingText: "INT. KELLER", oldHeadingText: "INT. KELLER", statusText: "gestrichen",
          lines: [{ change: "removed", line: 0, text: "Dunkel.", group: 0 }] },
        { number: "4", heading: "INT. DACH", status: "new", place: "kept",
          headingText: "INT. DACH", oldHeadingText: "INT. DACH", statusText: "neu",
          lines: [{ change: "added", line: 7, text: "Wind.", group: 0 }] }
    ]

    QtObject {
        id: fake
        property string scriptText: tc.book
        property string sceneId: ""
        property string projectJson: JSON.stringify({ scenes: tc.scenes })
        property string status: ""
        property bool loaded: true
        property var versions: []
        property var revision: []
        property var calls: []
        property int revisions: 0
        property int layouts: 0
        function selectScene(id) { sceneId = id; return true }
        function editScript(text) { scriptText = text }
        function paginateScript() { layouts += 1 }
        function scriptBreaksJson() { return JSON.stringify({ pages: 1, breaks: [], scenes: [] }) }
        function scriptKinds(text) {
            return text.split("\n").map(l => l.startsWith("INT.") || l.startsWith("EXT.") ? "h" : "a").join("")
        }
        function scriptOutlineJson() { return "[]" }
        function scriptPrintJson() { return JSON.stringify({ fontPt: 12, ascentPt: 10, pages: [] }) }
        function enterStep(line, col, kind, below) { return JSON.stringify({ step: "break" }) }
        function tabStep(line, kind) { return "" }
        function typedCaps(text, at, pending) { return "" }
        function titleFields(text) { return JSON.stringify({ "Title": "", "Credit": "", "Author": "", "Source": "", "Draft date": "", "Contact": "" }) }
        function sameFigureName(a, b) { return a === b }
        function scriptSpeakers(text) { return "[]" }
        function findInScript(text, find, matching, speaker) { return "[]" }
        function clearStatus() { status = "" }
        function scriptLooks() {
            return JSON.stringify({ columns: 60, looks: {
                "h": { kind: "heading", x: 0, width: 60, align: "left" },
                "a": { kind: "action", x: 0, width: 60, align: "left" }
            }, letters: { blank: " ", heading: "h", action: "a" } })
        }

        // The colour after the last version's, as the session counts.
        readonly property var order: ["white", "blue", "pink", "yellow"]
        readonly property var names: ({ white: "Weiß", blue: "Blau", pink: "Rosa", yellow: "Gelb" })
        function versionsJson() {
            const last = versions.length > 0 ? versions[versions.length - 1].color : ""
            const next = last === "" ? "" : order[(order.indexOf(last) + 1) % order.length]
            return JSON.stringify({ versions: versions,
                                    next: next === "" ? null : { color: next, colorName: names[next] } })
        }
        function lockVersion(name) {
            calls.push("lock:" + name)
            if (name.trim() === "") {
                return "Eine Fassung braucht einen Namen."
            }
            if (versions.some(v => v.name.toLowerCase() === name.trim().toLowerCase())) {
                return "Eine Fassung „" + name.trim() + "“ gibt es schon."
            }
            const color = order[versions.length % order.length]
            versions = versions.concat([{ id: "v" + versions.length, name: name.trim(), date: "2026-11-25",
                                          color: color, colorName: names[color] }])
            projectJson = JSON.stringify({ scenes: tc.scenes, locked: versions.length })
            return ""
        }
        function removeVersion(id) {
            calls.push("remove:" + id)
            versions = versions.filter(v => v.id !== id)
            return true
        }
        function revisionJson() {
            revisions += 1
            return JSON.stringify(revision)
        }
        function compareJson(from, to) {
            calls.push("compare:" + from + ">" + to)
            return JSON.stringify(tc.diff)
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: viewComponent
        ScreenplayView {
            texts: de
            controller: fake
            width: tc.width
            height: tc.height
            debounceMs: 50
            layoutRestMs: 200
            toolsSource: Qt.resolvedUrl("StubTools.qml")
        }
    }

    function init() {
        fake.scriptText = tc.book
        fake.versions = []
        fake.revision = []
        fake.calls = []
        fake.revisions = 0
        fake.projectJson = JSON.stringify({ scenes: tc.scenes })
    }

    function lockedTwice() {
        fake.versions = [
            { id: "v0", name: "Drehfassung", date: "2026-11-20", color: "white", colorName: "Weiß" },
            { id: "v1", name: "Nachdreh", date: "2026-11-25", color: "blue", colorName: "Blau" }
        ]
    }

    function make() {
        const view = createTemporaryObject(viewComponent, tc)
        verify(view)
        return view
    }

    // A menu entry as the user chooses it: not a click, a theme with native menus takes none.
    function choose(menu, name) {
        const item = findChild(menu.menu.contentItem, name) || findChild(menu.menu, name)
        verify(item, name)
        verify(item.enabled, name)
        if (item.checkable) {
            item.toggle()
        }
        item.triggered()
    }

    // Every descendant of `parent` with that object name (findChild finds only the first).
    function allChildren(parent, name) {
        const found = []
        const walk = item => {
            for (const c of item.children) {
                if (c.objectName === name) {
                    found.push(c)
                }
                walk(c)
            }
        }
        walk(parent)
        return found
    }

    // The row of scene `at`: a list makes its rows when it lays out.
    function row(dialog, at) {
        tryVerify(() => findChild(dialog.rows, "compare." + at) !== null)
        return findChild(dialog.rows, "compare." + at)
    }

    function item(view, name) {
        return findChild(view.versionMenu.menu.contentItem, name) || findChild(view.versionMenu.menu, name)
    }

    // DB-40: the button stands beside the layout switch, grey without a screenplay.
    function test_the_button_needs_a_screenplay() {
        const view = make()
        const button = view.versionMenu

        compare(button.text, "Fassung")
        verify(button.enabled)

        fake.scriptText = ""
        verify(!button.enabled)
    }

    function test_the_menu_without_versions_offers_only_to_lock() {
        const view = make()

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        compare(item(view, "lockVersion").text, "Fassung sperren …")
        verify(item(view, "lockVersion").enabled)
        compare(item(view, "compareVersions").text, "Vergleichen …")
        verify(!item(view, "compareVersions").enabled)
        verify(!item(view, "showRevisions").enabled)
        verify(item(view, "version.v0") === null)
        view.versionMenu.menu.close()
    }

    // The list: name and date in the row, the colour as a swatch with its name.
    function test_the_menu_lists_the_versions_with_colour_name_and_date() {
        lockedTwice()
        const view = make()

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        const row = item(view, "version.v1")
        verify(row)
        compare(row.text, "Nachdreh")
        compare(row.valueText, "25.11.2026")
        compare(row.Accessible.name, "Nachdreh, Blau, 25.11.2026")
        verify(findChild(row, "swatch.v1"))
        verify(item(view, "compareVersions").enabled)
        compare(item(view, "removeVersion.v0").text, "Löschen …")
        view.versionMenu.menu.close()
    }

    function test_a_row_compares_that_version_with_the_book_now() {
        lockedTwice()
        const view = make()

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        choose(view.versionMenu, "version.v0")

        verify(view.compareDialog.visible)
        compare(fake.calls[fake.calls.length - 1], "compare:v0>")
        view.compareDialog.close()
    }

    // DB-40: a small dialog with the name and the note; an empty or used name is told at the field.
    function test_the_lock_dialog_asks_for_a_name_and_says_what_it_does() {
        const view = make()

        view.askLock()
        const dialog = view.lockDialog
        tryCompare(dialog, "opened", true)
        compare(dialog.title, "Fassung sperren")
        compare(findChild(dialog, "versionHint").text, "Speichert das Projekt und sperrt Szenennummern")
        verify(!findChild(dialog, "versionError").visible)
        verify(dialog.nameField.activeFocus)
        dialog.close()
    }

    function test_an_empty_name_is_refused_at_the_field() {
        const view = make()
        view.askLock()
        const dialog = view.lockDialog
        tryCompare(dialog, "opened", true)

        dialog.accept()

        tryVerify(() => dialog.visible)
        compare(findChild(dialog, "versionError").text, "Eine Fassung braucht einen Namen.")
        verify(findChild(dialog, "versionError").visible)
        verify(dialog.nameField.invalid)
        compare(view.versions.length, 0)

        // Typing takes the message away.
        dialog.nameField.text = "Drehfassung"
        verify(!findChild(dialog, "versionError").visible)
        dialog.close()
    }

    function test_a_used_name_is_refused_and_a_free_one_locks() {
        lockedTwice()
        const view = make()
        view.askLock()
        const dialog = view.lockDialog
        tryCompare(dialog, "opened", true)

        dialog.nameField.text = "drehfassung"
        dialog.accept()
        tryVerify(() => dialog.visible && findChild(dialog, "versionError").visible)
        compare(findChild(dialog, "versionError").text, "Eine Fassung „drehfassung“ gibt es schon.")
        compare(view.versions.length, 2)

        dialog.nameField.text = "Schnitt"
        dialog.accept()
        tryVerify(() => !dialog.visible)
        compare(view.versions.length, 3)
        compare(view.versions[2].name, "Schnitt")
        compare(view.nextRevision.color, "yellow")
    }

    // What was typed goes to the session before the lock saves it.
    function test_the_lock_sends_the_typing_first() {
        const view = make()

        view.area.forceActiveFocus()
        view.area.insert(view.area.length, "Mehr.\n")
        view.askLock()

        compare(fake.scriptText, tc.book + "Mehr.\n")
        view.lockDialog.close()
    }

    function test_delete_asks_first_and_removes_on_yes() {
        lockedTwice()
        const view = make()
        const ask = view.removeDialog

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        item(view, "removeVersion.v0").clicked()
        tryCompare(ask, "opened", true)

        compare(findChild(ask, "question").text,
                "Fassung „Drehfassung“ löschen? Ihr Text geht verloren; die Szenennummern bleiben gesperrt.")
        compare(view.versions.length, 2)

        ask.accept()
        tryVerify(() => view.versions.length === 1)
        compare(view.versions[0].id, "v1")
        compare(fake.calls[fake.calls.length - 1], "remove:v0")
    }

    // The last version kept the page starts: without it the book is laid out anew, so the
    // window must ask for the new layout (the real call changes nothing else it could notice).
    function test_deleting_a_version_lays_the_book_out_anew() {
        lockedTwice()
        const view = make()
        const before = fake.layouts

        view.removeDialog.ask(view.versions[1])
        tryCompare(view.removeDialog, "opened", true)
        view.removeDialog.accept()

        tryVerify(() => view.versions.length === 1)
        verify(fake.layouts > before)
        verify(!view.stale)
    }

    // The Delete key asks like the button: the menu closes first.
    function test_the_delete_key_closes_the_menu_and_asks() {
        lockedTwice()
        const view = make()

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        const row = item(view, "version.v0")
        row.forceActiveFocus()
        keyClick(Qt.Key_Delete)

        tryCompare(view.removeDialog, "opened", true)
        verify(!view.versionMenu.menu.visible)
        compare(view.removeDialog.version.id, "v0")
        view.removeDialog.close()
    }

    function test_delete_can_be_declined() {
        lockedTwice()
        const view = make()

        view.removeDialog.ask(view.versions[0])
        tryCompare(view.removeDialog, "opened", true)
        view.removeDialog.reject()
        tryVerify(() => !view.removeDialog.visible)

        compare(view.versions.length, 2)
        verify(!fake.calls.includes("remove:v0"))
    }

    // DB-41: an `*` and a stroke at the left of every line new or changed since the last
    // version, in the colour of the running revision.
    function test_changed_lines_get_a_mark_in_the_colour_of_the_revision() {
        lockedTwice()
        fake.revision = [2, 6]
        const view = make()

        const marks = findChild(view.area, "revisionMarks")
        compare(view.revisionLines.join(), "2,6")
        compare(view.nextRevision.color, "pink")
        const a = view.area.positionToRectangle(view.doc.lineStart(2))
        const placed = []
        for (let i = 0; i < marks.children.length; i++) {
            if (marks.children[i].objectName === "revisionMark") {
                placed.push(marks.children[i])
            }
        }
        compare(placed.length, 2)
        compare(placed[0].y, a.y)
        verify(placed[0].visible)
        compare(placed[0].Accessible.name, "Seit der letzten Fassung geändert")
        compare(marks.ranges[1].from, view.doc.lineStart(6))
    }

    // The swatch and the marks take Kante's revision colours by the file's names.
    function test_the_revision_colours_are_kantes() {
        lockedTwice()
        const view = make()
        verify(KanteStyle.revisions.length === 9)

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        compare(findChild(item(view, "version.v1"), "swatch.v1").swatchColor, KanteStyle.revisionColor("blue"))
        compare(findChild(item(view, "version.v0"), "swatch.v0").swatchColor, KanteStyle.revisionColor("white"))
        view.versionMenu.menu.close()
    }

    function test_without_a_version_nothing_is_marked() {
        fake.revision = [2]
        const view = make()

        compare(view.revisionLines.length, 0)
        compare(fake.revisions, 0)
    }

    // The menu's switch hides the marks and shows them again.
    function test_the_switch_hides_the_marks() {
        lockedTwice()
        fake.revision = [2]
        const view = make()
        compare(view.revisionLines.length, 1)

        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        verify(item(view, "showRevisions").checked)
        choose(view.versionMenu, "showRevisions")
        compare(view.showRevisions, false)
        compare(view.revisionLines.length, 0)

        choose(view.versionMenu, "showRevisions")
        compare(view.showRevisions, true)
        compare(view.revisionLines.length, 1)
        view.versionMenu.menu.close()
    }

    // DN-02: the marks are made anew after the rest, not with every key.
    function test_typing_marks_anew_only_after_the_rest() {
        lockedTwice()
        fake.revision = [2]
        const view = make()
        const before = fake.revisions

        view.area.forceActiveFocus()
        view.area.insert(view.area.length, "a")
        view.area.insert(view.area.length, "b")
        view.area.insert(view.area.length, "c")
        compare(fake.revisions, before)
        verify(findChild(view.area, "revisionMarks").stale)

        fake.revision = [2, 6]
        tryVerify(() => view.revisionLines.length === 2)
        verify(!findChild(view.area, "revisionMarks").stale)
    }

    // DN-02: the marks keep the places of the last layout while typing; a mark is not placed
    // anew with every key (it cost 470 ms a key at 3000 marks).
    function test_typing_does_not_place_the_marks_anew() {
        lockedTwice()
        fake.revision = [2]
        const view = make()
        const marks = findChild(view.area, "revisionMarks")
        const at = marks.ranges[0].from
        compare(at, view.doc.lineStart(2))

        view.area.forceActiveFocus()
        view.area.cursorPosition = 0
        view.area.insert(0, "xx")

        verify(view.stale)
        compare(marks.ranges[0].from, at)
        tryVerify(() => !view.stale)
        compare(marks.ranges[0].from, at + 2)
    }

    // A line the text no longer has gets no mark.
    function test_a_line_beyond_the_text_has_no_range() {
        const view = make()

        view.doc.load("ab\n\ncd")
        compare(view.lineRanges([0, 2]).map(r => r.from + ":" + r.to).join(), "0:2,4:6")
        view.doc.load("ab\ncd")
        compare(view.lineRanges([1, 9]).map(r => r.from).join(), "3,-1")
    }

    // The marks at the end of the text: the empty last line is a line, one more is none.
    function test_the_marks_at_the_end_of_the_text() {
        lockedTwice()
        fake.revision = [0, 2, 3]
        const view = make()

        view.doc.load("ab\n\n")
        view.readRevision()

        compare(view.revisionRanges.map(r => r.from + ":" + r.to).join(), "0:2,4:4,-1:-1")
    }

    // DB-42: von / bis, the scenes with a word for what became of them, the lines of the one chosen.
    function test_the_comparison_lists_scenes_with_their_status() {
        lockedTwice()
        const view = make()

        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        compare(findChild(dialog, "fromBox").currentText, "Drehfassung")
        compare(findChild(dialog, "toBox").currentText, "Aktuelles Buch")
        compare(fake.calls[fake.calls.length - 1], "compare:v0>")
        compare(dialog.rows.count, 4)
        compare(row(dialog, 0).meta, "geändert")
        compare(row(dialog, 1).meta, "verschoben")
        compare(row(dialog, 2).meta, "gestrichen")
        compare(row(dialog, 3).meta, "neu")
        dialog.close()
    }

    // The old state is the left column, the new one the right: removed lines left and struck
    // through, added lines right and underlined, level with each other; both say so in words.
    function test_removed_lines_stand_left_and_added_lines_right_on_one_level() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        const first = row(dialog, 0)
        const removed = findChild(first, "line.old")
        const added = findChild(first, "line.new")
        verify(removed && added)
        compare(removed.Accessible.name, "gestrichen Er kommt an.")
        compare(added.Accessible.name, "neu Sie kommt an.")
        verify(removed.children[1].font.strikeout)
        verify(added.children[1].font.underline)
        verify(!added.children[1].font.strikeout)

        tryVerify(() => removed.mapToItem(first, 0, 0).x < added.mapToItem(first, 0, 0).x)
        compare(removed.mapToItem(first, 0, 0).y, added.mapToItem(first, 0, 0).y)
        dialog.close()
    }

    // One change is one row: when three lines replace one, the next removed line stands level
    // with the first line of its own replacement, not beside the second new line of the first.
    function test_each_change_starts_level_on_both_sides() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        const first = row(dialog, 0)
        tryVerify(() => allChildren(first, "line.old").length === 2 && allChildren(first, "line.new").length === 4)
        const ys = name => allChildren(first, name).map(l => l.mapToItem(first, 0, 0).y)
        tryVerify(() => ys("line.old")[1] > ys("line.old")[0])
        const old = ys("line.old")
        const now = ys("line.new")
        // Old lines: "Er kommt an." (change 1) and "Er geht." (change 2); new lines: three, then one.
        compare(old[0], now[0])
        verify(now[2] > now[0])
        compare(old[1], now[3])
        verify(old[1] > old[0])
        dialog.close()
    }

    // Old `[H, a, b, c, d]`, new `[H, a, c, Y]`: `b` goes alone, `Y` replaces `d` and stands level
    // with it, not with `b` (the lines between them are common ones).
    function test_changes_apart_are_not_one_hunk() {
        const view = make()
        const out = view.compareDialog.hunks({ lines: [
            { change: "removed", line: 2, text: "b", group: 0 },
            { change: "removed", line: 4, text: "d", group: 1 },
            { change: "added", line: 3, text: "Y", group: 1 }] })
        compare(out, [{ removed: ["b"], added: [] }, { removed: ["d"], added: ["Y"] }])
    }

    // The heads: the old heading left, the new one right.
    function test_the_left_head_is_the_old_heading() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        const first = row(dialog, 0)
        compare(findChild(first, "head.old").text, "1 INT. SAAL - NACHT · geändert")
        compare(findChild(first, "head.new").text, "1 INT. SAAL - TAG · geändert")
        dialog.close()
    }

    // A removed scene has a gap on the right, a new one on the left.
    function test_removed_and_new_scenes_leave_a_gap_on_the_other_side() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        const gone = row(dialog, 2)
        compare(findChild(gone, "head.old").text, "3 INT. KELLER · gestrichen")
        compare(findChild(gone, "head.new").text, "")
        compare(findChild(gone, "head.new").Accessible.name, "Nicht in dieser Fassung")
        verify(findChild(gone, "line.old") !== null)
        verify(findChild(gone, "line.new") === null)

        const came = row(dialog, 3)
        compare(findChild(came, "head.old").text, "")
        compare(findChild(came, "head.new").text, "4 INT. DACH · neu")
        verify(findChild(came, "line.old") === null)
        dialog.close()
    }

    // A moved scene says so in words in both columns (DB-44), and marks no line.
    function test_a_moved_scene_says_so_in_both_columns() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        const moved = row(dialog, 1)
        compare(findChild(moved, "head.old").text, "2 EXT. HOF - NACHT · verschoben")
        compare(findChild(moved, "head.new").text, "2 EXT. HOF - NACHT · verschoben")
        verify(findChild(moved, "line.old") === null)
        verify(findChild(moved, "line.new") === null)
        dialog.close()
    }

    // One list holds both columns: they scroll together, each side is as wide as the other.
    function test_both_columns_share_one_list_and_one_width() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        const first = row(dialog, 0)
        tryVerify(() => findChild(first, "head.old").width > 150)
        tryVerify(() => findChild(first, "head.new").width === findChild(first, "head.old").width)
        compare(findChild(first, "head.old").parent, findChild(first, "head.new").parent)
        dialog.close()
    }

    function test_clicking_or_the_arrow_keys_change_the_scene() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)
        dialog.rows.forceActiveFocus()
        compare(dialog.rows.currentIndex, 0)

        keyClick(Qt.Key_Down)
        compare(dialog.rows.currentIndex, 1)
        keyClick(Qt.Key_Down)
        compare(dialog.rows.currentIndex, 2)
        keyClick(Qt.Key_Up)
        compare(dialog.rows.currentIndex, 1)
        mouseClick(row(dialog, 3))
        compare(dialog.rows.currentIndex, 3)
        dialog.close()
    }

    // Another "from" or "to" compares again.
    function test_choosing_another_state_compares_again() {
        lockedTwice()
        const view = make()
        view.askCompare("")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        // Without a choice the last version against now.
        compare(findChild(dialog, "fromBox").currentText, "Nachdreh")

        const to = findChild(dialog, "toBox")
        to.currentIndex = 1
        to.activated(1)

        compare(fake.calls[fake.calls.length - 1], "compare:v1>v1")
        dialog.close()
    }

    function test_without_versions_the_comparison_says_so() {
        const view = make()

        view.askCompare("")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        verify(findChild(dialog, "noVersions").visible)
        compare(findChild(dialog, "noVersions").text, "Keine gesperrten Fassungen.")
        compare(dialog.scenes.length, 0)
        dialog.close()
    }

    // The words of the English surface.
    TestTexts { id: en; language: "en" }

    function test_the_english_menu() {
        lockedTwice()
        const view = make()
        view.texts = en

        compare(view.versionMenu.text, "Version")
        view.versionMenu.menu.popup()
        tryCompare(view.versionMenu.menu, "visible", true)
        compare(item(view, "lockVersion").text, "Lock version …")
        compare(item(view, "version.v1").valueText, "11/25/2026")
        compare(item(view, "showRevisions").text, "Show revisions")
        view.versionMenu.menu.close()
    }
}
