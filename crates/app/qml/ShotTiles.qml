import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The shots as storyboard tiles (D-05): the stage has the frame of the shot's camera, below
// it the number and size on the left and the lens on the right. The stage shows the camera view
// at the start of the shot (LH-36) until the shot has a storyboard picture (LH-74), which takes
// its place; the movement is written on it, and a jump over the axis (LH-41) as a
// warning chip, not only by the tier colour. A click selects the shot. The number stands on the
// stage in a chip in the colour of the shot's camera (LH-111) with the short name beside it, and
// the camera's label is in the caption: the colour is never alone.
Item {
    id: tiles

    required property var texts
    /// The rows (`ShotRow`s with `lens_label`), already in the order to show.
    property var shots: []
    property string selectedId: ""
    // `shotImage(id, widthPx, paletteJson)` and `pictureRevision`: the views are made again when
    // the revision, the width or the theme changes. Without a controller the stage stays empty.
    property var controller: null
    // The element that shows an image (`show(image)`); a Rust type the tests do not have.
    property url surfaceSource: Qt.resolvedUrl("CameraSurface.qml")

    signal picked(string id)

    readonly property bool canPaint: controller !== null && typeof controller.shotImage === "function"
    readonly property string paletteJson: colors.json
    // Counts the changes; a tile makes its view again when it shows an earlier round.
    property int round: 0

    // A change repaints the tiles with the next tick, however many changes come before it.
    function refresh() {
        round++
    }

    ViewPalette { id: colors }

    onPaletteJsonChanged: refresh()

    Connections {
        target: tiles.canPaint ? tiles.controller : null
        function onPictureRevisionChanged() {
            tiles.refresh()
        }
    }

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
                    // The tile's own height, the stage grown from its default width to this one.
                    height: implicitHeight + (width - implicitWidth) / stageAspect
                    stageAspect: modelData.aspect
                    selected: modelData.id === tiles.selectedId
                    tier: modelData.axis_jump ? KanteTile.Tier.Check : KanteTile.Tier.Good
                    name: modelData.camera_label + " · " + tiles.texts.t("size." + modelData.size)
                    figure: modelData.lens_label
                    source: modelData.image_url ? modelData.image_url : ""
                    onClicked: tiles.picked(modelData.id)

                    // The camera view over the placeholder (it paints nothing for a camera
                    // without marks), until a storyboard picture takes the stage.
                    readonly property bool wantsView: tiles.canPaint && tiles.visible && !modelData.image_url

                    function paint() {
                        if (!wantsView || width <= 0 || surface.item === null) {
                            return
                        }
                        const px = Math.max(1, Math.round(width * Screen.devicePixelRatio))
                        const image = tiles.controller.shotImage(modelData.id, px, tiles.paletteJson)
                        surface.item.show(image)
                    }

                    onWantsViewChanged: if (wantsView) { paintLater.restart() }
                    onWidthChanged: paintLater.restart()
                    Component.onCompleted: paintLater.restart()

                    Connections {
                        target: tiles
                        function onRoundChanged() {
                            paintLater.restart()
                        }
                    }
                    Timer {
                        id: paintLater
                        interval: 120
                        onTriggered: tile.paint()
                    }

                    Loader {
                        id: surface
                        objectName: "tileView-" + tile.modelData.id
                        anchors.fill: parent
                        active: tile.wantsView
                        source: tiles.surfaceSource
                        onLoaded: tile.paint()
                    }

                    Row {
                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.margins: Kirigami.Units.smallSpacing
                        spacing: Kirigami.Units.smallSpacing

                        KanteChip {
                            objectName: "numberChip-" + tile.modelData.id
                            interactive: false
                            chipColor: tile.modelData.camera_color ? KanteStyle.labelColor(tile.modelData.camera_color) : KanteStyle.textColor
                            text: tile.modelData.number
                        }
                        KanteChip {
                            objectName: "shortName-" + tile.modelData.id
                            visible: !!tile.modelData.short_name
                            interactive: false
                            text: tile.modelData.short_name
                        }
                    }

                    QQC2.Label {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.margins: Kirigami.Units.smallSpacing
                        color: KanteStyle.mutedTextColor
                        font: KanteStyle.labelFont()
                        text: tiles.texts.t("movement." + tile.modelData.movement)
                    }

                    KanteChip {
                        objectName: "axisJump-" + tile.modelData.id
                        visible: tile.modelData.axis_jump
                        anchors.right: parent.right
                        anchors.bottom: parent.bottom
                        anchors.margins: Kirigami.Units.smallSpacing
                        interactive: false
                        chipColor: KanteStyle.warningColor
                        iconName: "dialog-warning"
                        text: tiles.texts.t("shot.axisJump")
                    }
                }
            }
        }
    }
}
