//! The lamp list as cells, for the second sheet of ODS and XLSX (LH-50): the same `Lamp`s the
//! printed legend names.

use klappe_stellprobe::Lamp;

use crate::canvas::num;
use crate::page::LampTexts;
use crate::shots::{Kind, Table};

pub(crate) fn table(lamps: &[Lamp], words: &LampTexts) -> Table {
    let columns = &words.columns;
    let head = [
        &columns.label,
        &columns.kind,
        &columns.power_w,
        &columns.color_temp_k,
        &columns.beam_deg,
    ];

    let rows = lamps.iter().map(|lamp| row(lamp, words)).collect();

    Table {
        head: head.iter().map(|h| (*h).clone()).collect(),
        rows,
        kinds: vec![
            Kind::Text,
            Kind::Text,
            Kind::Numbers,
            Kind::Numbers,
            Kind::Numbers,
        ],
    }
}

fn row(lamp: &Lamp, words: &LampTexts) -> Vec<String> {
    // Bounce and flag give no light of their own: power and colour stay empty, as in print.
    let (power, temp) = match lamp.kind.gives_light() {
        true => (num(lamp.power_w), num(lamp.color_temp_k)),
        false => (String::new(), String::new()),
    };

    vec![
        lamp.label.clone(),
        words.kind_name(lamp.kind).to_owned(),
        power,
        temp,
        num(lamp.beam_deg),
    ]
}
