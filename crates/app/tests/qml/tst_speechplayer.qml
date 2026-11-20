import QtQuick
import QtTest
import "../../qml"

// SpeechPlayer (LH-64): hands each dialogue line to the voice once the playhead reaches it,
// stops on pause and on a seek. The voice is a fake: no sound in tests.
TestCase {
    id: tc
    name: "SpeechPlayer"
    width: 200
    height: 200
    when: windowShown

    QtObject {
        id: fakeController
        property real time: 0
        signal looped(real toS)
        readonly property string scene: JSON.stringify({
            language: "en",
            lines: [
                { index: 2, at_s: 2, text: "One." },
                { index: 4, at_s: 2, text: "Two." },
                { index: 6, at_s: 8, text: "Three." }
            ]
        })
        property string json: scene
        function speechJson() { return json }
    }

    QtObject {
        id: fakeVoice
        property var said: []
        property int stops: 0
        function say(text, language) { said.push(language + ":" + text) }
        function stop() { stops++ }
    }

    Component {
        id: playerComponent
        SpeechPlayer {
            controller: fakeController
            voice: fakeVoice
        }
    }

    function init() {
        fakeController.time = 0
        fakeController.json = fakeController.scene
        fakeVoice.said = []
        fakeVoice.stops = 0
    }

    // The playhead goes to `to` in ticks of 0.04 s, as the timeline's timer does.
    function playTo(to) {
        while (fakeController.time < to - 1e-9) {
            fakeController.time = Math.min(to, fakeController.time + 0.04)
        }
    }

    function make() {
        const player = createTemporaryObject(playerComponent, tc)
        verify(player)
        return player
    }

    function test_a_line_is_said_once_when_its_time_has_come() {
        const player = make()
        player.active = true
        compare(fakeVoice.said, [])

        playTo(1.9)
        compare(fakeVoice.said, [])
        playTo(2.04)
        compare(fakeVoice.said, ["en:One.", "en:Two."])
        playTo(7)
        compare(fakeVoice.said.length, 2)
        playTo(8.1)
        compare(fakeVoice.said, ["en:One.", "en:Two.", "en:Three."])
    }

    function test_a_line_at_the_playhead_is_said_at_the_start() {
        fakeController.time = 2
        const player = make()
        player.active = true

        compare(fakeVoice.said, ["en:One.", "en:Two."])
    }

    function test_pause_stops_the_voice_and_does_not_repeat_on_play() {
        const player = make()
        player.active = true
        playTo(2.04)
        fakeVoice.stops = 0

        player.active = false
        compare(fakeVoice.stops, 1)
        playTo(2.5)
        compare(fakeVoice.said.length, 2)

        player.active = true
        playTo(3)
        compare(fakeVoice.said.length, 2)
    }

    function test_a_seek_forward_skips_what_it_passes_and_stops_the_voice() {
        const player = make()
        player.active = true
        fakeVoice.stops = 0

        // The timeline's click: from 0 s to 5 s at once.
        fakeController.time = 5
        verify(fakeVoice.stops >= 1)
        compare(fakeVoice.said, [])

        playTo(8.1)
        compare(fakeVoice.said, ["en:Three."])
    }

    function test_a_seek_back_reads_the_lines_again() {
        const player = make()
        player.active = true
        playTo(2.04)
        compare(fakeVoice.said.length, 2)

        // A loop or the start button: the playhead goes back.
        fakeController.time = 0.5
        playTo(2.04)
        compare(fakeVoice.said.length, 4)
    }

    function test_a_scene_without_speech_says_nothing() {
        fakeController.json = "null"
        const player = make()
        player.active = true
        playTo(3)

        compare(fakeVoice.said, [])
    }

    function test_without_a_voice_nothing_happens() {
        const player = createTemporaryObject(playerComponent, tc, { voice: null })
        player.active = true
        playTo(3)
        compare(fakeVoice.said, [])
    }

    // A shot loop wraps to the shot's start plus the rest of the tick, and the controller says
    // so: the line at the start is read again (shot 5 to 10 s, lines at 5 and 7 s).
    function test_a_loop_wrap_restarts_at_the_loop_start() {
        fakeController.json = JSON.stringify({ language: "en", lines: [
            { index: 1, at_s: 5, text: "Shot." }, { index: 3, at_s: 7, text: "Middle." }] })
        fakeController.time = 5
        const player = createTemporaryObject(playerComponent, tc, {})
        player.active = true
        playTo(10)
        compare(fakeVoice.said, ["en:Shot.", "en:Middle."])

        fakeController.looped(5)
        fakeController.time = 5.04
        compare(fakeVoice.said, ["en:Shot.", "en:Middle.", "en:Shot."])
        playTo(7.1)
        compare(fakeVoice.said.length, 4)
    }

    // The reported jump decides, not the direction of the move: a loop entered from before
    // its shot reads the line at the shot's start, as a seek to 5.04 s would not.
    function test_a_reported_jump_restarts_even_when_time_did_not_get_smaller() {
        fakeController.json = JSON.stringify({ language: "en", lines: [
            { index: 1, at_s: 5, text: "Shot." }] })
        fakeController.time = 1
        const player = make()
        player.active = true

        fakeController.looped(5)
        fakeController.time = 5.04
        compare(fakeVoice.said, ["en:Shot."])
    }

    // A seek back inside the loop is no wrap: only what comes after it is read.
    function test_a_seek_back_inside_a_loop_is_no_wrap() {
        fakeController.json = JSON.stringify({ language: "en", lines: [
            { index: 1, at_s: 5, text: "Shot." }, { index: 3, at_s: 7, text: "Middle." }] })
        fakeController.time = 8
        const player = createTemporaryObject(playerComponent, tc, {})
        player.active = true
        compare(fakeVoice.said, [])

        fakeController.time = 5.8
        playTo(7.1)
        compare(fakeVoice.said, ["en:Middle."])
    }

    // The scene's end stops playing in the tick that said the last line: it is read to its end.
    function test_the_natural_end_lets_the_voice_finish() {
        const player = make()
        player.active = true
        playTo(8.1)
        fakeVoice.stops = 0

        player.letFinish = true
        player.active = false
        compare(fakeVoice.stops, 0)

        // The next play, then a pause: stops as ever.
        player.active = true
        fakeVoice.stops = 0
        player.active = false
        compare(fakeVoice.stops, 1)
    }
}
