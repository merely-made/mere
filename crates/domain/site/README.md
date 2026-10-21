# mere-site

Portable native-site profiles, page metadata, manifest validation and canonical
`PublishedSnapshotV1` records shared by Knot and Djinn. The default library has
no editor, filesystem, protocol server, runtime, identity or persistence dependencies.

Optional feature `history` adds `history::SiteHistory<B>` over a host-supplied
Muniment backend: explicit record/list/load operations, separately versioned
checkpoint journals and exact V1 snapshot blobs. Hosts assign `SiteId`, clock,
labels and capacity limits. Recording requires transactional read/write support;
competing changes refuse rather than overwrite. Failed journal commits may
leave unreferenced blobs. No pruning, collection or publication is automatic.

The snapshot's exact JSON representation and BLAKE3 digest preserve Knot's V1
handoff. Fixed compatibility fixtures cover Scroll, Gemini, Spartan and Micron,
including Unicode, CRLF and native metadata. V1's profile-specific filenames,
validation and byte ceilings are unchanged.

Knot supplies saved files and protocol-serving adapters. Djinn supplies durable
custody and listener lifetime. Replication and publication authority remain
owned by their existing layers.

Optional `publication` binds retained Gemini, Scroll and Micron snapshots to
Gemot's signed contributions and causal collections. Eidetic's `save_schema`
and `save_typed` retain `publication::snapshot_schema()` and the snapshot itself;
the typed binding preserves exact canonical V1 bytes. The host explicitly chooses
privacy, provenance, trust, backend and any acquisition. No content is promoted
to public automatically.

`publication::contribute` verifies locally retained PublicPortable content and
calls Gemot's stable-identity membership/capability gate. Select the returned
operation with Gemot's existing collection command and observed heads.
`SiteSelection` names that contribution at an exact `CollectionVersion`;
`publication::open` rechecks current effective selection around local resolution.
Anonymous public readers do not need Moot membership. `OpenedSite::page("/")`
maps to the native index and returns complete native metadata and bytes.

Opening is a point-in-time read. Hosts revalidate when their serving policy
requires current authority. This feature supplies neither a listener nor a
hosting promise, actor, automatic latest-version rule or cross-Moot signed
publication revision. Protocol replies and peer acquisition remain separate.
Spartan's existing V1 model stays available but is outside this first binding.

Run `cargo test -p mere-site` from the Mere workspace.
Run `cargo test -p mere-site --features history` for durable reopen, failure and
concurrent-checkpoint controls.
Run `cargo test -p mere-site --features publication,history` for signed selection,
fork/withdrawal controls and all three native publication profiles after reopen.
