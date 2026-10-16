# Batch 83: Link model amendment

Audit date: 2026-10-15. Re-judgment of the link model after an implementing agent's
review and Mark's L5–L6 rulings, against Mere `276004182`. Checked the review's code
claims: `crates/conatus/seiche/src/forces.rs`:184 (`EdgeSpring` acts along the endpoint
line only) and `crates/cambium/scenes/scenomise/src/projection.rs`:43
(`RelationshipProvenance` carries source, revision, method, version, provider and
evidence). Checked the PROV-O range of `prov:wasInvalidatedBy` (an activity).

## mere_docs/technical_architecture/2026-10-15_link_model.md

- disposition: current
- status line: "design, no code changed. Mark ruled L1–L4 on 2026-10-15 and L5–L6" - accurate: yes
- claims checked: 46 - holds: 44, stale: 0, unverifiable: 2

### Stale claims

None. §7.3's earlier claim that a lateral load deflects the bend has been removed; the
spring has no lateral state.

### Contradictions

None. L4 stands, and the review's closed-link direction cues are met by marks on the
single stroke. L5 and L6 replace open rulings O2 and O3.

### Recommended action

Implement P1 (bounded shared reading) next. Take O4 to Mark before P3's Woodshed proof.

### Notes

Supersedes batch 82's record for this document. Unverifiable: as in batch 82, Woodshed
and Knot pointers come from the 2026-10-15 survey.
