import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Timeline (round 2, design 1 and 5, D-06): cut order, then figures, cameras and lights, each
// with its own visibility switch; above it the transport bar (LH-60). The layers are in
// LayerPopover on the plan.
//
// Playing: a timer asks the controller to advance by the real time since its last tick; the
// controller (services) does the arithmetic and says when the end is reached.
//
// Dragging (LH-61, in KanteTrackTimeline): a mark of a figure or camera moves its arrival, an
// edge of a shot in the cut row moves that edge. While the button is down a line and the time
// show where it lands; the controller (services) snaps it to 0.1 s (Shift: free) and keeps it
// between its neighbours; on release one undo step is made. A locked layer is not grabbed.
Item {
    id: line

    required property var texts
    required property var controller

    readonly property var source: controller.tracksJson ? JSON.parse(controller.tracksJson) : []

    readonly property var speedSteps: [0.25, 0.5, 1]
    readonly property int tickMs: 40
    readonly property real msPerS: 1000
    /// Within this of the end (s), play starts again from the beginning.
    readonly property real endEpsS: 0.05

    readonly property var locks: controller.locksJson ? JSON.parse(controller.locksJson) : ({})
    // The layer whose lock keeps a row from being dragged, by kind of row.
    readonly property var layerOfKind: ({ cut: "cameras", figure: "figures", camera: "cameras" })

    property bool playing: false
    property bool looping: false
    property real speed: 1

    // "mm:ss,d" with the language's decimal separator, cut off like a timecode (59.96 → 00:59,9).
    readonly property int tenthsPerS: 10
    readonly property int tenthsPerMin: 600
    function clock(s) {
        const tenths = Math.floor(Math.max(0, s) * tenthsPerS + 1e-6)
        const sec = (tenths % tenthsPerMin) / tenthsPerS
        const min = Math.floor(tenths / tenthsPerMin)
        return (min < 10 ? "0" : "") + min + ":" + (sec < 10 ? "0" : "") + texts.fixed(sec, 1)
    }

    function togglePlay() {
        if (playing) {
            playing = false
            return
        }
        if (!looping && controller.time >= controller.duration - endEpsS) {
            controller.seek(0)
        }
        playing = true
    }

    // Locked (backup question), another project, scene or plan: stop, nobody pressed play there.
    onEnabledChanged: {
        if (!enabled) {
            playing = false
        }
    }
    Connections {
        target: line.controller
        function onSwitched() { line.playing = false }
    }

    Timer {
        id: clock
        interval: line.tickMs
        repeat: true
        running: line.playing
        property double lastMs: 0
        onRunningChanged: lastMs = Date.now()
        onTriggered: {
            const now = Date.now()
            const dt = (now - lastMs) / line.msPerS
            lastMs = now
            if (!line.controller.advance(dt, line.speed, line.looping ? "shot" : "off")) {
                line.playing = false
            }
        }
    }

    // Moves what a drag holds: the time it lands on while the button is down, the move itself on
    // release (the controller says it again, snapped).
    function drag(key, seconds, done, shift) {
        if (done) {
            controller.dropTime(key, lanes.dragSeconds, "free")
            return
        }
        lanes.dragSeconds = controller.dragTime(key, seconds, shift ? "free" : "tenths")
    }

    // The services' tracks in KanteTrackTimeline's shape.
    readonly property var tracks: source.map(tr => ({
        name: tr.kind === "cut" ? line.texts.t("timeline.cut") : tr.name,
        label: tr.label,
        shown: tr.shown,
        locked: line.locks[line.layerOfKind[tr.kind]] === true,
        // Only the edges of a shot are dragged, not the walks between marks or a light's span.
        segments: tr.segments.map(s => ({ from: s.from_s, to: s.to_s, text: s.text, selected: s.selected,
                                          locked: tr.kind !== "cut" })),
        markers: tr.markers.map(m => ({ at: m.at_s, text: m.text }))
    }))

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteTransportBar {
            id: bar
            objectName: "transport"
            Layout.fillWidth: true
            metronomeVisible: false
            countInVisible: false
            markersVisible: true
            timeFormat: "mm:ss,d"
            positionText: line.clock(line.controller.time) + " / " + line.clock(line.controller.duration)
            speedSteps: line.speedSteps
            playing: line.playing
            looping: line.looping
            speed: line.speed
            position: line.controller.time
            duration: line.controller.duration
            rewindText: line.texts.t("timeline.start")
            playText: line.texts.t("timeline.play")
            pauseText: line.texts.t("timeline.pause")
            loopText: line.texts.t("timeline.loop")
            speedText: line.texts.t("timeline.speed")
            markerBackText: line.texts.t("timeline.markerBack")
            markerOnText: line.texts.t("timeline.markerOn")
            positionName: line.texts.t("timeline.position")
            accessibleName: line.texts.t("timeline.transport")

            onRewind: line.controller.seek(0)
            onPlayToggled: line.togglePlay()
            onLoopToggled: line.looping = !line.looping
            onSpeedChangeRequested: s => line.speed = s
            onMarkerStepRequested: direction => line.controller.stepMark(direction)
        }

        KanteTrackTimeline {
            id: lanes
            objectName: "tracks"
            Layout.fillWidth: true
            Layout.fillHeight: true
            duration: Math.max(1, line.controller.duration)
            position: line.controller.time
            tracks: line.tracks
            hideText: line.texts.t("timeline.hide")
            showText: line.texts.t("timeline.show")
            accessibleName: line.texts.t("toggle.timeline")
            draggable: true
            formatTime: seconds => line.clock(seconds)

            onSeek: seconds => line.controller.seek(seconds)
            onToggleShown: index => {
                const tr = line.source[index]
                line.controller.setTrack(tr.id, tr.shown ? "hidden" : "shown")
            }
            onSegmentClicked: (index, segment) => {
                const tr = line.source[index]
                if (tr.kind === "cut") {
                    line.controller.selectShot(tr.segments[segment].key)
                }
            }

            onMarkDragged: (index, marker, seconds, done, shift) => {
                const tr = line.source[index]
                line.drag(tr.kind + ":" + marker + ":" + tr.id, seconds, done, shift)
            }
            onEdgeDragged: (index, segment, edge, seconds, done, shift) => {
                const tr = line.source[index]
                line.drag("shot-" + edge + ":0:" + tr.segments[segment].key, seconds, done, shift)
            }
        }
    }
}
