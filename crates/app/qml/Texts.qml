import QtQuick

// German/English texts from the Rust catalogues (crates/app/i18n). t("key", {name: value}).
QtObject {
    property string source: "{}"
    readonly property var map: JSON.parse(source)

    function t(key, args) {
        let text = map[key] !== undefined ? map[key] : key
        if (args) {
            for (const name in args) {
                text = text.split("{" + name + "}").join(String(args[name]))
            }
        }
        return text
    }
}
