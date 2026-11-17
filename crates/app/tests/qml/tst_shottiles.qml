import QtQuick
import QtTest
import "../../qml"

// ShotTiles: the stage shows the camera view of the shot (LH-36) unless a storyboard picture
// (LH-74) is there, and is made again when the revision changes.
TestCase {
    id: tc
    name: "ShotTiles"
    width: 480
    height: 400
    visible: true
    when: windowShown

    QtObject {
        id: fake
        property int pictureRevision: 0
        property var requests: []

        function shotImage(id, widthPx, paletteJson) {
            requests.push(id)
            return "image " + requests.length
        }
    }

    TestTexts { id: catalogue }

    function shot(id, imageUrl) {
        return {
            id: id, number: id, camera: "a", camera_label: "A", aspect: 1.78, size: "wide", movement: "static",
            lens_label: "35 mm", camera_color: "red", short_name: "", image_url: imageUrl, axis_jump: false
        }
    }

    Component {
        id: tilesComponent
        ShotTiles {
            width: 480
            height: 400
            texts: catalogue
            controller: fake
            surfaceSource: Qt.resolvedUrl("StubSurface.qml")
        }
    }

    function make(shots) {
        fake.requests = []
        fake.pictureRevision = 0
        const tiles = createTemporaryObject(tilesComponent, tc, { shots: shots })
        verify(tiles)
        return tiles
    }

    function test_a_tile_shows_the_view_of_its_shot() {
        const tiles = make([shot("s1", "")])
        const surface = findChild(tiles, "tileView-s1")

        tryVerify(() => surface.item !== null && surface.item.shows > 0)
        compare(fake.requests[0], "s1")
    }

    function test_a_storyboard_picture_takes_the_stage() {
        const tiles = make([shot("s1", "file:///x.png")])

        wait(300)
        compare(fake.requests.length, 0)
        compare(findChild(tiles, "tileView-s1").item, null)
    }

    function test_a_new_revision_makes_the_view_again() {
        const tiles = make([shot("s1", "")])
        const surface = findChild(tiles, "tileView-s1")
        tryVerify(() => surface.item !== null && surface.item.shows > 0)
        const before = surface.item.shows

        fake.pictureRevision++

        tryVerify(() => surface.item.shows > before)
    }
}
