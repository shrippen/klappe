import QtQuick
import Kante

// The colour of a revision (DB-41) as it is drawn: the file's name of the colour (`blue`) to a
// Kante colour. Kante has no revision colours; until it has, the nearest label colour stands in
// (white: the strong text colour, buff: the muted one). The colour's name is always shown beside
// it, so two colours that share a stand-in stay apart.
QtObject {
    function color(name) {
        switch (name) {
        case "white": return KanteStyle.strongTextColor
        case "blue": return KanteStyle.labelColor("blue")
        case "pink": return KanteStyle.labelColor("purple")
        case "yellow": return KanteStyle.labelColor("yellow")
        case "green": return KanteStyle.labelColor("green")
        case "goldenrod": return KanteStyle.labelColor("orange")
        case "buff": return KanteStyle.mutedTextColor
        case "salmon":
        case "cherry": return KanteStyle.labelColor("red")
        }
        return KanteStyle.textColor
    }
}
