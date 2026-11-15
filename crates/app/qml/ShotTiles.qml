import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The shots as storyboard tiles (D-05): the stage has the frame of the shot's camera, below
// it the number and size on the left and the lens on the right. The stage stays empty until
// the camera view (LH-36); the movement is written on it. A click selects the shot.
Item {
    id: tiles

    required property var texts
    /// The rows (`ShotRow`s with `lens_label`), already in the order to show.
    property var shots: []
    property string selectedId: ""

    signal picked(string id)

    readonly property real tileMin: KanteStyle.unit(168)
    readonly property real gap: Kirigami.Units.smallSpacing
    readonly property int columns: Math.max(1, Math.floor((flow.width + gap) / (tileMin + gap)))
    readonly property real tileWidth: (flow.width - (columns - 1) * gap) / columns

    QQC2.ScrollView {
        id: scroll
        anchors.fill: parent
        contentWidth: availableWidth

        Flow {
            id: flow
            x: tiles.gap
            width: scroll.availableWidth - 2 * tiles.gap
            topPadding: tiles.gap
            bottomPadding: tiles.gap
            spacing: tiles.gap

            Repeater {
                model: tiles.shots

                delegate: KanteTile {
                    id: tile
                    required property var modelData
                    objectName: "tile-" + modelData.id

                    width: tiles.tileWidth
                    height: width / stageAspect + KanteStyle.unit(34)
                    stageAspect: modelData.aspect
                    selected: modelData.id === tiles.selectedId
                    tier: modelData.axis_jump ? KanteTile.Tier.Check : KanteTile.Tier.Good
                    name: modelData.number + " · " + tiles.texts.t("size." + modelData.size)
                    figure: modelData.lens_label
                    onClicked: tiles.picked(modelData.id)

                    QQC2.Label {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.margins: Kirigami.Units.smallSpacing
                        color: KanteStyle.mutedTextColor
                        font: KanteStyle.labelFont()
                        text: tiles.texts.t("movement." + tile.modelData.movement)
                    }
                }
            }
        }
    }
}
