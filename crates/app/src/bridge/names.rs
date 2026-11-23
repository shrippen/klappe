//! The rules for names, as the session has them, for the pages of the window: which names are
//! the same, and how a name in capitals reads. The pages compare nothing themselves.

use super::{q, qobject};
use cxx_qt_lib::QString;

impl qobject::Controller {
    pub(super) fn same_figure_name(&self, a: &QString, b: &QString) -> bool {
        klappe_services::same_character_name(&a.to_string(), &b.to_string())
    }

    pub(super) fn same_place_name(&self, a: &QString, b: &QString) -> bool {
        klappe_services::same_place_name(&a.to_string(), &b.to_string())
    }

    pub(super) fn name_from_capitals(&self, capitals: &QString) -> QString {
        q(&klappe_services::name_from_capitals(&capitals.to_string()))
    }
}
