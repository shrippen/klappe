import QtQuick

// Ends a scenario that hangs: a failure, not a stuck test run. Generous, so a busy machine
// (debug build, 3D pictures) does not read as a hang: 10 s timed out under load.
Timer {
    interval: 60000
    running: true
    onTriggered: {
        console.error("scenario timed out")
        Qt.exit(1)
    }
}
