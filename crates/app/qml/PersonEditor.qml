import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-26: one person of the team. Name, department, function and agency, and the contact: phone,
// mail and address (one line each). Without an `item` the form makes a new person: name and
// department only, the rest is changed afterwards. Deleting asks first and says that the castings
// and responsibilities of the person go too.
EditorPane {
    id: form

    /// The department names in the catalogue's order.
    property var departments: []

    title: texts.t(item ? "team.edit" : "team.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: nameField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.person", { name: item.name }) : ""

    // The department the box shows; "" for none (the first entry).
    function chosenDepartment() {
        return departmentBox.currentIndex <= 0 ? "" : departments[departmentBox.currentIndex - 1]
    }

    function load() {
        nameField.text = item ? item.name : ""
        departmentBox.currentIndex = item && item.department ? departments.indexOf(item.department) + 1 : 0
        functionField.text = item ? item.function : ""
        agencyField.text = item ? item.agency : ""
        phoneField.text = item ? item.phone : ""
        mailField.text = item ? item.mail : ""
        addressField.text = item ? item.address : ""
    }

    onItemChanged: load()
    onDepartmentsChanged: load()

    // One edit for the whole form, only when something changed: one undo step.
    submit: function () {
        if (!item) {
            return controller.addPerson(nameField.text, chosenDepartment())
        }
        const changed = nameField.text !== item.name || chosenDepartment() !== (item.department || "")
            || functionField.text !== item.function || agencyField.text !== item.agency
            || phoneField.text !== item.phone || mailField.text !== item.mail
            || addressField.text !== item.address
        return !changed || controller.editPerson(item.id, nameField.text, chosenDepartment(),
                                                 functionField.text, agencyField.text,
                                                 phoneField.text, mailField.text, addressField.text)
    }
    remove: function () { return controller.removePerson(item.id) }

    KanteSectionLabel { text: form.texts.t("field.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("field.name")
    }

    KanteSectionLabel { text: form.texts.t("team.department") }
    KanteComboBox {
        id: departmentBox
        objectName: "departmentBox"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("team.department")
        model: [form.texts.t("team.noDepartment")].concat(form.departments.map(d => form.texts.t("department." + d)))
    }

    KanteSectionLabel { visible: form.item !== null; text: form.texts.t("team.function") }
    KanteTextField {
        id: functionField
        objectName: "functionField"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("team.function")
    }

    KanteSectionLabel { visible: form.item !== null; text: form.texts.t("team.agency") }
    KanteTextField {
        id: agencyField
        objectName: "agencyField"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("team.agency")
    }

    KanteSectionLabel { visible: form.item !== null; text: form.texts.t("team.phone") }
    KanteTextField {
        id: phoneField
        objectName: "phoneField"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("team.phone")
    }

    KanteSectionLabel { visible: form.item !== null; text: form.texts.t("team.mail") }
    KanteTextField {
        id: mailField
        objectName: "mailField"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("team.mail")
    }

    KanteSectionLabel { visible: form.item !== null; text: form.texts.t("team.address") }
    KanteTextField {
        id: addressField
        objectName: "addressField"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("team.address")
    }
}
