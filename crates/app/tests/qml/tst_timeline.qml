import QtQuick
import QtTest
import "../../qml"

// Timeline: texts come from the catalogue, numbers use its decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "Timeline"
    width: 900
    height: 300
    when: windowShown

    QtObject {
        id: fake
        property string tracksJson: "[]"
        property string layersJson: "{}"
        property real time: 6
        property real duration: 20
        property var seeks: []

        function seek(at) { seeks.push(at) }
        function setLayer() {}
        function setTrack() {}
        function selectShot() {}
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: timelineComponent
        Timeline {
            width: 900
            height: 300
            controller: fake
        }
    }

    function make(texts) {
        fake.seeks = []
        const line = createTemporaryObject(timelineComponent, tc, { texts: texts })
        verify(line)
        return line
    }

    function test_the_clock_uses_the_german_comma() {
        compare(findChild(make(de), "clock").text, "6,0 / 20,0 s")
    }

    function test_the_clock_uses_the_english_point() {
        compare(findChild(make(en), "clock").text, "6.0 / 20.0 s")
    }

    function test_the_start_button_has_a_readable_name() {
        const line = make(de)
        const start = findChild(line, "startButton")

        compare(start.text, de.t("timeline.start"))
        compare(start.Accessible.name, de.t("timeline.start"))

        start.clicked()
        compare(fake.seeks, [0])
    }
}
