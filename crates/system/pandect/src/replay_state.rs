// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the session's replay check compares (engine split D156): a graph's
//! snapshot at a fixed date and its facet store, as the compact JSON bytes
//! serde writes for them, with no `serde_json::Value` tree in between.
//!
//! Equal bytes mean equal `Value`s, as the check compared before. The same
//! serializer drops the same skipped fields and writes a NaN or infinite
//! float as `null` either way. Struct fields keep declaration order and every
//! map on the path is ordered (`BTreeMap`, or a `Value` object, which has no
//! `preserve_order` in Mere's graph), so equal states write equal bytes. The
//! one place `Value` equality is looser than bytes is a zero's sign, which
//! [`UnsignedZero`] writes away.
//!
//! The session edit compares only what two graphs do not share
//! ([`replay_equal`], engine split D164): the same bytes, entry by entry.

use kernel::graph::Graph;
use kernel::graph::snapshot::unshared::Unshared;
use serde::Serialize;
use serde_json::ser::{CompactFormatter, Formatter};
use std::io;

/// Compact JSON writing `-0.0` as `0.0`, since `Value` holds them equal.
struct UnsignedZero;

impl Formatter for UnsignedZero {
    fn write_f32<W: ?Sized + io::Write>(&mut self, writer: &mut W, value: f32) -> io::Result<()> {
        CompactFormatter.write_f32(writer, if value == 0.0 { 0.0 } else { value })
    }

    fn write_f64<W: ?Sized + io::Write>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        CompactFormatter.write_f64(writer, if value == 0.0 { 0.0 } else { value })
    }
}

/// `value` as the bytes the replay check compares.
pub(crate) fn state_bytes(value: &impl Serialize) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = Vec::new();
    value.serialize(&mut serde_json::Serializer::with_formatter(
        &mut bytes,
        UnsignedZero,
    ))?;
    Ok(bytes)
}

/// The replay check's state: the snapshot under one fixed date, so the check
/// never reads the clock (reservoir plan ruling 47), and the facet store.
pub(crate) fn replay_bytes(graph: &Graph) -> Result<Vec<u8>, serde_json::Error> {
    state_bytes(&(graph.to_snapshot_at(0), graph.facets()))
}

/// How [`replay_equal`] reached its answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Decided {
    /// From the entries the graphs do not share, this many pairs compared.
    Entries(usize),
    /// The pointer walk named a difference it can prove.
    Walk,
    /// The walk could not vouch; the whole states were compared.
    Whole,
}

/// Whether `a` and `b` have equal replay states, as [`replay_bytes`] would
/// find them (engine split D164). Two graphs cloned from one another share
/// every entry neither wrote since, so only the unshared entries are written
/// and compared, each as the bytes [`state_bytes`] gives it; where the walk
/// cannot vouch, the whole states are compared as before.
pub(crate) fn replay_equal(a: &Graph, b: &Graph) -> Result<(bool, Decided), serde_json::Error> {
    match a.unshared_state(b) {
        Unshared::Entries(pairs) => {
            for (x, y) in &pairs {
                if state_bytes(x)? != state_bytes(y)? {
                    return Ok((false, Decided::Walk));
                }
            }
            Ok((true, Decided::Entries(pairs.len())))
        },
        Unshared::Differ(_) => Ok((false, Decided::Walk)),
        Unshared::Whole(_) => Ok((replay_bytes(a)? == replay_bytes(b)?, Decided::Whole)),
    }
}

#[cfg(test)]
#[path = "replay_state_tests.rs"]
mod tests;
