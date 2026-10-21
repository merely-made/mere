# mere-site

Portable native-site profiles, page metadata, manifest validation and canonical
`PublishedSnapshotV1` records shared by Knot and Djinn. The library has no editor,
filesystem, protocol server, runtime, identity or persistence dependencies.

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
custody and listener lifetime. Revision history, replication and publication
authority are separate from this immutable interchange contract.

Run `cargo test -p mere-site` from the Mere workspace.
Run `cargo test -p mere-site --features history` for durable reopen, failure and
concurrent-checkpoint controls.
