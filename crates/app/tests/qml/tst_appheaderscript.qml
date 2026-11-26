import QtQuick
import QtTest
import "../../qml"

// AppHeader and the screenplay (E-09): "Load screenplay …" in the File menu and the palette,
// Save sends what was typed first (the modules: tst_modulebar.qml).
TestCase {
    id: tc
    name: "AppHeaderScript"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    FakeHeaderController { id: fake }
    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: headerComponent
        AppHeader { controller: fake; texts: de; width: tc.width }
    }

    SignalSpy { id: importSpy; signalName: "scriptImportRequested" }
    SignalSpy { id: titleSpy; signalName: "titlePageRequested" }
    SignalSpy { id: viewSpy; signalName: "viewToggled" }
    SignalSpy { id: flushSpy; signalName: "flushRequested" }

    function init() {
        fake.loaded = true
        fake.calls = []
    }

    function make() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
        for (const spy of [importSpy, titleSpy, viewSpy, flushSpy]) {
            spy.target = header
            spy.clear()
        }
        return header
    }

    function entry(header, menuName, name) {
        const button = findChild(header, menuName)
        verify(button, menuName)
        button.menu.open()
        const found = findChild(button.menu.contentItem, name) || findChild(button.menu, name)
        verify(found, name)
        return found
    }

    function pick(header, menuName, name) {
        const item = entry(header, menuName, name)
        verify(item.enabled, name)
        item.action.trigger()
        header.children.forEach(c => { if (c.menu) { c.menu.close() } })
    }

    function test_the_file_menu_loads_a_screenplay() {
        const header = make()

        compare(entry(header, "fileMenu", "scriptImport").text, "Drehbuch laden …")
        pick(header, "fileMenu", "scriptImport")

        compare(importSpy.count, 1)
    }

    function test_the_file_menu_asks_for_the_title_page() {
        const header = make()

        compare(entry(header, "fileMenu", "titlePage").text, "Titelseite")
        pick(header, "fileMenu", "titlePage")

        compare(titleSpy.count, 1)
        const found = header.commands.find(c => c.id === "script.titlePage")
        verify(found)
        verify(found.enabled)
    }

    function test_the_entry_is_in_the_palette() {
        const header = make()

        const found = header.commands.find(c => c.id === "script.import")
        verify(found)
        compare(found.text, "Drehbuch laden …")
        verify(found.enabled)
    }

    function test_it_is_english_too() {
        const header = createTemporaryObject(headerComponent, tc, { texts: en })

        compare(entry(header, "fileMenu", "scriptImport").text, "Load screenplay …")
    }

    function test_a_locked_window_cannot_load() {
        const header = make()
        header.locked = true

        verify(!entry(header, "fileMenu", "scriptImport").enabled)
    }

    function test_save_sends_the_typing_first() {
        const header = make()

        pick(header, "fileMenu", "save")

        compare(flushSpy.count, 1)
        compare(fake.calls, [["save"]])
    }

    // DB-70: the output menu has a section for the screenplay; its entry asks how first.
    function test_the_export_menu_outputs_the_screenplay() {
        const header = make()

        const item = entry(header, "exportMenu", "scriptPdf")
        compare(item.text, "Drehbuch als PDF …")
        pick(header, "exportMenu", "scriptPdf")

        const dialog = findChild(header, "scriptPdfDialog")
        tryCompare(dialog, "opened", true)
        // Starts from the project's paper.
        compare(dialog.paper, "letter")
        compare(fake.calls, [])
        dialog.close()
    }

    function test_the_screenplay_pdf_is_in_the_palette() {
        const header = make()

        const found = header.commands.find(c => c.id === "export.scriptpdf")
        verify(found)
        compare(found.text, "Drehbuch als PDF …")
        verify(found.enabled)
    }

    function test_the_screenplay_pdf_is_english_too() {
        const header = createTemporaryObject(headerComponent, tc, { texts: en })

        compare(entry(header, "exportMenu", "scriptPdf").text, "Screenplay as PDF …")
    }
}
