import QtQuick

// Stands in for the Rust element `CameraPicture` (the tests do not know Rust types): keeps
// the last image it was shown.
Item {
    property var shown: null
    property int shows: 0

    function show(image) {
        shown = image
        shows += 1
    }
}
