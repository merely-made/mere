# Engine to workspace contract

**Status:** 2026-11-16: draft for review. This records the P8 contract after rulings D147–D149, D161, D163, D166 and D169. It describes current ownership and the intended boundary; it does not claim the Graphshell host integration is implemented.

## Ownership

Mere owns the graph session, journal, revision replay, Resource-stratum dataset, and the live and pinned indexed views. Graphshell owns host interaction, view selection, arrangement, scene construction and presentation. A host may request and hold an engine view, but it does not mint the identity of the graph revision it displays. This follows the domain contract's separation between graph authority and client intents, and D44's separation of replicating arrangement state from device-local attention.

The engine's public revision is the pair **(session, journal cursor)**. The cursor identifies the local journal prefix. A kernel counter, Graphshell `projection_revision`, or `scenotime::Revision` is not an independent dataset revision; if retained for cache invalidation or scene compatibility, it must derive from the engine revision and remain scoped to that view. Where a revision crosses a replica or device boundary, its name is the digest of that journal prefix. The engine does not mint a separate revision identifier (D148, refining D141 F13).

## Engine to host

Opening a host view begins with a scene snapshot as a keyframe. Subsequent edits travel as deltas carrying the touched sources, so the host refreshes its held indexed dataset instead of rebuilding the whole index for each edit (D149). A reconnect or catch-up starts from a keyframe and applies later deltas in journal order. The live index is refreshed per edit. A pinned or historical view names its session and cursor, materializes from `graph_at` using the nearest valid checkpoint, and retains its resulting graph and dataset while held (D161, D163).

The dataset refresh budget is at most **8 ms after one edit at 100k nodes**, half of a 60 Hz frame. The engine edit target is at most 100 ms at 100k nodes; this is an edit-path target, not a per-frame budget. A host retains ownership of a `PinnedView` and its release cost. The index is built on the first dataset read. A host that cannot afford that work on its UI thread moves the read or drop to its worker; Mere does not add a hidden thread or registry (D161, D163, D166).

The exact dataset revision bound to a rendered view must accompany the keyframe and its subsequent deltas. Intents against a stale bound revision are refused. D148 determines revision identity; D149 determines transport shape. The current host wire's `Revision(u64)` remains a compatibility counter, not sufficient evidence of the journal prefix by itself.

## Arrangement and Resource content

Arrangement is host-owned view state and may also have RDF representation in the Surface stratum. An arrangement edit may advance the journal cursor when it changes graph-stored Surface facts. The invariant is content-specific: every Resource-stratum quad before the edit must remain present and byte-for-byte equivalent afterward (D147). A single revision counter does not split into per-stratum counters under this contract. Attention state such as camera and focus remains device-local as specified by the domain contract and D44.

The Resource-stratum digest in the third probe is computed over a deterministic ordering and canonical quad representation. The probe compares the full set or digest, not a sample, and requires a positive control in which a Resource edit changes it. This states the probe's comparison, not a newly selected digest encoding.

## Refusals and partial application

An import-shaped transaction that contains a refused edit refuses the whole change. `transact_live` also refuses the whole operation. A merge-in can apply accepted edits, but returns `Merged { applied, refused }`; each refusal records the edit position, optional part and reason text. Revert remains partial and reports kept edits. `PinnedView` may fail when the cursor is outside retained/replayable history; its caller owns lifetime, retry and release policy (D161, D163, D166, D169).

## Governing rulings

- D147: the arrangement probe preserves all Resource-stratum quads.
- D148: session and cursor name a local revision; journal-prefix digest names it across replicas.
- D149: a keyframe snapshot is followed by touched-source deltas.
- D161 and D163: live index refresh, checkpoint-based pinned views, 8 ms refresh budget, whole-change refusal for imports.
- D166 and D169: operation refusal, pin lifetime and cost, and merge refusal reporting.

## Open questions

This draft does not decide the normalized cross-process keyframe schema, recovery behavior when a host misses a delta, the canonical quad encoding for Resource digests, or how Surface arrangement facts and device-local attention share a host model. Record each answer in the engine split plan before changing this contract. The three probes and the existing host integration gap are described in `Code/testing/engine-split/p8_contract/report.md`.
