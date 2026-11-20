import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// DB-60 in the window: find and replace in the screenplay's text by the rules of `script/`, as
// the terminal does (the speaker filter stays the terminal's). Ctrl+F opens it above the text;
// Enter goes to the next hit, Shift+Enter back, Esc closes. The hits are found anew at each
// step and Ersetzen takes the marked hit, so typing between two steps never hits a stale one.
// A replacement is one step of the text's own undo history.
//
//  [Suchen: anna    ] [Ersetzen: Eva  ] (o) Groß-/Kleinschreibung  2 von 5  ‹ ›  Ersetzen  Alle  ✕
RowLayout {
    id: bar

    required property var texts
    required property var controller
    /// The text area searched in.
    required property var area
    /// Shows the stretch `from` to `to` of the text selected and in view.
    required property var showText
    /// Puts `text` in place of `from` to `to` as one undo step, no typing.
    required property var putText
    /// The hits of the last step: `[[start, end], …]`.
    property var hits: []
    /// The hit shown, -1 for none.
    property int current: -1
    readonly property string caseMode: caseSwitch.checked ? "match" : "ignore"
    readonly property alias findField: findField
    readonly property alias replaceField: replaceField
    /// The bar closed: the text takes the keys again.
    signal closed()

    visible: false
    spacing: Kirigami.Units.smallSpacing

    function open() {
        visible = true
        findField.selectAll()
        findField.forceActiveFocus()
    }

    function close() {
        visible = false
        closed()
    }

    function find() {
        hits = findField.text === "" ? [] : JSON.parse(controller.findInScript(area.text, findField.text, caseMode))
    }

    /// The first hit at or after `at`, wrapping to the first.
    function from(at) {
        find()
        const i = hits.findIndex(h => h[0] >= at)
        show(hits.length === 0 ? -1 : Math.max(i, 0))
    }

    function next() {
        from(area.selectionStart === area.selectionEnd ? area.cursorPosition : area.selectionStart + 1)
    }

    /// The last hit before the selection, wrapping to the last.
    function previous() {
        find()
        let i = -1
        hits.forEach((h, n) => { if (h[0] < area.selectionStart) { i = n } })
        show(hits.length === 0 ? -1 : (i < 0 ? hits.length - 1 : i))
    }

    function show(index) {
        current = index
        if (index >= 0) {
            showText(hits[index][0], hits[index][1])
        }
    }

    /// The marked hit becomes the replacement and the next one is shown; without a marked hit
    /// (typing moved it off) only the next one is shown.
    function replaceOne() {
        find()
        const marked = hits.findIndex(h => h[0] === area.selectionStart && h[1] === area.selectionEnd)
        if (marked < 0) {
            next()
            return
        }
        const after = hits[marked][0] + replaceField.text.length
        put(controller.replaceInScript(area.text, findField.text, caseMode, replaceField.text, marked))
        from(after)
    }

    function replaceAll() {
        put(controller.replaceInScript(area.text, findField.text, caseMode, replaceField.text, -1))
        find()
        current = -1
    }

    function put(json) {
        if (json === "") {
            return
        }
        const change = JSON.parse(json)
        putText(change.from, change.to, change.text)
    }

    KanteTextField {
        id: findField
        objectName: "findField"
        Layout.fillWidth: true
        placeholderText: bar.texts.t("search.find")
        Accessible.name: bar.texts.t("search.find")
        onTextEdited: bar.from(bar.area.selectionStart)
        Keys.onReturnPressed: event => (event.modifiers & Qt.ShiftModifier) ? bar.previous() : bar.next()
        Keys.onEnterPressed: event => (event.modifiers & Qt.ShiftModifier) ? bar.previous() : bar.next()
        Keys.onEscapePressed: bar.close()
    }
    KanteTextField {
        id: replaceField
        objectName: "replaceField"
        Layout.fillWidth: true
        placeholderText: bar.texts.t("search.replace")
        Accessible.name: bar.texts.t("search.replace")
        Keys.onReturnPressed: bar.replaceOne()
        Keys.onEnterPressed: bar.replaceOne()
        Keys.onEscapePressed: bar.close()
    }
    KanteSwitch {
        id: caseSwitch
        objectName: "caseSwitch"
        text: bar.texts.t("search.matchCase")
        onToggled: bar.from(bar.area.selectionStart)
    }
    QQC2.Label {
        objectName: "hitCount"
        text: bar.findField.text === "" ? ""
            : bar.hits.length === 0 ? bar.texts.t("search.none")
            : bar.current < 0 ? bar.texts.t("search.total", { total: bar.hits.length })
            : bar.texts.t("search.count", { current: bar.current + 1, total: bar.hits.length })
    }
    KanteToolButton {
        objectName: "previousHit"
        icon.name: "go-up"
        text: bar.texts.t("search.previous")
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: text
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        onClicked: bar.previous()
    }
    KanteToolButton {
        objectName: "nextHit"
        icon.name: "go-down"
        text: bar.texts.t("search.next")
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: text
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        onClicked: bar.next()
    }
    KanteButton {
        objectName: "replaceOne"
        text: bar.texts.t("search.replaceOne")
        enabled: bar.current >= 0
        onClicked: bar.replaceOne()
    }
    KanteButton {
        objectName: "replaceAll"
        text: bar.texts.t("search.replaceAll")
        enabled: bar.hits.length > 0
        onClicked: bar.replaceAll()
    }
    KanteToolButton {
        objectName: "closeSearch"
        icon.name: "window-close"
        text: bar.texts.t("search.close")
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: text
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        onClicked: bar.close()
    }
}
