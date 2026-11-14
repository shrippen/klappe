//! Kante's dark colours (Gruvbox base) for the terminal.
//!
//! The values are copied from `crates/export/kante/palette.json`, not read from it at run
//! time: a terminal frame wants plain constants. The tests below fail when a constant no
//! longer has the colour the app gives its role (`app.dark`).

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
/// `semantic.aqua.bright`: the app's green label.
const GREEN_HEX: u32 = 0x8e_c0_7c;
/// `semantic.orange.bright`.
const ORANGE_HEX: u32 = 0xfe_80_19;
/// `semantic.red.bright`.
const RED_HEX: u32 = 0xfb_49_34;
/// `semantic.purple.bright`.
const PURPLE_HEX: u32 = 0xd3_86_9b;
/// `semantic.cyan.bright`.
const CYAN_HEX: u32 = 0x5c_cf_c4;

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
    use serde_json::Value;

    const PALETTE: &str = include_str!("../../export/kante/palette.json");

    fn palette() -> Value {
        serde_json::from_str(PALETTE).unwrap_or_else(|e| unreachable!("palette.json: {e}"))
    }

    /// A token of `palette.json` as the app resolves it: a literal `#rrggbb`, a background
    /// or foreground name, or a semantic colour's bright shade.
    fn resolve(palette: &Value, token: &str) -> Color {
        let hex = if token.starts_with('#') {
            Some(token)
        } else {
            palette["backgrounds"][token]
                .as_str()
                .or_else(|| palette["foregrounds"][token].as_str())
                .or_else(|| palette["semantic"][token]["bright"].as_str())
        };
        let hex = hex.unwrap_or_else(|| unreachable!("unknown token {token}"));
        let value = u32::from_str_radix(hex.trim_start_matches('#'), 16)
            .unwrap_or_else(|e| unreachable!("{hex}: {e}"));

        rgb(value)
    }

    /// The colour of a role in `app.dark`.
    fn role(palette: &Value, name: &str) -> Color {
        let token = palette["app"]["dark"][name]
            .as_str()
            .unwrap_or_else(|| unreachable!("no app.dark.{name}"));

        resolve(palette, token)
    }

    #[test]
    fn labels_follow_the_app() {
        let palette = palette();
        for name in ["purple", "blue", "green", "yellow", "orange", "red", "cyan"] {
            let role_name = format!("label{}{}", name[..1].to_uppercase(), &name[1..]);
            assert_eq!(label(name), role(&palette, &role_name), "{name}");
        }
    }
}
