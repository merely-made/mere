# Merging back into a session: branches and picked-off parts

Status: design note, 2026-10-15. Mark ruled M1–M8 the same day (see
Rulings). The `picked_from` record (M2) is built (Mere `1cf67cf15`). The
pandect core of merge-in is built (see [Built](#built-phase-1-pandect-core)),
and so are the Djinn route and the mere-view cards (phase 2).

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
  own beyond its first change (`ChangeKind::Picked`), records
  `picked_from { session, at, container }` in its manifest rather than the
  fork fields, and is named `Evening reading (picked from Evening reading)`
  (`pandect::picked_session_name`).

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

The two cases differ only in how a source node finds its target node. A
branch is told apart from a fork by its manifest's `lineage` (M1); only a
branch, a picked-off part, or a thaw's branch can be merged in.

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
  was when it was picked, at `picked_from.at`.

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

## Rulings (Mark, 2026-10-15)

- **M1, branch in the manifest:** a `lineage: Fork | Branch` field on the
  session manifest, set at creation. A fork is never merged in; a branch is.
  Existing sessions read as `Fork`.
- **M2, the pick record:** `picked_from { session, at, container }`, in place
  of `parent_session`, `forked_at` and `ChangeKind::Forked`; a pick's first
  change is `ChangeKind::Picked`. Built: `pandect::PickedFrom`. Parts picked
  before it carry the fork fields and still load; merge-in treats one whose
  nodes carry `CopiedFrom` derivations into its `parent_session`'s container
  as a pick.
- **M3, picks of picks:** transitive. A part picked from a part merges into
  the original by walking derivations to the target's container.
- **M4, provenance:** each merged target node gets a `MergedFrom` derivation
  (a new `ProvenanceSubKind`, appended to keep ordinals).
- **M5, conflicts:** a pending merge with a choice per item, as proposed.
- **M6, the source after merge:** kept and **marked merged, with its history
  intact** (Mark revised his first reading, "Do we need the history? If not,
  trash after move", to "keep and mark the history"). The question his first
  reading asked has this answer: per-node `MergedFrom` provenance and the
  target's one `MergedIn` change say *that* a node came from the source and
  when, but not *how* it got there. The source's journal is the only record
  of the steps, authors and intermediate states behind a merged node, and of
  edits that were made and then undone before the merge. So the history is
  needed if anyone is to see why a merged node is the way it is, and keeping
  it costs only the source's existing storage.

  The mark is option (b): `merged: { into, through, at }` on the source's
  manifest, and the source read-only from then on, as a codicil thaw is
  (`codicil_read_only`). Read-only raises **one problem**: it ends the
  "merge a branch more than once" path above, where the previous `through` is
  the next base. The fix keeps (b): an edit to a merged, read-only branch
  makes a **new branch from it**, as a thaw's first edit makes a fork. Its
  lineage is the same target, and its base is the merge point, which the
  target holds at `through`, so the three-way base is still exact. A picked
  part has no such path, and none is wanted: picking again from the target
  gives a fresh part.
- **M7, deletions:** three-way. A source deletion deletes in the target; a
  deletion against an edit is a conflict.
- **M8, shared rules:** factor the per-facet merge rules out of
  `snapshot_merge` when merge-in is built, so Compose and merge-in share them.

## Built (phase 1, pandect core)

2026-10-15. What landed, and where it settled a point the rulings left open.

- **M1:** `GraphSessionManifest::lineage` (`Lineage::Fork | Branch`,
  default `Fork`, so every older manifest reads as a fork) and
  `MereSessions::branch_at` / `begin_branch_at`. A branch records
  `parent_session` and `forked_at` as a fork does, starts with
  `ChangeKind::Branched`, and is named `Session (branch of Session)`
  (`pandect::branched_session_name`).
- **M3:** `MereSessions::merge_in(target, source) -> PendingMerge`. A branch
  matches by id, against its own baseline. A branch whose parent was merged
  into the target also merges into it, so the M6 path (a new branch made from
  a merged one) works. A picked-off part matches through `CopiedFrom`
  derivations, walking every part in a chain. Its base is the target's graph
  where the chain was first picked, with the part's own edits replayed onto
  it (ids remapped), so what the copy changed in passing (visit stamps, its
  own derivation, dropped import provenance) never counts as an edit. A
  pre-`picked_from` part counts as a pick when its nodes carry derivations
  into its parent's container. In a chain, everything since the first pick
  comes back (Mark, 2026-10-15): each part between's edits to what the source
  holds, up to where the next part was picked from it, are replayed before the
  source's own. A part between's edits to nodes the source does not hold, and
  its field layer and import records, stay in that part. A node the part made arrives under a new id derived from the target's and
  the source node's ids (`Uuid::new_v5`), so planning again gives the same
  plan.
- **Three-way (M5, M7):** the comparison is undo's own `revert_change`. It
  reverts base→source against the target, then drops the kept parts where
  source and target agree. What is left is a `MergeConflict { part, base,
  source, target }`, values as JSON (`kernel::graph::part_value`).
  `MereSessions::apply_merge(target, source, pending, choices, author)` takes
  a `MergeChoice` per conflicted `Part`: `Source`, `Target`, `Hand(value)`
  (the part's value in `part_value`'s form, `null` for absent; applied by
  `kernel::graph::part_edits`) or `AddAsNew`. `AddAsNew` is offered
  (`MergeConflict::add_as_new`) for a picked node the target has since
  deleted. It adds the source node under a new id, with its relations, and
  leaves the deletion standing. `apply_merge` refuses with `Unresolved(n)`
  until every conflict has a choice, and with `NotMergeable` if either
  session moved since the plan or a choice does not fit. A delete against an
  edit is a `Part::Node` conflict both ways round. The node's relations
  follow its choice, so `Source` brings a deleted node back with them and
  `Target` keeps a node with them. Only relations between two nodes that are
  not in conflict are asked about.
- **M4:** `ProvenanceSubKind::MergedFrom` (and its persisted mirror), appended.
  Every node the merge made or changed records one, naming the source node and
  the source's container.
- **History:** one `ChangeKind::MergedIn { from, through }` on the target,
  which undo treats as one step. The source gets `ChangeKind::MergedInto` and
  `merged: { into, through, at }` (M6), and `ensure_writable` refuses it from
  then on. Both sessions are written in one batch. The mark lives in
  `sessions/<id>/merged.json`, which every read of a manifest lays over
  `merged`. Undoing the `MergedIn` deletes it in the target's own batch, so
  the source is editable and mergeable again (Mark, 2026-10-15), and redoing
  it writes it again. That works on every undo path, including a host that
  writes its own batches.
- **M8:** `pandect::merge_rules` holds same-id surface layering, edge
  signatures, `layer_resource_record`, the checked `merge_resource_facet`
  and the assertion-handle check. Compose calls them, and its tests are
  unchanged. Merge-in joins a resource facet that both sides changed by the
  facet's rule when it has one (properties, classifications, predicate
  declarations, legacy origins), and asks only if the rule itself refuses.
  Any other facet is compared whole.

## Built (phase 2, the route and the view)

2026-10-15.

- **Wire** (`mere_overmap::wire`): `mere.sessions.branch` and
  `mere.sessions.merge` on a session's card, with the `SessionsActionV1`
  payload. A pending merge is its own card in the sessions projection,
  source kind `mere.merge`, keyed by the source session. Its first media is
  a `PendingMergeV1` (`mere.merge.pending/v1`): the two sessions with their
  names, and each `MergeConflict`. The card offers `mere.merge.resolve` and
  `mere.merge.cancel` with a `MergeResolutionV1` (`mere.merge/v1`): the
  source and one `{ part, choice }` per conflict.
- **Djinn** (`resident_mere`): every session card offers Branch. A branch
  or picked-off part with nothing pending also offers Merge into source; its
  target is the branch's parent, or the session that parent was merged into,
  or the part's `picked_from`. A clean merge applies at once. One with
  conflicts is held by the resident and shown as the card until it is
  resolved or cancelled. A resolve missing a choice stays pending. Any other
  failure (a session moved, a value given by hand that does not fit) plans
  the merge again (Mark, 2026-10-15). A clean new plan applies; otherwise it
  waits with the choices that still apply carried over
  (`PendingMerge::carry_choices`). The card's material lists the carried and
  the dropped choices, each dropped one with its reason ("no longer in
  conflict", "adding as new no longer applies", "the value given by hand does
  not fit"). The view lists the dropped ones on the card as "Dropped choices". Undoing or redoing a
  merge through the session item reloads the source's mark in any host
  holding it open, and moves the sessions list. `MereHost` gained
  `plan_merge`, `apply_merge` and `reload_merged_mark`.
- **View** (`mere-view`): `SessionStep::Branch` ("Branch") and
  `SessionStep::Merge` ("Merge into source") join the steps, so they render
  wherever steps do. `MereViewModel::merge` is shown as a card over the view
  below its bar. Each conflict lists its part, its base, source and target
  values, and Source, Target, Edit by hand, and Add as new where it applies.
  Each also has a hand-value field, a `TextInput` draft kept in
  `MereViewState::merge_drafts` and fed by `MereViewEvent::MergeDraft`.
  Apply is disabled until every conflict has a choice; Cancel is always
  available. Requests are `MereViewRequest::Merge(MergeRequest::{Choose,
  Apply, Cancel})`.
- `MereViewRequest` and `SessionStep` are `#[non_exhaustive]` (Mark,
  2026-10-15), so a new request or step does not break a host. In-repo hosts
  keep a wildcard arm. Knot's merge wiring comes with its next repin.
- `graphshell_mere`, the native viewer, completes Branch and Merge too: it
  reads the pending merge's material, keeps choices until Apply, and adopts a
  new plan's carried and dropped choices.
- **Route adapter** (`mere_overmap::route`, `route_session`): reads the merge
  card's media, words each part (`Title of One`, `Relations from A to B`),
  keeps the choices made in the panel until Apply, and sends them as one
  resolve. A hand value is sent as text where the conflict's values are
  text, and as JSON otherwise.

Mark's rulings on the first build's open points (2026-10-15): both extra
choices; everything since the first pick; undo frees the source; derived ids
for a part's new nodes; a node's relations follow its choice. All are built
as described above.

## The options as they were put to Mark


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
