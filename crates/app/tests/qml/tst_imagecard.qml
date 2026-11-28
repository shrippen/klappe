import QtQuick
import QtTest
import "../../qml"

// The image card (LH-18): the opacity of the selected image in percent, and a new file for it.
TestCase {
    id: tc
    name: "ImageCard"
    width: 800
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property int pictureRevision: 0
        property string selectedKey: ""
        // Like the bridge's `describe`: reading it tells QML of no change.
        property var look: ({ opacity: 1 })
        property var opacities: []
        property var replaced: []
        property bool opacityOk: true

        function describe(key) {
            return JSON.stringify({ kind: key.indexOf("image:") === 0 ? "image" : "note", file: "bilder/a.png", opacity: look.opacity })
        }
        function setImageOpacity(text) {
            opacities.push(text)
            return opacityOk
        }
        function replaceImage(path) {
            replaced.push(path)
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: cardComponent
        ImageCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.opacities = []
        fake.replaced = []
        fake.opacityOk = true
        fake.look = { opacity: 1 }
        fake.pictureRevision = 0
    }

    function shown(key) {
        fake.selectedKey = key
        const card = createTemporaryObject(cardComponent, tc)
        waitForRendering(card)
        return card
    }

    function test_only_a_selected_image_has_a_card() {
        const card = shown("")
        verify(!card.visible)

        fake.selectedKey = "note:n1"
        verify(!card.visible)

        fake.selectedKey = "image:i1"
        verify(card.visible)
        // Its size grip is no image: the grip belongs to the drag.
        fake.selectedKey = "imagesize:i1"
        verify(!card.visible)
    }

    function test_the_field_shows_the_opacity_in_percent() {
        fake.look = { opacity: 0.4 }
        const card = shown("image:i1")

        compare(findChild(card, "imageOpacityField").text, "40")
    }

    // A drag shows its preview through the plan, and an edit at its end gives the same plan: the
    // card reads the image again on the picture revision, which every edit raises.
    function test_the_card_follows_an_edit_that_leaves_the_plan_as_previewed() {
        const card = shown("image:i1")
        fake.planJson = JSON.stringify({ preview: 1 })

        fake.look = { opacity: 0.25 }
        fake.pictureRevision++

        compare(findChild(card, "imageOpacityField").text, "25")
    }

    // D-46: Enter or leaving the field sets the opacity; no button.
    function test_enter_and_leaving_set_the_typed_opacity() {
        const card = shown("image:i1")
        const field = findChild(card, "imageOpacityField")

        field.text = "60"
        field.accepted()
        compare(fake.opacities, ["60"])

        field.forceActiveFocus()
        field.text = "35"
        card.forceActiveFocus()
        compare(fake.opacities, ["60", "35"])
        verify(!findChild(card, "imageOpacityApply"))
    }

    function test_an_untouched_field_sends_nothing() {
        const card = shown("image:i1")
        const field = findChild(card, "imageOpacityField")

        field.forceActiveFocus()
        card.forceActiveFocus()

        compare(fake.opacities, [])
    }

    function test_the_head_names_an_image() {
        compare(shown("image:i1").title, "Bild")
    }

    function test_a_refused_opacity_marks_the_field_until_it_is_edited() {
        const card = shown("image:i1")
        const field = findChild(card, "imageOpacityField")
        fake.opacityOk = false

        field.text = "5"
        field.accepted()
        verify(field.invalid)

        field.text = "50"
        verify(!field.invalid)
    }

    // The name sorts it last: an opened dialog takes the focus of the tests after it.
    function test_the_replace_button_asks_for_an_image_file() {
        const card = shown("image:i1")
        compare(findChild(card, "replaceDialog"), null)

        mouseClick(findChild(card, "imageReplace"))

        const dialog = findChild(card, "replaceDialog")
        verify(dialog)
        compare(dialog.nameFilters, [de.t("background.filter")])
        dialog.close()
    }
}
