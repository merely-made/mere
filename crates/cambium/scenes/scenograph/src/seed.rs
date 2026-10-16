// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Seeds and reset (graph grammar, Merely Design System DECISIONS 2026-10-16;
//! the reset baseline left open by projection-scenes §9.13 and editor plan B1).
//!
//! Reset is a refresh: every unheld occurrence eases back to its seed. The
//! seed is the recipe's starting arrangement: a data anchor where the data
//! has one, otherwise a position derived from the occurrence's identity, the
//! same on every device. Set seed makes the current arrangement the seed; the
//! recipe seed can be restored. Whether a saved seed travels with the recipe
//! is still open, so [`SeedBook`] is view-local and unserialized.

use std::collections::BTreeMap;

use sceno::Vec2;

/// The reset ease, the re-projection morph's duration.
pub const RESET_EASE_MS: u32 = 1050;

/// A point in the disc of `radius` around `center`, from `id` alone.
///
/// Integer hashing and rejection sampling only: no trigonometry, whose last
/// bit may differ between platform maths libraries, so every device agrees.
pub fn identity_seed(id: &str, center: Vec2, radius: f32) -> Vec2 {
    let mut stream = fnv1a(id.as_bytes());
    for _ in 0..64 {
        let x = unit(&mut stream) * 2.0 - 1.0;
        let y = unit(&mut stream) * 2.0 - 1.0;
        if x * x + y * y <= 1.0 {
            return Vec2::new(center.x + x * radius, center.y + y * radius);
        }
    }
    center
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// SplitMix64's next value, as a float in [0, 1) from its top 24 bits.
fn unit(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

/// A view's saved seed, if the person set one.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedBook<K: Ord> {
    saved: Option<BTreeMap<K, Vec2>>,
}

impl<K: Ord> Default for SeedBook<K> {
    fn default() -> Self {
        Self { saved: None }
    }
}

impl<K: Ord> SeedBook<K> {
    /// Set seed: the given arrangement becomes the seed.
    pub fn set(&mut self, positions: impl IntoIterator<Item = (K, Vec2)>) {
        self.saved = Some(positions.into_iter().collect());
    }

    /// Restore the recipe seed. Returns whether a saved seed was dropped.
    pub fn restore_recipe(&mut self) -> bool {
        self.saved.take().is_some()
    }

    pub fn saved(&self) -> Option<&BTreeMap<K, Vec2>> {
        self.saved.as_ref()
    }

    /// The saved seed for `key`, else the recipe's. An occurrence added after
    /// Set seed takes its recipe seed.
    pub fn seed_of(&self, key: &K, recipe: impl FnOnce() -> Option<Vec2>) -> Option<Vec2> {
        self.saved
            .as_ref()
            .and_then(|saved| saved.get(key).copied())
            .or_else(recipe)
    }

    pub fn retain(&mut self, mut live: impl FnMut(&K) -> bool) {
        if let Some(saved) = self.saved.as_mut() {
            saved.retain(|key, _| live(key));
        }
    }
}

/// One occurrence's reset: where it is and the seed it eases back to.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedMove<K> {
    pub key: K,
    pub from: Vec2,
    pub to: Vec2,
}

impl<K> SeedMove<K> {
    /// The position `t` of the way through the ease, `t` in [0, 1]:
    /// symmetric cubic ease-in-out.
    pub fn at(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let e = if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        };
        Vec2::new(
            self.from.x + (self.to.x - self.from.x) * e,
            self.from.y + (self.to.y - self.from.y) * e,
        )
    }
}

/// The moves a reset makes: every occurrence not `held` that has a seed and
/// is away from it. Position pins and anything else the host holds stay.
pub fn reset_moves<K>(
    current: impl IntoIterator<Item = (K, Vec2)>,
    held: impl Fn(&K) -> bool,
    seed: impl Fn(&K) -> Option<Vec2>,
) -> Vec<SeedMove<K>> {
    current
        .into_iter()
        .filter(|(key, _)| !held(key))
        .filter_map(|(key, from)| {
            let to = seed(&key)?;
            (to != from).then_some(SeedMove { key, from, to })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_seed_is_stable_and_inside_its_disc() {
        let center = Vec2::new(10.0, -4.0);
        let a = identity_seed("mere://a", center, 50.0);
        assert_eq!(a, identity_seed("mere://a", center, 50.0));
        assert_ne!(a, identity_seed("mere://b", center, 50.0));
        for i in 0..200 {
            let p = identity_seed(&format!("node-{i}"), center, 50.0);
            let (dx, dy) = (p.x - center.x, p.y - center.y);
            assert!(dx * dx + dy * dy <= 50.0 * 50.0 + 1e-3);
        }
    }

    #[test]
    fn identity_seed_is_pinned_to_its_derivation() {
        // A change here moves every recipe seed on every device: deliberate only.
        let p = identity_seed("mere://a", Vec2::ZERO, 100.0);
        assert_eq!((p.x, p.y), (-66.160416, 49.581028));
    }

    #[test]
    fn saved_seed_wins_then_falls_back_to_the_recipe() {
        let mut book = SeedBook::default();
        let recipe = || Some(Vec2::new(1.0, 1.0));
        assert_eq!(book.seed_of(&"a", recipe), Some(Vec2::new(1.0, 1.0)));
        book.set([("a", Vec2::new(5.0, 5.0))]);
        assert_eq!(book.seed_of(&"a", recipe), Some(Vec2::new(5.0, 5.0)));
        assert_eq!(book.seed_of(&"new", recipe), Some(Vec2::new(1.0, 1.0)));
        assert!(book.restore_recipe());
        assert!(!book.restore_recipe());
        assert_eq!(book.seed_of(&"a", recipe), Some(Vec2::new(1.0, 1.0)));
    }

    #[test]
    fn reset_moves_skip_held_seedless_and_home_occurrences() {
        let current = [
            ("free", Vec2::new(3.0, 0.0)),
            ("pinned", Vec2::new(9.0, 9.0)),
            ("home", Vec2::new(1.0, 1.0)),
            ("seedless", Vec2::new(2.0, 2.0)),
        ];
        let moves = reset_moves(
            current,
            |key| *key == "pinned",
            |key| (*key != "seedless").then_some(Vec2::new(1.0, 1.0)),
        );
        assert_eq!(
            moves,
            vec![SeedMove {
                key: "free",
                from: Vec2::new(3.0, 0.0),
                to: Vec2::new(1.0, 1.0)
            }]
        );
    }

    #[test]
    fn seed_move_eases_from_start_to_seed() {
        let m = SeedMove {
            key: (),
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(10.0, -10.0),
        };
        assert_eq!(m.at(0.0), m.from);
        assert_eq!(m.at(1.0), m.to);
        assert_eq!(m.at(0.5), Vec2::new(5.0, -5.0));
        assert!(m.at(0.25).x < 2.5, "eases in");
    }
}
