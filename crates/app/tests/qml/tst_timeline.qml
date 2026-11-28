import QtQuick
import QtTest
import "../../qml"

// Timeline: texts come from the catalogue, numbers use its decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "Timeline"
    width: 900
    height: 700
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string tracksJson: "[]"
        property string speech: "null"
        function speechJson() { return speech }
        property real time: 6
        property real duration: 20
        property var seeks: []
        property var steps: []
        property var advances: []
        property bool running: true
        // Where the next `advance` puts the playhead (-1: stays), e.g. a loop's wrap.
        property real nextTime: -1
        // The loop start `advance` reports with it (-1: none), as the bridge does.
        property real nextLoopS: -1
        signal switched()
        signal looped(real toS)

        function seek(at) { seeks.push(at) }
        function stepMark(direction) { steps.push(direction) }
        function advance(dt, speed, looping) {
            advances.push([speed, looping])
            if (nextTime >= 0) {
                if (nextLoopS >= 0) {
                    looped(nextLoopS)
                }
                time = nextTime
            }
            return running
        }
        property var trackSets: []
        function setTrack(id, state) { trackSets.push([id, state]) }
        property var selected: []
        function selectShot(key) { selected.push(key) }
        property string locksJson: "{}"
        property string selectedKey: ""

        property var drags: []
        property var drops: []
        function dragTime(grab, seconds, snap) {
            drags.push([grab, snap])
            return snap === "free" ? seconds : Math.round(seconds * 10) / 10
        }
        function dropTime(grab, seconds, snap) {
            drops.push([grab, seconds, snap])
            return true
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: timelineComponent
        Timeline {
            width: 900
            // The bar takes its height; the lanes need room below it.
            height: 700
            controller: fake
        }
    }

    // A cut row (shot s1 from 2 to 6 s) and a figure row (marks at 4 and 8 s).
    readonly property string tracksOfTwo: JSON.stringify([
        { kind: "cut", id: "cut", name: "", label: "yellow", shown: true,
          segments: [{ from_s: 2, to_s: 6, text: "1", key: "s1", selected: false }], markers: [] },
        { kind: "figure", id: "anna", name: "Anna", label: "red", shown: true,
          segments: [], markers: [{ at_s: 4, text: "A 1" }, { at_s: 8, text: "A 2" }] }
    ])

    function init() {
        fake.time = 6
        fake.speech = "null"
        fake.running = true
        fake.nextTime = -1
        fake.tracksJson = "[]"
        fake.drags = []
        fake.drops = []
        fake.selected = []
        fake.locksJson = "{}"
        // The figure is selected, so its row is on view (D-45).
        fake.selectedKey = "figure:anna"
    }

    readonly property string tracksOfFour: JSON.stringify([
        { kind: "cut", id: "cut", name: "", label: "yellow", shown: true, segments: [], markers: [] },
        { kind: "figure", id: "anna", name: "Anna", label: "red", shown: true, segments: [], markers: [] },
        { kind: "figure", id: "ben", name: "Ben", label: "blue", shown: true, segments: [], markers: [] },
        { kind: "camera", id: "a", name: "A", label: "green", shown: true, segments: [], markers: [] },
        { kind: "light", id: "l1", name: "L1", label: "orange", shown: true, segments: [], markers: [] }
    ])

    function rows(line) {
        return findChild(line, "tracks").tracks.map(t => t.name)
    }

    // D-45: the cut order alone, with the selected element's row; all of them on request.
    function test_folded_shows_the_cut_and_the_selected_row() {
        fake.tracksJson = tracksOfFour
        fake.selectedKey = ""
        const line = make(de)
        compare(rows(line), ["Schnittfolge"])
        fake.selectedKey = "camera:a"
        compare(rows(line), ["Schnittfolge", "A"])
        fake.selectedKey = "mark:ben:2"
        compare(rows(line), ["Schnittfolge", "Ben"])
        fake.selectedKey = "light:l1"
        compare(rows(line), ["Schnittfolge", "L1"])
    }

    // A bend of a way and a grip of a light belong to their element: its row stays.
    function test_a_bend_or_grip_keeps_its_row() {
        fake.tracksJson = tracksOfFour
        const line = make(de)
        fake.selectedKey = "figurevia:ben:1:at0"
        compare(rows(line), ["Schnittfolge", "Ben"])
        fake.selectedKey = "cameravia:a:1:at0"
        compare(rows(line), ["Schnittfolge", "A"])
        fake.selectedKey = "lightgrip:l1:beam"
        compare(rows(line), ["Schnittfolge", "L1"])
    }

    // Folded, the camera is the second row, not the fourth: its eye still hides the camera.
    function test_a_folded_row_hides_its_own_track() {
        fake.tracksJson = tracksOfFour
        fake.selectedKey = "camera:a"
        fake.trackSets = []
        const line = make(de)
        mouseClick(findChild(line, "toggle1"))
        compare(fake.trackSets, [["a", "hidden"]])
    }

    function test_unfolded_shows_all_rows_and_folds_again() {
        fake.tracksJson = tracksOfFour
        fake.selectedKey = ""
        const line = make(de)
        const lanes = findChild(line, "tracks")
        // The layout passes lazily: the height follows the rows, one track and then five.
        wait(300)
        const folded = line.implicitHeight
        line.allTracks = true
        compare(rows(line), ["Schnittfolge", "Anna", "Ben", "A", "L1"])
        tryCompare(line, "implicitHeight", folded + 4 * lanes.trackHeight)
        line.allTracks = false
        compare(rows(line), ["Schnittfolge"])
        tryCompare(line, "implicitHeight", folded)
    }

    function test_the_all_tracks_switch_asks_the_window() {
        const line = make(de)
        const sw = findChild(line, "allTracksSwitch")
        compare(sw.text, "Alle Spuren")
        const asked = []
        line.allTracksToggled.connect(on => asked.push(on))
        sw.toggle()
        sw.toggled()
        compare(asked, [true])
        verify(!sw.checked)
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
    function test_a_switch_stops_playing() {
        const line = make(de)
        line.togglePlay()
        fake.switched()
        verify(!line.playing)
    }

    QtObject {
        id: fakeVoice
        property bool available: true
        property var said: []
        property int stops: 0
        function say(text, language) { said.push(text) }
        function stop() { stops++ }
    }

    function speechOf(line) {
        return findChild(line, "speakSwitch")
    }

    // LH-64: without a voice the switch is off and says why; with one it asks the window.
    function test_the_speak_switch_is_off_and_explained_without_a_voice() {
        const sw = speechOf(make(de))

        verify(!sw.enabled)
        compare(sw.text, "Dialog vorlesen (keine Sprachausgabe gefunden)")
    }

    function test_the_speak_switch_asks_the_window() {
        const line = make(en)
        line.voice = fakeVoice
        const spy = createTemporaryObject(spyComponent, tc, { target: line, signalName: "speakToggled" })
        const sw = speechOf(line)

        verify(sw.enabled)
        compare(sw.text, "Read dialogue aloud")
        verify(!sw.checked)
        mouseClick(sw)
        compare(spy.count, 1)
        compare(spy.signalArguments[0][0], true)
        // The window did not turn it on: the switch follows the window.
        verify(!sw.checked)
        line.speak = true
        verify(sw.checked)
    }

    // The voice reads only while playing with the switch on; pause stops it.
    function test_the_voice_reads_only_while_playing_with_the_switch_on() {
        fakeVoice.said = []
        fake.speech = JSON.stringify({ language: "de", lines: [{ index: 1, at_s: 6, text: "Hallo." }] })
        const line = make(de)
        line.voice = fakeVoice
        line.speak = true

        fake.time = 6
        compare(fakeVoice.said, [])
        line.togglePlay()
        compare(fakeVoice.said, ["Hallo."])

        fakeVoice.stops = 0
        line.togglePlay()
        compare(fakeVoice.stops, 1)
    }

    // The voice of a line at 6 s speaks (playing, the switch on); then the scene ends by itself.
    function endedWithVoice() {
        fakeVoice.said = []
        fake.speech = JSON.stringify({ language: "de", lines: [{ index: 1, at_s: 6, text: "Ende." }] })
        const line = make(de)
        line.voice = fakeVoice
        line.speak = true
        line.togglePlay()
        compare(fakeVoice.said, ["Ende."])

        fakeVoice.stops = 0
        fake.running = false
        tryVerify(() => !line.playing)
        compare(fakeVoice.stops, 0)
        return line
    }

    // After the scene's end the last line is read on; the voice stops with the switch ...
    function test_the_voice_after_the_end_stops_with_the_switch() {
        const line = endedWithVoice()
        line.speak = false
        compare(fakeVoice.stops, 1)
    }

    // ... with another scene or project ...
    function test_the_voice_after_the_end_stops_on_a_switch() {
        endedWithVoice()
        fake.switched()
        compare(fakeVoice.stops, 1)
    }

    // ... with a seek ...
    function test_the_voice_after_the_end_stops_on_a_seek() {
        endedWithVoice()
        fake.time = 3
        compare(fakeVoice.stops, 1)
    }

    // ... and with the next start.
    function test_the_voice_after_the_end_stops_on_the_next_play() {
        const line = endedWithVoice()
        fake.running = true
        line.looping = true
        line.togglePlay()
        verify(fakeVoice.stops >= 1)
    }

    // A shot loop wraps to the shot's start plus the rest of a tick: the line there is read
    // again, the one before the shot is not (shot 2 to 6 s, lines at 1 s and 2 s).
    function test_a_loop_wrap_reads_the_line_at_the_shot_start() {
        fakeVoice.said = []
        fake.tracksJson = JSON.stringify([
            { kind: "cut", id: "cut", name: "", label: "yellow", shown: true,
              segments: [{ from_s: 2, to_s: 6, text: "1", key: "s1", selected: true }], markers: [] }])
        fake.speech = JSON.stringify({ language: "de", lines: [
            { index: 1, at_s: 1, text: "Before." }, { index: 3, at_s: 2, text: "Shot." }] })
        fake.time = 5.9
        const line = make(de)
        line.voice = fakeVoice
        line.speak = true
        line.looping = true
        line.togglePlay()
        compare(fakeVoice.said, [])

        fake.nextLoopS = 2
        fake.nextTime = 2.04
        tryCompare(fakeVoice.said, "length", 1)
        compare(fakeVoice.said, ["Shot."])
    }

    // Without a selected shot the loop is the scene: the wrap reads the line at 0 s again.
    function test_a_scene_loop_wrap_reads_the_line_at_the_scene_start() {
        fakeVoice.said = []
        fake.tracksJson = "[]"
        fake.speech = JSON.stringify({ language: "de", lines: [
            { index: 1, at_s: 0, text: "Start." }, { index: 3, at_s: 10, text: "End." }] })
        fake.time = 19.9
        const line = make(de)
        line.voice = fakeVoice
        line.speak = true
        line.looping = true
        line.togglePlay()
        tryCompare(fakeVoice.said, "length", 0)

        fake.nextLoopS = 0
        fake.nextTime = 0.04
        tryCompare(fakeVoice.said, "length", 1)
        compare(fakeVoice.said, ["Start."])
    }

    // In a loop, a jump back by the user is a seek, not a wrap: nothing from the loop start.
    function test_a_jump_back_in_a_loop_reads_nothing_from_its_start() {
        fakeVoice.said = []
        fake.tracksJson = JSON.stringify([
            { kind: "cut", id: "cut", name: "", label: "yellow", shown: true,
              segments: [{ from_s: 2, to_s: 6, text: "1", key: "s1", selected: true }], markers: [] }])
        fake.speech = JSON.stringify({ language: "de", lines: [
            { index: 1, at_s: 2, text: "Start." }, { index: 3, at_s: 3, text: "Middle." },
            { index: 5, at_s: 5.5, text: "Late." }] })
        fake.time = 5
        const line = make(de)
        line.voice = fakeVoice
        line.speak = true
        line.looping = true
        line.togglePlay()
        fake.nextTime = 5.04
        tryVerify(() => fake.time === 5.04)

        fake.time = 4.5
        wait(100)
        compare(fakeVoice.said, [])
    }

    // The scene's end does not stop the voice: the last line is read to its end.
    function test_the_natural_end_leaves_the_voice_alone() {
        endedWithVoice()
        wait(100)
        compare(fakeVoice.stops, 0)
    }

    Component {
        id: spyComponent
        SignalSpy {}
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
        compare(b.markerOnText, "Next mark")
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

    // Dragging (LH-61).
    function laneOf(line) {
        return findChild(line, "tracks")
    }

    function rowY(lanes, row) {
        return lanes.rulerHeight + (row + 0.5) * lanes.trackHeight
    }

    function test_a_dragged_mark_is_dropped_once_where_it_lands() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))
        const y = rowY(lanes, 1)

        mousePress(lanes, lanes.xAt(8), y)
        mouseMove(lanes, lanes.xAt(9.5), y)
        const line = findChild(lanes, "dragLine")
        verify(line.visible)
        mouseMove(lanes, lanes.xAt(10), y)
        mouseRelease(lanes, lanes.xAt(10), y)

        verify(!line.visible)
        compare(fake.drops.length, 1)
        compare(fake.drops[0][0], "figure:1:anna")
        compare(fake.drops[0][2], "free")
        fuzzyCompare(fake.drops[0][1], 10, 0.1)
        compare(fake.seeks, [])
    }

    function test_the_drag_shows_the_time_it_lands_on() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))
        const y = rowY(lanes, 1)

        mousePress(lanes, lanes.xAt(4), y)
        verify(!lanes.dragging)
        mouseMove(lanes, lanes.xAt(5), y)
        verify(lanes.dragging)
        fuzzyCompare(lanes.dragSeconds, 5, 0.1)
        compare(findChild(lanes, "dragText").text, "00:05,0")
        mouseRelease(lanes, lanes.xAt(5), y)
        verify(!lanes.dragging)
    }

    function test_the_shift_key_drags_free() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))
        const y = rowY(lanes, 1)

        mousePress(lanes, lanes.xAt(8), y)
        mouseMove(lanes, lanes.xAt(9), y, -1, Qt.LeftButton, Qt.ShiftModifier)
        mouseMove(lanes, lanes.xAt(9.5), y, -1, Qt.LeftButton, Qt.NoModifier)
        mouseRelease(lanes, lanes.xAt(9.5), y)

        compare(fake.drags[0][1], "free")
        compare(fake.drags[1][1], "tenths")
    }

    function test_the_end_of_a_shot_is_dragged() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))
        const y = rowY(lanes, 0)

        mousePress(lanes, lanes.xAt(6), y)
        mouseMove(lanes, lanes.xAt(7), y)
        mouseRelease(lanes, lanes.xAt(7), y)

        compare(fake.drops.length, 1)
        compare(fake.drops[0][0], "shot-to:0:s1")
    }

    function test_the_start_of_a_shot_is_dragged() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))
        const y = rowY(lanes, 0)

        mousePress(lanes, lanes.xAt(2), y)
        mouseMove(lanes, lanes.xAt(3), y)
        mouseRelease(lanes, lanes.xAt(3), y)

        compare(fake.drops[0][0], "shot-from:0:s1")
    }

    function test_a_press_beside_a_mark_still_seeks() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))

        mouseClick(lanes, lanes.xAt(12), rowY(lanes, 1))
        verify(!lanes.dragging)
        compare(fake.drops, [])
        compare(fake.seeks.length, 1)
        fuzzyCompare(fake.seeks[0], 12, 0.1)
    }

    function test_the_middle_of_a_shot_is_no_edge() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))

        mousePress(lanes, lanes.xAt(4), rowY(lanes, 0))
        verify(!lanes.dragging)
        mouseRelease(lanes, lanes.xAt(4), rowY(lanes, 0))
        compare(fake.selected, ["s1"])
    }

    // Review of PR 96: a click on a mark seeks, as before the dragging.
    function test_a_mark_pressed_and_not_moved_seeks_and_is_not_dropped() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))

        mouseClick(lanes, lanes.xAt(8), rowY(lanes, 1))
        compare(fake.drops, [])
        compare(fake.seeks.length, 1)
        fuzzyCompare(fake.seeks[0], 8, 0.1)
    }

    // Review of PR 96: the tooltip of a mark (hover) was taken by the drag layer.
    function test_hovering_a_mark_names_it() {
        fake.tracksJson = tracksOfTwo
        const lanes = laneOf(make(de))

        mouseMove(lanes, lanes.xAt(4), rowY(lanes, 1))
        compare(lanes.hovered, { track: 1, index: 0 })
    }

    // Review of PR 96: a shot narrower than twice the reach of an edge could not be picked.
    function test_a_narrow_shot_is_picked_by_a_click() {
        fake.tracksJson = JSON.stringify([
            { kind: "cut", id: "cut", name: "", label: "yellow", shown: true,
              segments: [{ from_s: 6, to_s: 6.2, text: "2", key: "s2", selected: false }], markers: [] }
        ])
        const lanes = laneOf(make(de))

        mouseClick(lanes, lanes.xAt(6.1), rowY(lanes, 0))
        compare(fake.selected, ["s2"])
        compare(fake.drops, [])
    }

    // Review of PR 96: with the layer locked the drag did not even start.
    function test_a_locked_layer_is_not_grabbed() {
        fake.tracksJson = tracksOfTwo
        fake.locksJson = JSON.stringify({ figures: true })
        const lanes = laneOf(make(de))

        mousePress(lanes, lanes.xAt(8), rowY(lanes, 1))
        mouseMove(lanes, lanes.xAt(9), rowY(lanes, 1))
        verify(!lanes.dragging)
        mouseRelease(lanes, lanes.xAt(9), rowY(lanes, 1))
        compare(fake.drops, [])
    }

    function test_a_locked_camera_layer_keeps_the_shot_edges() {
        fake.tracksJson = tracksOfTwo
        fake.locksJson = JSON.stringify({ cameras: true })
        const lanes = laneOf(make(de))

        mousePress(lanes, lanes.xAt(6), rowY(lanes, 0))
        mouseMove(lanes, lanes.xAt(7), rowY(lanes, 0))
        verify(!lanes.dragging)
        mouseRelease(lanes, lanes.xAt(7), rowY(lanes, 0))
        compare(fake.drops, [])
    }

    function test_a_hidden_row_is_not_dragged() {
        fake.tracksJson = tracksOfTwo.replace('"shown":true,"segments":[],', '"shown":false,"segments":[],')
        const lanes = laneOf(make(de))

        mousePress(lanes, lanes.xAt(8), rowY(lanes, 1))
        verify(!lanes.dragging)
        mouseRelease(lanes, lanes.xAt(8), rowY(lanes, 1))
    }
}
