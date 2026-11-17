import QtQuick
import "count.js" as Count

// Stands in for the Rust element `CameraPicture` and counts the pictures it is shown, to see
// when a tile is painted again.
Item {
    function show(image) {
        Count.shows += 1
    }
}
