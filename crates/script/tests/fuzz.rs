//! Random edits of generated scripts with the true lineage of every scene and line: a
//! `Matched` scene and an `At` line must be the same one the edits carried along. Scenes
//! and lines are drawn from a small vocabulary, so equal lines and twins occur.
#![allow(clippy::unwrap_used)]

use klappe_script::{align, parse, quote_at, scene_states, Element, Found, Outcome, SceneState};

const SEEDS: u64 = 3000;
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

    /// A scene with a line only it has, so scenes are never identical twins.
    fn scene(&mut self, number: u32) -> SceneM {
        let id = self.id();
        let marker = Block {
            id: self.id(),
            speaker: None,
            text: format!("Szene {id} beginnt."),
        };
        let mut blocks = vec![marker];
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
        match self.below(9) {
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
            5 => {
                let at = self.below(s[k].blocks.len() + 1);
                let b = self.block();
                s[k].blocks.insert(at, b);
            }
            6 => {
                let n = s[k].blocks.len();
                s[k].blocks.remove(self.below(n));
            }
            7 => self.change(&mut s[k]),
            _ => {
                let n = s[k].blocks.len();
                let (a, b) = (self.below(n), self.below(n));
                s[k].blocks.swap(a, b);
            }
        }
    }

    fn change(&mut self, sc: &mut SceneM) {
        let n = sc.blocks.len();
        if n == 0 {
            return;
        }
        let i = self.below(n);
        let talk = sc.blocks[i].speaker.is_some();
        sc.blocks[i].text = if talk {
            TALKS[self.below(TALKS.len())].into()
        } else {
            ACTIONS[self.below(ACTIONS.len())].into()
        };
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

/// The block id of every element of the scene (`None` for cue lines).
fn ids(s: &SceneM) -> Vec<Option<u32>> {
    let mut out = Vec::new();
    for b in &s.blocks {
        if b.speaker.is_some() {
            out.push(None);
        }
        out.push(Some(b.id));
    }
    out
}

/// Aligns `old` to `new` and checks every `Matched` scene and every `At` line against the
/// lineage. Returns how many of each were right.
fn check(seed: u64, old: &[SceneM], new: &[SceneM]) -> (usize, usize) {
    let (old_text, new_text) = (render(old), render(new));
    let (old_script, new_script) = (parse(&old_text), parse(&new_text));
    let (old_views, new_views) = (old_script.scenes(), new_script.scenes());
    assert_eq!(old_views.len(), old.len(), "{old_text}");
    assert_eq!(new_views.len(), new.len(), "{new_text}");

    let states = scene_states(&old_script);
    let al = align(&states, &new_script);
    let (mut scenes, mut lines) = (0, 0);

    for (o, outcome) in al.old.iter().enumerate() {
        let Outcome::Matched { scene: n, .. } = outcome else {
            continue;
        };
        let msg =
            format!("seed {seed}, old scene {o} -> {n}\n--- old\n{old_text}--- new\n{new_text}");
        assert_eq!(new[*n].id, old[o].id, "{msg}");
        scenes += 1;

        let (old_ids, new_ids) = (ids(&old[o]), ids(&new[*n]));
        assert_eq!(old_views[o].elements.len(), old_ids.len(), "{msg}");
        assert_eq!(new_views[*n].elements.len(), new_ids.len(), "{msg}");
        for (i, id) in old_ids.iter().enumerate() {
            let Some(id) = id else {
                continue;
            };
            let q = quote_at(old_views[o].elements, i).unwrap();
            let found = find(&states[o], new_views[*n].elements, &q);
            if let Found::At(j) = found {
                assert_eq!(new_ids[j], Some(*id), "{msg}\nline {i} found at {j}");
                lines += 1;
            }
        }
    }
    (scenes, lines)
}

fn find(old: &SceneState, scene: &[Element], q: &klappe_script::Quote) -> Found {
    let _ = old;
    klappe_script::find_quote(scene, q)
}

#[test]
fn matches_and_lines_agree_with_the_lineage() {
    let (mut scenes, mut lines) = (0, 0);
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
        let (s, l) = check(seed, &old, &new);
        scenes += s;
        lines += l;
    }
    println!("fuzz: {scenes} scenes and {lines} lines matched, all agree");
    assert!(scenes > 0 && lines > 0);
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
            for (i, id) in ids(&old[o]).iter().enumerate() {
                if id.is_none() {
                    continue;
                }
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
