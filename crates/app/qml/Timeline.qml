import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Timeline (round 2, design 1 and 5): cut order, then figures, cameras and lights, each with
// its own visibility switch; above it the switches for whole layers: shown and locked (a
// locked layer is drawn but cannot be picked or changed).
Item {
    id: line

    required property var texts
    required property var controller

    readonly property var source: controller.tracksJson ? JSON.parse(controller.tracksJson) : []
    readonly property var layers: controller.layersJson ? JSON.parse(controller.layersJson) : ({})
    readonly property var locks: controller.locksJson ? JSON.parse(controller.locksJson) : ({})

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

        RowLayout {
            spacing: Kirigami.Units.smallSpacing
            KanteToolButton {
                objectName: "startButton"
                text: line.texts.t("timeline.start")
                Accessible.name: text
                onClicked: line.controller.seek(0)
            }
            KanteSectionLabel {
                objectName: "clock"
                text: line.texts.t("timeline.clock", {
                    time: line.texts.fixed(line.controller.time, 1),
                    total: line.texts.fixed(line.controller.duration, 1),
                    unit: line.texts.t("unit.s")
                })
            }
            Item { Layout.fillWidth: true }
            KanteSectionLabel { text: line.texts.t("layer.title") }
            Repeater {
                model: ["set", "figures", "cameras", "lights", "notes", "drawings"]
                delegate: RowLayout {
                    required property string modelData
                    spacing: 0

                    KanteToolButton {
                        objectName: "layerButton_" + modelData
                        text: line.texts.t("layer." + modelData)
                        checkable: true
                        checked: line.layers[modelData] !== false
                        onToggled: line.controller.setLayer(modelData, checked ? "shown" : "hidden")
                    }
                    KanteToolButton {
                        objectName: "lockButton_" + modelData
                        text: line.texts.t("layer.locked")
                        Accessible.name: line.texts.t("layer.lockTip", { layer: line.texts.t("layer." + modelData) })
                        checkable: true
                        checked: line.locks[modelData] === true
                        onToggled: line.controller.setLock(modelData, checked ? "locked" : "unlocked")
                        QQC2.ToolTip.text: Accessible.name
                        QQC2.ToolTip.visible: hovered
                        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                    }
                }
            }
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
