# Merging back into a session: branches and picked-off parts

Status: design note, 2026-10-15. Nothing built. Options M1–M8 are open for
Mark.

## Problem

A session can be copied in two ways today, and neither can come back.

- **Fork** (`MereSessions::fork_at`, `begin_fork_at`): a copy of the whole
  session at a journal cursor that goes its own way. The fork keeps the
  parent's node ids, records `parent_session` and `forked_at` in its manifest,
  starts an empty journal whose provenance is the fork point, and is named
  `Session (fork of Session)` (`pandect::forked_session_name`).
- **Pick off** (`MereSessions::fork_component`, over
  `pandect::fork_component_graph`; Turnstone's tear-out): the connected
  component around one node, copied into a new session. Its nodes get new
  ids. Each one carries a `provenance.derivations` record,
  `NodeDerivation { sub_kind: CopiedFrom, source_node, source_graph }`, naming
  the donor node and the donor's container. It has no session history of its
  own beyond its first change, and is named `Session (picked from Evening
  reading)` (`pandect::picked_session_name`).

Mark's vocabulary (2026-10-15, now in [TERMINOLOGY.md](../../TERMINOLOGY.md))
adds a third: a **branch**, which also copies the whole session at a point but
keeps a lineage link to its parent and is meant to merge back. Branches are
offered only once merge-in exists. This note is that merge-in, for branches
and for picked-off parts. It is a design for review: rulings come first.

## What already exists

- **Codicil Compose** (`pandect::snapshot_merge`, used by
  `graph_codicil::compose_graph_codicils`) unions two frozen graph snapshots.
  Same-id surfaces layer `b`'s tags and new properties onto `a`'s; identical
  edges collapse; resources unite by canonical IRI, with predicate
  declarations, classifications and properties merged by their own checked
  rules ("conflicting resource data never selects an insertion-order
  survivor"); `a`'s field layer and navigation stay canonical; import records
  unite with remapped memberships. The result is a new, immutable `Derived`
  codicil whose `ProvenanceRecord.upstream` names both sources. Neither
  source changes.
- **Fork lineage**: `parent_session`, `forked_at`, `ChangeKind::Forked { from,
  at }`, and the fork's journal provenance `at`. The parent's graph at that
  cursor is recoverable (`graph_at`), so a three-way base exists.
- **Derivation provenance**: `Graph::node_derivations`, and
  `record_derivation` (idempotent per sub-kind and source).
- **Journals** of captured deltas, replayable onto a graph
  (`replay_captured_deltas_onto`).

Compose is the nearest thing to merge-in, and the difference matters. Compose
is a *two-way union of frozen snapshots into a new record*. Merge-in changes a
*live session* by bringing in what happened elsewhere since a known point. It
needs a base, it can delete as well as add, and it writes the parent's
journal.

## Proposal

### One operation, two matchers

Merge-in takes a **source** session (the branch or the picked-off part) and a
**target** (the parent, or the session the part was picked from), and
produces one change on the target's journal:
`ChangeKind::MergedIn { from, through }`, where `through` is the source's
journal cursor that was merged. The source is left as it is; whether it is
then trashed is a host decision (M6).

The two cases differ only in how a source node finds its target node.

- **Branch: by id.** A branch keeps its parent's node ids, so matching is the
  identity, the same rule Compose uses for surfaces.
- **Picked-off part: by derivation provenance.** A picked node `n` matches the
  target node named by its `CopiedFrom` derivation whose `source_graph` is the
  target's container. A node with no such derivation (made after the pick)
  is new and is added. Matching is transitive along a chain of picks (a part
  picked from a part), so merging into the original walks derivations until
  it reaches the target's container (M3).

### The base, and three-way decisions

A three-way merge needs the state both sides started from.

- For a branch the base is the parent's graph at `forked_at`.
- For a picked-off part the base is the part's own baseline, mapped back into
  the target's ids through its derivations. That is the donor component as it
  was when it was picked. Its pick cursor should be recorded for this
  (today `fork_component` records `forked_at` as the parent's live cursor, so
  that exists, but under the fork's name; see M2).

For each matched node, edge and facet, compare base, source and target:

| Base → source | Base → target | Result |
| --- | --- | --- |
| unchanged | changed | keep target |
| changed | unchanged | take source |
| changed | changed, equal | keep (no conflict) |
| changed | changed, different | **conflict** |
| added in source | — | add (new id for a picked part) |
| deleted in source | unchanged | delete in target |
| deleted in source | changed | **conflict** (delete versus edit) |

Facets merge per facet by their own rules where one exists, as Compose already
does for resource properties, classifications and predicate declarations.
Everything else is compared whole.

### Conflicts

A conflict never resolves by insertion order. That is Compose's rule, and
merge-in keeps it. A merge with conflicts produces a **pending merge**: the
proposed change plus a list of conflicts, each naming the node or edge, the
facet, and the three values. Nothing is written until every conflict has a
choice (source, target, or a hand edit). The pending-change machinery the
journal already has (`graph_session_pending_tests.rs`) is the natural home.

A picked-off part adds one conflict kind of its own: the **vanished source**.
The target node a derivation names has since been deleted from the target.
The choices are to resurrect it, to add the source node as new, or to drop it.

### History

- The target's journal gains one `MergedIn` change carrying the deltas, so
  undo takes the whole merge back in one step, and the change names its
  source. The source journal is not copied into the target. Its history stays
  in the source session, reachable through `from`.
- A merged picked-off node records a `MergedFrom` derivation on the target
  node (new sub-kind; M4), so provenance shows both directions.
- A branch merged more than once uses the previous `through` as the next base.
  The branch manifest records it (`merged_through`).

### Relation to codicil Compose

Compose stays what it is: a union of frozen archives into a new archive, with
no base and no deletions. Merge-in reuses Compose's *per-facet merge rules*
and its no-silent-survivor rule. It does not reuse its union semantics. The
shared part should move into a module both call: same-id layering, edge
signatures, resource-by-IRI unification and the checked facet merges. Then a
rule fixed for one is fixed for the other. A merge-in from a codicil thaw is
the branch case with the thaw as parent, and needs nothing extra.

## Where it would live

- pandect: `MereSessions::merge_in(target, source) -> PendingMerge`, the
  matchers, the three-way comparison and the change kind; the shared merge
  rules factored out of `snapshot_merge`.
- graph-kernel: the `MergedFrom` sub-kind (appended, to keep ordinals) and a
  derivation lookup by source node.
- mere-view: a Merge step on a branch's or picked part's card in the overmap
  tier, and a conflict view. Out of scope until the rulings land.

## Options for Mark

1. **M1, what a branch is in the manifest.**
   - (a) A `lineage: Fork | Branch` field on the session manifest, set at
     creation.
   - (b) A separate `branch_of` field beside `parent_session`.
   - (c) Every fork can merge back, and "branch" is only a word in the
     interface.
2. **M2, what records a pick.**
   - (a) Keep `parent_session` and `forked_at` on a picked part, as today.
   - (b) Rename them to a `picked_from { session, at, container }` record, so
     a pick never reads as a fork in the manifest.
3. **M3, picks of picks.**
   - (a) Merge back one step only, into the session the part was picked
     from.
   - (b) Walk derivations transitively, so a part picked from a part can
     merge into the original.
4. **M4, provenance after merge.**
   - (a) Add a `MergedFrom` derivation on each merged target node.
   - (b) Record the merge only in the journal change.
5. **M5, conflicts.**
   - (a) Pending merge with per-item choice, as proposed.
   - (b) Refuse a merge with any conflict and ask for edits in the source
     first.
   - (c) Choose the target by default and list what was overridden.
6. **M6, the source after merge.**
   - (a) Leave it as it is.
   - (b) Mark it merged and read-only.
   - (c) Trash it.
7. **M7, deletions.**
   - (a) A source deletion deletes in the target (three-way, as proposed).
   - (b) Merge-in only adds and edits; deletions stay manual.
8. **M8, shared rules with Compose.**
   - (a) Factor the per-facet merge rules out of `snapshot_merge` now, with
     merge-in.
   - (b) Duplicate them first and factor later.
