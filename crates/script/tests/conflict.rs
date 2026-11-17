//! Git conflict markers.

use klappe_script::conflict_line;

const CONFLICT: &str = "INT. SAAL

Sie wartet.

<<<<<<< HEAD
Er kommt.
=======
Er geht.
>>>>>>> feature

ENDE
";

#[test]
fn a_conflict_reports_its_first_line() {
    assert_eq!(conflict_line(CONFLICT), Some(5));
}

#[test]
fn a_marker_without_a_label_counts() {
    assert_eq!(conflict_line("<<<<<<<\na\n=======\nb\n>>>>>>>\n"), Some(1));
}

#[test]
fn a_page_break_alone_is_no_conflict() {
    assert_eq!(conflict_line("Er geht.\n\n=======\n\nSie kommt.\n"), None);
}

#[test]
fn an_incomplete_conflict_is_none() {
    assert_eq!(
        conflict_line("<<<<<<< HEAD\nEr kommt.\n=======\nEr geht.\n"),
        None
    );
    assert_eq!(
        conflict_line("Er kommt.\n=======\nEr geht.\n>>>>>>> x\n"),
        None
    );
}

#[test]
fn markers_inside_a_line_are_text() {
    assert_eq!(conflict_line("a <<<<<<< b\n=======\n>>>>>>> c\n"), None);
}

#[test]
fn windows_line_ends_work() {
    assert_eq!(
        conflict_line("<<<<<<< HEAD\r\na\r\n=======\r\nb\r\n>>>>>>> x\r\n"),
        Some(1)
    );
}

#[test]
fn a_text_without_markers_is_none() {
    assert_eq!(conflict_line(""), None);
}
