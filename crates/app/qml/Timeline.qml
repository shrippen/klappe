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

    property bool playing: false
    property bool looping: false
    property real speed: 1

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
        }
    }
}
