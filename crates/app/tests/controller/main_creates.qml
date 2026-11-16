import QtQuick
import "lib"

// The window the app opens is built, not only compiled: a page of the module that shadows a Qt
// type of the same name (e.g. `FolderDialog` from QtQuick.Dialogs) fails only when the window
// is made, with required properties left empty.
Item {
    Watchdog {}

    readonly property url main: "qrc:/qt/qml/io/github/shrippen/klappe/qml/Main.qml"

    Component.onCompleted: {
        const component = Qt.createComponent(main)
        const window = component.status === Component.Ready ? component.createObject(null) : null
        if (!window) {
            console.error("Main.qml is not made: " + component.errorString())
            Qt.exit(1)
            return
        }
        window.destroy()
        Qt.exit(0)
    }
}
