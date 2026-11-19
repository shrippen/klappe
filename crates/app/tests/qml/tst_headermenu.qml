import QtQuick
import QtTest
import Kante
import "../../qml"

// HeaderMenu: while one menu of the header is open, moving the mouse onto another opens that one
// and closes the first, as in a menu bar. Without an open menu, hovering opens nothing.
TestCase {
    id: tc
    name: "HeaderMenu"
    width: 600
    height: 400
    when: windowShown
    visible: true

    Component {
        id: barComponent
        Row {
            HeaderMenu { objectName: "file"; text: "Datei"; KanteMenuItem { text: "Neu" } }
            HeaderMenu { objectName: "export"; text: "Ausgeben"; KanteMenuItem { text: "PDF" } }
            HeaderMenu { objectName: "view"; text: "Ansicht"; KanteMenuItem { text: "Plan" } }
        }
    }

    function make() {
        const bar = createTemporaryObject(barComponent, tc)
        verify(bar)
        return [findChild(bar, "file"), findChild(bar, "export"), findChild(bar, "view")]
    }

    function test_hover_moves_the_open_menu() {
        const [file, exp, view] = make()

        mouseClick(file)
        tryVerify(() => file.menu.opened)

        mouseMove(exp)
        tryVerify(() => exp.menu.opened)
        tryVerify(() => !file.menu.visible)

        mouseMove(view)
        tryVerify(() => view.menu.opened)
        tryVerify(() => !exp.menu.visible)
    }

    function test_hover_alone_opens_nothing() {
        const [file, exp] = make()

        mouseMove(file)
        mouseMove(exp)
        wait(50)
        verify(!file.menu.visible)
        verify(!exp.menu.visible)
    }
}
