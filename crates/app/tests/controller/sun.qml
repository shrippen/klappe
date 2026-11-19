import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-52: sunlight on the plan. A window in the south wall of the example, the Sängersaal given
// a place and its scene a date and time through the forms' text: the plan gets the patch, the
// sun's arrow and the arrow of north, each form one undo step; a wrong place or date is refused
// with a message and changes nothing.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function shapes() {
        return JSON.parse(ctl.planJson).shapes
    }

    function project() {
        return JSON.parse(ctl.projectJson)
    }

    // The arrow of north carries an N, the sun's arrow its azimuth and elevation.
    function north() {
        return shapes().filter(s => s.text === "N").length
    }

    function rays() {
        return shapes().filter(s => String(s.text).indexOf("°") >= 0 && s.fill === "solid" && s.key === "").length
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            ctl.setSnapping(false)
            ctl.setTool("window", "")
            ctl.place(10.0, 13.0)
            ctl.setTool("select", "")

            const location = project().locations[0]
            const scene = project().scenes[0]
            const place = (lat, lon, north) => ctl.editLocation(
                location.id, location.name, "20", "13", "2,6", JSON.stringify([lat, lon, north]))
            const when = (date, time, offset) => ctl.editScene(
                scene.id, scene.number, scene.heading, scene.location, JSON.stringify([date, time, offset]))

            const before = shapes().length
            if (north() !== 0) {
                return fail("north shown without a place")
            }

            if (!place("52,52", "13,405", "0") || north() !== 0) {
                return fail("a place alone must show nothing: " + ctl.status)
            }
            if (!when("2026-06-21", "13:08", "+02:00")) {
                return fail("the date was refused: " + ctl.status)
            }
            // The patch, the sun's arrow and north.
            if (shapes().length !== before + 3 || north() !== 1 || rays() !== 1) {
                return fail("no sun on the plan: " + before + " -> " + shapes().length)
            }
            if (project().locations[0].latitude !== 52.52 || project().scenes[0].sunTime !== "13:08") {
                return fail("the form values did not come back: " + ctl.projectJson)
            }

            // At night only north stays.
            if (!when("2026-06-21", "01:00", "+02:00") || shapes().length !== before + 1 || north() !== 1) {
                return fail("the night still shows sun: " + shapes().length)
            }

            // A wrong place or date is refused and changes nothing.
            const json = ctl.projectJson
            if (place("95", "13", "") || ctl.status === "" || ctl.projectJson !== json) {
                return fail("a place off the globe was taken")
            }
            if (when("2026-02-30", "12:00", "") || when("2026-06-21", "25:00", "") || ctl.projectJson !== json) {
                return fail("a wrong date or time was taken")
            }

            // One undo step each.
            ctl.undo()
            ctl.undo()
            if (north() !== 0 || project().scenes[0].sunDate !== "") {
                return fail("undo did not take the sun back")
            }
            Qt.exit(0)
        }
    }
}
