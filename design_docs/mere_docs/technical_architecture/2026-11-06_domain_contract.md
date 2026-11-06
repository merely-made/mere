# Domain contract: what a domain supplies and what a client may do

**Status:** 2026-11-06: draft; nothing implemented. Mark ruled one kind of
store (O8) and C1 to C5 on 2026-11-06. Written for M0 of
[the domain adoption plan](../implementation_strategy/2026-11-05_domain_adoption_plan.md),
which made every application's authored store a reservoir mere (D8).

**For:** one written contract that every domain and every client builds
against, so that Woodshed, Redshank, Hocket, Knot and the rest move into the
reservoir the same way. Mark asked on 2026-11-05 whether this contract was
prepared; it was not.

**Not here:** each application's own move (its plan); device pairing
(`../implementation_strategy/2026-10-28_device_pairing_plan.md`); the grant
format (adoption plan O7, Meadowcap model); the optional domain vault, an
opt-in feature built later.

## One kind of store (Mark, 2026-11-06)

Mark, on O8: "I'm ok with a second kind, but then are there gonna be other
kinds? Isn't a graph a log…?" It is. Every store in question is an
operation log folded into state; they differ in how the log is ordered, not
in being graphs or logs. Measured on 2026-11-06:

| | Reservoir graph mere | Knot's operation store | Graphshell's personal replica |
|---|---|---|---|
| Entry | `AttributedDelta` (graph-kernel `graph/journal.rs`) | signed `Operation<KnotSyncExt>` (knot `sync.rs`) | signed operation with `PersonalGraphExt` |
| Order | one linear `Seq` per session | causal DAG over `parents` | causal DAG over `parents` |
| Across devices | none | Stickleback log sync | Stickleback log sync |
| Fold | replay into `Graph` | `KnotDocumentProjection` | deterministic fold into `Graph` |
| Check | `GraphValidator`, `GraphAdmissionValidator` before append | `KnotSyncPolicy::admit` before insert | Stickleback operation policy |
| Sealing | none on the live path | each event body, and checkpoints | in the header extension |
| Backend | muniment `DirectoryBackend` | muniment `RedbBackend` | muniment `RedbBackend` |

The reservoir's linear journal is the odd one out. It cannot replicate,
because a per-session counter has no meaning on a second device, and it is
not sealed. Knot's store and Graphshell's personal replica already have what
D2 and D6 require: a signed causal log on Stickleback, sealed per operation,
checked before insert, and synced between devices. Graphshell's replica also
shows a graph kept as the fold of such a log.

**Ruled** ("Yes, one kind"): a mere is one Stickleback causal operation log. A domain
supplies its event grammar, its fold and its admission policy; the reservoir
supplies everything else. A graph is the fold most domains use, not a
different kind of store. Knot's store fits as it is, which answers D4
without a second kind and without projecting documents into a graph. There
would be no other kinds: new domains add grammars and folds, not stores.

This also makes M3 (sealing) and M4 (replication) properties of the one
store rather than separate work on the linear journal.

## What a domain supplies

1. **A domain id** (`DomainId`), named for the data, not the application
   (adoption plan D5).
2. **An event grammar:** typed, versioned operation bodies, closed to
   unknown variants unless the grammar says how to tolerate them. Knot's
   `KnotSyncEvent` is the model.
3. **A deterministic fold** from the causally ordered log to the domain's
   state. The fold never orders by wall-clock time or insertion order;
   concurrent writes to the same item surface as a conflict with each
   writer's version, as Knot's `KnotDocumentConflict` does, until a resolving
   operation supersedes them.
4. **An admission policy** over the folded candidate an operation would
   produce, run where the operation is authored and again where it is
   folded in (C3, C4). A graph domain's existing
   `GraphValidator` and `GraphAdmissionValidator` become this policy
   unchanged; Cleromancy's `validate_divination_graph` keeps working.
5. **A pure crate** holding 2 to 4, with no I/O, implementing Stickleback's
   domain interface (C5), that the resident composes
   in both the daemon and the embedded resident (adoption plan M1, M2).
6. **Its move/stay split:** what is authored and replicates, and what is a
   device fact kept in the device-local store (D7).
7. **Blob references** for large media, content-addressed, fetched on demand
   (M5). Operations carry the reference, never the bytes.

## What the reservoir supplies

- **Lifetimes and the single owner** (D1): opening, durability and closing
  of every mere; the reservoir index lock refuses a second owner.
- **Sealing** under the persona's key, always (D6), with the optional domain
  vault layered on top when it exists.
- **Writers and ordering:** each device is its own writer, derived from the
  persona and the device, as Knot does now; parents record the frontier each
  operation saw.
- **Replication** to the persona's paired devices (D2, M4).
- **The route** `mere/<domain>` with read, edit and notice operations, the
  same through a Djinn door or an embedded resident (reservoir plan V5).
- **A device-local store** per application for what stays (D7), outside any
  mere and never replicated.

## What a client may do

Cleromancy's path is the precedent
(`cleromancy/src/ui/worker.rs`, `ports/mere-resident/src/embedded_reservoir.rs`):

- **Open:** try the Djinn door's `reservoir` route, `ensure` the domain,
  then open `mere/<domain>`. Embed the resident only when no door answers
  (not found or connection refused). A door that refuses, is locked or times
  out never leads to embedding, so no second store opens.
- **Read** the folded state through the route's projection.
- **Edit** by invoking intents that carry the revision the client observed;
  the reservoir runs the admission policy and refuses stale or invalid edits.
  A client never writes the store directly.
- **Observe** through the route's notices (the revision bell), not by
  polling files.
- **When locked or unavailable:** show the domain as unavailable and keep
  device-local state working. Never fall back to a private copy of
  authored data.

## Cleromancy against the contract

Its grammar today is graph edits, its fold is graph replay, and its policy is
`validate_divination_graph`; those carry over. What changes is beneath it:
its mere's linear journal becomes a causal log. Mark chose a fresh start
(C1): the new meres begin empty and the old stores stay on disk untouched.

## Rulings

- **C1. Existing meres: a fresh start (Mark, 2026-11-06).** Cleromancy's
  `divination` and `personal-facts` meres are not converted. The causal-log
  meres begin empty; the linear-journal stores stay on disk untouched and
  are no longer opened. Retiring them is Mark's call, later. Same choice as
  Woodshed's W5.
- **C2. Sessions are lines within the mere's one log (Mark, 2026-11-06).**
  Every operation names its session. Minting, forking and branching are
  operations that start a session from a frontier of the log; merging in is
  an operation that records the plan and each conflict choice, under the
  existing merge-in rules (`pandect/src/graph_session_merge.rs`,
  `merge_rules.rs`). Trash and restore are operations too. Concurrent edits
  from two devices within one session are concurrent heads that the fold
  joins or surfaces as a conflict; a deliberate branch is a session. The
  mere syncs and seals as one log.
- **C3. The full check runs where an edit is made, and again where it is
  received (Mark, 2026-11-06).** The authoring device runs the domain's full
  policy on the folded candidate before it signs the operation. Every
  receiving device runs it again when it folds the operation in, from a
  checkpoint of the fold rather than from the start of the log. An operation
  that fails on a receiver is kept as rejected and shown, never applied and
  never silently dropped, as Knot's `Tolerant` mode does with undecodable
  relations.
- **C4. Operations whose parents have not arrived wait (Mark, 2026-11-06).**
  They are stored as pending (Stickleback's `PendingCausalOperation`) and
  folded in, and checked under C3, once their parents arrive. Only the
  structural checks run before storing: signature, admitted writer, causal
  metadata bounds and a decodable body. An operation that fails those is
  refused.
- **C5. Code splits by layer (Mark, 2026-11-06).** The domain contract
  (grammar, fold, policy) is a Stickleback interface in Eidetic, so Knot and
  other non-graph domains depend on Stickleback, not on Mere's graph crates.
  The graph grammar and fold move from Graphshell's `personal_sync.rs` into
  pandect, where Mere's graph authority stays (resident split plan). The
  reservoir and the resident compose domains as they do now. M1's registry
  holds Stickleback domain implementations.

## Done when

This document names what a domain supplies and what a client may do,
and Cleromancy is checked against it with any changes recorded. O8 and C1
to C5 are ruled; what remains is the Cleromancy check.
