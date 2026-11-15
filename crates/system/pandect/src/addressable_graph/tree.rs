// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Lists stored as content-defined trees (catalog version 3, engine split
//! D177). A list's items are cut into leaf chunks, the leaves' digests into
//! index chunks, and so on up until one chunk is left, which the catalog's
//! root names with its depth ([`TreeRef`]). Every chunk is stored once under
//! `chunks/<digest>`.
//!
//! Where a chunk ends depends only on the items' keys: a key hashes to a
//! number, and its item closes the chunk at each level below the number of
//! times the [target](CHUNK_TARGET) divides that number. So a list gives the
//! same tree however it was reached, the root's digest (the generation) stays
//! a function of graph content, and an edit rewrites only the chunks on the
//! path from the changed items to the top.
use super::*;
use kernel::graph::history::NodeHistoryOwner;
use muniment::{TransactionReader, WriteOp};
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

/// Items per chunk on average, at every level (engine-split D178).
pub(super) const CHUNK_TARGET: u64 = 64;
/// The most levels a 64-bit key hash can close at, at a target of 2.
const MAX_DEPTH: u8 = 64;

#[cfg(test)]
thread_local! {
    /// The target this thread writes with: tests set a small one so that a
    /// small graph builds a deep tree.
    pub(super) static TARGET: std::cell::Cell<u64> = const { std::cell::Cell::new(CHUNK_TARGET) };
}
#[cfg(test)]
fn target() -> u64 {
    TARGET.with(|t| t.get())
}
#[cfg(not(test))]
fn target() -> u64 {
    CHUNK_TARGET
}

/// How many levels an item with `key` closes a chunk at: the times `target`
/// divides the key's hash.
fn rank(key: &[u8], target: u64) -> u8 {
    let digest = blake3::Hasher::new()
        .update(b"mere catalog chunk\0")
        .update(key)
        .finalize();
    let head: [u8; 8] = digest.as_bytes()[..8].try_into().expect("eight bytes");
    let mut value = u64::from_le_bytes(head);
    let mut rank = 0;
    while value != 0 && value.is_multiple_of(target) && rank < MAX_DEPTH {
        value /= target;
        rank += 1;
    }
    rank
}

/// A stored tree as the catalog's root names it: its top chunk and how many
/// index levels lie above the leaves. An empty list has no top.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct TreeRef {
    pub depth: u8,
    pub top: Option<String>,
}

/// A tree's chunks in memory, level by level from the leaves.
#[derive(Clone, Debug, Default)]
pub(super) struct Tree {
    levels: Vec<Vec<Span>>,
}
#[derive(Clone, Debug)]
struct Span {
    digest: String,
    /// The list's items under this chunk.
    items: Range<usize>,
    /// Its chunks on the level below; empty for a leaf.
    children: Range<usize>,
}
impl Tree {
    pub fn root(&self) -> TreeRef {
        match self.levels.last() {
            Some(top) => TreeRef {
                depth: (self.levels.len() - 1) as u8,
                top: Some(top[0].digest.clone()),
            },
            None => TreeRef::default(),
        }
    }
    /// Every chunk's digest.
    pub fn digests(&self) -> impl Iterator<Item = &str> {
        self.levels.iter().flatten().map(|s| s.digest.as_str())
    }
    /// The chunks of each level by digest.
    fn positions(&self) -> HashMap<&str, (usize, usize)> {
        let mut at = HashMap::new();
        for (level, spans) in self.levels.iter().enumerate() {
            for (i, span) in spans.iter().enumerate() {
                at.insert(span.digest.as_str(), (level, i));
            }
        }
        at
    }
    /// The chunk at `(level, i)` and every chunk under it, and its items.
    fn under(&self, (level, i): (usize, usize)) -> (Vec<&str>, Range<usize>) {
        let mut chunks = vec![];
        let mut at = i..i + 1;
        for spans in self.levels[..=level].iter().rev() {
            let spans = &spans[at.clone()];
            chunks.extend(spans.iter().map(|s| s.digest.as_str()));
            at = spans[0].children.start..spans[spans.len() - 1].children.end;
        }
        (chunks, self.levels[level][i].items.clone())
    }
    /// A tree read top down: each level's chunks, leaves first, with how many
    /// items (leaves) or children (index chunks) each holds.
    fn from_counts(levels: Vec<Vec<(String, usize)>>) -> Self {
        let mut tree = Self::default();
        for counts in levels {
            let below = tree.levels.last();
            let mut spans = Vec::with_capacity(counts.len());
            let mut next = 0;
            for (digest, count) in counts {
                let (items, children) = match below {
                    None => (next..next + count, 0..0),
                    Some(below) => {
                        let first = &below[next].items;
                        let last = &below[next + count - 1].items;
                        (first.start..last.end, next..next + count)
                    },
                };
                next += count;
                spans.push(Span {
                    digest,
                    items,
                    children,
                });
            }
            tree.levels.push(spans);
        }
        tree
    }
}

/// `items` as a tree whose chunks' puts are appended to `ops`, each chunk
/// once per publication (`written`). `key` is what places an item's chunk
/// boundaries, so it must not change when the item's content does.
pub(super) fn build<B: Backend, T: Serialize>(
    store: &AddressableGraphStore<B>,
    items: &[T],
    key: impl Fn(&T) -> Result<Vec<u8>, StoreError>,
    written: &mut HashSet<String>,
    ops: &mut Vec<WriteOp>,
) -> Result<Tree, StoreError> {
    let target = target();
    let mut put = |bytes: Vec<u8>| {
        let digest = hash(&bytes);
        if written.insert(digest.clone()) {
            ops.push(WriteOp::Put {
                key: store.chunk_key(&digest),
                value: bytes,
            });
        }
        digest
    };
    let mut tree = Tree::default();
    // Each chunk's closing rank: its last item's.
    let mut closing = vec![];
    let mut leaves = vec![];
    let mut start = 0;
    for (i, item) in items.iter().enumerate() {
        let rank = rank(&key(item)?, target);
        if rank > 0 || i + 1 == items.len() {
            let digest = put(encode(&items[start..=i])?);
            leaves.push(Span {
                digest,
                items: start..i + 1,
                children: 0..0,
            });
            closing.push(rank);
            start = i + 1;
        }
    }
    if leaves.is_empty() {
        return Ok(tree);
    }
    tree.levels.push(leaves);
    let mut height = 1;
    while let Some(below) = tree.levels.last().filter(|level| level.len() > 1) {
        let (mut spans, mut ranks) = (vec![], vec![]);
        let mut start = 0;
        for (i, span) in below.iter().enumerate() {
            if closing[i] > height || i + 1 == below.len() {
                let children: Vec<&str> =
                    below[start..=i].iter().map(|s| s.digest.as_str()).collect();
                let digest = put(encode(&children)?);
                spans.push(Span {
                    digest,
                    items: below[start].items.start..span.items.end,
                    children: start..i + 1,
                });
                ranks.push(closing[i]);
                start = i + 1;
            }
        }
        tree.levels.push(spans);
        closing = ranks;
        height += 1;
    }
    Ok(tree)
}

/// A chunk's entries, checked against its digest; a chunk is never empty.
fn chunk<T: DeserializeOwned>(bytes: &[u8], digest: &str) -> Result<Vec<T>, StoreError> {
    let entries: Vec<T> = checked(bytes, digest)?;
    if entries.is_empty() {
        return Err(fail("empty catalog chunk"));
    }
    Ok(entries)
}

/// The items of the tree `root` names, every chunk read and checked, and the
/// tree; `bytes` grows by what was read.
pub(super) async fn load<B: Backend, T: DeserializeOwned>(
    store: &AddressableGraphStore<B>,
    root: &TreeRef,
    bytes: &mut usize,
) -> Result<(Vec<T>, Tree), StoreError> {
    let Some(top) = &root.top else {
        if root.depth != 0 {
            return Err(fail("catalog tree without a top"));
        }
        return Ok((vec![], Tree::default()));
    };
    if root.depth >= MAX_DEPTH {
        return Err(fail("catalog tree too deep"));
    }
    let mut levels = vec![];
    let mut current = vec![top.clone()];
    for _ in 0..root.depth {
        let (mut next, mut level) = (vec![], vec![]);
        for digest in current {
            let data = store.required(&store.chunk_key(&digest)).await?;
            *bytes += data.len();
            let children: Vec<String> = chunk(&data, &digest)?;
            level.push((digest, children.len()));
            next.extend(children);
        }
        levels.push(level);
        current = next;
    }
    let (mut items, mut leaves) = (vec![], vec![]);
    for digest in current {
        let data = store.required(&store.chunk_key(&digest)).await?;
        *bytes += data.len();
        let entries: Vec<T> = chunk(&data, &digest)?;
        leaves.push((digest, entries.len()));
        items.extend(entries);
    }
    levels.push(leaves);
    levels.reverse();
    Ok((items, Tree::from_counts(levels)))
}

/// The keys the old tree `old` names, read inside a transaction: only the
/// chunks the new tree (`new`, whose items' keys are `new_keys`) lacks are
/// read; a chunk both have is the same items, so what lies under it comes
/// from the new tree. `key` spells an old item's key. `None` when a chunk
/// cannot be read.
pub(super) fn held<T: DeserializeOwned>(
    reader: &dyn TransactionReader,
    chunk_key: &dyn Fn(&str) -> String,
    old: &TreeRef,
    new: (&Tree, &[String]),
    key: impl Fn(&T) -> String,
    held: &mut HashSet<String>,
) -> Option<()> {
    let (tree, new_keys) = new;
    let shared = tree.positions();
    let mut stack: Vec<(String, u8)> = old.top.iter().map(|top| (top.clone(), old.depth)).collect();
    while let Some((digest, depth)) = stack.pop() {
        if let Some(&at) = (shared.get(digest.as_str())).filter(|at| at.0 == depth as usize) {
            let (chunks, items) = tree.under(at);
            held.extend(chunks.into_iter().map(chunk_key));
            held.extend(new_keys[items].iter().cloned());
            continue;
        }
        let bytes = reader.get(&chunk_key(&digest)).ok()??;
        held.insert(chunk_key(&digest));
        if depth == 0 {
            let items: Vec<T> = chunk(&bytes, &digest).ok()?;
            held.extend(items.iter().map(&key));
        } else {
            let children: Vec<String> = chunk(&bytes, &digest).ok()?;
            stack.extend(children.into_iter().map(|child| (child, depth - 1)));
        }
    }
    Some(())
}

/// The catalog's trees: its rows, navigation's entries, visits and owners,
/// and the pending owners. Empty for a catalog stored whole (versions 1, 2).
#[derive(Clone, Debug, Default)]
pub(super) struct Trees {
    pub rows: Tree,
    pub entries: Tree,
    pub visits: Tree,
    pub owners: Tree,
    pub pending: Tree,
}
impl Trees {
    pub fn digests(&self) -> impl Iterator<Item = &str> {
        [
            &self.rows,
            &self.entries,
            &self.visits,
            &self.owners,
            &self.pending,
        ]
        .into_iter()
        .flat_map(Tree::digests)
    }
}
/// Where a row's chunk ends depends on its address only, never its digest.
fn row_key(row: &IndexRow) -> Result<Vec<u8>, StoreError> {
    let tag = match row.address {
        GraphAddress::Surface(_) => b's',
        GraphAddress::Resource(_) => b'r',
    };
    let mut key = vec![tag];
    key.extend_from_slice(row.address.id().as_bytes());
    Ok(key)
}

/// `catalog`'s lists stored as trees, their chunks' puts appended to `ops`,
/// and the root naming them, encoded: its digest is the generation.
pub(super) fn store_catalog<B: Backend>(
    store: &AddressableGraphStore<B>,
    catalog: &mut Catalog,
    ops: &mut Vec<WriteOp>,
) -> Result<Vec<u8>, StoreError> {
    let mut written = HashSet::new();
    let digest_key = |digest: &String| Ok(digest.as_bytes().to_vec());
    let nav = &catalog.navigation;
    let owners: Vec<(&String, &NodeHistoryOwner)> =
        nav.owners.iter().zip(&nav.identities).collect();
    let trees = Trees {
        rows: build(store, &catalog.entries, row_key, &mut written, ops)?,
        entries: build(store, &nav.entries, digest_key, &mut written, ops)?,
        visits: build(store, &nav.visits, digest_key, &mut written, ops)?,
        owners: build(
            store,
            &owners,
            |(_, owner)| encode(owner),
            &mut written,
            ops,
        )?,
        pending: build(
            store,
            &catalog.pending.owners,
            |(id, _)| Ok(id.as_bytes().to_vec()),
            &mut written,
            ops,
        )?,
    };
    let root = Root {
        version: catalog.version,
        globals: catalog.globals.clone(),
        rows: trees.rows.root(),
        navigation: NavigationRoot {
            entries: trees.entries.root(),
            visits: trees.visits.root(),
            owners: trees.owners.root(),
            digest: nav.digest.clone(),
        },
        orphan_facets: catalog.orphan_facets.clone(),
        pending: PendingRoot {
            retention: catalog.pending.retention,
            owners: trees.pending.root(),
        },
    };
    catalog.trees = trees;
    encode(&root)
}

/// The catalog `root` names, every chunk read and checked; `size` grows by
/// the chunks' bytes.
pub(super) async fn load_catalog<B: Backend>(
    store: &AddressableGraphStore<B>,
    root: Root,
    size: &mut usize,
) -> Result<Catalog, StoreError> {
    let nav = &root.navigation;
    let (entries, rows) = load::<_, IndexRow>(store, &root.rows, size).await?;
    let (nav_entries, entries_tree) = load::<_, String>(store, &nav.entries, size).await?;
    let (visits, visits_tree) = load::<_, String>(store, &nav.visits, size).await?;
    let (owners, owners_tree) =
        load::<_, (String, NodeHistoryOwner)>(store, &nav.owners, size).await?;
    let (pending, pending_tree) =
        load::<_, (Uuid, String)>(store, &root.pending.owners, size).await?;
    let (owners, identities) = owners.into_iter().unzip();
    finish(Catalog {
        version: root.version,
        globals: root.globals,
        entries,
        navigation: NavigationIndex {
            entries: nav_entries,
            visits,
            owners,
            identities,
            digest: root.navigation.digest,
        },
        orphan_facets: root.orphan_facets,
        pending: PendingIndex {
            retention: root.pending.retention,
            owners: pending,
        },
        index: Default::default(),
        trees: Trees {
            rows,
            entries: entries_tree,
            visits: visits_tree,
            owners: owners_tree,
            pending: pending_tree,
        },
    })
}

#[cfg(test)]
impl Tree {
    /// The digests of the chunks at `level` (0 is the leaves).
    pub(super) fn level(&self, level: usize) -> Vec<&str> {
        self.levels[level]
            .iter()
            .map(|s| s.digest.as_str())
            .collect()
    }
}
