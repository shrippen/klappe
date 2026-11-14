//! Plans of a scene: select, add, copy, rename, remove.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::ShotsGo;

impl qobject::Controller {
    pub(super) fn select_plan(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if s.select_plan(&id.to_string()).is_err() {
            return false;
        }

        self.as_mut().rust_mut().get_mut().drop_drafts();
        self.as_mut().refresh();
        self.switched();
        true
    }

    pub(super) fn add_plan(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.add_plan(&name.to_string()).map(|_| ()));
        self.report(result)
    }

    pub(super) fn copy_plan(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.copy_plan(&name.to_string()).map(|_| ()));
        self.report(result)
    }

    pub(super) fn rename_plan(mut self: Pin<&mut Self>, id: &QString, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.rename_plan(&id.to_string(), &name.to_string()));
        self.report(result)
    }

    pub(super) fn remove_plan(self: Pin<&mut Self>, id: &QString) -> bool {
        self.drop_plan(id, ShotsGo::Refuse)
    }

    pub(super) fn remove_plan_with_shots(self: Pin<&mut Self>, id: &QString) -> bool {
        self.drop_plan(id, ShotsGo::Delete)
    }

    fn drop_plan(mut self: Pin<&mut Self>, id: &QString, shots_go: ShotsGo) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.remove_plan(&id.to_string(), shots_go));
        self.report(result)
    }
}
