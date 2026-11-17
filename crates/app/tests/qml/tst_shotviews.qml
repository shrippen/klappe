import QtQuick
import QtTest
import "../../qml"

// ShotViewCommands: the camera pictures, the long description and the shot order as commands,
// each with its key (LH-112, LH-113, LH-72).
TestCase {
    id: tc
    name: "ShotViewCommands"
    when: windowShown

    QtObject {
        id: fake
        property bool loaded: true
        property string selectedShot: ""
        property var stills: []
        function shotStill(id) { stills.push(id) }
    }
    TestTexts { id: de; language: "de" }

    Component {
        id: viewsComponent
        ShotViewCommands {
            texts: de
            controller: fake
            keyName: s => s.replace("Shift", "Umschalt")
        }
    }

    SignalSpy { id: toggledSpy; signalName: "toggled" }
    SignalSpy { id: orderSpy; signalName: "orderPicked" }

    function make() {
        const views = createTemporaryObject(viewsComponent, tc)
        toggledSpy.target = views
        toggledSpy.clear()
        orderSpy.target = views
        orderSpy.clear()
        return views
    }

    function test_the_switches_name_their_area_and_have_a_key() {
        const views = make()
        verify(views.cameraViewsAction.checked)
        verify(!views.descriptionAction.checked)
        compare(views.cameraViewsAction.keys, "Umschalt+C")
        compare(views.descriptionAction.keys, "Umschalt+S")

        views.cameraViewsAction.toggle()
        views.descriptionAction.toggle()

        compare(Array.from(toggledSpy.signalArguments[0]), ["cameraViews", false])
        compare(Array.from(toggledSpy.signalArguments[1]), ["shotDescription", true])
    }

    function test_the_order_follows_the_window_and_is_picked() {
        const views = make()
        verify(views.cutAction.checked)

        views.shotOrder = "shooting"
        verify(views.shootingAction.checked)
        views.cutAction.trigger()

        compare(orderSpy.signalArguments[0][0], "cut")
        compare(views.actions.map(a => a.cmd), ["view.cameraViews", "view.shotDescription", "order.cut", "order.shooting", "shot.still"])
    }

    // LH-75: the command takes the still of the selected shot, and only with one.
    function test_the_still_is_taken_for_the_selected_shot() {
        fake.stills = []
        const views = make()
        verify(!views.stillAction.enabled)

        fake.selectedShot = "s12-2"
        verify(views.stillAction.enabled)
        views.stillAction.trigger()

        compare(fake.stills, ["s12-2"])
        fake.selectedShot = ""
    }

    function test_nothing_works_without_a_project() {
        fake.loaded = false
        const views = make()

        verify(!views.cameraViewsAction.enabled)
        verify(!views.cutAction.enabled)
        fake.loaded = true
    }
}
