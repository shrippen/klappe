//! The text as lines; all positions count characters, not bytes.

/// A place in the text: line and character column, both from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Buffer {
    lines: Vec<String>,
}

/// Byte offset of the character column; the line end for a column past it.
fn byte_at(line: &str, col: usize) -> usize {
    line.char_indices().nth(col).map_or(line.len(), |(i, _)| i)
}

impl Buffer {
    /// Splits at `\n`; a text ending in `\n` ends in an empty line, so `text()` gives it back.
    pub(crate) fn from_text(text: &str) -> Self {
        let text = text.replace("\r\n", "\n");
        Self {
            lines: text.split('\n').map(str::to_owned).collect(),
        }
    }

    pub(crate) fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub(crate) fn lines(&self) -> &[String] {
        &self.lines
    }

    pub(crate) fn line(&self, index: usize) -> &str {
        self.lines.get(index).map_or("", String::as_str)
    }

    pub(crate) fn line_len(&self, index: usize) -> usize {
        self.line(index).chars().count()
    }

    /// Moves a position onto the text.
    pub(crate) fn clamp(&self, pos: Pos) -> Pos {
        let line = pos.line.min(self.lines.len().saturating_sub(1));
        Pos {
            line,
            col: pos.col.min(self.line_len(line)),
        }
    }

    /// Returns the position after the character.
    pub(crate) fn insert_char(&mut self, pos: Pos, c: char) -> Pos {
        let pos = self.clamp(pos);
        let Some(line) = self.lines.get_mut(pos.line) else {
            return pos;
        };
        line.insert(byte_at(line, pos.col), c);

        Pos {
            line: pos.line,
            col: pos.col + 1,
        }
    }

    /// Splits the line at the position; returns the start of the new line.
    pub(crate) fn insert_newline(&mut self, pos: Pos) -> Pos {
        let pos = self.clamp(pos);
        let Some(line) = self.lines.get_mut(pos.line) else {
            return pos;
        };
        let rest = line.split_off(byte_at(line, pos.col));
        self.lines.insert(pos.line + 1, rest);

        Pos {
            line: pos.line + 1,
            col: 0,
        }
    }

    /// Removes the character before the position, or joins the line with the one above.
    pub(crate) fn backspace(&mut self, pos: Pos) -> Pos {
        let pos = self.clamp(pos);
        if pos.col > 0 {
            if let Some(line) = self.lines.get_mut(pos.line) {
                line.remove(byte_at(line, pos.col - 1));
            }
            return Pos {
                line: pos.line,
                col: pos.col - 1,
            };
        }
        if pos.line == 0 {
            return pos;
        }

        let col = self.line_len(pos.line - 1);
        let joined = self.lines.remove(pos.line);
        if let Some(above) = self.lines.get_mut(pos.line - 1) {
            above.push_str(&joined);
        }

        Pos {
            line: pos.line - 1,
            col,
        }
    }

    /// Removes the character at the position, or joins the next line to this one.
    pub(crate) fn delete(&mut self, pos: Pos) {
        let pos = self.clamp(pos);
        if pos.col < self.line_len(pos.line) {
            if let Some(line) = self.lines.get_mut(pos.line) {
                line.remove(byte_at(line, pos.col));
            }
            return;
        }
        if pos.line + 1 >= self.lines.len() {
            return;
        }

        let next = self.lines.remove(pos.line + 1);
        if let Some(line) = self.lines.get_mut(pos.line) {
            line.push_str(&next);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: usize, col: usize) -> Pos {
        Pos { line, col }
    }

    #[test]
    fn text_round_trips() {
        for text in ["", "a", "a\n", "a\n\nb\n", "ÄÖÜ – ä\nß\n"] {
            assert_eq!(Buffer::from_text(text).text(), text);
        }
    }

    #[test]
    fn crlf_becomes_lf() {
        assert_eq!(Buffer::from_text("a\r\nb").text(), "a\nb");
    }

    #[test]
    fn insert_counts_characters_not_bytes() {
        let mut buf = Buffer::from_text("Äpfel");
        let pos = buf.insert_char(at(0, 1), '–');

        assert_eq!(buf.text(), "Ä–pfel");
        assert_eq!(pos, at(0, 2));
    }

    #[test]
    fn newline_splits_and_backspace_joins() {
        let mut buf = Buffer::from_text("abcd");
        let pos = buf.insert_newline(at(0, 2));
        assert_eq!(buf.text(), "ab\ncd");
        assert_eq!(pos, at(1, 0));

        let pos = buf.backspace(pos);
        assert_eq!(buf.text(), "abcd");
        assert_eq!(pos, at(0, 2));
    }

    #[test]
    fn backspace_removes_umlaut() {
        let mut buf = Buffer::from_text("aÖb");
        let pos = buf.backspace(at(0, 2));

        assert_eq!(buf.text(), "ab");
        assert_eq!(pos, at(0, 1));
    }

    #[test]
    fn backspace_at_start_does_nothing() {
        let mut buf = Buffer::from_text("a");
        assert_eq!(buf.backspace(at(0, 0)), at(0, 0));
        assert_eq!(buf.text(), "a");
    }

    #[test]
    fn delete_removes_char_or_joins() {
        let mut buf = Buffer::from_text("aü\nb");
        buf.delete(at(0, 1));
        assert_eq!(buf.text(), "a\nb");

        buf.delete(at(0, 1));
        assert_eq!(buf.text(), "ab");

        buf.delete(at(0, 2));
        assert_eq!(buf.text(), "ab");
    }

    #[test]
    fn clamp_moves_onto_text() {
        let buf = Buffer::from_text("ab\nc");
        assert_eq!(buf.clamp(at(9, 9)), at(1, 1));
        assert_eq!(buf.clamp(at(0, 9)), at(0, 2));
    }
}
