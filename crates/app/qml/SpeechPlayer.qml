import QtQuick

// Reads the dialogue while the scene plays (LH-64). The services say what is read when
// (`controller.speechJson()`); this keeps the place in that list and hands each line to the
// voice once the playhead has reached its time.
//
// A step of the playhead is small (a timer tick). A move back or by more than `jumpS` is a
// seek (the timeline, a mark): speech stops and goes on from the first line that has not come
// yet. A move back during a step that the timeline says wraps a loop (`wrapping`) goes on from
// the loop's start, so the line there is read again. `active` off (paused, stopped, switched
// off) stops it too, but not at the scene's end (`letFinish`): then the next move of the
// playhead, `stop()` or the next start ends it.
Item {
    id: reader

    required property var controller
    /// `say(text, language)` and `stop()`: SpeechVoice, or a fake in tests.
    property var voice: null
    /// Reading is wanted and the scene plays.
    property bool active: false
    /// Playing ends by itself (the scene's end): the voice reads on instead of stopping. Set
    /// before `active` goes off, taken back by `stop()` and at the next start.
    property bool letFinish: false
    /// The timeline is in a step that may wrap a shot loop to `loopStartS` plus the rest of
    /// the tick: a move back in it is no seek.
    property bool wrapping: false
    property real loopStartS: 0

    readonly property real jumpS: 1
    readonly property real epsS: 1e-6

    property var lines: []
    property string language: "de"
    property int next: 0
    property real lastS: 0

    visible: false

    // The list is read again at each start: the script and its links may have changed.
    function load() {
        const speech = JSON.parse(controller.speechJson() || "null")
        lines = speech ? speech.lines : []
        language = speech ? speech.language : "de"
    }

    function skipTo(timeS) {
        if (voice) {
            voice.stop()
        }
        const first = lines.findIndex(l => l.at_s >= timeS - epsS)
        next = first < 0 ? lines.length : first
    }

    function stop() {
        letFinish = false
        if (voice) {
            voice.stop()
        }
    }

    function follow() {
        // After the scene's end the voice still reads; any move of the playhead is a seek.
        if (!active && letFinish) {
            stop()
            return
        }
        if (!active || !voice) {
            return
        }

        const t = controller.time
        if (t < lastS || t - lastS > jumpS) {
            skipTo(wrapping && t < lastS ? loopStartS : t)
        }
        lastS = t

        while (next < lines.length && lines[next].at_s <= t + epsS) {
            voice.say(lines[next].text, language)
            next++
        }
    }

    onActiveChanged: {
        if (!voice) {
            return
        }
        if (!active) {
            if (!letFinish) {
                voice.stop()
            }
            return
        }

        letFinish = false
        load()
        lastS = controller.time
        skipTo(lastS)
        follow()
    }

    Connections {
        target: reader.controller
        function onTimeChanged() { reader.follow() }
    }
}
