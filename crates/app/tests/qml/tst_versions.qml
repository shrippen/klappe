import QtQuick
import QtTest
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
        { number: "1", heading: "INT. SAAL - TAG", status: "changed", place: "kept",
          lines: [{ change: "removed", line: 2, text: "Er kommt an." }, { change: "added", line: 2, text: "Sie kommt an." }] },
        { number: "2", heading: "EXT. HOF - NACHT", status: "same", place: "moved", lines: [] },
        { number: "3", heading: "INT. KELLER", status: "removed", place: "kept",
          lines: [{ change: "removed", line: 0, text: "Dunkel." }] },
        { number: "4", heading: "INT. DACH", status: "new", place: "kept",
          lines: [{ change: "added", line: 7, text: "Wind." }] }
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
        function scriptPrintJson() { return JSON.stringify({ fontPt: 12, ascentPt: 10, pages: [] }) }
        function enterStep(line, col, kind, below) { return JSON.stringify({ step: "break" }) }
        function tabStep(line, kind) { return "" }
        function typedCaps(text, at, pending) { return "" }
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
            projectJson = JSON.stringify({ scenes: tc.scenes, locked: versions.length })
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
        compare(row.valueText, "2026-11-25")
        compare(row.Accessible.name, "Nachdreh, Blau, 2026-11-25")
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
        const a = view.area.positionToRectangle(view.lineStart(tc.book, 2))
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
        compare(marks.ranges[1].from, view.lineStart(tc.book, 6))
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

    // A line the text no longer has gets no mark.
    function test_a_line_beyond_the_text_has_no_range() {
        const view = make()

        compare(view.lineRanges([0, 2], "ab\n\ncd").map(r => r.from + ":" + r.to).join(), "0:2,4:6")
        compare(view.lineRanges([1, 9], "ab\ncd").map(r => r.from).join(), "3,-1")
    }

    // DB-42: von / bis, the scenes with a word for what became of them, the lines of the one chosen.
    function test_the_comparison_lists_scenes_with_their_status() {
        lockedTwice()
        const view = make()

        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        compare(findChild(dialog, "fromBox").currentText, "Drehfassung")
        compare(findChild(dialog, "toBox").currentText, "Jetzt")
        compare(fake.calls[fake.calls.length - 1], "compare:v0>")
        compare(dialog.rows.count, 4)
        compare(row(dialog, 0).meta, "geändert")
        compare(row(dialog, 1).meta, "verschoben")
        compare(row(dialog, 2).meta, "gestrichen")
        compare(row(dialog, 3).meta, "neu")
        dialog.close()
    }

    // New lines are underlined, removed ones struck through; both say so in words too.
    function test_the_text_shows_new_and_removed_lines() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        // The first scene that changed shows first.
        compare(dialog.rows.currentIndex, 0)
        compare(findChild(dialog, "compareHeading").text, "INT. SAAL - TAG")
        const removed = findChild(findChild(dialog, "compareText"), "line.removed")
        const added = findChild(findChild(dialog, "compareText"), "line.added")
        verify(removed && added)
        compare(removed.Accessible.name, "gestrichen Er kommt an.")
        compare(added.Accessible.name, "neu Sie kommt an.")
        verify(removed.children[1].font.strikeout)
        verify(added.children[1].font.underline)
        verify(!added.children[1].font.strikeout)
        dialog.close()
    }

    // A moved scene's text is not marked (DB-44): it says nothing changed.
    function test_a_moved_scene_has_no_marked_text() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)

        mouseClick(row(dialog, 1))

        compare(dialog.rows.currentIndex, 1)
        verify(findChild(dialog, "compareNoChange").visible)
        compare(findChild(dialog, "compareNoChange").text, "Keine Zeile geändert.")
        verify(findChild(findChild(dialog, "compareText"), "line.added") === null)
        dialog.close()
    }

    function test_the_arrow_keys_change_the_scene() {
        lockedTwice()
        const view = make()
        view.askCompare("v0")
        const dialog = view.compareDialog
        tryCompare(dialog, "opened", true)
        dialog.rows.forceActiveFocus()

        keyClick(Qt.Key_Down)
        compare(dialog.rows.currentIndex, 1)
        keyClick(Qt.Key_Down)
        compare(findChild(dialog, "compareHeading").text, "INT. KELLER")
        keyClick(Qt.Key_Up)
        compare(dialog.rows.currentIndex, 1)
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
        compare(item(view, "showRevisions").text, "Show revisions")
        view.versionMenu.menu.close()
    }
}
