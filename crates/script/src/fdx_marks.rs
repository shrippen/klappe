//! The text of an FDX paragraph as Fountain: runs with a style get its marks.
//!
//! Fountain reads a run of stars as one mark, so the marks of two runs that touch must not
//! stand side by side (`**Nie***mals*` is no bold and italic, it is stars). Where a run
//! cannot be marked without that, it is left plain and counted. Every line is read back
//! with the reader of the screenplay; if it does not say what the file says, the line is
//! written without emphasis, so text is never lost, only counted emphasis.

use crate::fdx::{Style, STYLE, TEXT};
use crate::inline::visible_runs;
use crate::xml::Node;

/// Characters that, directly before a mark, change what it means: `/*` opens a boneyard.
const SPOILERS: [char; 1] = ['/'];

/// Characters of the inline syntax; each is written with a backslash, whatever its
/// neighbours, so that a run's text reads back as itself next to any other run.
const SPECIAL: [char; 4] = ['\\', '*', '_', '['];

/// One run's text on one line, as the file has it, and how it is set.
struct Frag {
    style: Style,
    /// The style the run has in the file; `style` is what is left of it.
    wanted: Style,
    text: String,
}

impl Frag {
    fn plain(text: &str) -> Frag {
        Frag {
            style: Style::default(),
            wanted: Style::default(),
            text: text.to_owned(),
        }
    }

    fn stars(&self) -> bool {
        self.style.bold || self.style.italic
    }

    /// The marks stand directly at the first or last character.
    fn touches_start(&self) -> bool {
        !self.text.starts_with(char::is_whitespace)
    }

    fn touches_end(&self) -> bool {
        !self.text.ends_with(char::is_whitespace)
    }
}

/// How Klappe reads the text of an element kind.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reading {
    /// With the inline syntax of Fountain (action, dialogue): marks make styles.
    Inline,
    /// As it stands (heading, name, transition): no marks, nothing to escape.
    Literal,
}

/// The paragraph's text as Fountain. Read `Inline`, literal special characters get a
/// backslash so they stay text (`/*` is protected when the book is written); read `Literal`,
/// the text stands as it is. `lost` counts the runs that had to give up their style, and
/// every styled run of a `Literal` paragraph.
pub(crate) fn marked_text(par: &Node, reading: Reading, lost: &mut usize) -> String {
    if reading == Reading::Literal {
        let runs = runs(par);
        *lost += runs.iter().filter(|(s, _)| *s != Style::default()).count();

        return runs.into_iter().map(|(_, text)| text).collect();
    }

    let mut lines: Vec<Vec<Frag>> = vec![Vec::new()];

    for (style, text) in runs(par) {
        for (n, line) in text.split('\n').enumerate() {
            if n > 0 {
                lines.push(Vec::new());
            }
            if !line.is_empty() {
                let at = lines.len() - 1;
                lines[at].push(Frag {
                    style,
                    wanted: style,
                    text: line.to_owned(),
                });
            }
        }
    }

    let done: Vec<String> = lines
        .into_iter()
        .map(|line| safe_line(settle(line), lost))
        .collect();

    done.join("\n")
}

/// The line as Fountain; if reading it back gives other text or style than the line has,
/// it is written plain instead. Counts the runs that lost their style.
fn safe_line(mut line: Vec<Frag>, lost: &mut usize) -> String {
    let mut text = render(&line);
    if !reads_back(&line, &text) {
        line.iter_mut().for_each(|f| f.style = Style::default());
        text = render(&line);
    }

    *lost += line.iter().filter(|f| f.style != f.wanted).count();

    text
}

/// Whether `text` says what `line` has: the same characters, and the style of every
/// character that is not a blank.
fn reads_back(line: &[Frag], text: &str) -> bool {
    let mut seen = Vec::new();
    for (marks, part) in visible_runs(text) {
        if marks.note || marks.boneyard {
            return false;
        }

        let style = Style {
            bold: marks.bold,
            italic: marks.italic,
            underline: marks.underline,
        };
        seen.extend(part.chars().map(|c| (c, style)));
    }

    let wanted = line
        .iter()
        .flat_map(|f| f.text.chars().map(|c| (c, f.style)));

    seen.len() == wanted.clone().count()
        && seen
            .into_iter()
            .zip(wanted)
            .all(|(s, w)| s.0 == w.0 && (s.0.is_whitespace() || s.1 == w.1))
}

/// The runs of the paragraph; neighbours of one style join, and an empty run is no run.
fn runs(par: &Node) -> Vec<(Style, String)> {
    let mut runs: Vec<(Style, String)> = Vec::new();

    for run in par.nodes().filter(|n| n.name == TEXT) {
        let style = Style::parse(run.attr(STYLE).unwrap_or_default());
        let text = run.text();
        if text.is_empty() {
            continue;
        }

        match runs.last_mut() {
            Some((last, joined)) if *last == style => joined.push_str(&text),
            _ => runs.push((style, text)),
        }
    }

    runs
}

/// Takes the style off the runs whose marks would touch another mark or a character that
/// changes them, and gives it back to a run whose neighbour gave way in the end. A run of
/// blanks has no word to mark and is plain.
fn settle(line: Vec<Frag>) -> Vec<Frag> {
    let mut out: Vec<Frag> = line.into_iter().flat_map(split_tail).collect();
    for frag in out.iter_mut().filter(|f| f.text.trim().is_empty()) {
        frag.style = Style::default();
    }

    for i in 1..out.len() {
        if !meets(&out[i - 1], &out[i]) {
            continue;
        }
        // The shorter run of two stars gives way; a tie costs the later one.
        let star_clash = clash(&out[i - 1], &out[i]);
        let earlier = star_clash && out[i - 1].text.len() < out[i].text.len();
        let victim = if earlier { i - 1 } else { i };
        out[victim].style = Style::default();
    }

    for i in 0..out.len() {
        let given_up = out[i].style != out[i].wanted && !out[i].text.trim().is_empty();
        if !given_up {
            continue;
        }

        // Without the neighbour that took the clash, the run may keep its style.
        out[i].style = out[i].wanted;
        let before = i > 0 && meets(&out[i - 1], &out[i]);
        let after = i + 1 < out.len() && meets(&out[i], &out[i + 1]);
        if before || after {
            out[i].style = Style::default();
        }
    }

    out
}

/// Whether `b` cannot keep its marks behind `a`: they would touch another mark, or a `/`
/// stands before them.
fn meets(a: &Frag, b: &Frag) -> bool {
    let opens_with_star = b.stars() && !b.style.underline;
    let spoiled = a.text.ends_with(SPOILERS) && opens_with_star && b.touches_start();

    spoiled || clash(a, b)
}

/// Whether the closing stars of `a` and the opening stars of `b` would touch. An underscore
/// between them keeps them apart.
fn clash(a: &Frag, b: &Frag) -> bool {
    let stars = a.stars() && b.stars();
    let apart = a.style.underline || b.style.underline;

    stars && !apart && a.touches_end() && b.touches_start()
}

/// A run with stars that ends in `/` would put its closing stars behind it: the tail leaves
/// the run and stands plain.
fn split_tail(frag: Frag) -> Vec<Frag> {
    if !frag.stars() {
        return vec![frag];
    }

    let core = frag.text.trim_end();
    let kept = core.trim_end_matches(SPOILERS);
    if kept.len() == core.len() {
        return vec![frag];
    }

    let plain = |text: &str| Frag {
        wanted: frag.wanted,
        ..Frag::plain(text)
    };
    if kept.trim().is_empty() {
        return vec![plain(&frag.text)];
    }

    vec![
        Frag {
            text: kept.to_owned(),
            ..frag
        },
        plain(&frag.text[kept.len()..]),
    ]
}

fn render(line: &[Frag]) -> String {
    line.iter().map(|f| mark_line(&f.text, f.style)).collect()
}

/// `line` with the marks of `style` around its words; blanks stay outside.
fn mark_line(line: &str, style: Style) -> String {
    let core = line.trim();
    let under = if style.underline { "_" } else { "" };
    let stars = match (style.bold, style.italic) {
        (true, true) => "***",
        (true, false) => "**",
        (false, true) => "*",
        (false, false) => "",
    };
    if core.is_empty() {
        return line.to_owned();
    }

    let lead = &line[..line.len() - line.trim_start().len()];
    let trail = &line[line.trim_end().len()..];

    format!("{lead}{under}{stars}{}{stars}{under}{trail}", escape(core))
}

/// `text` as Fountain that reads back as itself: every special character gets a backslash.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        if SPECIAL.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }

    out
}
