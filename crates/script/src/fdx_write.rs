//! A [`Script`] to FDX.

use crate::fdx::{
    FdxLoss, Style, ACTION, ALIGNMENT, CENTER, CHARACTER, CONTENT, DIALOGUE, DUAL, HEADING, KEY,
    LEFT, NEW_PAGE, NUMBER, PARAGRAPH, PARENTHETICAL, RIGHT, ROOT, STYLE, TEXT, TITLE_PAGE,
    TRANSITION, TYPE, YES,
};
use crate::inline::{inline_runs, run_text, Marks};
use crate::title::TITLE_KEYS;
use crate::xml::{Kid, Node};
use crate::{Element, Script};

const DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>\n";
/// The key of the draft date in `TITLE_KEYS`: it sits at the right of the sheet.
const DRAFT_DATE_RANK: usize = 4;
/// Keys up to this rank (title, credit, author, source) are centered.
const CENTERED_RANKS: usize = 4;

/// Writes the script as an FDX document and says what the file could not take.
pub fn write_fdx(script: &Script) -> (String, FdxLoss) {
    let mut writer = Writer {
        loss: FdxLoss::default(),
        pending_break: false,
    };
    let content = Node {
        name: CONTENT.to_owned(),
        attrs: Vec::new(),
        kids: writer
            .content(&script.elements)
            .into_iter()
            .map(Kid::Node)
            .collect(),
    };

    let mut root = Node::new(ROOT)
        .with_attr("DocumentType", "Script")
        .with_attr("Template", "No")
        .with_attr("Version", "5");
    root.kids.push(Kid::Node(content));
    if !script.title_page.is_empty() {
        root.kids.push(Kid::Node(title_page(&script.title_page)));
    }

    let mut out = String::from(DECLARATION);
    root.write(&mut out, 0);

    (out, writer.loss)
}

struct Writer {
    loss: FdxLoss,
    /// A page break came and no paragraph has taken it yet.
    pending_break: bool,
}

impl Writer {
    fn content(&mut self, elements: &[Element]) -> Vec<Node> {
        let mut out = Vec::new();
        let mut i = 0;

        while i < elements.len() {
            let el = &elements[i];
            if let Element::Character { dual: false, .. } = el {
                i = self.voices(elements, i, &mut out);
                continue;
            }
            i += 1;

            match el {
                Element::PageBreak => self.page_break(),
                Element::Section { .. } => self.loss.sections += 1,
                Element::Synopsis(_) => self.loss.synopses += 1,
                Element::Note(_) => self.loss.notes += 1,
                other => {
                    let node = self.paragraph(other);
                    self.place(node, &mut out);
                }
            }
        }

        if self.pending_break {
            self.loss.page_breaks += 1;
        }

        out
    }

    fn page_break(&mut self) {
        if self.pending_break {
            self.loss.page_breaks += 1;
        }
        self.pending_break = true;
    }

    /// Adds `node`; it starts the page a page break asked for.
    fn place(&mut self, node: Node, out: &mut Vec<Node>) {
        out.push(node);
        if !self.pending_break {
            return;
        }
        self.pending_break = false;

        let Some(last) = out.last_mut() else {
            return;
        };
        // A dual dialogue starts the page with its first paragraph.
        let target = if last.name == DUAL {
            last.kids.iter_mut().find_map(first_node)
        } else {
            Some(last)
        };
        if let Some(par) = target {
            par.attrs.insert(0, (NEW_PAGE.to_owned(), YES.to_owned()));
        }
    }

    /// The cue at `start` with its lines, and the dual voice after it if there is one. Returns
    /// the index after what was written.
    fn voices(&mut self, elements: &[Element], start: usize, out: &mut Vec<Node>) -> usize {
        let first_end = block_end(elements, start);
        let second = elements.get(first_end);
        let dual = matches!(second, Some(Element::Character { dual: true, .. }));
        let end = if dual {
            block_end(elements, first_end)
        } else {
            first_end
        };

        let paragraphs: Vec<Node> = elements[start..end]
            .iter()
            .map(|el| self.paragraph(el))
            .collect();
        if !dual {
            for par in paragraphs {
                self.place(par, out);
            }

            return end;
        }

        let mut group = Node::new(DUAL);
        group.kids = paragraphs.into_iter().map(Kid::Node).collect();
        self.place(group, out);

        end
    }

    /// One paragraph of an element FDX has a type for.
    fn paragraph(&mut self, el: &Element) -> Node {
        match el {
            Element::SceneHeading { text, number } => {
                let mut par = self.par(HEADING, text);
                if let Some(n) = number {
                    par.attrs.push((NUMBER.to_owned(), n.clone()));
                }
                par
            }
            Element::Character {
                name, extension, ..
            } => {
                let text = match extension {
                    Some(ext) => format!("{name} ({ext})"),
                    None => name.clone(),
                };
                self.par(CHARACTER, &text)
            }
            Element::Parenthetical(text) => self.par(PARENTHETICAL, &format!("({text})")),
            Element::Dialogue(text) => self.par(DIALOGUE, text),
            Element::Transition(text) => self.par(TRANSITION, text),
            Element::Centered(text) => {
                let par = self.par(ACTION, text);
                par.with_attr(ALIGNMENT, CENTER)
            }
            Element::Lyrics(text) => {
                self.loss.lyrics += 1;
                self.par(ACTION, text)
            }
            Element::Action(text) => self.par(ACTION, text),
            // Counted and left out by `content`.
            Element::Section { .. } | Element::Synopsis(_) | Element::Note(_) => {
                Node::new(PARAGRAPH)
            }
            Element::PageBreak => Node::new(PARAGRAPH),
        }
    }

    fn par(&mut self, kind: &str, text: &str) -> Node {
        let mut par = Node::new(PARAGRAPH).with_attr(TYPE, kind);
        for (style, piece) in self.runs(text) {
            let mut run = Node::new(TEXT).with_text(&piece);
            if let Some(attr) = style.attr() {
                run.attrs.push((STYLE.to_owned(), attr));
            }
            par.kids.push(Kid::Node(run));
        }

        par
    }

    /// The text as runs of one style each: the marks of Fountain's emphasis become the style,
    /// notes inside the text are left out and counted.
    fn runs(&mut self, text: &str) -> Vec<(Style, String)> {
        let mut pieces: Vec<(Style, String)> = Vec::new();

        for (n, line) in text.split('\n').enumerate() {
            if n > 0 {
                push_piece(&mut pieces, Style::default(), "\n");
            }
            let chars: Vec<char> = line.chars().collect();
            let (runs, _) = inline_runs(line, false);

            for run in runs {
                if run.marks.note {
                    self.loss.notes += 1;
                    continue;
                }
                let part = run_text(&chars[run.start..run.end], run.marks);
                push_piece(&mut pieces, style_of(run.marks), &part);
            }
        }

        pieces
    }
}

fn first_node(kid: &mut Kid) -> Option<&mut Node> {
    match kid {
        Kid::Node(n) => Some(n),
        Kid::Text(_) => None,
    }
}

fn style_of(marks: Marks) -> Style {
    Style {
        bold: marks.bold,
        italic: marks.italic,
        underline: marks.underline,
    }
}

/// Neighbours of one style join into one run.
fn push_piece(pieces: &mut Vec<(Style, String)>, style: Style, text: &str) {
    if text.is_empty() {
        return;
    }

    match pieces.last_mut() {
        Some((last, joined)) if *last == style => joined.push_str(text),
        _ => pieces.push((style, text.to_owned())),
    }
}

/// The index after the cue at `start` and its parentheticals and dialogue.
fn block_end(elements: &[Element], start: usize) -> usize {
    let lines = elements[start + 1..]
        .iter()
        .take_while(|el| matches!(el, Element::Parenthetical(_) | Element::Dialogue(_)))
        .count();

    start + 1 + lines
}

/// One paragraph per key, laid out as a sheet: title to source centered, the date at the
/// right, the rest at the left; the key is kept for reading it back.
fn title_page(pairs: &[(String, String)]) -> Node {
    let mut content = Node::new(CONTENT);

    for (key, value) in pairs {
        let rank = TITLE_KEYS
            .iter()
            .position(|k| k.eq_ignore_ascii_case(key.trim()));
        let alignment = match rank {
            Some(r) if r < CENTERED_RANKS => CENTER,
            Some(DRAFT_DATE_RANK) => RIGHT,
            _ => LEFT,
        };
        let par = Node::new(PARAGRAPH)
            .with_attr(ALIGNMENT, alignment)
            .with_attr(KEY, key)
            .with_node(Node::new(TEXT).with_text(value));
        content = content.with_node(par);
    }

    Node::new(TITLE_PAGE).with_node(content)
}
