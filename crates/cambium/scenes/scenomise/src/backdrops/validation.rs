// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl SceneInput {
    pub fn parse(json: &str) -> Result<Self, SceneInputError> {
        check(
            json.len() <= MAX_SCENE_INPUT_BYTES,
            "input",
            "input exceeds one MiB",
        )?;
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| error("input", e.to_string()))?;
        strict_core_records(&value, "input")?;
        let input: Self =
            serde_json::from_value(value).map_err(|e| error("input", e.to_string()))?;
        input.validate()?;
        Ok(input)
    }
    pub fn validate(&self) -> Result<(), SceneInputError> {
        check(
            self.version == BACKDROP_SCENE_VERSION,
            "version",
            "unsupported backdrop scene version",
        )?;
        source(&self.source, "source")?;
        check(
            !self.revision.trim().is_empty(),
            "revision",
            "a disclosed revision is required",
        )?;
        match &self.facts {
            SceneFacts::MetreGrid(grid) => {
                check(
                    (0..=32).contains(&grid.feet)
                        && (1..=8).contains(&grid.positions_per_foot)
                        && grid.lines.len() <= 256,
                    "metre",
                    "metre extent exceeds limits",
                )?;
                unique(grid.lines.iter().map(|l| l.key.as_str()), "metre.lines")?;
                let keys: Vec<String> = grid
                    .lines
                    .iter()
                    .flat_map(|line| {
                        std::iter::once(line.key.clone()).chain(
                            (0..line.syllables.len()).map(|i| format!("{}/syllable/{i}", line.key)),
                        )
                    })
                    .collect();
                unique(keys.iter().map(String::as_str), "metre.occurrences")?;
                let mut count = grid.lines.len();
                for line in &grid.lines {
                    source(&line.source, "metre.line.source")?;
                    check(
                        line.syllables.len() <= 512,
                        "metre.line.syllables",
                        "too many syllables",
                    )?;
                    if let Some(alignment) = &line.alignment {
                        check(
                            alignment.len() == line.syllables.len()
                                && alignment.iter().flatten().all(|span| {
                                    span.length > 0
                                        && span.end().is_some_and(|end| end <= grid.extent())
                                }),
                            "metre.line.alignment",
                            "one bounded positive span or unplaced value is required per syllable",
                        )?;
                    }
                    count += line.syllables.len();
                    check(
                        line.rhyme_tail
                            .as_ref()
                            .is_none_or(|s| !s.trim().is_empty())
                            && line
                                .rhyme_group
                                .as_ref()
                                .is_none_or(|s| !s.trim().is_empty()),
                        "metre.line.rhyme",
                        "unknown rhyme values must be absent, not blank",
                    )?;
                }
                check(count <= MAX_MARKS, "metre", "too many marks")?;
            },
            SceneFacts::TemplatedSequence(sequence) => {
                sequence.validate().map_err(|e| error("template", e))?
            },
            SceneFacts::Fretboard(board) => {
                check(
                    board.strings.iter().all(|s| s.open_midi <= 127),
                    "fretboard.strings",
                    "tuning must use valid MIDI pitches",
                )?;
                check(
                    !board.strings.is_empty()
                        && board.strings.len() <= 24
                        && (1..=48).contains(&board.frets)
                        && board.positions.len() <= MAX_MARKS,
                    "fretboard",
                    "instrument extent exceeds limits",
                )?;
                unique(
                    board.positions.iter().map(|p| p.key.as_str()),
                    "fretboard.positions",
                )?;
                let mut occupied = HashSet::new();
                for p in &board.positions {
                    source(&p.source, "fretboard.position.source")?;
                    let Some(string) = board.strings.get(p.string_index as usize) else {
                        return Err(error("fretboard.position.string_index", "missing string"));
                    };
                    check(
                        p.fret <= board.frets
                            && p.pitch_class < 12
                            && (u16::from(string.open_midi) + p.fret) <= 127
                            && (u16::from(string.open_midi) + p.fret) % 12
                                == u16::from(p.pitch_class),
                        "fretboard.position",
                        "position disagrees with the disclosed tuning or pitch class",
                    )?;
                    check(
                        occupied.insert((p.string_index, p.fret)),
                        "fretboard.positions",
                        "two occurrences occupy one instrument position",
                    )?;
                }
            },
            SceneFacts::TerrainCoverage(map) => {
                let bound = map.extent.w.max(map.extent.h) * 4.0;
                check(
                    map.contours
                        .iter()
                        .chain(map.unmeasured.iter())
                        .flat_map(|v| v.iter())
                        .chain(map.coverage.iter().flat_map(|c| c.contour.iter()))
                        .all(|p| p.x.abs() <= bound && p.y.abs() <= bound),
                    "terrain.contours",
                    "geometry exceeds the disclosed plane's supported margin",
                )?;
                let vertices: usize = map
                    .contours
                    .iter()
                    .map(Vec::len)
                    .chain(map.unmeasured.iter().map(Vec::len))
                    .chain(map.coverage.iter().map(|c| c.contour.len()))
                    .sum();
                check(
                    vertices <= 131_072,
                    "terrain",
                    "total contour budget exceeded",
                )?;
                check(
                    map.extent.w.is_finite()
                        && map.extent.h.is_finite()
                        && map.extent.w > 0.0
                        && map.extent.h > 0.0
                        && map.extent.w <= 100_000.0
                        && map.extent.h <= 100_000.0,
                    "terrain.extent",
                    "extent must be finite and positive",
                )?;
                check(
                    map.devices.len() <= 512
                        && map.contours.len() <= 512
                        && map.coverage.len() <= 1536
                        && map.unmeasured.len() <= 512
                        && map.links.len() <= 4096,
                    "terrain",
                    "scene exceeds geometry limits",
                )?;
                unique(
                    map.devices.iter().map(|d| d.key.as_str()),
                    "terrain.devices",
                )?;
                for device in &map.devices {
                    source(&device.source, "terrain.device.source")?;
                    if let Some(p) = device.position {
                        check(
                            finite(p)
                                && p.x >= 0.0
                                && p.y >= 0.0
                                && p.x <= map.extent.w
                                && p.y <= map.extent.h,
                            "terrain.device.position",
                            "position lies outside its disclosed plane",
                        )?;
                    }
                }
                for points in &map.contours {
                    poly(points, 2, "terrain.contours")?;
                }
                for points in &map.unmeasured {
                    poly(points, 3, "terrain.unmeasured")?;
                }
                for c in &map.coverage {
                    source(&c.source, "terrain.coverage.source")?;
                    poly(&c.contour, 3, "terrain.coverage.contour")?;
                    check(
                        !c.basis.trim().is_empty(),
                        "terrain.coverage.basis",
                        "coverage needs its disclosed basis",
                    )?;
                    check(
                        map.devices
                            .iter()
                            .any(|d| d.key == c.device && d.position.is_some()),
                        "terrain.coverage.device",
                        "coverage requires a placed device",
                    )?;
                }
                for link in &map.links {
                    check(
                        map.devices.iter().any(|d| d.key == link.from)
                            && map.devices.iter().any(|d| d.key == link.to)
                            && !link.why.trim().is_empty(),
                        "terrain.links",
                        "link needs known endpoints and a reason",
                    )?;
                }
            },
        }
        Ok(())
    }
    /// Pure compilation. Rejection cannot mutate a host's active scene.
    pub fn compile(&self) -> Result<BackdropScene, SceneInputError> {
        self.validate()?;
        let mut out = BackdropScene {
            version: BACKDROP_SCENE_VERSION,
            revision: self.revision.clone(),
            title: self.title.clone(),
            scene: Scene::new(),
            marks: vec![],
            guides: vec![],
            relations: vec![],
            fields: vec![],
            unplaced: vec![],
            diagnostics: vec![],
        };
        let src = out.scene.intern_source(self.source.clone());
        match &self.facts {
            SceneFacts::MetreGrid(grid) => compile_metre(&mut out, src, grid),
            SceneFacts::TemplatedSequence(sequence) => compile_sequence(&mut out, src, sequence),
            SceneFacts::Fretboard(board) => compile_fretboard(&mut out, src, board),
            SceneFacts::TerrainCoverage(map) => compile_terrain(&mut out, src, map),
        }
        unique(out.marks.iter().map(|m| m.key.as_str()), "marks")?;
        let bytes = serde_json::to_vec(self).map_err(|e| error("input", e.to_string()))?;
        out.scene.generation = bytes.into_iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        });
        Ok(out)
    }
}
