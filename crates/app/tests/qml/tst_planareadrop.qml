import QtQuick
import QtTest
import "../../qml"

// PlanArea drop (LH-18): an image file dragged in from the file manager is placed at the drop
// point, as the image tool places one. The drag itself comes from the window system and cannot
// be played here; what the drop area calls is tested.
TestCase {
    id: tc
    name: "PlanAreaDrop"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property bool viewFree: false
        property string planJson: JSON.stringify({ width: 20, depth: 13, shapes: [] })
        property string choicesJson: ""
        property string selectedKey: ""
        property int selectedCount: 0
        property var images: []

        function addImage(file, x, y) {
            images.push([file, x, y])
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.images = []
        fake.viewFree = false
    }

    function drawn() {
        const area = createTemporaryObject(areaComponent, tc)
        waitForRendering(area)
        return area
    }

    function test_the_plan_takes_drops_and_the_3d_view_does_not() {
        const area = drawn()
        const drop = findChild(area, "imageDrop")

        verify(drop.enabled)
        compare(drop.width, findChild(area, "planCanvas").width)
        fake.viewFree = true
        verify(!drop.enabled)
    }

    function test_the_first_image_file_is_placed_at_the_drop_point() {
        const area = drawn()
        const canvas = findChild(area, "planCanvas")

        verify(area.dropImage(["file:///home/a/notizen.txt", "file:///home/a/Skizze.JPG", "file:///home/a/zwei.png"], 100, 120))

        compare(fake.images.length, 1)
        compare(fake.images[0][0], "file:///home/a/Skizze.JPG")
        const at = canvas.mapToModel(100, 120)
        compare(fake.images[0][1], at.x)
        compare(fake.images[0][2], at.y)
    }

    function test_files_that_are_no_images_place_nothing() {
        const area = drawn()

        verify(!area.dropImage(["file:///home/a/notizen.txt", "file:///home/a/ordner", "file:///x/y.pdf"], 100, 120))

        compare(fake.images.length, 0)
    }

    function test_the_formats_are_those_of_the_background() {
        const area = drawn()

        for (const name of ["a.png", "a.PNG", "a.jpg", "a.jpeg"]) {
            compare(area.imageIn(["file:///x/" + name]), "file:///x/" + name)
        }
        for (const name of ["a.gif", "a.svg", "png", "a.png.txt"]) {
            compare(area.imageIn(["file:///x/" + name]), undefined)
        }
    }
}
