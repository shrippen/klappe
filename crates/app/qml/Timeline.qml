import QtQuick
import QtQuick.Controls as QQC2
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
// Dragging (LH-61): a mark of a figure or camera moves its arrival, an edge of a shot in the cut
// row moves that edge. While the button is down a line and the time show where it lands; the
// controller (services) snaps it to 0.1 s (Shift: free) and keeps it between its neighbours;
// on release one undo step is made.
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

    /// Reach (px) within which a shot edge is grabbed.
    readonly property real edgeReach: KanteStyle.unit(5)

    // The held mark or shot edge: { key, seconds }, the key as for controller.dragTime and the
    // time it started at; null when nothing is dragged.
    property var grab: null
    property real dragSeconds: 0
    readonly property bool dragging: grab !== null

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

    // What a press at (x, y) of the lanes (their own coordinates) would hold: a mark of a
    // shown figure or camera row, or the nearest edge of a shot of the cut row within reach.
    function grabAt(x, y) {
        const tr = source[Math.floor((y - lanes.rulerHeight) / lanes.trackHeight)]
        if (!tr || !tr.shown) {
            return null
        }

        if (tr.kind === "cut") {
            return edgeAt(tr, x)
        }
        if (tr.kind !== "figure" && tr.kind !== "camera") {
            return null
        }

        const hit = lanes.markerAt(x, y)
        if (!hit) {
            return null
        }
        return { key: tr.kind + ":" + hit.index + ":" + tr.id, seconds: tr.markers[hit.index].at_s }
    }

    function edgeAt(tr, x) {
        let found = null
        let nearest = edgeReach
        for (const seg of tr.segments) {
            const edges = [["shot-from", seg.from_s], ["shot-to", seg.to_s]]
            for (const [kind, seconds] of edges) {
                const distance = Math.abs(lanes.xAt(seconds) - x)
                if (distance <= nearest) {
                    nearest = distance
                    found = { key: kind + ":0:" + seg.key, seconds: seconds }
                }
            }
        }
        return found
    }

    // The services' tracks in KanteTrackTimeline's shape.
    readonly property var tracks: source.map(tr => ({
        name: tr.kind === "cut" ? line.texts.t("timeline.cut") : tr.name,
        label: tr.label,
        shown: tr.shown,
        segments: tr.segments.map(s => ({ from: s.from_s, to: s.to_s, text: s.text, selected: s.selected })),
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

            // Takes a press on a mark or a shot edge; any other press goes on to the lanes
            // below (seek, select).
            MouseArea {
                id: retime
                objectName: "retime"
                x: lanes.labelWidth
                width: lanes.laneWidth
                height: lanes.height
                cursorShape: line.dragging ? Qt.SizeHorCursor : Qt.ArrowCursor

                function _held(mouse) {
                    return line.grabAt(mouse.x + lanes.labelWidth, mouse.y)
                }

                onPressed: mouse => {
                    const held = _held(mouse)
                    if (!held) {
                        mouse.accepted = false
                        return
                    }
                    line.grab = held
                    line.dragSeconds = held.seconds
                }
                onPositionChanged: mouse => {
                    if (!line.dragging) {
                        return
                    }
                    const snap = (mouse.modifiers & Qt.ShiftModifier) ? "free" : "tenths"
                    const at = lanes.timeAt(mouse.x + lanes.labelWidth)
                    const g = line.grab
                    line.dragSeconds = line.controller.dragTime(g.key, at, snap)
                }
                onReleased: {
                    const g = line.grab
                    line.grab = null
                    if (g && line.dragSeconds !== g.seconds) {
                        // Already snapped while dragging.
                        line.controller.dropTime(g.key, line.dragSeconds, "free")
                    }
                }
                onCanceled: line.grab = null

                HoverHandler {
                    id: hover
                    cursorShape: line.grabAt(hover.point.position.x + lanes.labelWidth,
                                             hover.point.position.y) ? Qt.SizeHorCursor
                                                                      : Qt.ArrowCursor
                }

                // Where the drag lands, with its time.
                Rectangle {
                    id: ghost
                    objectName: "dragLine"
                    visible: line.dragging
                    x: lanes.xAt(line.dragSeconds) - lanes.labelWidth - width / 2
                    width: KanteStyle.unit(2)
                    height: parent.height
                    color: KanteStyle.accentColor

                    QQC2.ToolTip {
                        visible: line.dragging
                        text: line.clock(line.dragSeconds)
                    }
                }
            }
        }
    }
}
