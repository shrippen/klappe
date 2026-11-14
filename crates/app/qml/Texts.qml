import QtQuick

// German/English texts from the Rust catalogues (crates/services/i18n). t("key", {name: value}).
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

    // `value` with the catalogue's decimal separator (unit.decimal): "39,6" in German.
    // fixed keeps `digits` decimals ("6,0"), num drops trailing zeros ("35", "1,6").
    function fixed(value, digits) {
        return Number(value).toFixed(digits).replace(".", t("unit.decimal"))
    }

    function num(value, digits) {
        return String(Number(Number(value).toFixed(digits))).replace(".", t("unit.decimal"))
    }
}
