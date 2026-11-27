import QtQuick
import QtQuick.Controls as QQC2
import Kante

// D-37, D-43: the context menu of the screenplay's text, in place of the style's own (which
// copies and cuts the text without its marks and has no step to take back). Right click, the
// Menu key and Shift+F10 (a Shortcut of the view, which comes before the key reaches the
// style's own handler) open it; the entries do what the keys of the editor do.
//
//  ┌──────────────────────────┐
//  │ Ausschneiden      Strg+X │
//  │ Kopieren          Strg+C │
//  │ Einfügen          Strg+V │
//  ├──────────────────────────┤
//  │ Rückgängig        Strg+Z │
//  │ Wiederholen Strg+Umschalt+Z │
//  ├──────────────────────────┤
//  │ ✓ Fett            Strg+B │
//  │   Kursiv          Strg+I │
//  │   Unterstrichen   Strg+U │
//  ├──────────────────────────┤
//  │ Notiz an dieser Stelle   │
//  └──────────────────────────┘
QQC2.Menu {
    id: menu

    required property var texts
    /// The editor (ScreenplayView) the entries work on.
    required property var editor

    /// The emphasis at the selection or cursor (`ScriptDoc.markState`); read when the menu opens.
    property var marks: ({ bold: false, italic: false, underline: false, heading: false })
    readonly property bool selected: editor.doc.selectedText !== ""
    readonly property bool canUndo: editor.tools ? editor.tools.canUndo : false
    readonly property bool canRedo: editor.tools ? editor.tools.canRedo : false

    onAboutToShow: marks = editor.doc.markState()
    onClosed: editor.area.forceActiveFocus()

    /// What a note is: its brackets, between which the cursor goes.
    readonly property string noteBrackets: "[[]]"

    /// A right click: the cursor goes to the pointer unless that is in the selection; then the menu.
    function openAt(point) {
        const area = editor.area
        const at = area.positionAt(point.x, point.y)
        if (!selected || at < area.selectionStart || at > area.selectionEnd) {
            area.cursorPosition = at
        }
        area.forceActiveFocus()
        popup(area, point.x, point.y)
    }

    /// The Menu key and Shift+F10: the menu opens at the cursor.
    function openAtCursor() {
        const rect = editor.area.cursorRectangle
        popup(editor.area, rect.x, rect.y + rect.height)
    }

    /// The Fountain of the selection on the clipboard, and then out of the text.
    function cut() {
        if (editor.copySelection()) {
            editor.doc.cutSelection()
        }
    }

    /// A note at the cursor (behind the selection), which stands between its brackets.
    function insertNote() {
        const doc = editor.doc
        const at = selected ? doc.selectionEnd : doc.cursor

        editor.putText(at, at, noteBrackets)
        doc.setCursor(at + noteBrackets.length / 2)
        editor.area.forceActiveFocus()
    }

    /// "Ctrl+Shift+Z" as the keyboard of the language names it.
    function keyName(sequence) {
        return sequence.replace("Ctrl", texts.t("key.ctrl")).replace("Shift", texts.t("key.shift"))
    }

    KantePopupSkin { popup: menu }

    KanteMenuItem {
        objectName: "menuCut"
        text: menu.texts.t("screenplay.cut")
        shortcutText: menu.keyName("Ctrl+X")
        enabled: menu.selected
        onTriggered: menu.cut()
    }
    KanteMenuItem {
        objectName: "menuCopy"
        text: menu.texts.t("screenplay.copy")
        shortcutText: menu.keyName("Ctrl+C")
        enabled: menu.selected
        onTriggered: menu.editor.copySelection()
    }
    KanteMenuItem {
        objectName: "menuPaste"
        text: menu.texts.t("screenplay.paste")
        shortcutText: menu.keyName("Ctrl+V")
        enabled: menu.editor.area.canPaste
        onTriggered: menu.editor.area.paste()
    }

    QQC2.MenuSeparator {}

    KanteMenuItem {
        objectName: "menuUndo"
        text: menu.texts.t("menu.undo")
        shortcutText: menu.keyName("Ctrl+Z")
        enabled: menu.canUndo
        onTriggered: menu.editor.undoText()
    }
    KanteMenuItem {
        objectName: "menuRedo"
        text: menu.texts.t("menu.redo")
        shortcutText: menu.keyName("Ctrl+Shift+Z")
        enabled: menu.canRedo
        onTriggered: menu.editor.redoText()
    }

    QQC2.MenuSeparator {}

    // A tick is the state at the selection or cursor: it goes back to that after a click.
    KanteMenuItem {
        objectName: "menuBold"
        text: menu.texts.t("screenplay.bold")
        shortcutText: menu.keyName("Ctrl+B")
        checkable: true
        checked: menu.marks.bold
        // A heading is bold throughout.
        enabled: !menu.marks.heading
        onTriggered: {
            menu.editor.toggleMark("bold")
            checked = Qt.binding(() => menu.marks.bold)
        }
    }
    KanteMenuItem {
        objectName: "menuItalic"
        text: menu.texts.t("screenplay.italic")
        shortcutText: menu.keyName("Ctrl+I")
        checkable: true
        checked: menu.marks.italic
        onTriggered: {
            menu.editor.toggleMark("italic")
            checked = Qt.binding(() => menu.marks.italic)
        }
    }
    KanteMenuItem {
        objectName: "menuUnderline"
        text: menu.texts.t("screenplay.underline")
        shortcutText: menu.keyName("Ctrl+U")
        checkable: true
        checked: menu.marks.underline
        onTriggered: {
            menu.editor.toggleMark("underline")
            checked = Qt.binding(() => menu.marks.underline)
        }
    }

    QQC2.MenuSeparator {}

    KanteMenuItem {
        objectName: "menuNote"
        text: menu.texts.t("screenplay.noteHere")
        onTriggered: menu.insertNote()
    }
}
