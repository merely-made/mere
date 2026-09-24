# Batch 20 — ambiance design (new design doc)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/design/2026-09-23_ambiance_design.md | current | yes | 54 | 54 | 0 | 0 |
| **Totals** |  |  | **54** | **54** | **0** | **0** |

**Totals: 1 doc, 54 claims checked (54 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `03c05dbd` (2026-09-23), with this pass's edits in the
working tree:
- the new document;
- its `DOC_README.md` entry, and a sentence added to the family composition
  thesis's entry;
- in `TERMINOLOGY.md`: new *ambiance* and *reservoir* entries, the *mere* entry
  rewritten per data domain and as a repository, and a sentence in the
  *Eidetic* entry;
- a dated note at the top of `2026-08-12_family_composition_thesis_brief.md`.

Sibling sources were read from the local checkouts on the same day. The IBM
Redbook abstract was fetched from `redbooks.ibm.com`. `archive_docs/` is
excluded.

This batch exists because the document is new. The record is taken after the
document's third revision, all on the same day. The 2026-09-23 rulings were
given in a Cleromancy design session and are recorded in this document first.
They are the primary record, not claims about the tree, so they are not
counted below.

## mere_docs/design/2026-09-23_ambiance_design.md

- disposition: current
- status line: "Status (2026-09-23): recorded; every question raised while writing it was ruled the same day (§9)." — accurate: yes
- claims checked: 54 — holds: 54, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The family composition thesis's "every application its own datalake"
  is superseded on its unit by the 2026-09-23 per-domain ruling. The brief
  carries a dated note and keeps its body as history.

### Recommended action

- none for this record. Alembic's own open decisions stand, event-log shape
  included, and the document says so.

### Notes

The claims checked, and where each holds:

- **Isocosm wing design plan**
  (`repos/isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`), 10 claims:
  - rulings 69, 70, 71 and 80;
  - reading D11;
  - the record's one-ladder reading of Woodshed's four states;
  - "the ambient tier is regenerated from the merged facts and can never conflict";
  - §3.4.1's three tiers of keeping;
  - the 2026-09-20 finding that the phrase was absent from Mere's and Woodshed's documents;
  - the record setting the ambient pulse and living backdrop beside the ambient tier.
- **Isocosm sim plan**, 1 claim: the wilderness row.
- **Woodshed musical projections plan**
  (`repos/woodshed/design_docs/2026-09-04_musical_projections_plan.md`), 10 claims:
  - the ambient context boundary;
  - the reason-and-reading rule;
  - "Focus never adds a Card";
  - the distinct-columns rule;
  - unavailable scores distinct from zero;
  - breadth as a user preference;
  - the eviction rules;
  - the Grid/Snake/Circle rule;
  - the discovery-history rule;
  - the 2026-09-08 status and the readings it lists as built.
- **Alembic memory and engrams**
  (`mere_docs/technical_architecture/2026-06-09_alembic_memory_and_engrams.md`), 6 claims:
  - §2's three levels, with their promotion, eviction and deletion rules;
  - Athanor's proposal emission, its non-mutation of graph truth, and forgetting;
  - Eidetic R0;
  - §9's structural event log, and its undo and Timeline roles;
  - "local-until-engram is the privacy boundary";
  - §10's open event-log shape.
- **Alembic implementation plan**, 1 claim: its status line.
- **Pandect**, 3 claims:
  - the multiplexer thesis quoted in `crates/system/pandect/src/lib.rs`;
  - the package description in its `Cargo.toml`;
  - the freeze, thaw, fork and compose behaviour in
    `crates/system/pandect/src/graph_codicil.rs` and `snapshot_merge.rs`.
- **Browser multiplexer framing**, 1 claim: the durable graph session.
- **Lexicon brief** §4.1, 1 claim: "Backed by `eidetic`" and "All four tiers
  are **forkable**".
- **The existing senses of *session***, 1 claim: projection, admitted, the
  Session subgraph binding, and Cleromancy's reading session.
- **Earlier Mere documents**, 11 claims:
  - swatch primitive design §2;
  - subgraph derivation;
  - projection scenes §4 and receipt 7;
  - the Scenograph content catalog's Rosette entry, quoted without attribution
    to Mark because the catalog gives none;
  - the family composition thesis's title and quote, and its new dated note
    (2 claims);
  - DOC_README's t1–t4 vocabulary;
  - the physics scenes plan;
  - the meaningful physics signals plan;
  - the family shared identity plan's personae vault path;
  - the suite census's Knot vault.
- **TERMINOLOGY**, 7 claims:
  - "pay-as-you-go";
  - the retired per-application *mere* examples;
  - the former unminted *reservoir* gloss;
  - Turnstone's history;
  - *Standing* as reputation;
  - *Codicil* replacing *Engram*;
  - the new and revised entries.
- **The reservoir plan**, 1 claim: §9's pointer, which says the plan implements
  the per-domain meres, their sessions and archive, and the access and ambient
  grants. The session-senses claim now also names the per-persona web session
  store, from the native session store plan.
- **IBM Redbook SG24-8274**, *Designing and Operating a Data Reservoir*, 1
  claim. Its abstract reads: "The data reservoir is a reference architecture
  that balances the desire for easy access to data with information governance
  and security". Fetched 2026-09-23.

How the document changed on the way to this record:
- **First draft.** It narrowed a phrase-absence claim to Mere and Woodshed, as
  the Isocosm record states it. It softened "most of" to "several of" for
  Woodshed. It removed an unverified claim about Woodshed's progression
  relations.
- **Second draft.** It proposed keeping levels named "noted" and "asserted",
  and a working-state word, "checkout". Both were made before reading Mere's
  session and memory design.
- **Third revision.** Reading pandect, the multiplexer framing, the lexicon
  and Alembic showed that Mere already has both: sessions and codicils, and the
  short-term, long-term and codicil levels. Mark retired the assistant's words
  in favour of Mere's own. The TERMINOLOGY *ambiance* entry was corrected the
  same way.
