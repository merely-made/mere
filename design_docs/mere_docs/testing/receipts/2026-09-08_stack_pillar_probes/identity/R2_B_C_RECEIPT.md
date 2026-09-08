# R2-B/C disposable model receipt

Date: 2026-09-08

Scope: research models for capture-result correlation and representation custody.
These files do not implement or modify Genet, Mere, or Turnstone. They exercise
the proposed contract shapes in memory, including deliberately broken controls.

## Evidence baseline

- Mere: `cc0b6aa50dc41dbb42a202cf1faf2d3408062d2a`; the family composition brief
  has a concurrent working-tree edit.
- Turnstone: `fa4cca57363f2beef02d882fbc401565741fba33`.
- Genet: `ee0b314b3e9ac4a2fadb07fb7816990fa3f2b71d`; scripted DOM/runtime files have
  concurrent working-tree edits. They were inspected but not changed or built.
- Model: `identity_custody_probe.py`, SHA-256
  `C87FB4F96FFF3FFB2A7B95A1E42B301D32897047E60798174F338230401C27FE`.

The model uses SHA-256 as a standard-library stand-in for key equality.
Production Muniment uses BLAKE3. The experiment does not compare algorithms,
collision resistance, persistence backends, serialization, or byte throughput.

## Command and result

From `C:\Users\mark_\Code`:

```powershell
py -3 scratch\pillar-research-20260908\identity\identity_custody_probe.py
```

Result after the request-reuse and reference-owner reviews: exit 0, 14 tests,
0 failures, 0 errors, about 0.003 seconds of test time.

The passing cases establish within the model:

1. A completion resolves by `(surface, request)` and then compares the complete
   frozen target `(session, durable node, document generation, surface)`.
2. Navigation, node replacement, session switch, and surface replacement all
   refuse the late completion before a blob or observation is admitted.
3. A `(surface instance, request id)` pair is single-use through terminal
   success or refusal. Re-registration is refused, and a repeated or delayed
   completion is typed as unknown or duplicate.
4. Equal bytes share one artifact entry while two successful requests retain
   distinct observation identities.
5. Every permutation of retiring live-page, tombstone, and download references
   preserves the blob through the penultimate reference and deletes it only
   after the final reference is gone.
6. References retain `(owner, artifact hash)`, not only a category-level set of
   hashes. Removing live page A therefore leaves live page B's independent
   equal-blob reference intact.
7. Live-to-tombstone and tombstone-to-live moves add the destination reference
   before removing the source. An observation between those steps still sees a
   reference and cannot propose the blob as an orphan. Production must serialize
   the whole move with GC proposal/apply; ordering alone is defense in depth.
8. Delete and recovery preserve the same durable node id and envelope value.
9. GC apply rechecks current references after proposal. A reference inserted
   between propose and apply prevents deletion.
10. Redacted export omits the capture reference and records that omission while
   leaving local custody unchanged.

The broken controls are observable:

- Attaching to the current target after navigation produces a target unequal to
  the frozen page A target.
- Using artifact hash as observation identity collapses two equal-byte captures
  to one stored observation, rather than merely returning equal ids while
  retaining hidden entries.
- Applying a stale GC proposal without rechecking deletes a blob which once
  again has a live reference.
- Removing the live reference before adding the tombstone reference exposes a
  momentary false orphan to an interleaved GC proposal.

## Contract selected by the model

Capture correlation should use two stages. The host first resolves a pending
request by the surface-scoped key `(surface-instance identity, request id)`.
That pair must never be reused during the surface instance's lifetime, after
either success or refusal. A production allocator could use a non-wrapping
counter or make an epoch part of surface identity; this experiment does not
claim either allocator exists yet. The host then
compares the entire immutable target captured at request admission: session id,
durable node UUID, document/navigation generation, and surface identity. Any
mismatch is terminal and deposits neither blob nor envelope. Current focus,
URL, appearance identity, and arena `NodeId` are unsuitable attachment keys.

A successful observation needs an identity separate from its artifact hash.
The content hash addresses immutable bytes and deduplicates storage. The
observation records the request, target, acquisition facts, and artifact hash.
Equal bytes may therefore support several observations without duplicating the
blob or merging provenance.

Custody should be owner-reference based. Turnstone supplies the complete set of
`(reference class, owner identity, artifact hash)` entries from live capture
envelopes, surviving tombstone envelopes, and download facets. The owner is
required because several live pages or downloads may independently cite one
deduplicated blob. The storage/retirement seam may propose deletion, but apply
must recheck the current reference set. Export redaction controls disclosure of
references; it does not mutate local custody. Recovery moves the same reference
and envelope from tombstone state to live state and retains the durable node
UUID. That state transfer must be serialized with GC; within the boundary it
adds the destination before removing the source so even an intermediate reader
does not observe a zero-reference interval.

## Limits and implementation gate

This is model evidence only. It does not establish production adapter wiring,
Muniment/redb transaction behavior, persisted schema compatibility, actor
ordering, Weld pixels, Livery capture, place authority, remote deletion, or a
headed receipt. It does not close Turnstone capture P2/P3 or lifecycle L-P2/P3.

Before implementation, select a versioned envelope format and its observation-id
minting rule; define generation mint/increment ownership; define typed terminal
errors and retry behavior; and choose a schema migration or refusal rule for old
tombstones. Define the atomicity/serialization boundary covering reference-state
transfers and GC proposal/apply. Production tests should replay the same negative
cases through the actual action/effect and serialized custody actor paths. A
cross-product custody contract remains gated on a real heterogeneous second
product consumer.
