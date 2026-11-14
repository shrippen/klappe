import "../../qml"

// The real catalogue of the app (crates/app/i18n), read synchronously; needs
// QML_XHR_ALLOW_FILE_READ=1 (scripts/check-qml.sh sets it).
Texts {
    property string language: "de"

    source: {
        const request = new XMLHttpRequest()
        request.open("GET", Qt.resolvedUrl("../../i18n/" + language + ".json"), false)
        request.send()
        return request.responseText
    }
}
