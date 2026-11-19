import QtQuick
import "lib"

// D-09: the width tiers are the window's width. The module bar (D-11) takes some of it, so the
// work area at a 1440 px window is narrower than 1400 and must still be the wide step: the
// script column and the shot panel beside the plan, not drawers.
Item {
    Watchdog {}

    readonly property url main: "qrc:/qt/qml/io/github/shrippen/klappe/qml/Main.qml"
    readonly property url example: Qt.resolvedUrl("../../../../testdata/beispiel.klappe")

    property var window: null
    property var workspace: null

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

    // A window of `width` px: the tier the work area reports, after the layout settled.
    function check(width, tier, then) {
        window.width = width
        settle.callback = () => {
            if (workspace.tier !== tier) {
                return fail("window " + width + " px: tier " + workspace.tier + ", wanted " + tier
                            + " (work area " + workspace.width + " px)")
            }
            then()
        }
        settle.restart()
    }

    Timer {
        id: settle
        property var callback: null
        interval: 300
        onTriggered: callback()
    }

    Component.onCompleted: {
        const component = Qt.createComponent(main)
        window = component.status === Component.Ready ? component.createObject(null) : null
        if (!window) {
            return fail("Main.qml is not made: " + component.errorString())
        }
        window.show()
        const header = find(window.contentItem, "header")
        workspace = find(window.contentItem, "workspace")
        if (!header || !workspace) {
            return fail("header or workspace not found")
        }
        header.moduleRequested("stellprobe")
        if (!header.controller.openProject(example)) {
            return fail("project did not open")
        }

        // 0 = Wide, 1 = Medium, 2 = Narrow (Workspace.Tier).
        check(1440, 0, () => check(1400, 0, () => check(1399, 1, () => check(1000, 1, () => check(999, 2, () => {
            window.destroy()
            Qt.exit(0)
        })))))
    }
}
