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
    SignalSpy { id: chips; signalName: "chipClicked" }
    SignalSpy { id: plans; signalName: "planRequested" }

    Component {
        id: panelComponent
        ScriptPanel {
            width: 600
            height: 500
            texts: de
            controller: fake
            scene: { "id": "m4djbtj6", "number": "12", "heading": "INT. SÄNGERSAAL – TAG" }
            lines: [
                { kind: "action", text: "Elisabeth tritt ein.", current: false, character: "",
                  chips: [{ kind: "mark", text: "E · M2", target: "elisabeth", index: 1, label: "purple", at_s: 6 }] },
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
        chips.target = panel
        chips.clear()
        plans.target = panel
        plans.clear()
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

    // DB-63: from the script to the plan.
    function test_a_chip_names_its_link() {
        const panel = make()
        const chip = findChild(panel, "chip")
        compare(chip.text, "E · M2")

        chip.clicked()

        compare(chips.count, 1)
        compare(Array.from(chips.signalArguments[0]), ["mark", "elisabeth", 1, 6])
    }

    function test_enter_on_a_line_takes_its_first_chip() {
        const panel = make()

        panel.showLine(0)
        keyClick(Qt.Key_Return)
        compare(Array.from(chips.signalArguments[0]), ["mark", "elisabeth", 1, 6])

        // A line without a chip does nothing.
        panel.showLine(2)
        keyClick(Qt.Key_Return)
        compare(chips.count, 1)
    }

    function test_the_head_button_asks_for_the_plan() {
        const panel = make()

        findChild(panel, "toPlan").clicked()

        compare(plans.count, 1)
    }

    // DB-63: from the plan to the script.
    function test_show_line_and_show_head_take_the_focus() {
        const panel = make()
        const head = findChild(panel, "sceneHead")
        const list = findChild(panel, "lineList")

        panel.showLine(1)
        verify(list.activeFocus)
        compare(list.currentIndex, 1)
        verify(!head.activeFocus)

        panel.showHead()
        verify(head.activeFocus)
    }

    // The list and its rows are named for a screen reader: role and the line's text.
    function test_list_and_rows_are_named() {
        const panel = make()
        const list = findChild(panel, "lineList")
        const row = list.itemAtIndex(0)

        compare(list.Accessible.role, Accessible.List)
        verify(list.Accessible.name !== "")
        compare(row.Accessible.role, Accessible.ListItem)
        compare(row.Accessible.name, panel.lines[0].text)
    }
}
