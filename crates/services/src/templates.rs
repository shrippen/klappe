//! The templates as the session sees them: the built-in ones and the project's own (LH-07).

use klappe_model::Template;
use klappe_store::builtin_templates;

use crate::views::TemplateEntry;

/// The template `id`: the project's own first, then the built-in ones.
pub(crate) fn find<'a>(own: &'a [Template], id: &str) -> Option<&'a Template> {
    own.iter().chain(builtin_templates()).find(|t| t.id == id)
}

/// Every template in the order a list shows them: built-in, then own. A built-in one an own
/// one has the id of is left out, as `find` would never reach it.
pub(crate) fn entries(own: &[Template]) -> Vec<TemplateEntry> {
    let built_in = builtin_templates()
        .iter()
        .filter(|b| !own.iter().any(|o| o.id == b.id))
        .map(|t| TemplateEntry::of(t, false));
    let custom = own.iter().map(|t| TemplateEntry::of(t, true));

    built_in.chain(custom).collect()
}
