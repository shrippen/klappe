//! How the way of a camera is drawn: split by rig, one line form each (LH-33).

use klappe_model::{CameraMark, Dash, Rig};

/// How a rig's way is drawn: a rail solid, a dolly dashed, everything freer dotted (LH-33).
/// Decided here once; screen and printout only translate the dash.
pub(crate) fn rig_dash(rig: Rig) -> Dash {
    match rig {
        Rig::Tripod | Rig::Track => Dash::Solid,
        Rig::Dolly => Dash::Dashed,
        Rig::Crane | Rig::Handheld | Rig::Gimbal => Dash::Dotted,
    }
}

/// The ways of a camera, split where the rig changes: each run starts at the mark before its
/// first way, so the runs join. The rig of a way is that of the mark it leads to.
///
/// ```text
///  marks   1     2     3     4
///  rig        dolly dolly crane
///  runs    [1 2 3]    [3 4]
/// ```
pub(crate) fn rig_runs(marks: &[CameraMark]) -> Vec<&[CameraMark]> {
    let mut runs = Vec::new();
    let mut start = 0;

    for i in 1..marks.len() {
        let ends = marks.get(i + 1).is_none_or(|next| next.rig != marks[i].rig);
        if !ends {
            continue;
        }

        runs.push(&marks[start..=i]);
        start = i;
    }

    runs
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::Point;

    fn runs(rigs: &[Rig]) -> Vec<(Dash, usize)> {
        let marks: Vec<CameraMark> = rigs
            .iter()
            .enumerate()
            .map(|(i, rig)| CameraMark {
                at_s: i as f64,
                at: Point::new(i as f64, 0.0),
                facing_deg: 0.0,
                height_m: 1.6,
                tilt_deg: 0.0,
                hold_s: 0.0,
                via: None,
                rig: *rig,
                lens_mm: None,
            })
            .collect();

        rig_runs(&marks)
            .iter()
            .map(|run| (rig_dash(run[1].rig), run.len()))
            .collect()
    }

    /// A way is drawn in the line of the rig it leads to; neighbours with one rig join, and
    /// the first mark's rig counts for nothing (LH-33).
    #[test]
    fn a_camera_path_changes_its_line_with_the_rig() {
        let rigs = [
            Rig::Crane,
            Rig::Dolly,
            Rig::Dolly,
            Rig::Track,
            Rig::Handheld,
        ];

        assert_eq!(
            runs(&rigs),
            [(Dash::Dashed, 3), (Dash::Solid, 2), (Dash::Dotted, 2)]
        );
    }

    #[test]
    fn one_mark_has_no_way() {
        assert!(runs(&[Rig::Dolly]).is_empty());
        assert!(runs(&[]).is_empty());
    }
}
