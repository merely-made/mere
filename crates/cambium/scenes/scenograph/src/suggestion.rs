// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Suggestion settings (graph grammar, DECISIONS 2026-10-16, *Suggestions
//! and derived content*): per host, all the person's to change, and off by
//! default (Mark, 2026-10-16).
//!
//! The number of things the person keeps track of never limits their
//! content. It only paces suggestions: they taper over its last tenth, and
//! again as the person's content fills the plate.

use serde::{Deserialize, Serialize};

/// How a suggestion is drawn: the smallest mark, by the person's choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionMark {
    Dot,
    Ring,
    Glyph,
    /// The favicon where there is one, else the derived glyph.
    #[default]
    Face,
    Word,
}

impl SuggestionMark {
    pub fn id(self) -> &'static str {
        match self {
            Self::Dot => "dot",
            Self::Ring => "ring",
            Self::Glyph => "glyph",
            Self::Face => "face",
            Self::Word => "word",
        }
    }
}

/// One host's suggestion settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SuggestionSettings {
    pub on: bool,
    /// How many suggestions show at once.
    pub at_once: u32,
    /// How many things the person keeps track of (200 or 5000 are both
    /// reasonable).
    pub tracked: u32,
    pub mark: SuggestionMark,
    /// The sources suggestions may come from; empty means every source.
    pub sources: Vec<String>,
}

impl Default for SuggestionSettings {
    fn default() -> Self {
        Self {
            on: false,
            at_once: 6,
            tracked: 1000,
            mark: SuggestionMark::Face,
            sources: Vec::new(),
        }
    }
}

impl SuggestionSettings {
    /// A host's defaults: off, with DECISIONS Open 8's proposed pace and mark.
    /// Unknown hosts take [`Self::default`].
    pub fn for_host(host: &str) -> Self {
        let (at_once, tracked, mark) = match host {
            "graphshell" => (8, 2000, SuggestionMark::Face),
            "turnstone" => (6, 5000, SuggestionMark::Face),
            "woodshed" => (5, 500, SuggestionMark::Word),
            "knot" => (6, 1000, SuggestionMark::Word),
            "signalman" => (10, 300, SuggestionMark::Glyph),
            "djinn" => (4, 200, SuggestionMark::Ring),
            "distillery" => (4, 1000, SuggestionMark::Ring),
            _ => return Self::default(),
        };
        Self {
            at_once,
            tracked,
            mark,
            ..Self::default()
        }
    }

    /// How many suggestions may show now, with `tracked_now` things kept and
    /// the plate `plate_fill` full (0 to 1).
    pub fn budget(&self, tracked_now: usize, plate_fill: f32) -> usize {
        if !self.on || self.tracked == 0 {
            return 0;
        }
        let tracked = self.tracked as f32;
        let taper_from = tracked * 0.9;
        let now = tracked_now as f32;
        let taper = if now <= taper_from {
            1.0
        } else {
            ((tracked - now) / (tracked - taper_from)).max(0.0)
        };
        let room = 1.0 - plate_fill.clamp(0.0, 1.0);
        (self.at_once as f32 * taper * room).round() as usize
    }

    /// The suggestions to show from `candidates` (each with its source), in
    /// order, within the budget and the allowed sources.
    pub fn admit<K>(
        &self,
        candidates: impl IntoIterator<Item = (K, String)>,
        tracked_now: usize,
        plate_fill: f32,
    ) -> Vec<K> {
        let budget = self.budget(tracked_now, plate_fill);
        candidates
            .into_iter()
            .filter(|(_, source)| self.sources.is_empty() || self.sources.contains(source))
            .take(budget)
            .map(|(key, _)| key)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(at_once: u32, tracked: u32) -> SuggestionSettings {
        SuggestionSettings {
            on: true,
            at_once,
            tracked,
            ..SuggestionSettings::default()
        }
    }

    #[test]
    fn off_by_default_for_every_host() {
        for host in [
            "graphshell",
            "turnstone",
            "woodshed",
            "knot",
            "signalman",
            "djinn",
            "distillery",
            "other",
        ] {
            let settings = SuggestionSettings::for_host(host);
            assert!(!settings.on, "{host}");
            assert_eq!(settings.budget(0, 0.0), 0);
        }
        assert_eq!(
            SuggestionSettings::for_host("signalman").mark,
            SuggestionMark::Glyph
        );
    }

    #[test]
    fn suggestions_taper_over_the_last_tenth_and_as_the_plate_fills() {
        let settings = on(10, 1000);
        assert_eq!(settings.budget(100, 0.0), 10);
        assert_eq!(settings.budget(900, 0.0), 10);
        assert_eq!(settings.budget(950, 0.0), 5);
        assert_eq!(settings.budget(1000, 0.0), 0);
        assert_eq!(
            settings.budget(4000, 0.0),
            0,
            "content past the number is never limited; suggestions are"
        );
        assert_eq!(settings.budget(100, 0.5), 5);
        assert_eq!(settings.budget(100, 1.0), 0);
    }

    #[test]
    fn admit_keeps_order_within_budget_and_sources() {
        let mut settings = on(2, 100);
        let offers = || {
            vec![
                ("a", "web".to_string()),
                ("b", "mesh".to_string()),
                ("c", "web".to_string()),
            ]
        };
        assert_eq!(settings.admit(offers(), 0, 0.0), ["a", "b"]);
        settings.sources = vec!["web".into()];
        assert_eq!(settings.admit(offers(), 0, 0.0), ["a", "c"]);
    }

    #[test]
    fn settings_round_trip_and_fill_missing_fields() {
        let settings = on(3, 200);
        let json = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<SuggestionSettings>(&json).unwrap(),
            settings
        );
        let sparse: SuggestionSettings =
            serde_json::from_str(r#"{"on":true,"mark":"ring"}"#).unwrap();
        assert_eq!(
            (sparse.on, sparse.mark, sparse.at_once),
            (true, SuggestionMark::Ring, 6)
        );
    }
}
