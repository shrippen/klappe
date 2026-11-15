import QtQuick
import "lib"

// The window the app opens comes from the built module, not from the files the other tests
// load: a page missing from `PAGES` in build.rs fails only here (and in the screenshots).
Item {
    Watchdog {}

    readonly property url main: "qrc:/qt/qml/io/github/shrippen/klappe/qml/Main.qml"

    Component.onCompleted: {
        const component = Qt.createComponent(main, Component.PreventCreation)
        if (component.status !== Component.Ready) {
            console.error("Main.qml does not compile: " + component.errorString())
            Qt.exit(1)
            return
        }
        Qt.exit(0)
    }
}
