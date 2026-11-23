import QtQuick
import Kante

// A breakdown element (D-22) as a chip; the colour of its square is its status: open is muted,
// in progress warns, ready is positive. The status is also in the tooltip, so colour is not
// the only sign. `compact` is the square alone (a list row's marker).
KanteChip {
    id: chip

    /// `open`, `working` or `ready`.
    property string status: "open"

    chipColor: status === "ready" ? KanteStyle.positiveTextColor
        : status === "working" ? KanteStyle.warningColor : KanteStyle.mutedTextColor
}
