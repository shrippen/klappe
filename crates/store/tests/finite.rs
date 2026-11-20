//! No number in a project's TOML file may be NaN or endless: such a project could be shown
//! nowhere and never saved again (`save` refuses them). Files are written by hand here.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// A copy of the example with `from` replaced by `to` in its location file, once.
fn copy_with(name: &str, from: &str, to: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    save(&dir, &load(Path::new(EXAMPLE)).unwrap().project, None).unwrap();
    let file = dir.join("drehorte/saengersaal.toml");
    let text = fs::read_to_string(&file).unwrap();
    assert!(text.contains(from), "{from} not in the file");
    fs::write(&file, text.replacen(from, to, 1)).unwrap();

    dir
}

/// Review of PR 281: a note whose place is no number loaded.
#[test]
fn a_note_placed_at_no_number_does_not_load() {
    let dir = copy_with(
        "finite-note-at",
        "x_m = 3.0\ny_m = 11.5",
        "x_m = nan\ny_m = 11.5",
    );

    let err = load(&dir).unwrap_err();

    assert!(err.to_string().contains("x_m"), "{err}");
}

#[test]
fn an_endless_object_does_not_load() {
    for value in ["inf", "-inf", "nan"] {
        let dir = copy_with(
            "finite-object",
            "width_m = 6.0",
            &format!("width_m = {value}"),
        );

        let err = load(&dir).unwrap_err();

        assert!(err.to_string().contains("width_m"), "{value}: {err}");
    }
}
