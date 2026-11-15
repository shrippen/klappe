//! Templates (LH-07): place on the plan on view, file the plan as one.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

impl qobject::Controller {
    pub(super) fn apply_template(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let id = id.to_string();
        let result = self.as_mut().session().map(|s| {
            let at = s.plan_middle();

            s.apply_template(&id, at)
        });

        self.report(result)
    }

    pub(super) fn save_template(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let language = self.rust().language;
        let name = name.to_string();
        let result = self.as_mut().session().map(|s| s.save_template(&name));

        if !self.as_mut().report(result) {
            return false;
        }

        // The change cleared the status line; the result goes there.
        let saved = language
            .text("template.saved")
            .replace("{name}", name.trim());
        self.as_mut().say(saved);
        true
    }
}
