import QtQuick

// The system's speech voice (LH-64): Qt TextToSpeech, made when the module and an engine are
// there. The module is made from text, not imported here, so a system without
// QtTextToSpeech still starts (`available` is false) and the build needs nothing of it.
//   say(text, language)   reads `text` after what is being read; `language` is "de" or "en"
//   stop()                stops reading and drops what waits
// Tests hand the window a fake with the same three members.
QtObject {
    id: voice

    readonly property bool available: engine !== null && engine.available
    property var engine: null

    // The engines are tried in Qt's order for one that starts and has a voice: the default
    // (often speechd) fails where its service does not run. "mock" makes no sound.
    readonly property string engineSource: `
        import QtQuick
        import QtTextToSpeech
        Item {
            id: speaker
            property bool available: false
            TextToSpeech { id: tts }

            Component.onCompleted: {
                for (const name of tts.availableEngines()) {
                    if (name === "mock") {
                        continue
                    }
                    tts.engine = name
                    if (tts.state !== TextToSpeech.Error && tts.availableLocales().length > 0) {
                        available = true
                        return
                    }
                }
            }

            // The first voice of the language; none: the engine's own.
            function say(text, language) {
                const match = tts.availableLocales().find(l => l.name.indexOf(language) === 0)
                if (match && tts.locale.name !== match.name) {
                    tts.locale = match
                }
                tts.enqueue(text)
            }
            function stop() { tts.stop() }
        }`

    function say(text, language) {
        if (engine) {
            engine.say(text, language)
        }
    }

    function stop() {
        if (engine) {
            engine.stop()
        }
    }

    Component.onCompleted: {
        try {
            engine = Qt.createQmlObject(engineSource, voice, "speechEngine")
        } catch (e) {
            engine = null
        }
    }
}
