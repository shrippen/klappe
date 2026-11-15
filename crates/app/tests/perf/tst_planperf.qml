import QtQuick
import QtTest
import "../../qml/Kante"

// NF-05: one frame of KanteShapeCanvas with the shapes of 50 figures, 10 cameras and 300
// objects. Run by `make perf`, not `make qml`: a timing needs a quiet machine. The numbers are
// in planung/STAND.md.
TestCase {
    id: tc
    name: "PlanPerf"
    width: 800
    height: 800
    when: windowShown
    visible: true

    readonly property real setM: 40
    readonly property int frames: 20
    // Four times the 16 ms budget of NF-05.
    readonly property int limitMs: 64

    function at(n) {
        return [(n * 37 % 997) / 997 * (setM - 2) + 1, (n * 91 % 991) / 991 * (setM - 2) + 1]
    }

    // The kinds the display list makes: objects as polygons, figures and marks as circles
    // with a caption, ways as polylines, camera views as polygons.
    function bigPlan() {
        const shapes = []
        for (let i = 0; i < 300; ++i) {
            const [x, y] = at(i + 2000)
            shapes.push({ kind: "polygon", points: [x, y, x + 1.2, y, x + 1.2, y + 0.8, x, y + 0.8],
                          label: "muted", fill: "tint", key: "object:o" + i })
        }
        for (let i = 0; i < 50; ++i) {
            let way = []
            for (let k = 0; k < 4; ++k) {
                const [x, y] = at(i * 7 + k)
                way.push(x, y)
                shapes.push({ kind: "circle", points: [x, y], radius: 0.22, label: "red",
                              fill: "none", text: String(k + 1), key: "mark:c" + i + ":" + k })
            }
            shapes.push({ kind: "polyline", points: way, label: "red", stroke: "dashed", key: "way:c" + i })
            shapes.push({ kind: "circle", points: way.slice(0, 2), radius: 0.4, label: "red",
                          fill: "solid", text: "F" + i, key: "figure:c" + i })
        }
        for (let i = 0; i < 10; ++i) {
            const [x, y] = at(i + 1000)
            shapes.push({ kind: "polygon", points: [x, y, x + 6, y - 2, x + 6, y + 2],
                          label: "accent", fill: "tint", text: "A", key: "camera:cam" + i })
        }
        return shapes
    }

    KanteShapeCanvas {
        id: canvas
        width: 800
        height: 800
        contentWidth: tc.setM
        contentHeight: tc.setM
        gridStep: 1
    }

    // Median milliseconds to set `shapes` and have the canvas paint and render them.
    function frameMs(shapes) {
        const times = []
        canvas.shapes = shapes
        grabImage(canvas)

        for (let i = 0; i < frames; ++i) {
            // A new array each time, as every redraw of the app hands one over.
            const start = Date.now()
            canvas.shapes = shapes.slice()
            grabImage(canvas)
            times.push(Date.now() - start)
        }
        times.sort((a, b) => a - b)
        return times[Math.floor(frames / 2)]
    }

    function test_a_frame_of_the_large_plan_is_fast() {
        const plan = bigPlan()
        // An empty plan still costs the grid and the grab: what a frame has at least.
        const floor = frameMs([])
        const median = frameMs(plan)

        console.log("perf canvas frame:", plan.length, "shapes,", median, "ms (median); empty plan", floor, "ms")
        verify(median < limitMs, "frame took " + median + " ms, limit " + limitMs)
    }
}
