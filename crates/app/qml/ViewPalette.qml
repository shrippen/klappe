import QtQuick
import Kante

// Kante's colours as the theme has them now, for the controller to colour the roles of a
// picture (the camera view's and the 3D view's): `{ name: [r, g, b] }` as JSON.
QtObject {
    readonly property string json: JSON.stringify(palette())

    function rgb(c) {
        return [Math.round(c.r * 255), Math.round(c.g * 255), Math.round(c.b * 255)]
    }

    function palette() {
        const colors = { ground: rgb(KanteStyle.backgroundColor),
                         text: rgb(KanteStyle.textColor),
                         muted: rgb(KanteStyle.mutedTextColor),
                         accent: rgb(KanteStyle.accentColor) }
        for (const name of KanteStyle.labels) {
            colors[name] = rgb(KanteStyle.labelColor(name))
        }
        return colors
    }
}
