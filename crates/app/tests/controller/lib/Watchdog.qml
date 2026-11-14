import QtQuick

// Ends a scenario that hangs: a failure, not a stuck test run.
Timer {
    interval: 10000
    running: true
    onTriggered: {
        console.error("scenario timed out")
        Qt.exit(1)
    }
}
