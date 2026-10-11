# Batch 65: graph model research review

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| 2026-08-12_family_composition_thesis_brief.md | current | yes | 4 | 4 | 0 | 0 |
| mere_docs/research/2026-09-02_port_gui_composition_and_comparable_stacks_brief.md | current | yes | 2 | 2 | 0 | 0 |
| mere_docs/technical_architecture/2026-07-08_mere_as_the_unifying_graph.md | current | yes | 2 | 2 | 0 | 0 |
| **Totals** | | | **8** | **8** | **0** | **0** |

**Totals: 3 docs, 8 claims checked (8 holds, 0 stale, 0 unverifiable),
0 remaining contradictions within this pass; 0 status lines wrong.**

Audit base: Mere `4a47b488` plus these owned research amendments, 2026-10-11.
The implementing agent judged the corrected passages and research addition;
these counts do not constitute a fresh whole-document audit. Concurrent
Fleece/Eidetic moves, their new plan, DOC_README and Forme edits are outside
this pass. No runtime work is opened or qualified.

The evidence is the primary dataspace paper (Franklin, Halevy and Maier,
SIGMOD Record 2005), iDM (Dittrich and Vaz Salles, VLDB 2006), W3C Web
Annotation and Solid Protocol, all read on 2026-10-11 and linked in the
family brief. Architectural applications of those sources are explicitly
marked as readings for Mere rather than adopted external contracts.

Validation: the three owned documents introduce no mechanical audit findings
or count changes against their committed versions. All five new relative
targets and anchors resolve; the three D2 records and eight claims parse
without errors. Scoped `git diff --check` passes. Concurrent source moves
were left intact and no runtime checks are claimed.

## 2026-08-12_family_composition_thesis_brief.md

- disposition: current
- status line: "### Research review: logical model and usable sharing (2026-10-11)" — accurate: yes
- claims checked: 4 — holds: 4, stale: 0, unverifiable: 0

### Stale claims

Corrected the Solid row's one-pod and universal-schema assumptions. Removed
the unsupported adoption verdict as an architectural justification.

### Contradictions

The former Solid contrast implied that linked-data structure entails uniform
application schemas. The new comparison separates those architectural choices.

### Recommended action

Use the identified logical-model and capability distinctions in the existing
consumer proofs. Preserve the lifecycle proposals and implementation gates.

### Notes

Checked four claims: the named internal predecessors exist; iDM supplies a
logical graph with on-demand components without mandatory materialization;
the dataspace and annotation readings preserve autonomy and source/part/state
distinctions; Solid supports multi-storage clients and media-typed resources
without imposing a universal application schema. The source comparisons do
not establish Mere runtime interoperability.

## mere_docs/research/2026-09-02_port_gui_composition_and_comparable_stacks_brief.md

- disposition: current
- status line: "**Research qualification (2026-10-11).**" — accurate: yes
- claims checked: 2 — holds: 2, stale: 0, unverifiable: 0

### Stale claims

Corrected the table's universal-schema objection to Solid.

### Contradictions

The old objection is explicitly withdrawn; the dated port inventory remains
dated rather than being presented as freshly measured implementation.

### Recommended action

Use the family brief's canonical research review for the current comparison.

### Notes

Checked two claims: the Solid row matches its primary protocol's resource and
storage model; the qualification links to the current review while preserving
the earlier source/consumer evidence boundary.

## mere_docs/technical_architecture/2026-07-08_mere_as_the_unifying_graph.md

- disposition: current
- status line: "**Status:** vision (Mark, 2026-07-08)." — accurate: yes
- claims checked: 2 — holds: 2, stale: 0, unverifiable: 0

### Stale claims

Qualified the inference from content addresses and predicate IRIs to general
cross-vault semantic intelligibility. The original vision remains dated.

### Contradictions

The qualification distinguishes the architectural aim from interoperability
requiring compatible vocabularies or adapters. It also identifies the current
domain unit and narrowed orrery meaning.

### Recommended action

Read the original vision through the current model and consumer evidence.

### Notes

Checked two claims: identity alone does not establish shared interpretation;
the current domain/clip/live-entity direction has its named canonical home.
