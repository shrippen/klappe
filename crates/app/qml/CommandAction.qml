import QtQuick.Controls as QQC2

// An action that the command palette lists too: the menus, the buttons and the palette run the
// same action, so no logic is written twice.
QQC2.Action {
    /// Id the palette hands back.
    property string cmd: ""
    /// Heading in the palette: the suffix of the key `palette.group.<group>`.
    property string group: ""
    /// The shortcut as the palette shows it ("Strg+K"); empty for none.
    property string keys: ""
}
