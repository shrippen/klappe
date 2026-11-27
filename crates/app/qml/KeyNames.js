.pragma library

// "Ctrl+Shift+Z" as the keyboard of the language names it (`texts` is the catalogue, Texts.qml).
function name(texts, sequence) {
    return sequence.replace("Ctrl", texts.t("key.ctrl")).replace("Shift", texts.t("key.shift"))
        .replace("Delete", texts.t("key.delete")).replace("Space", texts.t("key.space"))
}
