// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `wheel` scenario verb: one wheel notch, delivered through the host's own
//! wheel path.
//!
//! ```text
//! wheel <dx> <dy> [lines|px] [shift] [ctrl] [alt] [meta] at <x> <y>
//! wheel <dx> <dy> [lines|px] [shift] [ctrl] [alt] [meta] on <selector>
//! ```
//!
//! `dx` and `dy` are the scroll delta in the direction the content moves, as
//! the host's `wheel` takes it: a positive `dy` advances toward the end of the
//! document. Pixels are the default unit; `lines` counts mouse-wheel notches.
//! `at` names a logical window point; `on` takes the same selector grammar as
//! `click` and turns the wheel at the centre of what it matches. The modifiers
//! are held for the notch only.
//!
//! Taproot parses `click`'s selector but does not export that parser, so
//! [`parse_selector`] restates its grammar; taproot's own scenario tests hold
//! the original, and this module's tests hold the restatement.

use cambium_rootstock::{HostPointer, Modifiers, WheelUnit};
use taproot::Selector;

/// The largest delta a scenario may state, in its own unit: a bound that
/// refuses a typo like `1e30` rather than delivering it.
const LIMIT: f32 = 100_000.0;

/// Where a wheel notch lands.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum WheelAt {
    /// A logical window point.
    Point(f32, f32),
    /// The centre of whatever the selector matches.
    Selector(Selector),
}

/// A parsed `wheel` line.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WheelStep {
    pub dx: f32,
    pub dy: f32,
    pub unit: WheelUnit,
    pub modifiers: Modifiers,
    pub at: WheelAt,
}

impl WheelStep {
    /// The queued host input, once the position is known.
    pub fn at_point(&self, x: f32, y: f32) -> HostPointer {
        HostPointer::Wheel {
            x,
            y,
            dx: self.dx,
            dy: self.dy,
            unit: self.unit,
            modifiers: self.modifiers,
        }
    }
}

/// Parse a whole `wheel ...` line, verb included.
pub(crate) fn parse(line: &str) -> Result<WheelStep, String> {
    const USAGE: &str = "wheel wants '<dx> <dy> [lines|px] [shift|ctrl|alt|meta]... \
                         (at <x> <y> | on <selector>)'";
    let mut rest = line.trim();
    rest = rest
        .strip_prefix("wheel")
        .filter(|tail| tail.is_empty() || tail.starts_with(char::is_whitespace))
        .ok_or_else(|| USAGE.to_string())?
        .trim_start();

    let number = |word: Option<&str>, what: &str| -> Result<f32, String> {
        let word = word.ok_or_else(|| format!("{USAGE}; missing {what}"))?;
        word.parse::<f32>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("wheel {what} wants a finite number, got '{word}'"))
    };

    let mut words = rest.split_whitespace();
    let dx = number(words.next(), "dx")?;
    let dy = number(words.next(), "dy")?;
    let mut unit = WheelUnit::Pixels;
    let mut modifiers = Modifiers::NONE;
    let mut at = None;
    while let Some(word) = words.next() {
        match word {
            "px" => unit = WheelUnit::Pixels,
            "lines" => unit = WheelUnit::Lines,
            "shift" => modifiers.shift = true,
            "ctrl" => modifiers.ctrl = true,
            "alt" => modifiers.alt = true,
            "meta" => modifiers.meta = true,
            "at" => {
                let x = number(words.next(), "x")?;
                let y = number(words.next(), "y")?;
                if let Some(extra) = words.next() {
                    return Err(format!("wheel at wants two numbers, then '{extra}'"));
                }
                at = Some(WheelAt::Point(x, y));
            },
            "on" => {
                let selector = text_after(line, "on")?;
                if selector.is_empty() {
                    return Err("wheel 'on' wants a selector".into());
                }
                at = Some(WheelAt::Selector(parse_selector(selector)?));
                break;
            },
            other => return Err(format!("unknown wheel word '{other}'; {USAGE}")),
        }
    }
    if dx.abs() > LIMIT || dy.abs() > LIMIT {
        return Err(format!("wheel delta {dx} {dy} is past the {LIMIT} limit"));
    }
    let at = at.ok_or_else(|| format!("{USAGE}; missing where"))?;
    Ok(WheelStep {
        dx,
        dy,
        unit,
        modifiers,
        at,
    })
}

/// The rest of `line` after the first standalone `word`, which carries the
/// selector's own spaces.
fn text_after<'a>(line: &'a str, word: &str) -> Result<&'a str, String> {
    let mut offset = 0;
    for token in line.split_inclusive(char::is_whitespace) {
        let start = offset;
        offset += token.len();
        if token.trim_end() == word && start > 0 {
            return Ok(line[offset..].trim());
        }
    }
    Err(format!("wheel '{word}' wants a selector"))
}

/// A `click` selector: `.class` or `role:name`, optionally `surface:NAME` first,
/// then an optional `@attr=value` or free text. The grammar of taproot's
/// private `parse_selector`.
pub(crate) fn parse_selector(rest: &str) -> Result<Selector, String> {
    let (head, tail) = split_first(rest.trim());
    let (scope, head, tail) = if let Some(surface) = head.strip_prefix("surface:") {
        if surface.is_empty() {
            return Err("wheel surface wants a nonempty name".into());
        }
        let (head, tail) = split_first(tail);
        (Some(surface), head, tail)
    } else {
        (None, head, tail)
    };
    let mut selector = if let Some(class) = head.strip_prefix('.') {
        Selector::class(class)
    } else if let Some(role) = head.strip_prefix("role:") {
        Selector::role(role)
    } else {
        return Err(format!(
            "wheel on wants '.class' or 'role:name', got '{head}'"
        ));
    };
    if let Some(scope) = scope {
        selector = selector.on_surface(scope);
    }
    let tail = tail.trim();
    if let Some(attr) = tail.strip_prefix('@') {
        let (name, value) = attr
            .split_once('=')
            .ok_or_else(|| "wheel @attr wants 'name=value'".to_string())?;
        selector = selector.with_attr(name, value);
    } else if !tail.is_empty() {
        selector = selector.containing(tail);
    }
    Ok(selector)
}

fn split_first(line: &str) -> (&str, &str) {
    match line.trim().split_once(char::is_whitespace) {
        Some((a, b)) => (a, b.trim()),
        None => (line.trim(), ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_wheel_defaults_to_pixels_with_no_modifiers() {
        let step = parse("wheel 0 -3 at 120 40.5").unwrap();
        assert_eq!(
            step,
            WheelStep {
                dx: 0.0,
                dy: -3.0,
                unit: WheelUnit::Pixels,
                modifiers: Modifiers::NONE,
                at: WheelAt::Point(120.0, 40.5),
            }
        );
        assert_eq!(
            step.at_point(1.0, 2.0),
            HostPointer::Wheel {
                x: 1.0,
                y: 2.0,
                dx: 0.0,
                dy: -3.0,
                unit: WheelUnit::Pixels,
                modifiers: Modifiers::NONE,
            }
        );
    }

    #[test]
    fn units_and_modifiers_are_words_in_any_order() {
        let step = parse("wheel 1.5 2 lines ctrl shift at 0 0").unwrap();
        assert_eq!(step.unit, WheelUnit::Lines);
        assert!(step.modifiers.ctrl && step.modifiers.shift);
        assert!(!step.modifiers.alt && !step.modifiers.meta);
        let step = parse("wheel 0 1 meta alt px at 0 0").unwrap();
        assert_eq!(step.unit, WheelUnit::Pixels);
        assert!(step.modifiers.meta && step.modifiers.alt);
    }

    #[test]
    fn a_selector_keeps_its_own_spaces_and_grammar() {
        let by_class = parse("wheel 0 2 on .graph-canvas-swatch").unwrap();
        assert_eq!(
            by_class.at,
            WheelAt::Selector(Selector::class("graph-canvas-swatch"))
        );
        let by_text = parse("wheel 0 2 lines ctrl on role:button Zoom in").unwrap();
        assert_eq!(by_text.unit, WheelUnit::Lines);
        assert!(by_text.modifiers.ctrl);
        assert_eq!(
            by_text.at,
            WheelAt::Selector(Selector::role("button").containing("Zoom in"))
        );
        let by_attr = parse("wheel 0 2 on surface:app .canvas @data-key=a b").unwrap();
        assert_eq!(
            by_attr.at,
            WheelAt::Selector(
                Selector::class("canvas")
                    .on_surface("app")
                    .with_attr("data-key", "a b")
            )
        );
    }

    #[test]
    fn a_selector_that_contains_the_word_on_is_not_cut_short() {
        let step = parse("wheel 0 1 on role:button Turn on the lights").unwrap();
        assert_eq!(
            step.at,
            WheelAt::Selector(Selector::role("button").containing("Turn on the lights"))
        );
    }

    #[test]
    fn mistakes_are_loud() {
        for (line, want) in [
            ("wheel", "missing dx"),
            ("wheel 1", "missing dy"),
            ("wheel x 1 at 0 0", "finite number"),
            ("wheel 1 inf at 0 0", "finite number"),
            ("wheel 1 1", "missing where"),
            ("wheel 1 1 at 5", "missing y"),
            ("wheel 1 1 at 5 6 7", "two numbers"),
            ("wheel 1 1 sideways at 0 0", "unknown wheel word"),
            ("wheel 1 1 on", "wants a selector"),
            ("wheel 1 1 on button", "'.class' or 'role:name'"),
            ("wheel 1e30 0 at 0 0", "limit"),
            ("wheelx 1 1 at 0 0", "wheel wants"),
        ] {
            let error = parse(line).expect_err(line);
            assert!(error.contains(want), "{line}: {error}");
        }
    }
}
