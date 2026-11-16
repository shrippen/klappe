//! The title page of a Fountain text: reading the fields of the form and writing them back
//! without touching the rest of the text.

use crate::parse::{parse, title_page_span};
use crate::{write, Script};

/// The keys of the form, in the order Fountain lists them.
pub const TITLE_KEYS: [&str; 6] = [
    "Title",
    "Credit",
    "Author",
    "Source",
    "Draft date",
    "Contact",
];

/// A key as compared: no case, no spaces.
fn same_key(a: &str, b: &str) -> bool {
    let plain = |s: &str| {
        s.chars()
            .filter(|c| !c.is_whitespace())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };

    plain(a) == plain(b)
}

fn rank(key: &str) -> Option<usize> {
    TITLE_KEYS.iter().position(|k| same_key(k, key))
}

/// The value of each key of the form (`""` when the title page lacks it); other keys of the
/// title page are not in the list.
pub fn title_fields(text: &str) -> Vec<(String, String)> {
    let page = parse(text).title_page;

    TITLE_KEYS
        .iter()
        .map(|key| {
            let value = page.iter().find(|(k, _)| same_key(k, key));
            (
                (*key).to_owned(),
                value.map(|(_, v)| v.clone()).unwrap_or_default(),
            )
        })
        .collect()
}

/// The pairs of the title page with the form's keys set from `fields`: a key keeps its place
/// and its spelling, an emptied one goes, a new one is put after the form key before it.
fn merged(old: Vec<(String, String)>, fields: &[(String, String)]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();

    for (key, value) in old {
        let Some(rank) = rank(&key) else {
            pairs.push((key, value));
            continue;
        };
        let new = fields.iter().find(|(k, _)| same_key(k, TITLE_KEYS[rank]));
        match new {
            Some((_, v)) if v.trim().is_empty() => {}
            Some((_, v)) => pairs.push((key, v.trim().to_owned())),
            None => pairs.push((key, value)),
        }
    }

    for key in TITLE_KEYS {
        let present = pairs.iter().any(|(k, _)| same_key(k, key));
        let value = fields
            .iter()
            .find(|(k, _)| same_key(k, key))
            .map(|(_, v)| v.trim());
        let Some(value) = value.filter(|v| !present && !v.is_empty()) else {
            continue;
        };

        // After the last form key that comes before this one, else at the top.
        let at = pairs
            .iter()
            .rposition(|(k, _)| rank(k).is_some_and(|r| r < rank(key).unwrap_or(0)))
            .map_or(0, |i| i + 1);
        pairs.insert(at, (key.to_owned(), value.to_owned()));
    }

    pairs
}

/// The text with the form's keys of the title page set to `fields`. The rest of the text,
/// body and other title keys, stays as it is (a boneyard inside the old title page goes, one before it stays); a text without title page gets one.
pub fn set_title_fields(text: &str, fields: &[(String, String)]) -> String {
    let old = parse(text).title_page;
    let pairs = merged(old, fields);

    // What stands before the title page (a boneyard) stays; the page ends where the reader
    // says. A comment inside the old page goes with it.
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let (start, end) = title_page_span(&text).unwrap_or((0, 0));
    let head = lines[..start.min(lines.len())].join("\n");
    let rest = lines[end.min(lines.len())..].join("\n");
    let rest = rest.trim_start_matches('\n');
    let head = match head.is_empty() {
        true => head,
        false => head + "\n",
    };
    if pairs.is_empty() {
        return head + rest;
    }

    let page = write(&Script {
        title_page: pairs,
        elements: Vec::new(),
    });
    if rest.is_empty() {
        return head + page.trim_end() + "\n";
    }

    head + &page + rest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    const BODY: &str = "INT. A - TAG\n\nAnna geht.\n\nANNA\nHallo.\n";

    #[test]
    fn a_text_without_title_page_gets_one() {
        let text = set_title_fields(
            BODY,
            &fields(&[("Title", "Nachtwache"), ("Author", "A. B.")]),
        );

        assert_eq!(text, format!("Title: Nachtwache\nAuthor: A. B.\n\n{BODY}"));
        assert_eq!(parse(&text).elements, parse(BODY).elements);
    }

    #[test]
    fn fields_are_read_in_form_order() {
        let text = format!("Author: A\nCopyright: X\ntitle: T\n\n{BODY}");

        assert_eq!(
            title_fields(&text),
            fields(&[
                ("Title", "T"),
                ("Credit", ""),
                ("Author", "A"),
                ("Source", ""),
                ("Draft date", ""),
                ("Contact", ""),
            ])
        );
    }

    #[test]
    fn other_keys_and_the_body_stay() {
        let text = format!("Title: T\nCopyright: X\n\n{BODY}");
        let new = set_title_fields(
            &text,
            &fields(&[
                ("Title", "Neu"),
                ("Credit", "von"),
                ("Author", "Eva"),
                ("Draft date", "1.1."),
            ]),
        );

        assert_eq!(
            new,
            format!(
                "Title: Neu\nCredit: von\nAuthor: Eva\nDraft date: 1.1.\nCopyright: X\n\n{BODY}"
            )
        );
        assert_eq!(parse(&new).elements, parse(BODY).elements);
    }

    #[test]
    fn an_emptied_field_goes_and_the_last_one_takes_the_page() {
        let text = format!("Title: T\nAuthor: A\n\n{BODY}");

        let one = set_title_fields(&text, &fields(&[("Author", "")]));
        assert_eq!(one, format!("Title: T\n\n{BODY}"));

        let none = set_title_fields(&one, &fields(&[("Title", " ")]));
        assert_eq!(none, BODY);
    }

    #[test]
    fn multi_line_values_round_trip() {
        let text = format!("Title: T\nContact:\n   Eva\n   Weg 1\n\n{BODY}");
        let same = set_title_fields(&text, &title_fields(&text));

        assert_eq!(same, text);
        assert_eq!(title_fields(&same)[5].1, "Eva\nWeg 1");
    }

    #[test]
    fn a_title_only_text_stays_a_title_page() {
        let new = set_title_fields("", &fields(&[("Title", "T")]));

        assert_eq!(new, "Title: T\n");
        assert_eq!(parse(&new).title_page, fields(&[("Title", "T")]));
    }

    #[test]
    fn a_boneyard_before_the_title_page_does_not_make_a_second_one() {
        let text = format!("/* weg */\nTitle: Alt\n\n{BODY}");
        assert_eq!(title_fields(&text)[0].1, "Alt");

        let new = set_title_fields(&text, &fields(&[("Title", "Neu")]));
        assert_eq!(parse(&new).title_page, fields(&[("Title", "Neu")]));
        assert_eq!(parse(&new).elements, parse(BODY).elements);
        assert_eq!(new.matches("Title:").count(), 1, "{new}");
        assert!(new.starts_with("/* weg */\nTitle: Neu\n\n"), "{new}");
    }
}
