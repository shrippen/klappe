//! The object library as the session sees it: the built-in objects and the project's own
//! (LH-11, LH-12).

use klappe_model::LibraryItem;
use klappe_store::builtin_library;

use crate::views::LibraryEntry;

/// The entry for `kind`: the project's own first, then the built-in ones.
pub(crate) fn find<'a>(own: &'a [LibraryItem], kind: &str) -> Option<&'a LibraryItem> {
    own.iter().chain(builtin_library()).find(|i| i.kind == kind)
}

/// Every entry in the order a list shows them: built-in, then own. A built-in entry an own
/// one has the kind of is left out, as `find` would never reach it.
pub(crate) fn entries(own: &[LibraryItem]) -> Vec<LibraryEntry> {
    let built_in = builtin_library()
        .iter()
        .filter(|b| !own.iter().any(|o| o.kind == b.kind))
        .map(|item| LibraryEntry::of(item, false));
    let custom = own.iter().map(|item| LibraryEntry::of(item, true));

    built_in.chain(custom).collect()
}
