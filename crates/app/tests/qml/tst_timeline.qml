import QtQuick
import QtTest
import "../../qml"

// Timeline: texts come from the catalogue, numbers use its decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "Timeline"
    width: 900
    height: 300
    when: windowShown

    QtObject {
        id: fake
        property string tracksJson: "[]"
        property real time: 6
        property real duration: 20
        property var seeks: []
        property var steps: []
        property var advances: []
        property bool running: true
        signal switched()
        property string sceneId: "s12"

        function seek(at) { seeks.push(at) }
        function stepMark(direction) { steps.push(direction) }
        function advance(dt, speed, looping) {
            advances.push([speed, looping])
            return running
        }
        function setTrack() {}
        function selectShot() {}
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: timelineComponent
        Timeline {
            width: 900
            height: 300
            controller: fake
        }
    }

    function init() {
        fake.time = 6
        fake.running = true
    }

    // Play stops when the timeline is locked (backup question) and when another scene or project
    // comes in: the time is that of the new one, nobody pressed play there.
    function test_a_locked_timeline_stops_playing() {
        const line = make(de)
        line.togglePlay()
        verify(line.playing)
        line.enabled = false
        verify(!line.playing)
    }

    // The controller says `switched` for another project, scene or plan (also one with the
    // same title or scene id).
    function test_a_scene_gone_by_an_edit_stops_playing() {
        const line = make(de)
        line.togglePlay()
        fake.sceneId = fake.sceneId === "s12" ? "s13" : "s12"
        verify(!line.playing)
    }

    function test_a_switch_stops_playing() {
        const line = make(de)
        line.togglePlay()
        fake.switched()
        verify(!line.playing)
    }

    function make(texts) {
        fake.seeks = []
        fake.steps = []
        fake.advances = []
        const line = createTemporaryObject(timelineComponent, tc, { texts: texts })
        verify(line)
        return line
    }

    function bar(line) {
        return findChild(line, "transport")
    }

    function test_the_bar_shows_markers_and_tenths_without_metronome() {
        const b = bar(make(de))

        verify(b.markersVisible)
        verify(!b.metronomeVisible)
        verify(!b.countInVisible)
        compare(b.timeFormat, "mm:ss,d")
        compare(b.positionText, "00:06,0 / 00:20,0")
        compare(b.speedSteps, [0.25, 0.5, 1])
    }

    function test_the_clock_uses_the_english_point() {
        compare(bar(make(en)).positionText, "00:06.0 / 00:20.0")
    }

    function test_the_clock_cuts_off_like_a_timecode() {
        fake.time = 59.96
        compare(bar(make(de)).positionText, "00:59,9 / 00:20,0")
    }

    function test_the_texts_come_from_the_catalogue() {
        const b = bar(make(en))

        compare(b.rewindText, "To start")
        compare(b.loopText, "Loop shot")
        compare(b.markerOnText, "Next marker")
    }

    function test_rewind_goes_to_the_start() {
        const line = make(de)

        bar(line).rewind()
        compare(fake.seeks, [0])
    }

    function test_the_marker_buttons_ask_the_controller_to_jump() {
        const line = make(de)

        bar(line).markerStepRequested(-1)
        bar(line).markerStepRequested(1)
        compare(fake.steps, [-1, 1])
    }

    function test_play_ticks_the_controller_with_the_speed() {
        const line = make(de)

        bar(line).speedChangeRequested(0.5)
        compare(line.speed, 0.5)
        bar(line).playToggled()
        verify(line.playing)
        tryVerify(() => fake.advances.length > 0)
        compare(fake.advances[0], [0.5, "off"])

        bar(line).playToggled()
        verify(!line.playing)
    }

    function test_the_loop_plays_the_shot_in_a_loop() {
        const line = make(de)

        bar(line).loopToggled()
        verify(line.looping)
        bar(line).playToggled()
        tryVerify(() => fake.advances.length > 0)
        compare(fake.advances[0][1], "shot")
    }

    function test_play_stops_when_the_controller_reaches_the_end() {
        const line = make(de)
        fake.running = false

        bar(line).playToggled()
        tryVerify(() => !line.playing)
    }

    function test_play_at_the_end_starts_from_the_beginning() {
        const line = make(de)
        fake.time = 20

        bar(line).playToggled()
        compare(fake.seeks, [0])
        verify(line.playing)
    }
}
