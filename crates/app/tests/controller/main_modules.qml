import QtQuick
import "lib"

// D-11, D-12: the window opens in Stellprobe; the bar, the header's module request, the jump of
// DB-63 and the empty-state button of the script column switch the module.
Item {
    Watchdog {}

    readonly property url main: "qrc:/qt/qml/io/github/shrippen/klappe/qml/Main.qml"

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function find(item, name) {
        if (item.objectName === name) {
            return item
        }
        for (const child of item.children) {
            const found = find(child, name)
            if (found) {
                return found
            }
        }
        return null
    }

    Component.onCompleted: {
        const component = Qt.createComponent(main)
        const window = component.status === Component.Ready ? component.createObject(null) : null
        if (!window) {
            return fail("Main.qml is not made: " + component.errorString())
        }
        const header = find(window.contentItem, "header")
        const workspace = find(window.contentItem, "workspace")
        if (!header || !workspace) {
            return fail("header or workspace not found")
        }

        // The module of an earlier run is kept in the settings: start from a known one.
        header.moduleRequested("stellprobe")
        if (window.module !== "stellprobe" || header.module !== "stellprobe") {
            return fail("not Stellprobe: " + window.module)
        }
        if (window.modules.map(m => m.id).join() !== "screenplay,stellprobe") {
            return fail("modules: " + JSON.stringify(window.modules))
        }

        header.moduleRequested("screenplay")
        if (window.module !== "screenplay" || header.module !== "screenplay") {
            return fail("module request: " + window.module)
        }

        // A jump to the plan or the script (DB-63) goes to Stellprobe.
        header.planRequested()
        if (window.module !== "stellprobe") {
            return fail("plan jump: " + window.module)
        }
        header.moduleRequested("screenplay")
        header.scriptRequested(-1)
        if (window.module !== "stellprobe") {
            return fail("script jump: " + window.module)
        }

        // "Open screenplay" in the script column.
        workspace.screenplayRequested()
        if (window.module !== "screenplay") {
            return fail("open screenplay: " + window.module)
        }

        header.moduleRequested("stellprobe")
        window.destroy()
        Qt.exit(0)
    }
}
