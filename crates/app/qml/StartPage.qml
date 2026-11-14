import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Shown while no project is open (LH-01, D-07): the projects opened last as tiles with a
// preview of their first plan, the example as the last tile, and the two ways to start another.
// Arrow keys move between the tiles, Enter opens one; a project whose folder is gone is dimmed
// and can be taken off the list. The first tile has the keyboard focus whenever the page shows.
Item {
    id: start

    required property var texts
    required property var controller

    signal newRequested()
    signal openRequested()
    signal recentRequested(string path)
    signal exampleRequested()
    signal forgetRequested(string path)

    /// `[{ path, shortPath, name, state, scenes, modifiedS, preview }]`, state: loading, ready,
    /// missing, unreadable. `shortPath` is the path with `~` for the home folder (the tooltip).
    readonly property var recent: controller.recentJson ? JSON.parse(controller.recentJson) : []

    readonly property string stateMissing: "missing"
    readonly property string stateUnreadable: "unreadable"
    readonly property string stateExample: "example" // the last row: no project, the example
    readonly property real stageAspect: 1.414 // an A4 sheet, what the preview is drawn on
    readonly property real tileWidth: KanteStyle.unit(240)
    // Under the stage: the tile's own caption, then the subtitle and the remove button.
    readonly property real captionHeight: KanteStyle.unit(34)
    readonly property real belowTile: KanteStyle.unit(56) + Kirigami.Units.largeSpacing

    // Rows are updated in place while the previews arrive, so the tiles and the focus stay. The
    // last row is the example tile.
    ListModel { id: rows }

    function row(entry) {
        return {
            path: entry.path,
            shortPath: entry.shortPath || entry.path,
            name: entry.name,
            state: entry.state,
            scenes: entry.scenes || 0,
            modifiedS: entry.modifiedS || 0,
            preview: entry.preview || ""
        }
    }

    function sync() {
        const same = recent.length + 1 === rows.count
            && recent.every((entry, i) => rows.get(i).path === entry.path)
        if (same) {
            recent.forEach((entry, i) => rows.set(i, row(entry)))
            return
        }

        rows.clear()
        recent.forEach(entry => rows.append(row(entry)))
        rows.append(row({ path: "", name: "", state: stateExample }))
    }

    // The page may show after its tiles were made (a project is closed, the window opens).
    function focusFirst() {
        if (!visible || grid.count === 0) {
            return
        }
        if (grid.currentIndex < 0) {
            grid.currentIndex = 0
        }
        grid.currentItem.focusTile()
    }

    onVisibleChanged: focusFirst()

    function subtitle(state, scenes, modifiedS) {
        if (state === stateMissing) {
            return texts.t("start.missing")
        }
        if (state === stateUnreadable) {
            return texts.t("start.unreadable")
        }
        if (state !== "ready") {
            return ""
        }

        const count = scenes === 1 ? texts.t("start.scene1") : texts.t("start.scenes", { count: scenes })
        if (modifiedS === 0) {
            return count
        }

        const date = new Date(modifiedS * 1000).toLocaleDateString(Qt.locale(), Locale.ShortFormat)
        return count + " · " + date
    }

    onRecentChanged: sync()
    Component.onCompleted: sync()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        spacing: Kirigami.Units.largeSpacing

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            KanteHeading {
                Layout.fillWidth: true
                text: start.texts.t("start.title")
                level: 2
            }
            KanteButton {
                objectName: "newButton"
                emphasis: KanteButton.Emphasis.Primary
                text: start.texts.t("start.new")
                onClicked: start.newRequested()
            }
            KanteButton {
                objectName: "openButton"
                text: start.texts.t("start.open")
                onClicked: start.openRequested()
            }
        }

        QQC2.Label {
            objectName: "noRecent"
            Layout.fillWidth: true
            visible: start.recent.length === 0
            color: KanteStyle.mutedTextColor
            text: start.texts.t("start.noRecent")
        }

        GridView {
            id: grid
            objectName: "tiles"
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: rows
            cellWidth: start.tileWidth + Kirigami.Units.largeSpacing
            cellHeight: start.tileWidth / start.stageAspect + start.captionHeight + start.belowTile
            keyNavigationEnabled: true
            keyNavigationWraps: false

            Keys.onDeletePressed: {
                if (currentItem && currentItem.missing) {
                    start.forgetRequested(currentItem.path)
                }
            }

            delegate: Item {
                id: cell

                required property int index
                required property string path
                required property string shortPath
                required property string name
                required property string state
                required property int scenes
                required property real modifiedS
                required property string preview

                readonly property bool isExample: state === start.stateExample
                readonly property bool missing: state === start.stateMissing
                readonly property bool broken: missing || state === start.stateUnreadable

                width: grid.cellWidth
                height: grid.cellHeight

                function focusTile() {
                    if (GridView.isCurrentItem && start.visible) {
                        tile.forceActiveFocus()
                    }
                }

                GridView.onIsCurrentItemChanged: focusTile()
                Component.onCompleted: focusTile()

                ColumnLayout {
                    x: 0
                    y: 0
                    width: start.tileWidth
                    spacing: Kirigami.Units.smallSpacing

                    KanteTile {
                        id: tile
                        objectName: cell.isExample ? "example" : "recent"
                        implicitWidth: start.tileWidth
                        Layout.preferredWidth: start.tileWidth
                        stageAspect: start.stageAspect
                        name: cell.isExample ? start.texts.t("start.example") : cell.name
                        tier: cell.broken ? KanteTile.Tier.Check : KanteTile.Tier.Good
                        stale: cell.missing
                        source: cell.preview !== "" ? "file://" + encodeURI(cell.preview) : ""
                        sourceSize: Qt.size(start.tileWidth * 2, start.tileWidth * 2 / start.stageAspect)
                        QQC2.ToolTip.text: cell.shortPath
                        QQC2.ToolTip.visible: tipHover.hovered && cell.shortPath !== ""
                        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                        onClicked: {
                            if (cell.isExample) {
                                start.exampleRequested()
                                return
                            }
                            if (!cell.missing) {
                                start.recentRequested(cell.path)
                            }
                        }
                        onActiveFocusChanged: {
                            if (activeFocus) {
                                grid.currentIndex = cell.index
                            }
                        }

                        // The example has no preview: the tile's own placeholder shows.
                        HoverHandler { id: tipHover }
                    }
                    QQC2.Label {
                        objectName: "recentSub"
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                        color: KanteStyle.mutedTextColor
                        text: start.subtitle(cell.state, cell.scenes, cell.modifiedS)
                    }
                    KanteButton {
                        objectName: "forgetButton"
                        visible: cell.missing
                        text: start.texts.t("start.forget")
                        onClicked: start.forgetRequested(cell.path)
                    }
                }
            }
        }
    }
}
