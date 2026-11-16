//! Images of the project folder (`bilder/`, FORMAT.md): copied in, never deleted by `save`.

use std::fs;
use std::path::{Component, Path};

use klappe_imagehead::{jpeg_frame, png_head, PNG_SIGNATURE};

use crate::error::StoreError;
use crate::files::{self, io_err};
use crate::IMAGES_DIR;

const NOT_AN_IMAGE: &str = "not a readable PNG or JPEG image";
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
    check_new(&bytes).map_err(|message| bad(&message))?;

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

/// Whether `file` names a file of `bilder/`: a relative path without `..`, e.g.
/// `bilder/saal.png`.
pub fn is_image_path(file: &str) -> bool {
    let mut parts = Path::new(file).components();
    let first = parts.next();

    first == Some(Component::Normal(IMAGES_DIR.as_ref()))
        && parts.clone().next().is_some()
        && parts.all(|c| matches!(c, Component::Normal(_)))
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

    dimensions(&bytes).map_err(|message| StoreError::BadImage {
        path: dir.join(file),
        message,
    })
}

/// What an image must be to be added: readable, and printable by the PDF export.
fn check_new(bytes: &[u8]) -> Result<(), String> {
    dimensions(bytes)?;

    match jpeg_frame(bytes) {
        Some(frame) => frame.check_printable(),
        None => Ok(()),
    }
}

/// `(width, height)` of a PNG or JPEG; `Err` says why not. Whether the export can print a
/// JPEG is checked when an image is added, not here: Qt and SVG show what the PDF cannot.
fn dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    if bytes.starts_with(&PNG_SIGNATURE) {
        let head = png_head(bytes)?;

        return Ok((head.width, head.height));
    }

    let frame = jpeg_frame(bytes).ok_or(NOT_AN_IMAGE)?;

    Ok((frame.width, frame.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG header with `w` x `h` pixels, 8-bit RGB; the rest of the file is not needed.
    fn png_head(w: u32, h: u32) -> Vec<u8> {
        let mut b = PNG_SIGNATURE.to_vec();
        b.extend_from_slice(&13_u32.to_be_bytes());
        b.extend_from_slice(b"IHDR");
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&h.to_be_bytes());
        b.extend_from_slice(&[8, 2, 0, 0, 0]);
        b
    }

    #[test]
    fn reads_png_size() {
        assert_eq!(dimensions(&png_head(1000, 640)), Ok((1000, 640)));
    }

    #[test]
    fn rejects_other_data_and_zero_size() {
        assert!(dimensions(b"GIF89a").is_err());
        assert!(dimensions(&png_head(0, 5)).is_err());
        assert!(dimensions(&png_head(1, 1)[..20]).is_err());
        assert!(dimensions(&[0xFF, 0xD8, 0xFF]).is_err());
    }
}
