import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// DB-03: the choices under the line being typed in the flowing text (D-27). A card
// below the cursor line, over the text: nothing moves. The keys stay with the text area (Up, Down,
// Esc in `ScreenplayView`, Enter and Tab in `ScriptFlow`); a click takes a choice too.
//
//   INT. SA▌
//   ┌─────────────────┐
//   │ SAAL - ▌        │  ← marked
//   │ SALON - ▌       │
//   └─────────────────┘
QQC2.Popup {
    id: pop

    required property var flow
    required property var area
    required property var doc
    required property var texts

    readonly property var options: flow.suggestions ? flow.suggestions.options : []
    // Under the cursor's line, at the cursor.
    readonly property rect caret: doc.rectOf(doc.cursor)

    parent: area
    x: caret.x
    y: caret.y + caret.height
    width: Kirigami.Units.gridUnit * 16
    padding: Kirigami.Units.smallSpacing
    margins: 0
    // The text area keeps the keys and the focus.
    focus: false
    closePolicy: QQC2.Popup.NoAutoClose

    onOptionsChanged: {
        if (options.length > 0) {
            open()
            return
        }
        close()
    }

    KantePopupSkin { popup: pop }

    contentItem: Column {
        Accessible.role: Accessible.List
        Accessible.name: pop.texts.t("screenplay.suggestions")
        Repeater {
            model: pop.options
            delegate: KanteListRow {
                required property string modelData
                required property int index
                objectName: "suggestion." + index
                width: pop.availableWidth
                density: KanteListRow.Density.Compact
                rule: false
                focusOnClick: false
                text: modelData
                selected: index === pop.flow.suggestSel
                onClicked: {
                    pop.flow.suggestSel = index
                    pop.flow.acceptSuggest()
                }
            }
        }
    }
}
