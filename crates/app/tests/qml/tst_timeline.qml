import QtQuick
import QtTest
import "../../qml"

// Timeline: texts come from the catalogue, numbers use its decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "Timeline"
    width: 900
    height: 300
    when: windowShown

    QtObject {
        id: fake
        property string tracksJson: "[]"
        property string layersJson: "{}"
        property string locksJson: "{}"
        property var locks: []
        property real time: 6
        property real duration: 20
        property var seeks: []

        function seek(at) { seeks.push(at) }
        function setLayer() {}
        function setLock(layer, state) { locks.push([layer, state]) }
        function setTrack() {}
        function selectShot() {}
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: timelineComponent
        Timeline {
            width: 900
            height: 300
            controller: fake
        }
    }

    function init() {
        fake.locks = []
        fake.locksJson = "{}"
        fake.layersJson = "{}"
    }

    function make(texts) {
        fake.seeks = []
        const line = createTemporaryObject(timelineComponent, tc, { texts: texts })
        verify(line)
        return line
    }

    function test_the_clock_uses_the_german_comma() {
        compare(findChild(make(de), "clock").text, "6,0 / 20,0 s")
    }

    function test_the_clock_uses_the_english_point() {
        compare(findChild(make(en), "clock").text, "6.0 / 20.0 s")
    }

    function test_the_start_button_has_a_readable_name() {
        const line = make(de)
        const start = findChild(line, "startButton")

        compare(start.text, de.t("timeline.start"))
        compare(start.Accessible.name, de.t("timeline.start"))

        start.clicked()
        compare(fake.seeks, [0])
    }

    function test_every_layer_has_a_lock_next_to_its_switch() {
        const line = make(de)

        for (const layer of ["set", "figures", "cameras", "lights", "notes"]) {
            const lock = findChild(line, "lockButton_" + layer)
            verify(lock, layer)
            verify(findChild(line, "layerButton_" + layer))
            compare(lock.text, "gesperrt")
            verify(!lock.checked)
        }
    }

    function test_the_lock_tells_the_controller_and_the_layer_name() {
        const line = make(de)
        const lock = findChild(line, "lockButton_notes")

        lock.toggle()
        lock.toggled()
        lock.toggle()
        lock.toggled()

        compare(fake.locks, [["notes", "locked"], ["notes", "unlocked"]])
        compare(lock.Accessible.name, "„Notizen“ sperren: nichts darauf kann gewählt oder geändert werden")
    }

    function test_a_locked_layer_shows_its_lock_checked_and_stays_visible() {
        fake.locksJson = JSON.stringify({ set: true, notes: false })
        fake.layersJson = JSON.stringify({ set: true })
        const line = make(en)

        verify(findChild(line, "lockButton_set").checked)
        verify(!findChild(line, "lockButton_notes").checked)
        verify(findChild(line, "layerButton_set").checked)
        compare(findChild(line, "lockButton_set").text, "locked")
    }
}
