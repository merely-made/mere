# Engine to workspace contract

**Status:** 2026-11-16: draft for review. This records the P8 contract after rulings D147–D149, D158, D161, D163, D166, D169 and D179. It describes current ownership and the intended boundary; it does not claim the Graphshell host integration is implemented.

## Ownership

Mere owns the graph session, journal, revision replay, Resource-stratum dataset, and the live and pinned indexed views. Graphshell owns host interaction, view selection, arrangement, scene construction and presentation. MereHost takes and owns a PinnedView and builds its pinned scene from that view; the pin supplies its session, cursor and digest. The host owns the pin lifetime and drop cost. It does not mint the identity of the graph revision it displays (D179). This follows the domain contract's separation between graph authority and client intents, and D44's separation of replicating arrangement state from device-local attention.

The engine's public revision is the pair **(session, journal cursor)**. The cursor identifies the local journal prefix. A kernel counter, Graphshell `projection_revision`, or `scenotime::Revision` is not an independent dataset revision; if retained for cache invalidation or scene compatibility, it must derive from the engine revision and remain scoped to that view. Where a revision crosses a replica or device boundary, its name is the **v2 chained digest** of that journal prefix. The engine does not mint a separate revision identifier (D148, refined by D154 and D158; D141 F13).

## Engine to host

Opening a host view begins with a scene snapshot as a keyframe. Subsequent edits travel as deltas carrying the touched sources, so the host refreshes its held indexed dataset instead of rebuilding the whole index for each edit (D149). A reconnect or catch-up starts from a keyframe and applies later deltas in journal order. The live index is refreshed per edit. A pinned or historical view names its session and cursor, materializes from `graph_at` using the nearest valid checkpoint, and retains its resulting graph and dataset while held (D161, D163).

The dataset refresh budget is at most **8 ms after one edit at 100k nodes**, half of a 60 Hz frame. The engine edit target is at most 100 ms at 100k nodes; this is an edit-path target, not a per-frame budget. A host retains ownership of a `PinnedView` and its release cost. The index is built on the first dataset read. A host that cannot afford that work on its UI thread moves the read or drop to its worker; Mere does not add a hidden thread or registry (D161, D163, D166).

The exact dataset revision bound to a rendered view must accompany the keyframe and its subsequent deltas. Intents against a stale bound revision are refused. D148 determines revision identity; D149 determines transport shape. The current host wire's `Revision(u64)` remains a compatibility counter, not sufficient evidence of the journal prefix by itself. Graphshell `origin/main` still constructs its scene by reading `self.graph()` (`graphshell/host/crates/graphshell/graphshell-host/src/mere_host.rs:1329-1353`) and stamps it with `self.projection_revision` (`graphshell/host/crates/graphshell/graphshell-host/src/mere_host.rs:1415-1420`); stale-intent rejection also compares that host counter (`graphshell/host/crates/graphshell/graphshell-host/src/mere_host.rs:1578-1582`). Thus the current code does not yet implement this binding contract.

## Arrangement and Resource content

Arrangement is host-owned view state and may also have RDF representation in the Surface stratum. An arrangement edit may advance the journal cursor when it changes graph-stored Surface facts. The invariant is content-specific: every Resource-stratum quad before the edit must remain present and byte-for-byte equivalent afterward (D147). A single revision counter does not split into per-stratum counters under this contract. Attention state such as camera and focus remains device-local as specified by the domain contract and D44.

The third probe makes a graph-stored Surface arrangement edit and compares the exact full Resource-stratum quad sets before and after that edit (D179), with equality in both directions: no prior quad disappears and no new quad appears. Its negative control deliberately corrupts Resource content and must fail the comparison. Portable digest encoding stays open; the probe need not choose one because it can compare the full sets in-process.

The headless revision probe records the `(session, cursor)` pair and the running v2 journal-prefix digest at that cursor, through `PinnedView::session`, `cursor`, and `digest` (`crates/system/pandect/src/graph_session_pin.rs:31-48`). Pandect holds `DigestChain` in its session and rebuilds it on open (`crates/system/pandect/src/graph_session.rs:45,751,950`); the pin API already exposes the v2 digest. The probe records the lookup cost for cursor-only identity and for cursor plus digest at its selected journal sizes. It does not recompute the v1 `prefix_digest` over the entire history. D158's recorded results are 2–10 μs per append, 2–8 ns per digest lookup, 32 bytes per entry, and 0.8–5.1 s to build a 600k-entry chain; this handoff does not rerun those measurements.

## Refusals and partial application

An import-shaped transaction that contains a refused edit refuses the whole change. `transact_live` also refuses the whole operation. A merge-in can apply accepted edits, but returns `Merged { applied, refused }`; each refusal records the edit position, optional part and reason text. Revert remains partial and reports kept edits. `PinnedView` may fail when the cursor is outside retained/replayable history; its caller owns lifetime, retry and release policy (D161, D163, D166, D169).

## Governing rulings

- D147: the arrangement probe preserves all Resource-stratum quads.
- D148, D154 and D158: session and cursor name a local revision; the v2 chained journal-prefix digest names it across replicas.
- D149: a keyframe snapshot is followed by touched-source deltas.
- D161 and D163: live index refresh, checkpoint-based pinned views, 8 ms refresh budget, whole-change refusal for imports.
- D166 and D169: operation refusal, pin lifetime and cost, and merge refusal reporting.
- D179: the host owns its pin; the third probe covers graph-stored Surface arrangement, and P8 settles the three transport forks.

## Open questions

Ruled 2026-11-18 (D181): the keyframe is a versioned CBOR envelope carrying the engine session, cursor and full v2 digest with the projection epoch and revision around the projection snapshot; a host that misses a delta resumes from its acknowledged identity and epoch, replaying a retained suffix that verifies, or receives a fresh keyframe; the portable Resource digest is a versioned, domain-separated hash of RDFC-1.0 canonical N-Quads, since replaced by sorted RDF 1.2 N-Quads that refuse blank nodes, `mere-resource-sorted-nquads-blake3` version 1 (D190, D194). Ruled 2026-11-23 (D201): a host retains its last 64 deltas, evicted early past one keyframe's bytes, and replays a suffix with one clone. Ruled 2026-11-24 (D206): until placement keeps edits local, every gap is answered with a keyframe and no deltas are retained. Arrangement and attention share a host model as C7 ruled (D133, D138): arrangement stays in the view log, attention lives in a device-local `AttentionState` slot. Ruled 2026-11-24 (D207): the envelope is built before P8 closes. Still open: the envelope's schema and its vectors. See `Code/testing/engine-split/p8_binding/transport_forks.md` for options and recommendations; record each ruling in the engine split plan before changing this contract. The three probes and the existing host integration gap are described in `Code/testing/engine-split/p8_contract/report.md`.
