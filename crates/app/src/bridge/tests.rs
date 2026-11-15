use klappe_services::Session;

pub(super) fn example() -> Session {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    Session::open(&dir).unwrap()
}
