# Batch 31 — graph semantics plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-04_graph_semantics_plan.md | current | yes | 29 | 29 | 0 | 0 |
| **Totals** |  |  | **29** | **29** | **0** | **0** |

**Totals: 1 doc, 29 claims checked (29 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `68d2a928` (2026-10-04), with this pass's edits in the
working tree: the new plan, its `DOC_README.md` entry at the head of the
`mere_docs/implementation_strategy/` list, and dated ruling annotations in
TERMINOLOGY (link), the petgraph-RDF plan, the statements-over-schema stance,
the node navigation lineage plan, the graph query layer plan, the subgraph
derivation design, the ambiance design and the family composition thesis.
Sources read: the code paths cited in the plan's §2 and the archived
2026-05-18 node identity brief and plan. No web sources; `archive_docs/` is
excluded from the audit but was read as evidence.

This batch exists because the document is new.

## mere_docs/implementation_strategy/2026-10-04_graph_semantics_plan.md

- disposition: current
- status line: "Status (2026-10-05): in progress. Rulings 1 to 19; P1 implemented on branch `graph-semantics` (`459cad84`), not merged; P2 under way on that branch, whose plan copy predates rulings 9 to 19 (§3). C5 to C8 open (§5)." — accurate: yes
- claims checked: 29 — holds: 29, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

Claims checked, by finding: F1 the `assert_statement` dedup key and in-place
overwrite (`edge_data.rs` 157-198) and the same in `insert_statement`
(214-251); F2 the six `GraphScope` variants (`types.rs` 83-91), ingest as the
one scope-carrying production writer (`ingest/apply.rs` 125-155),
`AssertSemanticPredicate` landing in `Default` (`edge_ops.rs` 116), and the
`Source`/`User` hits sitting in test modules (`lib.rs` tests from 695,
`query.rs` from 227, `serialize.rs` from 93); F3 the `Author` envelope
(`journal.rs` 64-98); F4 retract by id (`edge_data.rs` 203); F5 random-UUID
`add_node` with a `Vec` URL index (`mod.rs` 438-480), UUIDv5 at ingest
(`ingest/apply.rs` 74), verbatim URLs (`address.rs` 153), eidetic's content
fingerprint and tracking-parameter strip (`browsing/page.rs`); F6
`NavigateNode` and `update_node_url` (`apply.rs` 1152-1171, `mod.rs` 519-543),
`navigate_member` (`nodes.rs` 300-315), canvas `visit` (`input.rs` 417)
called only from canvas tests; F7 `CONSTRUCT`/`DESCRIBE` unsupported in
`query.rs`, `SubgraphBinding` and the nine `SubgraphKind`s (`subgraph.rs`
51-88), `Container.nested` (`container.rs` 76-79), and no freeze path (a
search for freeze, materialize and nest operations over subgraphs found only
`copy_component_from`, which forks a component into a new session graph); F8
`from_snapshot` (`snapshot/from.rs` 48) and `pending_targets`
(`statements.rs` 80-83) with no reader outside that file; F9 the narrowing
gradient (family composition thesis, 146-152). Added with rulings 5 to 8: the
2026-06-18 held-RDF-truth multiples (about 11x memory, 19x load, 18x mutation;
petgraph-RDF plan 38-44) quoted in ruling 6; the kernel and eidetic-core both
depending on chartulary and not on each other (their `Cargo.toml`s), which
places the shared canonicalizer in P2; and the seven `ContainmentSubKind`s
(`edge_taxonomy.rs` 119-127) named in checkpoint C2. Added with rulings 9 to
15: journal entries as `AttributedDelta { author, delta }` (`journal.rs`
161-170) with minted statement ids captured by `ReplaySetEdgesByIds`
(`apply.rs` 518-530); sessions opening from a baseline plus the journal
(`pandect/src/graph_session.rs` 406-430); the default semantic write storing
no provenance or time (`edge_payload.rs` 271-279); and the full Semantic,
Imported, Provenance and Arrangement sub-kind lists behind the placement
table (`edge_taxonomy.rs`). Round 4 adds no code claims: its inventory is of
document vocabulary (TERMINOLOGY, the swatch and ambiance designs, the family
composition thesis), and "aspect" appears in code only as `AspectRatio` and
wgpu's `TextureAspect`. Round 5 checked against branch `graph-semantics` (its
base `36893553`, its C1, C2 and C3 records at `9ec20ffb`, `5666943e`,
`4025f55f` and `2bf393ac`, and its C4 fork text) and `SurfaceId` at
`graph-kernel/src/accessibility.rs` 49.
