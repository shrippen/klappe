//! Kante's dark colours (Gruvbox base) for the terminal.
//!
//! The values are copied from `crates/export/kante/palette.json`, not read from it at run
//! time: reading it needs a JSON parser the TUI does not depend on (it would be a new
//! dependency), and a terminal frame wants plain constants. The test below fails when a
//! value is no longer in that file.

use ratatui::style::Color;

/// `backgrounds.bg0`: the ground.
const GROUND_HEX: u32 = 0x28_28_28;
/// `backgrounds.bg1`: bars and the heading band.
const SURFACE_HEX: u32 = 0x3c_38_36;
/// `backgrounds.bg2`: the selected row.
const RAISED_HEX: u32 = 0x50_49_45;
/// `foregrounds.fg1`: text.
const TEXT_HEX: u32 = 0xeb_db_b2;
/// `foregrounds.fg0`: text that stands out.
const STRONG_HEX: u32 = 0xfb_f1_c7;
/// `foregrounds.fg2`: text a step back.
const SOFT_HEX: u32 = 0xd5_c4_a1;
/// `foregrounds.fg3`: quiet text (numbers, hints).
const MUTED_HEX: u32 = 0xa8_99_84;
/// `semantic.yellow.bright`: the accent.
const ACCENT_HEX: u32 = 0xfa_bd_2f;
/// `semantic.blue.bright`.
const BLUE_HEX: u32 = 0x83_a5_98;
/// `semantic.green.bright`.
const GREEN_HEX: u32 = 0xb8_bb_26;
/// `semantic.orange.bright`.
const ORANGE_HEX: u32 = 0xfe_80_19;
/// `semantic.red.bright`.
const RED_HEX: u32 = 0xfb_49_34;
/// `semantic.purple.bright`.
const PURPLE_HEX: u32 = 0xd3_86_9b;
/// `semantic.cyan.bright`.
const CYAN_HEX: u32 = 0x5c_cf_c4;

/// Every value above, for the test.
#[cfg(test)]
const ALL_HEX: [u32; 14] = [
    GROUND_HEX,
    SURFACE_HEX,
    RAISED_HEX,
    TEXT_HEX,
    STRONG_HEX,
    SOFT_HEX,
    MUTED_HEX,
    ACCENT_HEX,
    BLUE_HEX,
    GREEN_HEX,
    ORANGE_HEX,
    RED_HEX,
    PURPLE_HEX,
    CYAN_HEX,
];

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub(crate) const GROUND: Color = rgb(GROUND_HEX);
pub(crate) const SURFACE: Color = rgb(SURFACE_HEX);
pub(crate) const RAISED: Color = rgb(RAISED_HEX);
pub(crate) const TEXT: Color = rgb(TEXT_HEX);
pub(crate) const STRONG: Color = rgb(STRONG_HEX);
pub(crate) const SOFT: Color = rgb(SOFT_HEX);
pub(crate) const MUTED: Color = rgb(MUTED_HEX);
pub(crate) const ACCENT: Color = rgb(ACCENT_HEX);
pub(crate) const BLUE: Color = rgb(BLUE_HEX);

/// A figure's colour by Kante's label name (`ScriptLine` chips: `purple`, `blue` …); the
/// text colour for `""` and unknown names.
pub(crate) fn label(name: &str) -> Color {
    match name {
        "purple" => rgb(PURPLE_HEX),
        "blue" => rgb(BLUE_HEX),
        "green" => rgb(GREEN_HEX),
        "yellow" => rgb(ACCENT_HEX),
        "orange" => rgb(ORANGE_HEX),
        "red" => rgb(RED_HEX),
        "cyan" => rgb(CYAN_HEX),
        _ => SOFT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PALETTE: &str = include_str!("../../export/kante/palette.json");

    #[test]
    fn colours_are_in_the_kante_palette() {
        for hex in ALL_HEX {
            let text = format!("\"#{hex:06x}\"");
            assert!(PALETTE.contains(&text), "{text} is not in palette.json");
        }
    }
}
