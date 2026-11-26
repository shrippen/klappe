import QtQuick
import Kante

// The colour of a revision (DB-41) as it is drawn: the file's name of the colour (`blue`) to
// Kante's revision colour. `RevisionColor::name` and `KanteStyle.revisions` use the same names,
// so this is the one place that maps them. The colour's name is shown beside it all the same.
QtObject {
    function color(name) {
        return KanteStyle.revisionColor(name)
    }
}
