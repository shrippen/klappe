import QtQuick

// Play or pause, the jumps between marks and reading the dialogue aloud (LH-64), as commands:
// menu, palette and keys. The window does the playing (`requested`) and keeps the choice to
// read (`speakRequested`).
Item {
    id: playing

    required property var texts
    required property var controller
    property bool locked: false
    /// The system has a speech voice.
    property bool speechAvailable: false
    property var keyName: sequence => sequence

    signal requested()
    signal speakRequested()

    /// In the order of the palette.
    readonly property var actions: [actPlay, actMarkPrev, actMarkNext, actSpeak]

    visible: false
    width: 0
    height: 0

    // Play or pause: K works everywhere. Space does too, except where it already means
    // something: the plan pans with Space held (Kante), a button presses with it, a text field
    // types it. A text field keeps its keys anyway: it takes the key before a shortcut.
    CommandAction {
        id: actPlay
        cmd: "playback.toggle"
        group: "playback"
        text: playing.texts.t("menu.play")
        shortcut: "K"
        keys: playing.keyName("Space") + " / K"
        enabled: playing.controller.loaded && !playing.locked
        onTriggered: playing.requested()
    }
    readonly property Item focused: Window.window ? Window.window.activeFocusItem : null
    Shortcut {
        sequence: "Space"
        enabled: actPlay.enabled && !(playing.focused && playing.focused.down !== undefined)
            && !(playing.focused && playing.focused.objectName === "planCanvas")
        onActivated: actPlay.trigger()
    }

    // Marks: the playhead jumps to the previous or next mark of the shot.
    CommandAction {
        id: actMarkPrev
        cmd: "playback.markPrev"
        group: "playback"
        text: playing.texts.t("menu.markPrev")
        shortcut: ","
        keys: ","
        enabled: playing.controller.loaded && !playing.locked
        onTriggered: playing.controller.stepMark(-1)
    }
    CommandAction {
        id: actMarkNext
        cmd: "playback.markNext"
        group: "playback"
        text: playing.texts.t("menu.markNext")
        shortcut: "."
        keys: "."
        enabled: playing.controller.loaded && !playing.locked
        onTriggered: playing.controller.stepMark(1)
    }

    // Dialogue aloud on or off. Shift+V: V is the select tool.
    CommandAction {
        id: actSpeak
        cmd: "playback.speak"
        group: "playback"
        text: playing.texts.t("menu.speak")
        shortcut: "Shift+V"
        keys: playing.keyName("Shift+V")
        enabled: playing.controller.loaded && !playing.locked && playing.speechAvailable
        onTriggered: playing.speakRequested()
    }
}
