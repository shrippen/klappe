import QtQuick
import QtTest
import "../../qml"

// LayerPopover (D-06): the layers in a list with an eye and a lock each; a dot on the button
// when a layer is hidden or locked. The same controller calls as the old switches in the timeline.
TestCase {
    id: tc
    name: "LayerPopover"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string layersJson: "{}"
        property string locksJson: "{}"
        property string fadesJson: "{}"
        property string choicesJson: JSON.stringify({
            layer: ["set", "figures", "cameras", "lights", "notes", "drawings"]
        })
        property var layerCalls: []
        property var lockCalls: []
        property var fadeCalls: []

        function setLayer(layer, state) { layerCalls.push([layer, state]) }
        function setLock(layer, state) { lockCalls.push([layer, state]) }
        function cycleFade(layer) { fadeCalls.push(layer) }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: layersComponent
        LayerPopover {
            x: 400
            y: 20
            controller: fake
        }
    }

    function init() {
        fake.layersJson = "{}"
        fake.locksJson = "{}"
        fake.fadesJson = "{}"
        fake.fadeCalls = []
        fake.layerCalls = []
        fake.lockCalls = []
    }

    function make(texts) {
        const layers = createTemporaryObject(layersComponent, tc, { texts: texts })
        verify(layers)
        return layers
    }

    // The rows live in the popup's own item tree, not under the layers item.
    function row(layers, name) {
        const popover = findChild(layers, "layersPopover")
        return findChild(popover.contentItem, name)
    }

    function opened(layers) {
        const popover = findChild(layers, "layersPopover")
        popover.openAt(findChild(layers, "layersButton"))
        tryVerify(() => popover.opened)
        return popover
    }

    function test_the_button_has_no_dot_while_every_layer_is_on_and_free() {
        const layers = make(de)

        compare(findChild(layers, "layersButton").text, "Ebenen")
        verify(!findChild(layers, "layersDot").visible)
    }

    function test_a_hidden_layer_shows_the_dot() {
        fake.layersJson = JSON.stringify({ notes: false })
        const layers = make(de)

        verify(findChild(layers, "layersDot").visible)
        compare(findChild(layers, "layersButton").Accessible.name, "Ebenen (eine ist aus oder gesperrt)")
    }

    function test_a_locked_layer_shows_the_dot() {
        fake.locksJson = JSON.stringify({ set: true })

        verify(findChild(make(en), "layersDot").visible)
    }

    function test_every_layer_has_an_eye_and_a_lock() {
        const layers = make(de)
        opened(layers)

        for (const name of ["set", "figures", "cameras", "lights", "notes", "drawings"]) {
            const eye = row(layers, "layerButton_" + name)
            const lock = row(layers, "lockButton_" + name)
            verify(eye, name)
            verify(lock, name)
            verify(eye.checked)
            verify(!lock.checked)
            compare(eye.icon.name, "view-visible")
            compare(lock.icon.name, "object-unlocked")
        }
    }

    function test_the_eye_tells_the_controller() {
        const layers = make(de)
        opened(layers)
        const eye = row(layers, "layerButton_figures")

        eye.toggle()
        eye.toggled()
        eye.toggle()
        eye.toggled()

        compare(fake.layerCalls, [["figures", "hidden"], ["figures", "shown"]])
        compare(eye.Accessible.name, "„Figuren“ zeigen")
    }

    function test_the_lock_tells_the_controller_and_the_layer_name() {
        const layers = make(de)
        opened(layers)
        const lock = row(layers, "lockButton_notes")

        lock.toggle()
        lock.toggled()
        lock.toggle()
        lock.toggled()

        compare(fake.lockCalls, [["notes", "locked"], ["notes", "unlocked"]])
        compare(lock.Accessible.name, "„Notizen“ sperren: nichts darauf kann gewählt oder geändert werden")
    }

    function test_the_rows_show_what_the_controller_says() {
        fake.layersJson = JSON.stringify({ set: false })
        fake.locksJson = JSON.stringify({ set: true, notes: false })
        const layers = make(en)
        opened(layers)

        verify(!row(layers, "layerButton_set").checked)
        compare(row(layers, "layerButton_set").icon.name, "view-hidden")
        verify(row(layers, "layerButton_figures").checked)
        verify(row(layers, "lockButton_set").checked)
        compare(row(layers, "lockButton_set").icon.name, "object-locked")
        verify(!row(layers, "lockButton_notes").checked)
    }

    // LH-15: a button per layer shows its strength and takes it one step on.
    function test_every_layer_has_a_fade_button_that_shows_its_step() {
        fake.fadesJson = JSON.stringify({ figures: "60", notes: "30" })
        const layers = make(de)
        opened(layers)

        compare(row(layers, "fadeButton_set").text, "100 %")
        compare(row(layers, "fadeButton_figures").text, "60 %")
        compare(row(layers, "fadeButton_notes").text, "30 %")
        compare(row(layers, "fadeButton_figures").Accessible.name,
                "„Figuren“ abblenden: jetzt 60 %, Klick für die nächste Stufe (100, 60, 30 %)")
    }

    function test_the_fade_button_tells_the_controller_the_layer() {
        const layers = make(en)
        opened(layers)

        row(layers, "fadeButton_cameras").clicked()

        compare(fake.fadeCalls, ["cameras"])
    }

    function test_a_faded_layer_shows_the_dot() {
        fake.fadesJson = JSON.stringify({ lights: "60" })

        verify(findChild(make(en), "layersDot").visible)
    }
}
