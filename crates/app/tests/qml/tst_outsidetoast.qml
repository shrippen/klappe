import QtQuick
import QtTest
import "../../qml"

// OutsideToast: says what the controller tells of a file taken over from outside.
TestCase {
    id: tc
    name: "OutsideToast"
    width: 900
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        signal outsideTaken(string text)
    }

    Component {
        id: toastComponent
        OutsideToast { controller: fake }
    }

    function test_it_shows_the_text_of_a_take_over() {
        const toast = createTemporaryObject(toastComponent, tc)
        verify(toast)
        verify(!toast.shown)

        fake.outsideTaken("Neu geladen: Drehbuch (außerhalb geändert).")

        verify(toast.shown)
        compare(toast.text, "Neu geladen: Drehbuch (außerhalb geändert).")
    }
}
