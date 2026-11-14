//! Images of the project folder (`bilder/`, FORMAT.md): copied in, never deleted by `save`.

use std::fs;
use std::path::{Component, Path};

use crate::error::StoreError;
use crate::files::{self, io_err};
use crate::IMAGES_DIR;

const PNG_SIGNATURE: [u8; 8] = *b"\x89PNG\r\n\x1a\n";
/// Signature, then the IHDR chunk: length (4), type (4), width (4), height (4).
const PNG_HEADER_LEN: usize = 24;
const PNG_WIDTH_AT: usize = 16;
const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

/// Copies the image `source` into `bilder/` of `dir` and returns its path relative to `dir`,
/// e.g. `bilder/saal.png`. The same content under the same name is not copied twice; other
/// content under that name gets `-2`, `-3` … before the extension. Only PNG and JPEG.
pub fn add_image(dir: &Path, source: &Path) -> Result<String, StoreError> {
    let bad = |message: &str| StoreError::BadImage {
        path: source.to_path_buf(),
        message: message.to_owned(),
    };
    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| bad("no file name"))?;
    if !files::safe_name(name) {
        return Err(StoreError::BadId {
            id: name.to_owned(),
        });
    }

    let (stem, ext) = name.rsplit_once('.').ok_or_else(|| bad("no extension"))?;
    if !EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()) {
        return Err(bad("only .png, .jpg and .jpeg images are supported"));
    }

    let bytes = fs::read(source).map_err(|e| io_err(source, e))?;
    if dimensions(&bytes).is_none() {
        return Err(bad("not a readable PNG or JPEG image"));
    }

    for number in 1_u32.. {
        let candidate = match number {
            1 => name.to_owned(),
            n => format!("{stem}-{n}.{ext}"),
        };
        let target = dir.join(IMAGES_DIR).join(&candidate);
        let relative = format!("{IMAGES_DIR}/{candidate}");

        if !files::exists(&target)? {
            files::write_atomic(&target, &bytes)?;
            return Ok(relative);
        }
        if fs::read(&target).map_err(|e| io_err(&target, e))? == bytes {
            return Ok(relative);
        }
    }

    Err(bad("no free file name"))
}

/// The bytes of `file` (relative to `dir`, inside it), e.g. for the export.
pub fn read_image(dir: &Path, file: &str) -> Result<Vec<u8>, StoreError> {
    let inside = Path::new(file)
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if !inside {
        return Err(StoreError::BadId {
            id: file.to_owned(),
        });
    }

    let path = dir.join(file);
    fs::read(&path).map_err(|e| io_err(&path, e))
}

/// What tells whether an image file changed: its length and modification time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageStamp {
    len: u64,
    modified: Option<std::time::SystemTime>,
}

/// The stamp of `file` (relative to `dir`, inside it), without reading it.
pub fn image_stamp(dir: &Path, file: &str) -> Result<ImageStamp, StoreError> {
    let inside = Path::new(file)
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if !inside {
        return Err(StoreError::BadId {
            id: file.to_owned(),
        });
    }

    let path = dir.join(file);
    let meta = fs::metadata(&path).map_err(|e| io_err(&path, e))?;

    Ok(ImageStamp {
        len: meta.len(),
        modified: meta.modified().ok(),
    })
}

/// Width and height in pixels of `file` (relative to `dir`), read from the file header.
pub fn image_size(dir: &Path, file: &str) -> Result<(u32, u32), StoreError> {
    let bytes = read_image(dir, file)?;

    dimensions(&bytes).ok_or_else(|| StoreError::BadImage {
        path: dir.join(file),
        message: "not a readable PNG or JPEG image".to_owned(),
    })
}

/// `(width, height)` of a PNG or JPEG; `None` for anything else, or for zero pixels.
fn dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let size = match bytes {
        b if b.starts_with(&PNG_SIGNATURE) => png_size(b),
        b => klappe_imagehead::jpeg_frame(b).map(|f| (f.width, f.height)),
    };

    size.filter(|(w, h)| *w > 0 && *h > 0)
}

fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let head = bytes.get(..PNG_HEADER_LEN)?;
    let be = |at: usize| Some(u32::from_be_bytes(head.get(at..at + 4)?.try_into().ok()?));

    Some((be(PNG_WIDTH_AT)?, be(PNG_WIDTH_AT + 4)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG header with `w` x `h` pixels; the rest of the file is not needed.
    fn png_head(w: u32, h: u32) -> Vec<u8> {
        let mut b = PNG_SIGNATURE.to_vec();
        b.extend_from_slice(&13_u32.to_be_bytes());
        b.extend_from_slice(b"IHDR");
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&h.to_be_bytes());
        b
    }

    #[test]
    fn reads_png_size() {
        assert_eq!(dimensions(&png_head(1000, 640)), Some((1000, 640)));
    }

    #[test]
    fn rejects_other_data_and_zero_size() {
        assert_eq!(dimensions(b"GIF89a"), None);
        assert_eq!(dimensions(&png_head(0, 5)), None);
        assert_eq!(dimensions(&png_head(1, 1)[..20]), None);
        assert_eq!(dimensions(&[0xFF, 0xD8, 0xFF]), None);
    }
}
