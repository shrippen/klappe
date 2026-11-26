import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// "Fassung ▾" in the screenplay's head (DB-40 to DB-42): lock a version, compare, show the
// revisions, and the list of the locked versions with a way to delete each. A row of the list
// compares that version with the book now.
//
//  Fassung ▾
//    Fassung sperren …
//    Vergleichen …
//    [x] Revisionen zeigen
//    ── Gesperrte Fassungen ──
//    Drehfassung        ■ Weiß   2026-11-20  Löschen …
//    Nachdreh           ■ Blau   2026-11-25  Löschen …
HeaderMenu {
    id: chooser

    required property var texts
    /// The locked versions, `{ id, name, date, color, colorName }`, in the order they were locked.
    property var versions: []
    property bool showRevisions: true

    /// "Lock version …" was chosen.
    signal lockRequested()
    /// "Compare …" or a row was chosen; `from` is a version's id, empty for none.
    signal compareRequested(string from)
    /// "Delete …" of a row was chosen.
    signal removeRequested(var version)
    signal revisionsToggled(bool shown)

    text: texts.t("version.menu")
    Accessible.name: texts.t("version.menu")

    RevisionInk { id: revisionInk }

    KanteMenuItem {
        objectName: "lockVersion"
        text: chooser.texts.t("version.lock")
        onTriggered: chooser.lockRequested()
    }
    KanteMenuItem {
        objectName: "compareVersions"
        text: chooser.texts.t("version.compare")
        enabled: chooser.versions.length > 0
        onTriggered: chooser.compareRequested("")
    }
    KanteMenuItem {
        objectName: "showRevisions"
        text: chooser.texts.t("version.showRevisions")
        checkable: true
        checked: chooser.showRevisions
        enabled: chooser.versions.length > 0
        onTriggered: chooser.revisionsToggled(!chooser.showRevisions)
    }
    KanteMenuSection {
        objectName: "versionList"
        visible: chooser.versions.length > 0
        text: chooser.texts.t("version.list")
    }

    Instantiator {
        model: chooser.versions
        delegate: KanteMenuItem {
            id: row
            required property var modelData
            objectName: "version." + modelData.id
            text: modelData.name
            valueText: modelData.date
            Accessible.name: chooser.texts.t("version.row", {
                name: modelData.name, color: modelData.colorName, date: modelData.date })
            onTriggered: chooser.compareRequested(modelData.id)
            Keys.onDeletePressed: chooser.removeRequested(modelData)

            trailing: [
                KanteSwatch {
                    objectName: "swatch." + row.modelData.id
                    size: KanteSwatch.Size.Dense
                    swatchColor: revisionInk.color(row.modelData.color)
                },
                QQC2.Label {
                    text: row.modelData.colorName
                    color: row.ink
                    font: KanteStyle.smallFont
                },
                KanteToolButton {
                    objectName: "removeVersion." + row.modelData.id
                    text: chooser.texts.t("version.remove")
                    onClicked: {
                        chooser.menu.close()
                        chooser.removeRequested(row.modelData)
                    }
                }
            ]
        }
        onObjectAdded: (index, object) => chooser.menu.addItem(object)
        onObjectRemoved: (index, object) => chooser.menu.removeItem(object)
    }
}
