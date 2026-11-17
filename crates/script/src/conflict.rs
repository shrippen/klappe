//! Git conflict markers in a text (DATENMODELL §4).

const OURS: &str = "<<<<<<<";
const SEPARATOR: &str = "=======";
const THEIRS: &str = ">>>>>>>";

/// The 1-based line of the first conflict's opening marker, if the text has a complete
/// one: `<<<<<<<` (alone or followed by a space and a label), later `=======` alone on
/// its line, later `>>>>>>>`. A lone `=======` is a Fountain page break, so the order is
/// required.
pub fn conflict_line(text: &str) -> Option<usize> {
    let mut open: Option<usize> = None;
    let mut split = false;

    for (i, line) in text.lines().enumerate() {
        match open {
            None if is_marker(line, OURS) => open = Some(i + 1),
            Some(_) if !split && line.trim_end() == SEPARATOR => split = true,
            Some(start) if split && is_marker(line, THEIRS) => return Some(start),
            _ => {}
        }
    }
    None
}

fn is_marker(line: &str, marker: &str) -> bool {
    line.strip_prefix(marker)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
}
