import QtQuick
import QtTest
import "../../qml"

// CameraThumbs: a small picture beside each camera body in view (LH-113), made at the small
// width and not more often than the interval allows (NF-05).
TestCase {
    id: tc
    name: "CameraThumbs"
    width: 600
    height: 400
    visible: true
    when: windowShown

    QtObject {
        id: fake
        property int pictureRevision: 0
        property var spots: [
            { id: "a", x_m: 100, y_m: 100, aspect: 2 },
            { id: "b", x_m: 5000, y_m: 100, aspect: 1.5 }
        ]
        property var requests: []

        function cameraSpots() {
            return JSON.stringify(spots)
        }
        function cameraImageOf(id, widthPx, paletteJson) {
            requests.push([id, widthPx])
            return "image " + requests.length
        }
    }

    // The canvas as the plan has it: model metres to pixels.
    Item {
        id: fakeCanvas
        function mapFromModel(x, y) {
            return { x: x, y: y }
        }
    }

    TestTexts { id: catalogue }

    Component {
        id: thumbsComponent
        CameraThumbs {
            width: 600
            height: 400
            texts: catalogue
            controller: fake
            canvas: fakeCanvas
            surfaceSource: Qt.resolvedUrl("StubSurface.qml")
        }
    }

    function make() {
        fake.requests = []
        fake.pictureRevision = 0
        const thumbs = createTemporaryObject(thumbsComponent, tc)
        verify(thumbs)
        tryVerify(() => thumbs.spots.length > 0)
        return thumbs
    }

    function test_a_camera_in_view_gets_a_picture_beside_its_body() {
        const thumbs = make()
        const a = findChild(thumbs, "cameraThumb-a")

        tryVerify(() => a.shownRound === thumbs.round)
        verify(a.visible)
        compare(a.x, 100 + thumbs.gap)
        compare(a.width, thumbs.pictureWidth)
        compare(a.height, thumbs.pictureWidth / 2)
        compare(fake.requests[0][0], "a")
        compare(fake.requests[0][1], Math.round(thumbs.pictureWidth * Screen.devicePixelRatio))
    }

    function test_a_camera_out_of_view_is_not_painted() {
        const thumbs = make()
        const b = findChild(thumbs, "cameraThumb-b")

        verify(!b.visible)
        compare(fake.requests.filter(r => r[0] === "b").length, 0)
    }

    function test_a_burst_of_changes_makes_one_round() {
        const thumbs = make()
        tryVerify(() => findChild(thumbs, "cameraThumb-a").shownRound === thumbs.round)
        fake.requests = []

        for (let i = 0; i < 20; i++) {
            fake.pictureRevision++
        }

        wait(thumbs.interval * 3)
        compare(fake.requests.length, 1)
    }

    function test_hidden_pictures_ask_for_nothing() {
        const thumbs = make()
        thumbs.visible = false
        fake.requests = []

        fake.pictureRevision++
        wait(thumbs.interval * 3)

        compare(fake.requests.length, 0)
    }

    // NF-05: a round makes one picture per tick, not all at once, and keeps the items.
    function test_a_round_paints_one_camera_per_tick_and_keeps_the_items() {
        fake.spots = [1, 2, 3, 4].map(n => ({ id: "c" + n, x_m: 50 * n, y_m: 100, aspect: 2 }))
        const thumbs = make()
        wait(thumbs.interval * 8)
        const first = findChild(thumbs, "cameraThumb-c1")
        fake.requests = []

        fake.pictureRevision++
        wait(thumbs.interval * 1.5)

        verify(fake.requests.length < 4, "all at once: " + fake.requests.length)
        tryVerify(() => fake.requests.length === 4)
        verify(findChild(thumbs, "cameraThumb-c1") === first, "the item was made again")
    }

    function cleanup() {
        fake.spots = [{ id: "a", x_m: 100, y_m: 100, aspect: 2 }, { id: "b", x_m: 5000, y_m: 100, aspect: 1.5 }]
    }
}
