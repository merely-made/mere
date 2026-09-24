# Batch 22 — insigne proofs plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| dramatis_docs/implementation_strategy/2026-09-23_insigne_proofs_plan.md | current | yes | 14 | 14 | 0 | 0 |
| **Totals** |  |  | **14** | **14** | **0** | **0** |

**Totals: 1 doc, 14 claims checked (14 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `97b3865f` (2026-09-23). Turnstone, knot-editor, mer3ly,
hocket, woodshed and cleromancy sources were searched in their local checkouts
on the same day. `archive_docs/` is excluded.

This batch exists because the plan is new. Mark's 2026-09-23 ruling on the
split is recorded first in the crate consolidation plan's insigne row and the
gaz founding plan; it is not counted again here.

## dramatis_docs/implementation_strategy/2026-09-23_insigne_proofs_plan.md

- disposition: current
- status line: "Status: plan. Mark agreed the split on 2026-09-23; how issuing is expressed (§2) is his call before phase A starts. Nothing has moved yet." — accurate: yes
- claims checked: 14 — holds: 14, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The plan moves the delegation grammar out of personae, which the
  2026-08-11 reconciliation placed there; it cites that ruling and changes the
  grammar's location, not its content.

### Recommended action

- none for this record. §2's decision is Mark's and is marked as open.

### Notes

The claims checked: the moved type list against `delegation.rs` and
`provider.rs`; the domain strings; `id` as blake3 over the signing bytes; checks
ending in `Ed25519PublicKey::verify`; `ed25519-dalek` 3 as personae's
dependency; `issue` as inherent methods; the key accessors' personae return
type; the orphan-rule and cycle constraints; 787 references in 97 files, 120
call sites in 60 files, and 11 `IdentityProvider` implementations, by ripgrep;
the repos those span; and notochord's `AdmittedPrincipal` as local and
non-`Serialize`.
