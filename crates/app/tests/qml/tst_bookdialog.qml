import QtQuick
import QtTest
import "../../qml"

// BookDialog: asks the open questions about the book one at a time, sends the answer by the
// answer's position (the last one is "not in the book"), goes on to the next question and closes after
// the last. Close leaves the questions open.
TestCase {
    id: tc
    name: "BookDialog"
    width: 900
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string projectJson: ""
        property var pending: []
        property var places: []
        property var calls: []

        // Words the questions as the session does (`book_questions`, tested in services): the
        // dialog only shows them.
        function show() {
            const scenes = pending.map(q => ({
                kind: "scene", key: q.id, title: de.t("book.title"),
                ask: de.t("book.ask", { number: q.number, heading: q.heading }),
                answers: q.answers.map(a => ({ number: a.number, text: a.none ? de.t("book.none") : a.heading }))
            }))
            const keys = { create: "book.placeNew", rename: "book.placeRename", variant: "book.placeVariant" }
            const named = places.map(q => ({
                kind: "place", key: q.name, title: de.t("book.placeTitle"),
                ask: de.t("book.placeAsk", { name: q.name, count: q.headings }),
                answers: q.answers.map(a => ({ number: "", text: de.t(keys[a.kind], { name: q.name, old: a.name, new: q.name }) }))
            }))
            projectJson = JSON.stringify({ scenes: [], questions: scenes.concat(named) })
        }
        // Answered: the question is gone, as the session has it.
        function answerQuestion(kind, key, index) {
            calls.push([key, index])
            if (kind === "place") {
                places = places.filter(p => p.name !== key)
            } else {
                pending = pending.filter(p => p.id !== key)
            }
            show()
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: dialogComponent
        BookDialog { texts: de; controller: fake }
    }

    function init() {
        fake.calls = []
        fake.pending = [
            { id: "a", number: "4", heading: "INT. KELLER – NACHT", answers: [
                { number: "1", heading: "INT. HOF – NACHT" }, { number: "2", heading: "INT. KELLER – NACHT" },
                { number: "", heading: "", none: true }] },
            { id: "b", number: "5", heading: "INT. HOF – NACHT", answers: [
                { number: "1", heading: "INT. HOF – NACHT" }, { number: "", heading: "", none: true }] }
        ]
        fake.places = []
        fake.show()
    }

    function placeQuestions() {
        fake.places = [
            { name: "HOF", headings: 2, answers: [
                { kind: "create", name: "" }, { kind: "rename", name: "SÄNGERSAAL" }, { kind: "variant", name: "SÄNGERSAAL" }] }
        ]
        fake.show()
    }

    function make() {
        const book = createTemporaryObject(dialogComponent, tc)
        verify(book)
        return book
    }

    function test_the_first_question_lists_the_candidates_and_not_in_the_book() {
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        compare(findChild(book, "ask").text, "Eintrag 4 INT. KELLER – NACHT — welche Szene des Buchs ist das?")
        compare(book.options.map(o => o.text), ["INT. HOF – NACHT", "INT. KELLER – NACHT", "Nicht im Buch"])
        compare(book.options.map(o => o.index), [0, 1, 2])
    }

    function test_an_answer_goes_to_the_controller_and_the_next_question_follows() {
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        book.answer(1)

        compare(fake.calls, [["a", 1]])
        compare(findChild(book, "ask").text, "Eintrag 5 INT. HOF – NACHT — welche Szene des Buchs ist das?")
        verify(book.dialog.opened)
    }

    function test_not_in_the_book_is_the_last_answer_and_the_last_answer_closes() {
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        book.answer(0)
        book.answer(1)

        compare(fake.calls, [["a", 0], ["b", 1]])
        tryVerify(() => !book.dialog.visible)
    }

    function test_enter_answers_the_chosen_row() {
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)
        const rows = findChild(book, "rows")

        keyClick(Qt.Key_Down)
        keyClick(Qt.Key_Return)

        compare(rows.currentIndex, 0, "the next question starts at the top")
        compare(fake.calls, [["a", 1]])
    }

    function test_escape_keeps_the_questions() {
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        keyClick(Qt.Key_Escape)

        tryVerify(() => !book.dialog.visible)
        compare(fake.calls, [])
        compare(book.questions.length, 2)
    }

    function test_without_questions_it_does_not_open() {
        fake.pending = []
        fake.show()
        const book = make()

        book.open()

        verify(!book.dialog.visible)
    }

    function test_the_places_come_after_the_scenes() {
        placeQuestions()
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        compare(book.questions.map(q => q.kind), ["scene", "scene", "place"])
        book.answer(0)
        book.answer(0)

        compare(findChild(book, "ask").text, "HOF, in 2 Szenenkopf/-köpfen — welcher Spielort ist das?")
        compare(book.dialog.title, "Spielort der Szenenköpfe")
    }

    function test_a_place_question_lists_new_rename_and_variant() {
        fake.pending = []
        placeQuestions()
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        compare(book.options.map(o => o.text), ["Neuer Spielort HOF", "SÄNGERSAAL heißt jetzt HOF", "Schreibweise von SÄNGERSAAL"])
        compare(findChild(book, "rows").Accessible.name, findChild(book, "ask").text)
    }

    function test_a_place_answer_goes_to_the_controller_by_name_and_position_and_the_last_closes() {
        fake.pending = []
        placeQuestions()
        const book = make()
        book.open()
        tryVerify(() => book.dialog.opened)

        keyClick(Qt.Key_Down)
        keyClick(Qt.Key_Return)

        compare(fake.calls, [["HOF", 1]])
        tryVerify(() => !book.dialog.visible)
    }

}
