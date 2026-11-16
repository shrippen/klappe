//! Saved views (LH-119): the plan's layers, cameras and figures under a name.

use super::qobject;
use core::pin::Pin;
use cxx_qt_lib::QString;

impl qobject::Controller {
    pub(super) fn save_view(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.save_view(&name.to_string()).map(|_| ()));
        self.report(result)
    }

    pub(super) fn apply_view(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.apply_view(&id.to_string()));
        self.report(result)
    }

    pub(super) fn remove_view(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.remove_view(&id.to_string()));
        self.report(result)
    }
}
