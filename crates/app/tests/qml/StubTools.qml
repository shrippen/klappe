import QtQuick

// Stands in for `ScriptTools` (the tests do not know native types): keeps what it was given.
Item {
    property var document: null
    property color mutedColor
    property bool canUndo: false
    property bool canRedo: false
    property string kinds: ""
    property var looks: ({})
    property int forgets: 0
    property int undos: 0

    function setLooks(kinds, looks) {
        this.kinds = kinds
        this.looks = looks
    }

    function faceOf(line) {
        return ""
    }

    function forget() {
        forgets += 1
    }

    function undo() {
        undos += 1
        return -1
    }

    function redo() {
        return -1
    }
}
