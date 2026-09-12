# mere-subgraph

Pure subgraph derivation and shape classification for the Mere graph family
(lib name `subgraph`). It owns the per-session subgraph index
(`SessionSubgraphs`), the two linked-subgraph derivations — `Component`
(connected component around a root) and `Ego` (bounded neighborhood) — their
reconciliation against the live kernel graph, and the shape classifier in
`classifier` that ranks a member set as corridor / loop / frontier / facet.
The index persists as the `subgraphs.json` sidecar beside a session's
`graph.json`. Depends only on `forme`, `kernel`, and serde: no UI, no I/O
beyond the sidecar.
