//! The register of what is shot (DATENMODELL §3, §4): the kind of an entry, whether the book
//! still holds it, the fingerprints of its lines, and the random ids entries get.

use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::{BuildHasher, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// What an entry of the register is: a scene of the screenplay, or a content item, a unit of
/// the shoot that is no scene of a book (F-07a).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Scene,
    Interview,
    Statement,
    Cutaways,
    Vignette,
    Intro,
    Vlog,
}

/// A scene number of the Sperrstand (DB-33): the numbers of the book as Klappe saved it last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedNumber {
    pub number: String,
    /// The register entry that held the scene; empty for a struck heading.
    pub id: String,
}

/// Whether the screenplay still holds an entry. A scene struck from the book stays with its
/// plans until the user assigns or deletes it (DATENMODELL §4).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookState {
    #[default]
    InBook,
    NotInBook,
}

/// Kind and hash of one element of a scene (what `klappe_script::Fingerprint` has), kept to
/// find the scene again after the book changed. Written as `<tag>:<hash in 8 hex digits>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Print {
    pub tag: u8,
    pub hash: u32,
}

impl fmt::Display for Print {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:08x}", self.tag, self.hash)
    }
}

impl FromStr for Print {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let bad = || format!("{text:?} is not a fingerprint like 1:0a1b2c3d");
        let (tag, hash) = text.split_once(':').ok_or_else(bad)?;

        Ok(Print {
            tag: tag.parse().map_err(|_| bad())?,
            hash: u32::from_str_radix(hash, 16).map_err(|_| bad())?,
        })
    }
}

impl Serialize for Print {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Print {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;

        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Names the content of a scene of the book (number, heading, prints), so an answer of the
/// user about that scene still applies after the book moved, but not after it changed.
/// Written as 16 hex digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneKey(pub u64);

impl fmt::Display for SceneKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl FromStr for SceneKey {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let bad = || format!("{text:?} is not a scene key like 0a1b2c3d4e5f6071");
        if text.len() != 16 {
            return Err(bad());
        }

        u64::from_str_radix(text, 16)
            .map(SceneKey)
            .map_err(|_| bad())
    }
}

impl Serialize for SceneKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for SceneKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;

        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Characters of an id.
const ID_LEN: usize = 8;
const ID_BITS: u32 = 5;
/// Lower-case base32 (RFC 4648): no look-alikes of `0`, `1` among the digits.
const ID_ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// A new id, short and random, so that two people adding an entry at the same time in Git do
/// not get the same id (DATENMODELL §2). Seeded by the operating system through the keys of
/// `RandomState`, plus a counter and the clock so that no two calls repeat; a caller that needs
/// a free id takes `free_random_id`.
pub fn random_id() -> String {
    static CALLS: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(CALLS.fetch_add(1, Ordering::Relaxed));
    hasher.write_u128(nanos);

    id_from_seed(hasher.finish())
}

/// How a taken id is compared with a new one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IdCase {
    Exact,
    /// For ids that are file names, which may not differ by case only.
    Ignored,
}

/// A `random_id` that none of `taken` has.
pub fn free_random_id<'a>(
    taken: impl IntoIterator<Item = &'a str> + Clone,
    case: IdCase,
) -> String {
    loop {
        let id = random_id();
        if !taken.clone().into_iter().any(|t| is_taken(t, &id, case)) {
            return id;
        }
    }
}

/// Whether the taken id `taken` blocks `id`.
fn is_taken(taken: &str, id: &str, case: IdCase) -> bool {
    match case {
        IdCase::Exact => taken == id,
        IdCase::Ignored => taken.eq_ignore_ascii_case(id),
    }
}

/// The id a seed gives: the same seed, the same id. For fixtures that must come out the same
/// every time (the demo); everything else uses `random_id`.
pub fn id_from_seed(seed: u64) -> String {
    let mask = (1u64 << ID_BITS) - 1;

    (0..ID_LEN)
        .map(|i| ID_ALPHABET[((seed >> (ID_BITS * i as u32)) & mask) as usize] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn random_ids_are_short_lowercase_and_differ() {
        let ids: HashSet<String> = (0..1000).map(|_| random_id()).collect();

        assert_eq!(ids.len(), 1000);
        for id in &ids {
            assert_eq!(id.len(), ID_LEN);
            assert!(id.bytes().all(|b| ID_ALPHABET.contains(&b)), "{id}");
        }
    }

    #[test]
    fn a_free_id_is_never_a_taken_one() {
        let taken: Vec<String> = (0..1000).map(|_| random_id()).collect();
        let refs = || taken.iter().map(String::as_str);

        for _ in 0..200 {
            assert!(!taken.contains(&free_random_id(refs(), IdCase::Exact)));
        }
    }

    /// Scene ids are file names: another case of a taken id is taken too.
    #[test]
    fn the_case_rule_decides_what_is_taken() {
        assert!(is_taken("abcdefgh", "abcdefgh", IdCase::Exact));
        assert!(!is_taken("ABCDEFGH", "abcdefgh", IdCase::Exact));
        assert!(is_taken("ABCDEFGH", "abcdefgh", IdCase::Ignored));
        assert!(!is_taken("abcdefgx", "abcdefgh", IdCase::Ignored));
    }

    #[test]
    fn a_seed_gives_the_same_id() {
        assert_eq!(id_from_seed(42), id_from_seed(42));
        assert_ne!(id_from_seed(42), id_from_seed(43));
    }

    #[test]
    fn a_print_survives_its_text() {
        let print = Print {
            tag: 2,
            hash: 0xab12_c3d4,
        };

        assert_eq!(print.to_string(), "2:ab12c3d4");
        assert_eq!("2:ab12c3d4".parse::<Print>(), Ok(print));
        assert!("ab12".parse::<Print>().is_err());
        assert!("x:ab12c3d4".parse::<Print>().is_err());
    }
}
