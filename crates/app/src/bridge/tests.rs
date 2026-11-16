use klappe_services::Session;

pub(super) fn example() -> Session {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    Session::open(&dir).unwrap()
}

#[cfg(test)]
mod marking {
    use super::example;
    use crate::bridge::view::plan_json;
    use crate::catalog::Language;
    use crate::selection::Selection;

    /// A selected shot marks its camera's body, view cone and whole path, and no other
    /// camera's (LH-73); nothing is selected by it.
    #[test]
    fn a_shot_marks_its_cameras_cone_and_path() {
        let mut s = example();
        assert!(s.select_shot("s12-1").is_ok());

        let selected = Selection::default();
        let json = plan_json(
            &s,
            &s.display_list_for(None),
            &selected,
            Language::German,
            (&[], None, None),
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let shapes = v["shapes"].as_array().unwrap();
        let of = |key: &str| -> Vec<bool> {
            shapes
                .iter()
                .filter(|sh| sh["key"] == key)
                .map(|sh| sh["selected"].as_bool().unwrap_or(false))
                .collect()
        };

        // Path, cone and body of camera a; camera b is a single mark, so cone and body only.
        let a = of("camera:a");
        assert_eq!(a.len(), 3);
        assert!(a.iter().all(|marked| *marked));
        let b = of("camera:b");
        assert!(!b.is_empty());
        assert!(b.iter().all(|marked| !*marked));
        assert!(selected.is_empty());
    }
}
