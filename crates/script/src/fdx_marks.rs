//! The text of an FDX paragraph as Fountain: runs with a style get its marks.
//!
//! Fountain reads a run of stars as one mark, so the marks of two runs that touch must not
//! stand side by side (`**Nie***mals*` is no bold and italic, it is stars). Where a run
//! cannot be marked without that, it is left plain and counted.

use crate::fdx::{Style, STYLE, TEXT};
use crate::xml::Node;

/// Characters that, directly before a mark, change what it means: `\*` is a literal star and
/// `/*` opens a boneyard.
const SPOILERS: [char; 2] = ['/', '\\'];

/// One run's text on one line, escaped, and how it is set.
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

    fn marked(&self) -> bool {
        self.style != Style::default()
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

/// The paragraph's text as Fountain. A literal `*`, `_` or `[[` gets a backslash so it
/// stays text (`/*` is protected when the book is written); `lost` counts the runs that
/// had to give up their style.
pub(crate) fn marked_text(par: &Node, lost: &mut usize) -> String {
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
                    text: escape(line),
                });
            }
        }
    }

    let done: Vec<String> = lines
        .into_iter()
        .map(|line| render(settle(line, lost)))
        .collect();

    done.join("\n")
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
/// changes them, and gives it back to a run whose neighbour gave way in the end.
fn settle(line: Vec<Frag>, lost: &mut usize) -> Vec<Frag> {
    let mut out: Vec<Frag> = Vec::new();
    for frag in line {
        out.extend(split_tail(frag, lost));
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
        let given_up = out[i].style != out[i].wanted;
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

    *lost += out.iter().filter(|f| f.style != f.wanted).count();

    out
}

/// Whether `b` cannot keep its marks behind `a`: they would touch another mark, or a `\` or
/// `/` stands before them.
fn meets(a: &Frag, b: &Frag) -> bool {
    let spoiled = a.text.ends_with(SPOILERS) && b.marked() && b.touches_start();

    spoiled || clash(a, b)
}

/// Whether the closing stars of `a` and the opening stars of `b` would touch. An underscore
/// between them keeps them apart.
fn clash(a: &Frag, b: &Frag) -> bool {
    let stars = a.stars() && b.stars();
    let apart = a.style.underline || b.style.underline;

    stars && !apart && a.touches_end() && b.touches_start()
}

/// A marked run that ends in `/` or `\` would put its closing mark behind it: the tail
/// leaves the run and stands plain.
fn split_tail(frag: Frag, lost: &mut usize) -> Vec<Frag> {
    if !frag.marked() {
        return vec![frag];
    }

    let core = frag.text.trim_end();
    let kept = core.trim_end_matches(SPOILERS);
    if kept.len() == core.len() {
        return vec![frag];
    }

    *lost += 1;
    let tail = Frag::plain(&frag.text[kept.len()..]);
    if kept.trim().is_empty() {
        return vec![Frag::plain(&frag.text)];
    }

    vec![
        Frag {
            style: frag.style,
            wanted: frag.style,
            text: kept.to_owned(),
        },
        tail,
    ]
}

fn render(line: Vec<Frag>) -> String {
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
    if core.is_empty() || (under.is_empty() && stars.is_empty()) {
        return line.to_owned();
    }

    let lead = &line[..line.len() - line.trim_start().len()];
    let trail = &line[line.trim_end().len()..];

    format!("{lead}{under}{stars}{core}{stars}{under}{trail}")
}

/// `line` as Fountain text that reads back as itself: a backslash before a character it
/// would escape (`*`, `_`, `[` or another backslash) is escaped too.
fn escape(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());

    for (i, &c) in chars.iter().enumerate() {
        let next = chars.get(i + 1).copied();
        let escapes_next = c == '\\' && matches!(next, Some('*' | '_' | '[' | '\\'));
        if matches!(c, '*' | '_') || escapes_next {
            out.push('\\');
        }
        out.push(c);
    }

    // `[[` would open a note; the backslash keeps the first bracket text.
    out.replace("[[", "\\[[")
}
