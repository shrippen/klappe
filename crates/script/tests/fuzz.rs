//! Random edits of generated scripts with the true lineage of every scene and line: a
//! `Matched` scene and an `At` line must be the same one the edits carried along. Scenes
//! and lines are drawn from a small vocabulary, so equal lines and twins occur; half of
//! the scenes have no line of their own, half of the inserted lines come from the
//! vocabulary, lines move, and no two quotes may land on one line.
//!
//! Only what the text cannot tell is excused, and counted: a scene that became equal in
//! content to its old one (twins), and a line for which reading it as the found line
//! explains the new script with no more edits (changes, moves, insertions, deletions)
//! than the true lineage, or a diff of the two scenes could pair them (an inserted "Ja."
//! next to an edited one, equal lines in another order).
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use klappe_script::{align, parse, quote_at, scene_states, Element, Found, Outcome, SceneState};

const SEEDS: u64 = 1000;
const HEADS: [&str; 3] = ["KÜCHE", "SAAL", "HOF"];
const SPEAKERS: [&str; 3] = ["ANNA", "BEN", "CARLA"];
const ACTIONS: [&str; 4] = ["Er wartet.", "Sie geht zur Tür.", "Es regnet.", "Stille."];
const TALKS: [&str; 5] = ["Ja.", "Nein.", "Doch.", "Komm her.", "Warum?"];
const MAX_OPS: u64 = 5;
const LCG_MUL: u64 = 6_364_136_223_846_793_005;
const LCG_ADD: u64 = 1_442_695_040_888_963_407;

#[derive(Clone, Debug)]
struct Block {
    id: u32,
    speaker: Option<&'static str>,
    text: String,
}

#[derive(Clone, Debug)]
struct SceneM {
    id: u32,
    number: u32,
    head: usize,
    blocks: Vec<Block>,
}

struct Gen {
    state: u64,
    next_id: u32,
}

impl Gen {
    fn below(&mut self, n: usize) -> usize {
        self.state = self.state.wrapping_mul(LCG_MUL).wrapping_add(LCG_ADD);
        ((self.state >> 33) as usize) % n
    }

    fn id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    fn block(&mut self) -> Block {
        let id = self.id();
        if self.below(2) == 0 {
            return Block {
                id,
                speaker: None,
                text: ACTIONS[self.below(ACTIONS.len())].into(),
            };
        }
        Block {
            id,
            speaker: Some(SPEAKERS[self.below(SPEAKERS.len())]),
            text: TALKS[self.below(TALKS.len())].into(),
        }
    }

    /// A line for an insertion: half from the vocabulary, half with text no other has.
    fn inserted(&mut self) -> Block {
        let mut b = self.block();
        if self.below(2) == 0 {
            return b;
        }
        b.text = format!("Neu {}.", b.id);
        b
    }

    /// A scene, half of them with a line only they have.
    fn scene(&mut self, number: u32) -> SceneM {
        let id = self.id();
        let mut blocks = Vec::new();
        if self.below(2) == 0 {
            blocks.push(Block {
                id: self.id(),
                speaker: None,
                text: format!("Szene {id} beginnt."),
            });
        }
        for _ in 0..3 + self.below(4) {
            blocks.push(self.block());
        }
        SceneM {
            id,
            number,
            head: self.below(HEADS.len()),
            blocks,
        }
    }

    fn script(&mut self) -> Vec<SceneM> {
        let n = 5 + self.below(5);
        (1..=n as u32).map(|i| self.scene(i)).collect()
    }

    fn edit(&mut self, s: &mut Vec<SceneM>) {
        if s.is_empty() {
            return;
        }
        let k = self.below(s.len());
        match self.below(11) {
            0 => {
                s.remove(k);
            }
            1 => {
                let number = 100 + self.below(900) as u32;
                let at = self.below(s.len() + 1);
                s.insert(at, self.scene(number));
            }
            2 => {
                let moved = s.remove(k);
                let at = self.below(s.len() + 1);
                s.insert(at, moved);
            }
            3 => {
                let shift = 1 + self.below(20) as u32;
                for sc in s.iter_mut() {
                    sc.number += shift;
                }
            }
            4 => s[k].head = self.below(HEADS.len()),
            op => self.edit_lines(op, &mut s[k]),
        }
    }

    /// The edits of the lines of one scene.
    fn edit_lines(&mut self, op: usize, sc: &mut SceneM) {
        let n = sc.blocks.len();
        if op == 5 {
            let at = self.below(n + 1);
            let b = self.inserted();
            sc.blocks.insert(at, b);
            return;
        }
        if n == 0 {
            return;
        }

        let i = self.below(n);
        match op {
            6 => {
                sc.blocks.remove(i);
            }
            7 => self.change(sc),
            8 => {
                let b = sc.blocks.remove(i);
                let at = self.below(n);
                sc.blocks.insert(at, b);
            }
            9 => {
                let other = SPEAKERS[self.below(SPEAKERS.len())];
                if sc.blocks[i].speaker.is_some() {
                    sc.blocks[i].speaker = Some(other);
                }
            }
            _ => {
                let j = self.below(n);
                // Swapping equal lines changes nothing a reader could tell.
                let (x, y) = (&sc.blocks[i], &sc.blocks[j]);
                if (x.speaker, &x.text) != (y.speaker, &y.text) {
                    sc.blocks.swap(i, j);
                }
            }
        }
    }

    fn change(&mut self, sc: &mut SceneM) {
        let n = sc.blocks.len();
        if n == 0 {
            return;
        }
        let i = self.below(n);
        let id = self.id();
        sc.blocks[i].text = format!("Geändert {id}.");
    }
}

fn render(scenes: &[SceneM]) -> String {
    let mut out = String::new();
    for s in scenes {
        out.push_str(&format!("INT. {} – TAG #{}#\n\n", HEADS[s.head], s.number));
        for b in &s.blocks {
            if let Some(who) = b.speaker {
                out.push_str(&format!("{who}\n"));
            }
            out.push_str(&format!("{}\n\n", b.text));
        }
    }
    out
}

/// Marks the id of a cue line: the cue and the speech of a block share its id.
const CUE: u32 = 1 << 31;

/// The block id of every element of the scene, `CUE` set on cue lines.
fn ids(s: &SceneM) -> Vec<u32> {
    let mut out = Vec::new();
    for b in &s.blocks {
        if b.speaker.is_some() {
            out.push(b.id | CUE);
        }
        out.push(b.id);
    }
    out
}

/// The content of a scene without its number.
fn content(s: &SceneM) -> (usize, Vec<(Option<&'static str>, &str)>) {
    let blocks = s
        .blocks
        .iter()
        .map(|b| (b.speaker, b.text.as_str()))
        .collect();
    (s.head, blocks)
}

#[derive(Default)]
struct Tally {
    scenes: usize,
    lines: usize,
    excused: usize,
}

/// Aligns `old` to `new` and checks every `Matched` scene and every `At` line against the
/// lineage, and that no two quotes land on one line.
fn check(seed: u64, old: &[SceneM], new: &[SceneM], tally: &mut Tally) {
    let (old_text, new_text) = (render(old), render(new));
    let (old_script, new_script) = (parse(&old_text), parse(&new_text));
    let (old_views, new_views) = (old_script.scenes(), new_script.scenes());
    assert_eq!(old_views.len(), old.len(), "{old_text}");
    assert_eq!(new_views.len(), new.len(), "{new_text}");

    let states = scene_states(&old_script);
    let al = align(&states, &new_script);

    for (o, outcome) in al.old.iter().enumerate() {
        let Outcome::Matched { scene: n } = outcome else {
            continue;
        };
        let msg = format!(
            "seed {seed}, old scene {o} -> {n}\n--- old\n{old_text}--- new\n{new_text}\n{:?}\n{:?}",
            old[o], new[*n]
        );
        tally.scenes += 1;
        if new[*n].id != old[o].id {
            // Twins: the new scene reads exactly like the old one.
            assert_eq!(content(&new[*n]), content(&old[o]), "{msg}");
            tally.excused += 1;
        }

        let (old_ids, new_ids) = (ids(&old[o]), ids(&new[*n]));
        assert_eq!(old_views[o].elements.len(), old_ids.len(), "{msg}");
        assert_eq!(new_views[*n].elements.len(), new_ids.len(), "{msg}");

        let mut landed: Vec<usize> = Vec::new();
        for (i, id) in old_ids.iter().enumerate() {
            let q = quote_at(old_views[o].elements, i).unwrap();
            let Found::At(j) = find(&states[o], new_views[*n].elements, &q) else {
                continue;
            };
            assert!(!landed.contains(&j), "{msg}\nline {i} found at {j} again");
            landed.push(j);

            tally.lines += 1;
            if new_ids[j] != *id {
                assert!(
                    excused(&old[o], &new[*n], *id, new_ids[j]),
                    "{msg}\nline {i} found at {j}"
                );
                tally.excused += 1;
            }
        }
    }
}

/// The text cannot tell the old line `id` from the found line `found` if reading `id` as
/// `found` explains the new script with no more edits (changes, moves, insertions,
/// deletions) than the true lineage does: a "Stille." edited and another inserted next to
/// it, or equal lines in another order.
fn excused(old: &SceneM, new: &SceneM, id: u32, found: u32) -> bool {
    if (id & CUE) != (found & CUE) {
        return false;
    }
    let cue = id & CUE != 0;
    let (id, found) = (id & !CUE, found & !CUE);

    let truth: HashMap<u32, u32> = old
        .blocks
        .iter()
        .filter(|b| new.blocks.iter().any(|n| n.id == b.id))
        .map(|b| (b.id, b.id))
        .collect();

    // Read `id` as `found`; the block that was `found` takes over `id`'s successor.
    let mut alt = truth.clone();
    let successor = alt.remove(&id);
    alt.insert(id, found);
    if found != id && old.blocks.iter().any(|b| b.id == found) {
        alt.remove(&found);
        if let Some(s) = successor {
            alt.insert(found, s);
        }
    }

    edits(old, new, &alt) <= edits(old, new, &truth) || same_diff(old, new, id, found, cue)
}

/// Some longest common subsequence of the two scenes' lines pairs `id` with `found`: a
/// diff, which knows nothing of lineage, could pick either. For a cue line only the
/// speaker counts.
fn same_diff(old: &SceneM, new: &SceneM, id: u32, found: u32, cue: bool) -> bool {
    let key = |b: &Block| {
        if !cue {
            return Some((b.speaker, b.text.clone()));
        }
        b.speaker.map(|s| (Some(s), String::new()))
    };
    let side = |s: &SceneM| -> Vec<(u32, (Option<&'static str>, String))> {
        s.blocks
            .iter()
            .filter_map(|b| key(b).map(|k| (b.id, k)))
            .collect()
    };
    let (a, b) = (side(old), side(new));
    let (Some(i), Some(j)) = (
        a.iter().position(|x| x.0 == id),
        b.iter().position(|x| x.0 == found),
    ) else {
        return false;
    };
    if a[i].1 != b[j].1 {
        return false;
    }

    // lcs[x][y]: longest common subsequence of a[x..] and b[y..].
    let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for x in (0..a.len()).rev() {
        for y in (0..b.len()).rev() {
            lcs[x][y] = if a[x].1 == b[y].1 {
                lcs[x + 1][y + 1] + 1
            } else {
                lcs[x + 1][y].max(lcs[x][y + 1])
            };
        }
    }
    // Before the pair, `a[..i]` against `b[..j]`: computed on the reversed prefix.
    let before = {
        let mut t = vec![vec![0usize; j + 1]; i + 1];
        for x in 0..i {
            for y in 0..j {
                t[x + 1][y + 1] = if a[x].1 == b[y].1 {
                    t[x][y] + 1
                } else {
                    t[x][y + 1].max(t[x + 1][y])
                };
            }
        }
        t[i][j]
    };
    before + 1 + lcs[i + 1][j + 1] == lcs[0][0]
}

/// Changes, moves, deletions and insertions that turn `old` into `new` if `map` says which
/// new block each old block became.
fn edits(old: &SceneM, new: &SceneM, map: &HashMap<u32, u32>) -> usize {
    let new_at = |id: u32| new.blocks.iter().position(|b| b.id == id);
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    let mut changed = 0;
    for (i, b) in old.blocks.iter().enumerate() {
        let Some(&to) = map.get(&b.id) else {
            continue;
        };
        let Some(j) = new_at(to) else {
            continue;
        };
        let n = &new.blocks[j];
        changed += usize::from((b.speaker, &b.text) != (n.speaker, &n.text));
        pairs.push((i, j));
    }

    // Longest run that keeps its order; the rest moved.
    let mut best: Vec<usize> = vec![1; pairs.len()];
    for i in 0..pairs.len() {
        for k in 0..i {
            if pairs[k].1 < pairs[i].1 {
                best[i] = best[i].max(best[k] + 1);
            }
        }
    }
    let kept = best.iter().copied().max().unwrap_or(0);
    let lost = (old.blocks.len() - pairs.len()) + (new.blocks.len() - pairs.len());
    changed + (pairs.len() - kept) + lost
}

fn find(old: &SceneState, scene: &[Element], q: &klappe_script::Quote) -> Found {
    klappe_script::find_quote(old, scene, q)
}

#[test]
fn matches_and_lines_agree_with_the_lineage() {
    let mut tally = Tally::default();
    for seed in 1..=SEEDS {
        let mut g = Gen {
            state: seed,
            next_id: 0,
        };
        let old = g.script();
        let mut new = old.clone();
        for _ in 0..1 + g.below(MAX_OPS as usize) {
            g.edit(&mut new);
        }
        check(seed, &old, &new, &mut tally);
    }
    println!(
        "fuzz: {} scenes and {} lines matched, {} excused as unreadable from the text",
        tally.scenes, tally.lines, tally.excused
    );
    assert!(tally.scenes > 0 && tally.lines > 0);
}

#[test]
fn an_unchanged_script_yields_only_matches() {
    for seed in 1..=SEEDS {
        let mut g = Gen {
            state: seed,
            next_id: 0,
        };
        let old = g.script();
        let text = render(&old);
        let script = parse(&text);
        let states = scene_states(&script);
        let al = align(&states, &script);

        for (o, outcome) in al.old.iter().enumerate() {
            assert!(
                matches!(outcome, Outcome::Matched { scene, .. } if *scene == o),
                "seed {seed}, scene {o}: {outcome:?}\n{text}"
            );
        }

        let views = script.scenes();
        for (o, view) in views.iter().enumerate() {
            for i in 0..ids(&old[o]).len() {
                let q = quote_at(view.elements, i).unwrap();
                let found = find(&states[o], view.elements, &q);
                assert_eq!(
                    found,
                    Found::At(i),
                    "seed {seed}, scene {o}, line {i}\n{text}"
                );
            }
        }
        assert!(al.unmatched.is_empty());
    }
}
