pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The shot list as a table (D-05, LH-71): the columns of the exported list (15), a row per shot in
// the order given, in shooting order a band per setup (LH-72). A click on a row selects the
// shot; double click or Enter edits the cell, Enter or a click away sends it, Esc drops it.
//
//  Nr.  Kam. Obj.  Größe  …       Up/Down: row, Left/Right: column
//  ── Setup 1 ───────────────     Enter or F2: edit the cell
//  12.1  A   35    Totale …
FocusScope {
    id: table

    /// How a column is edited: text as typed, one of a list, or not at all (computed).
    enum Kind {
        Text,
        Choice,
        Fixed
    }

    required property var texts
    /// The rows (`ShotRow`s with `lens_label`), already in the order to show.
    property var shots: []
    property string selectedId: ""
    /// "shooting" puts a band with the setup in front of each group.
    property string order: "cut"
    /// Choices of the Choice columns, as in the shot form.
    property var cameras: []
    property var sizes: []
    property var angles: []
    property var movements: []

    signal picked(string id)
    /// A cell was edited: the column's key is the field `editShot` names.
    signal edited(string id, string field, string value)

    // The shot and column in the editor; no id: no editor.
    property string editId: ""
    property int editColumn: 0
    property int currentColumn: 0
    // Counts refused values; the cells' texts read it, so an editor goes back to the model.
    property int refused: 0

    readonly property real rowHeight: KanteStyle.heightMedium
    readonly property real _unit: Kirigami.Units.gridUnit

    readonly property var columns: [
        { key: "number", kind: ShotTable.Kind.Text, title: "field.number", width: _unit * 4 },
        { key: "camera", kind: ShotTable.Kind.Choice, title: "field.camera", width: _unit * 5 },
        { key: "lens", kind: ShotTable.Kind.Text, title: "field.lens", width: _unit * 5 },
        { key: "setup", kind: ShotTable.Kind.Fixed, title: "field.setup", width: _unit * 4 },
        { key: "size", kind: ShotTable.Kind.Choice, title: "field.size", width: _unit * 8 },
        { key: "angle", kind: ShotTable.Kind.Choice, title: "field.perspective", width: _unit * 8 },
        { key: "movement", kind: ShotTable.Kind.Choice, title: "field.movement", width: _unit * 8 },
        { key: "from_s", kind: ShotTable.Kind.Fixed, title: "field.from", width: _unit * 4 },
        { key: "to_s", kind: ShotTable.Kind.Fixed, title: "field.to", width: _unit * 4 },
        { key: "duration", kind: ShotTable.Kind.Fixed, title: "field.duration", width: _unit * 4 },
        { key: "description", kind: ShotTable.Kind.Text, title: "field.description", width: _unit * 16 },
        { key: "props", kind: ShotTable.Kind.Text, title: "field.props", width: _unit * 10 },
        { key: "gear", kind: ShotTable.Kind.Text, title: "field.gear", width: _unit * 10 },
        { key: "crew", kind: ShotTable.Kind.Text, title: "field.crew", width: _unit * 10 },
        { key: "note", kind: ShotTable.Kind.Text, title: "field.note", width: _unit * 12 }
    ]

    readonly property var planKeys: [Qt.Key_F2, Qt.Key_Return, Qt.Key_Enter, Qt.Key_Delete, Qt.Key_R, Qt.Key_G, Qt.Key_D]

    // The rows plus a band before each new setup in shooting order.
    readonly property var items: {
        const out = []
        let setup = 0
        for (const shot of shots) {
            if (order === "shooting" && shot.setup !== setup) {
                setup = shot.setup
                out.push({ band: true, setup: setup })
            }
            out.push({ band: false, shot: shot })
        }
        return out
    }

    // Row and column of the shot with `id` in `items`; -1 if it is not listed.
    function rowOf(id) {
        return items.findIndex(i => !i.band && i.shot.id === id)
    }

    // What the cell of `shot` shows.
    function cellText(shot, key) {
        switch (key) {
        case "camera": return shot.camera_label
        case "lens": return shot.lens_label
        case "size": return texts.t("size." + shot.size)
        case "angle": return texts.t("angle." + shot.angle)
        case "movement": return texts.t("movement." + shot.movement)
        case "from_s": return texts.num(shot.from_s, 1)
        case "to_s": return texts.num(shot.to_s, 1)
        case "duration": return texts.num(shot.to_s - shot.from_s, 1)
        case "setup": return String(shot.setup)
        default: return shot[key]
        }
    }

    // What a text editor starts with: the lens only if the shot has its own.
    function editText(shot, key) {
        if (key === "lens") {
            return shot.own_lens_mm !== null ? texts.num(shot.own_lens_mm, 1) : ""
        }
        return shot[key]
    }

    // The values of a Choice column, and the one the shot has now.
    function choicesOf(key) {
        switch (key) {
        case "camera": return { ids: cameras.map(c => c.id), names: cameras.map(c => c.name) }
        case "size": return { ids: sizes, names: sizes.map(s => texts.t("size." + s)) }
        case "angle": return { ids: angles, names: angles.map(a => texts.t("angle." + a)) }
        default: return { ids: movements, names: movements.map(m => texts.t("movement." + m)) }
        }
    }
    function choiceNow(shot, key) {
        return key === "camera" ? shot.camera : shot[key]
    }

    // Whether the cell can be edited: not the computed columns, nor a lens the marks give.
    function editable(shot, column) {
        const col = columns[column]
        if (col.kind === ShotTable.Kind.Fixed) {
            return false
        }
        return !(col.key === "lens" && shot.lens_from_marks)
    }

    function edit(id, column) {
        const row = rowOf(id)
        if (row < 0 || !editable(items[row].shot, column)) {
            return
        }
        picked(id)
        currentColumn = column
        editColumn = column
        editId = id
    }

    function stopEdit() {
        editId = ""
        list.forceActiveFocus()
    }

    // Sends the cell's value unless it is what it was.
    function commit(shot, key, value, was) {
        stopEdit()
        if (value === was) {
            return
        }
        edited(shot.id, key, value)
    }

    // Moves the selection to the next or previous shot (bands are skipped).
    function step(delta) {
        let row = rowOf(selectedId)
        const next = row < 0 ? 0 : row + delta
        for (row = next; row >= 0 && row < items.length; row += delta) {
            if (!items[row].band) {
                picked(items[row].shot.id)
                return
            }
        }
    }

    implicitHeight: rowHeight * 6

    Flickable {
        id: sideways
        anchors.fill: parent
        clip: true
        contentWidth: Math.max(width, head.implicitWidth)
        contentHeight: height
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.HorizontalFlick
        QQC2.ScrollBar.horizontal: QQC2.ScrollBar {}

        KanteTableHeader {
            id: head
            width: sideways.contentWidth
            implicitWidth: table.columns.reduce((sum, c) => sum + c.width, 0) + (table.columns.length - 1) * spacing
            columns: table.columns.map(c => ({ title: table.texts.t(c.title), width: c.width }))
        }

        ListView {
            id: list
            objectName: "shotRows"
            anchors.top: head.bottom
            anchors.bottom: parent.bottom
            width: sideways.contentWidth
            clip: true
            focus: true
            model: table.items
            currentIndex: table.rowOf(table.selectedId)
            QQC2.ScrollBar.vertical: QQC2.ScrollBar {}

            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)

            Keys.onUpPressed: table.step(-1)
            Keys.onDownPressed: table.step(1)
            Keys.onLeftPressed: table.currentColumn = Math.max(0, table.currentColumn - 1)
            Keys.onRightPressed: table.currentColumn = Math.min(table.columns.length - 1, table.currentColumn + 1)
            Keys.onReturnPressed: table.edit(table.selectedId, table.currentColumn)
            Keys.onEnterPressed: table.edit(table.selectedId, table.currentColumn)
            // The plan's window shortcuts (F2, Enter, Delete, R, G, Ctrl+D) are not for a row.
            Keys.onShortcutOverride: event => {
                event.accepted = table.planKeys.indexOf(event.key) >= 0
            }
            Keys.onPressed: event => {
                if (event.key !== Qt.Key_F2) {
                    return
                }
                table.edit(table.selectedId, table.currentColumn)
                event.accepted = true
            }

            delegate: Item {
                id: line
                required property int index
                required property var modelData
                readonly property bool band: modelData.band
                readonly property var shot: modelData.shot
                readonly property bool selected: !band && shot.id === table.selectedId

                width: list.width
                height: band ? table.rowHeight * 0.75 : table.rowHeight

                Rectangle {
                    objectName: line.band ? "" : "rowFill-" + line.shot.id
                    anchors.fill: parent
                    color: line.selected ? KanteStyle.selectionColor : "transparent"
                }
                Rectangle {
                    objectName: line.band ? "" : "rowEdge-" + line.shot.id
                    visible: line.selected
                    width: KanteStyle.edgeWidth
                    height: parent.height
                    color: KanteStyle.focusColor
                }
                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: KanteStyle.borderWidth
                    color: KanteStyle.ruleColor
                }

                // Setup band (shooting order).
                QQC2.Label {
                    objectName: "setupBand"
                    visible: line.band
                    anchors.verticalCenter: parent.verticalCenter
                    x: Kirigami.Units.smallSpacing
                    color: KanteStyle.mutedTextColor
                    font: KanteStyle.labelFont()
                    text: line.band ? table.texts.t("field.setup") + " " + line.modelData.setup : ""
                }

                TapHandler {
                    enabled: !line.band
                    onTapped: {
                        list.forceActiveFocus()
                        table.picked(line.shot.id)
                    }
                }

                Repeater {
                    model: line.band ? 0 : table.columns.length

                    delegate: Item {
                        id: cell
                        required property int index
                        readonly property var col: table.columns[index]
                        readonly property bool editing: table.editId === line.shot.id && table.editColumn === index
                        x: head.columnX(index)
                        width: head.columnWidth(index)
                        height: line.height

                        // The current cell of the selected row: a frame.
                        Rectangle {
                            anchors.fill: parent
                            anchors.margins: KanteStyle.borderWidth
                            visible: line.selected && table.currentColumn === cell.index && list.activeFocus
                            color: "transparent"
                            border.width: KanteStyle.borderWidth
                            border.color: KanteStyle.focusColor
                        }

                        QQC2.Label {
                            objectName: "cell-" + cell.col.key
                            visible: !cell.editing
                            anchors.fill: parent
                            verticalAlignment: Text.AlignVCenter
                            elide: Text.ElideRight
                            color: cell.col.kind === ShotTable.Kind.Fixed ? KanteStyle.mutedTextColor : KanteStyle.textColor
                            text: { table.refused; return table.cellText(line.shot, cell.col.key) }
                        }

                        // A click makes the cell current, a double click edits it.
                        function open() {
                            table.currentColumn = index
                            table.edit(line.shot.id, index)
                        }

                        TapHandler {
                            acceptedButtons: Qt.LeftButton
                            onDoubleTapped: cell.open()
                            onTapped: table.currentColumn = cell.index
                        }

                        Loader {
                            anchors.fill: parent
                            active: cell.editing
                            sourceComponent: cell.col.kind === ShotTable.Kind.Choice ? choiceEditor : textEditor
                            onLoaded: item.start(line.shot, cell.col)
                        }
                    }
                }
            }
        }
    }

    Component {
        id: textEditor

        KanteTextField {
            id: field
            objectName: "cellEditor"
            property var shot: null
            property string key: ""
            property bool dropped: false

            function start(row, col) {
                shot = row
                key = col.key
                text = table.editText(row, col.key)
                forceActiveFocus()
                selectAll()
            }

            inputMethodHints: key === "lens" ? Qt.ImhFormattedNumbersOnly : Qt.ImhNone
            Keys.onEscapePressed: {
                dropped = true
                table.stopEdit()
            }
            onEditingFinished: {
                if (dropped || !shot) {
                    return
                }
                table.commit(shot, key, text, table.editText(shot, key))
            }
        }
    }

    Component {
        id: choiceEditor

        KanteComboBox {
            id: box
            objectName: "cellEditor"
            property var shot: null
            property string key: ""
            readonly property var choices: shot ? table.choicesOf(key) : ({ ids: [], names: [] })

            function start(row, col) {
                shot = row
                key = col.key
                forceActiveFocus()
                popup.open()
            }

            model: choices.names
            currentIndex: shot ? choices.ids.indexOf(table.choiceNow(shot, key)) : -1
            onActivated: index => table.commit(shot, key, choices.ids[index], table.choiceNow(shot, key))

            Connections {
                target: box.popup
                function onClosed() {
                    table.stopEdit()
                }
            }
        }
    }
}
