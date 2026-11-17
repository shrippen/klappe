import QtQuick
import QtTest
import "../../qml"

// ShotTiles: the stage shows the camera view of the shot (LH-36) unless a storyboard picture
// (LH-74) is there. It is made again when the plan is edited, not when only the selection
// does, and one view per tick, for the tiles on screen only (NF-05).
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
        property int editRevision: 0
        property var requests: []
        property var times: []

        function shotImage(id, widthPx, paletteJson) {
            requests.push(id)
            times.push(Date.now())
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
        fake.times = []
        fake.pictureRevision = 0
        fake.editRevision = 0
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

    function test_an_edit_makes_the_view_again() {
        const tiles = make([shot("s1", "")])
        const surface = findChild(tiles, "tileView-s1")
        tryVerify(() => surface.item !== null && surface.item.shows > 0)
        const before = surface.item.shows

        fake.editRevision++

        tryVerify(() => surface.item.shows > before)
    }

    function test_a_selection_does_not_make_the_views_again() {
        const tiles = make([shot("s1", ""), shot("s2", "")])
        tryVerify(() => fake.requests.length === 2)

        tiles.selectedId = "s2"
        fake.pictureRevision++
        wait(400)

        compare(fake.requests.length, 2)
    }

    function test_a_tile_off_screen_is_not_painted() {
        const shots = []
        for (let i = 0; i < 40; i++) {
            shots.push(shot("s" + i, ""))
        }
        const tiles = make(shots)
        const below = findChild(tiles, "tile-s39")
        tryVerify(() => below.y > tiles.height)

        wait(1000)

        verify(fake.requests.length > 0)
        compare(fake.requests.indexOf("s39"), -1)
    }

    function test_one_view_is_made_per_tick() {
        const tiles = make([shot("s1", ""), shot("s2", ""), shot("s3", "")])
        tryVerify(() => fake.requests.length === 3, 3000)

        for (let i = 1; i < fake.times.length; i++) {
            verify(fake.times[i] - fake.times[i - 1] >= tiles.interval - 20)
        }
    }
}
