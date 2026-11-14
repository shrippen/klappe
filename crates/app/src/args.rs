//! Command line of the app, read once at start.

use std::path::PathBuf;

pub const USAGE: &str = "\
klappe [--open FOLDER] [--language de|en] [--time SECONDS] [--shot ID]
       [--screenshot PNG] [--screenshot-after MS]

  --open FOLDER         project folder (*.klappe) to open
  --language de|en      language of the app (default: from the system)
  --time SECONDS        start with the playhead here
  --shot ID             start with this shot selected
  --screenshot PNG      save the window as PNG, then quit (tests, offscreen)
  --screenshot-after MS wait this long before the screenshot (default 1500)
";

/// Wait before a screenshot so the window has laid out and painted.
const SCREENSHOT_AFTER_MS: i32 = 1500;

/// Qt's own options that take a value (`-platform offscreen`); every other single-dash
/// option is a flag (`-reverse`) and leaves the next argument alone.
const QT_VALUE_OPTIONS: &[&str] = &[
    "-platform",
    "-platformpluginpath",
    "-platformtheme",
    "-plugin",
    "-qwindowgeometry",
    "-qwindowtitle",
    "-qwindowicon",
    "-style",
    "-display",
    "-geometry",
    "-session",
];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppArgs {
    pub open: Option<PathBuf>,
    pub language: Option<String>,
    pub time_s: f64,
    pub shot: Option<String>,
    pub screenshot: Option<PathBuf>,
    pub screenshot_after_ms: i32,
    pub help: bool,
    /// The first argument nobody understood.
    pub unknown: Option<String>,
}

impl AppArgs {
    pub fn from_env() -> Self {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut out = AppArgs {
            screenshot_after_ms: SCREENSHOT_AFTER_MS,
            ..Default::default()
        };
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => out.help = true,
                "--open" => out.open = args.next().map(PathBuf::from),
                "--language" => out.language = args.next(),
                "--time" => out.time_s = args.next().and_then(|t| t.parse().ok()).unwrap_or(0.0),
                "--shot" => out.shot = args.next(),
                "--screenshot" => out.screenshot = args.next().map(PathBuf::from),
                "--screenshot-after" => {
                    out.screenshot_after_ms = args
                        .next()
                        .and_then(|t| t.parse().ok())
                        .unwrap_or(SCREENSHOT_AFTER_MS);
                }
                // Qt's own options (e.g. -qwindowgeometry, -platform) pass through.
                other if other.starts_with('-') && !other.starts_with("--") => {
                    if QT_VALUE_OPTIONS.contains(&other) {
                        let _ = args.next();
                    }
                }
                other => {
                    if out.unknown.is_none() {
                        out.unknown = Some(other.to_owned());
                    }
                }
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> AppArgs {
        AppArgs::parse(args.iter().map(|a| a.to_string()))
    }

    #[test]
    fn options_with_values() {
        let a = parse(&["--open", "x.klappe", "--language", "en", "--time", "9.5"]);

        assert_eq!(a.open, Some(PathBuf::from("x.klappe")));
        assert_eq!(a.language.as_deref(), Some("en"));
        assert_eq!(a.time_s, 9.5);
        assert_eq!(a.screenshot_after_ms, SCREENSHOT_AFTER_MS);
    }

    #[test]
    fn qt_options_pass_unknown_is_kept() {
        let a = parse(&["-qwindowgeometry", "800x600", "--frob"]);

        assert_eq!(a.unknown.as_deref(), Some("--frob"));
    }

    #[test]
    fn qt_flags_without_value_keep_the_next_argument() {
        let a = parse(&["-reverse", "--open", "x.klappe"]);

        assert_eq!(a.unknown, None);
        assert_eq!(a.open, Some(PathBuf::from("x.klappe")));
    }

    #[test]
    fn qt_options_with_value_swallow_it() {
        let a = parse(&[
            "-platform",
            "offscreen",
            "-style",
            "Fusion",
            "--open",
            "x.klappe",
        ]);

        assert_eq!(a.unknown, None);
        assert_eq!(a.open, Some(PathBuf::from("x.klappe")));
    }
}
