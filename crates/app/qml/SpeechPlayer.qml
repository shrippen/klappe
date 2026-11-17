import QtQuick

// Reads the dialogue while the scene plays (LH-64). The services say what is read when
// (`controller.speechJson()`); this keeps the place in that list and hands each line to the
// voice once the playhead has reached its time.
//
// A step of the playhead is small (a timer tick). A move back or by more than `jumpS` is a
// seek (the timeline, a mark): speech stops and goes on from the first line that has not come
// yet. A move back to within `jumpS` of the loop's start is the loop wrapping: it goes on from
// the start itself, so the line there is read again. `active` off (paused, stopped,
// switched off) stops it too, but not at the scene's end (`letFinish`).
Item {
    id: reader

    required property var controller
    /// `say(text, language)` and `stop()`: SpeechVoice, or a fake in tests.
    property var voice: null
    /// Reading is wanted and the scene plays.
    property bool active: false
    /// Playing ends by itself (the scene's end): the voice reads on instead of stopping. Set
    /// before `active` goes off, taken back at the next start.
    property bool letFinish: false
    /// A shot loop wraps to the start of its range plus the rest of the tick: that is no seek.
    property bool looping: false
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

    function follow() {
        if (!active || !voice) {
            return
        }

        const t = controller.time
        if (t < lastS || t - lastS > jumpS) {
            const wrapped = looping && t < lastS && t - loopStartS <= jumpS
            skipTo(wrapped ? loopStartS : t)
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
            letFinish = false
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
