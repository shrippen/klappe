use std::path::Path;

use super::export::write_file;
use super::Session;
use crate::catalog::Language;
use crate::error::ServiceError;
use klappe_script::FdxLoss;

impl Session {
    /// Writes the screenplay as a Final Draft file (DB-72) and says what FDX has no place for.
    pub fn export_script_fdx(&self, path: &Path) -> Result<FdxLoss, ServiceError> {
        if self.script_text().trim().is_empty() {
            return Err(ServiceError::NoScript);
        }

        let script = klappe_script::parse(self.script_text());
        let (xml, mut loss) = klappe_script::write_fdx(&script);
        // The parser has cut the boneyards from the script already.
        loss.boneyards = klappe_script::boneyard_count(self.script_text());
        write_file(path, xml.into_bytes())?;

        Ok(loss)
    }
}

/// What `export_script_fdx` left out, as one line ("Nicht in der FDX-Datei: Notizen 2");
/// empty when nothing was.
pub fn fdx_loss_text(language: Language, loss: &FdxLoss) -> String {
    let counted = [
        ("script.fdxSections", loss.sections),
        ("script.fdxSynopses", loss.synopses),
        ("script.fdxNotes", loss.notes),
        ("script.fdxLyrics", loss.lyrics),
        ("script.fdxBoneyards", loss.boneyards),
        ("script.fdxBreaks", loss.page_breaks),
    ];
    let parts: Vec<String> = counted
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(key, count)| language.text(key).replace("{count}", &count.to_string()))
        .collect();
    if parts.is_empty() {
        return String::new();
    }

    language
        .text("script.fdxLost")
        .replace("{list}", &parts.join(", "))
}
