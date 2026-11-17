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
}
