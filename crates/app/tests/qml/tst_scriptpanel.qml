import QtQuick
import QtTest
import "../../qml"

// ScriptPanel (D-08): the scene's heading and the names above the dialogue open a card with
// the entry's properties and where it occurs; the card's button asks for the project dialog.
TestCase {
    id: tc
    name: "ScriptPanel"
    width: 700
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property var asked: []
        // Replaces the character's scenes when set.
        property string noScenes: ""
        property string projectJson: JSON.stringify({
            scenes: [{ id: "m4djbtj6", number: "12", heading: "INT. SÄNGERSAAL – TAG",
                       location: "saengersaal", locationName: "Sängersaal" }],
            locations: [],
            characters: [{ id: "elisabeth", name: "Elisabeth", short: "E", color: "purple", group: 0 }]
        })
        function usesOf(kind, id) {
            asked.push([kind, id])
            if (kind === "scene") {
                return JSON.stringify({
                    time: "TAG", plans: 2, shots: 5,
                    cast: [{ id: "elisabeth", name: "Elisabeth", color: "purple", cues: 3 }]
                })
            }
            if (noScenes !== "") {
                return noScenes
            }
            return JSON.stringify([
                { id: "m4djbtj6", number: "12", heading: "INT. SÄNGERSAAL – TAG", cues: 3, figures: 2 },
                { id: "szene-014", number: "14", heading: "EXT. HOF", cues: 1, figures: 0 }
            ])
        }
    }

    TestTexts { id: de; language: "de" }

    SignalSpy { id: edits; signalName: "editRequested" }

    Component {
        id: panelComponent
        ScriptPanel {
            width: 600
            height: 500
            texts: de
            controller: fake
            scene: { "id": "m4djbtj6", "number": "12", "heading": "INT. SÄNGERSAAL – TAG" }
            lines: [
                { kind: "action", text: "Elisabeth tritt ein.", current: false, chips: [], character: "" },
                { kind: "character", text: "ELISABETH", current: false, chips: [], character: "elisabeth" },
                { kind: "dialogue", text: "Guten Tag.", current: false, chips: [], character: "" }
            ]
        }
    }

    function init() {
        fake.asked = []
        fake.noScenes = ""
    }

    function make() {
        const panel = createTemporaryObject(panelComponent, tc)
        verify(panel)
        edits.target = panel
        edits.clear()
        // The list makes its rows when it has room.
        tryVerify(() => findChild(panel, "cue") !== null)
        return panel
    }

    function test_the_scene_head_opens_the_scene_card() {
        const panel = make()
        const head = findChild(panel, "sceneHead")
        compare(head.text, "12 INT. SÄNGERSAAL – TAG")

        mouseClick(head)

        const card = findChild(panel, "refCard")
        tryCompare(card, "opened", true)
        compare(fake.asked, [["scene", "m4djbtj6"]])
        compare(findChild(card, "cardTitle").text, "12 INT. SÄNGERSAAL – TAG")
        verify(findChild(card, "sceneFacts").visible)
        verify(!findChild(card, "characterFacts").visible)
    }

    function test_a_cue_opens_the_character_card_with_its_scenes() {
        const panel = make()

        mouseClick(findChild(panel, "cue"))

        const card = findChild(panel, "refCard")
        tryCompare(card, "opened", true)
        compare(fake.asked, [["character", "elisabeth"]])
        compare(findChild(card, "cardTitle").text, "Elisabeth")
        verify(findChild(card, "characterFacts").visible)
        verify(!findChild(card, "sceneFacts").visible)
        verify(!findChild(card, "noUses").visible)
    }

    function test_other_lines_open_nothing() {
        const panel = make()

        mouseClick(findChild(panel, "line"))

        compare(findChild(panel, "refCard").opened, false)
        compare(fake.asked, [])
    }

    function test_the_card_hands_the_entry_to_the_project_dialog() {
        const panel = make()
        mouseClick(findChild(panel, "cue"))
        const card = findChild(panel, "refCard")
        tryCompare(card, "opened", true)

        findChild(card, "editInProject").clicked()

        compare(edits.count, 1)
        compare(edits.signalArguments[0][0], "character")
        compare(edits.signalArguments[0][1], "elisabeth")
        compare(card.opened, false)
    }

    function test_a_character_without_scenes_says_so() {
        const panel = make()
        const card = findChild(panel, "refCard")

        fake.noScenes = "[]"
        mouseClick(findChild(panel, "cue"))

        tryCompare(card, "opened", true)
        verify(findChild(card, "noUses").visible)
    }
}
