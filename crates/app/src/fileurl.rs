//! Paths and `file://` URLs, both ways: what the canvas and the file dialogs speak.

use std::path::{Path, PathBuf};

const FILE_URL: &str = "file://";

/// A `file://` URL for the absolute `path`. Anything but unreserved characters and `/` is
/// percent-encoded.
pub fn from_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");

    // A Windows path (`C:/…`) needs the slash after `file://`.
    let mut url = String::from(FILE_URL);
    if !path.starts_with('/') {
        url.push('/');
    }
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                url.push(char::from(byte));
            }
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }

    url
}

/// `file://` URL or plain path as the file dialog hands it over, as a path. Percent-escapes
/// in a URL are decoded.
pub fn to_path(url: &str) -> PathBuf {
    let Some(rest) = url.strip_prefix(FILE_URL) else {
        return PathBuf::from(url);
    };

    let mut bytes = Vec::with_capacity(rest.len());
    let raw = rest.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        let escaped = (raw[i] == b'%')
            .then(|| raw.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                bytes.push(byte);
                i += 3;
            }
            None => {
                bytes.push(raw[i]);
                i += 1;
            }
        }
    }

    let path = String::from_utf8_lossy(&bytes).into_owned();
    // `file:///C:/x`: no slash before a drive letter.
    match path.as_bytes() {
        [b'/', drive, b':', ..] if drive.is_ascii_alphabetic() => PathBuf::from(&path[1..]),
        _ => PathBuf::from(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dialog_url_becomes_a_path() {
        assert_eq!(
            to_path("file:///home/a%20b/Plan%C3%A4.png"),
            PathBuf::from("/home/a b/Planä.png")
        );
        assert_eq!(to_path("/home/x/p.png"), PathBuf::from("/home/x/p.png"));
        assert_eq!(to_path("file:///C:/x/p.png"), PathBuf::from("C:/x/p.png"));
        assert_eq!(to_path("file:///a%zz"), PathBuf::from("/a%zz"));
    }

    /// A path with a space, a `#` and an umlaut: the URL gives the same path back.
    #[test]
    fn a_path_reads_back_from_its_url() {
        for path in ["/home/a b/p#1/Plan \u{e4}.png", "C:/x y/p.png"] {
            let url = from_path(Path::new(path));

            assert_eq!(to_path(&url), PathBuf::from(path), "{url}");
        }
    }
}
