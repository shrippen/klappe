import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The pages of the window judge names as the session does: the same place name, the same
// character name and the name made of capitals come from the controller, not from the pages.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }

    PlaceEditor {
        id: place
        texts: texts
        controller: ctl
    }
    CharacterEditor {
        id: figure
        texts: texts
        controller: ctl
        characters: []
    }
    PlanNotShot {
        id: notShot
        texts: texts
        controller: ctl
        locations: []
        place: null
        notShot: ({ place: "", place_name: "CHARLOTTENBURG-NORD" })
    }

    TextEdit { id: text }
    ScriptDoc { id: seam; area: text; controller: ctl }
    ScriptSearch {
        id: search
        texts: texts
        controller: ctl
        doc: seam
        showText: () => {}
        putText: () => {}
    }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The child of `item` called `name`, at any depth.
    function child(item, name) {
        for (const c of item.children) {
            const found = c.objectName === name ? c : child(c, name)
            if (found) {
                return found
            }
        }
        return null
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            // Two spaces and the capital sharp s do not make another variant.
            place.variants = ["Hafen Nord", "Straße"]
            if (!place.hasVariant("HAFEN  NORD") || !place.hasVariant("STRAẞE")) {
                return root.fail("the place form takes a spelling the session calls the same")
            }
            if (place.hasVariant("Hafen Süd")) {
                return root.fail("the place form refuses another spelling")
            }

            figure.aliases = ["Frau Brandes", "Straße"]
            if (!figure.hasAlias("FRAU BRANDES") || figure.hasAlias("Brandes")) {
                return root.fail("the character form compares names on its own")
            }
            // `ẞ` agrees with `SS` and `ß` for the session, which upper-casing alone does not.
            if (!figure.hasAlias("STRAẞE")) {
                return root.fail("the character form does not equate the capital sharp s")
            }

            // The search finds the chosen speaker again when the cue is spelled the other way.
            text.text = "@Straße\nHallo\n"
            search.refreshSpeakers()
            search.speakers = ["Straße", "Anna"]
            child(search, "speakerBox").currentIndex = 1
            text.text = "@STRASSE\nHallo\n\n@Anna\nHi\n"
            search.refreshSpeakers()
            if (search.speaker !== "STRASSE") {
                return root.fail("the search lost the speaker " + JSON.stringify(search.speaker))
            }

            // The window shows the name the session would create.
            const asked = []
            notShot.newLocationRequested.connect((name, at) => asked.push(name))
            child(notShot, "newLocation").clicked()
            if (asked.length !== 1 || asked[0] !== "Charlottenburg-Nord") {
                return root.fail("the empty page asks for " + JSON.stringify(asked))
            }

            Qt.exit(0)
        }
    }
}
