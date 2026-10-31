# Source audit repairs: routing the open findings

**Status:** 2026-10-28: cut from `2026-10-16_full_source_audit_plan.md`, archived in `archive_docs/2026-10-28/`. The audit itself is done: the full-source read of Mere (A1) and the contained repairs (A2, 24 findings) landed in October 2026, and its two main reds closed on 2026-10-18. What is open is the repair of the other 441 findings (Q001 to Q465 less the repaired), the 212 unconfirmed candidates, Q073, Q069 with its producers Q074 to Q079, the personae half of R4, R5's reading, and runtime qualification the audit never ran.

**For:** routing. The findings live in `support/receipts/authority-success-audit/review_notes.json` (findings, 212 `candidates`, 23 `ruled_out`) with the ownership map in `authority_map.json` (A001 to A1536); the old plan held the dated log of how each was found. This plan owns no code. It assigns each finding id range to the owner plan that fits, keeps the audit's own open items, and records where the register and the tree now disagree. Opened by the audit's closeout, which called the repairs and wider runtime qualification "named followup work" with no owner or schedule.
Not here: the repairs themselves, which each owner plan takes into its own phases; the sibling repositories' plans where a crate moved with the family promotion (Eidetic, Armillary, ESP, Gemot, Errand); the audit's A2 sweep, landed.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- Where the record lives: the dated audit log is in the archived copy, moved byte for byte from the scenograph editor plan, and the relocation receipt binds both paths and hashes (R2, audit 48-65). Cite the register, not the log.
- R4: the castellan residue was fixed by option B (build the slot summary so no uninitialised bytes are copied), with option A, personae `from_seed` taking `&[u8; 32]`, logged against dramatis; B alone does not prove the stack clean (audit 90-106, 3279-3297).
- R5: `metre.occurrences` wins over the later `marks` check for a duplicated occurrence key; `SceneInputError.path` stays a free string, a reading and not a ruling (audit 109-124).
- Reading source and preserving old receipts do not turn conditional or unexecuted tests into live acceptance; the audit's probes are source inspection, synthetic boundary probes, freshly compiled public APIs and, rarely, actual product runs (audit 3217).
- "Documented, not repaired" is the status the audit's one-hour diversion limit produced; a repair that follows takes the whole boundary, not one field (audit 179-184).

## Phases

### R1. Route the documented findings
Owner names are this set's plans (for example `graph-semantics` is `2026-10-28_graph_semantics_plan.md`); "Eidetic repo" and "retinue repo" are outside the set. Where a row names a crate the register's `source` field decides, and every assignment is the closest fit for the owner to confirm. Q001 to Q065 are in the register only; the old log opens at Q066.

| Findings | Old plan lines | Families | Owner |
|---|---|---|---|
| Q001-Q065 | none | chartulary, graph-kernel, pandect sessions and wallet, castellan, Djinn resident, testkit, graphshell host and client | Q001-Q003, Q009-Q020, Q031-Q034 `graph-semantics` (assertion and link-reading ones with `link-model`); Q007-Q008 Eidetic repo; Q021, Q023-Q029, Q035-Q042 `custody`; Q022 `alembic-implementation`; Q043-Q047, Q049 `family-update-resident`; Q050 `device-pairing`; Q048, Q051-Q055 `djinn-test-harness`; Q056-Q065 `graphshell-host` |
| Q066-Q085 | 144-300 | scenograph, scenotime, scenomise, chirograph, graphshell client | Q066-Q068 `graphshell-host`; Q069, Q073 `scenograph-editor` (phases below); Q070-Q072 repaired; Q074-Q085 `projection-grammar` |
| Q086-Q106 | 301-601 | cartography, pictograph canvas and signals | Q086-Q090 `projection-grammar`; Q091-Q106 `graph-view-curation`, Q094, Q097, Q099, Q102, Q103 (fields, settling, signals) `dynamics-grammar` |
| Q107-Q122 | 602-1099 | seiche, armillary pool, numen, tenant, conatus, modulus, root build | Q107-Q111 `physics-catalog`; Q112-Q113 repaired (see Open questions); Q114-Q115 `dynamics-grammar`; Q116-Q121 `conatus-engine`; Q122 `workspace-hygiene` |
| Q123-Q153 | 1100-1470 | mere facade, forme, platen, workbench, document-canvas, inker | Q123-Q125 `graph-view-curation`; Q126-Q139 `cambium-presentation` (portable tiles); Q140-Q153 `presentation-engines` |
| Q154-Q195 | 1471-1736 | document-lanes, nematic, errand, fetch, page-load, content-contract, state-witness | Q154-Q178 `smolweb-fidelity` (Djot and Knot grammars Q162-Q165 `presentation-engines`); Q179-Q190 `smolweb-fidelity` (Q182 Content-Range to `2026-09-20_ranged_fetch_plan.md`); Q191-Q193 `presentation-engines`; Q194 `dynamics-grammar`; Q195 `distillery-models` |
| Q196-Q213 | 1737-1902 | mesh board, lease and checkpoint, pictograph suggestions, fleece | Q196-Q208 `distillery-models`; Q209-Q210 `graph-view-curation`; Q211-Q213 fleece: no plan in the set (page capture, Turnstone's plan) |
| Q214-Q249 | 1903-2122 | Djinn pairing and sync, distillery inference and mesh host, eidetic-core, graphshell capture and transfer | Q214-Q215 `device-pairing`; Q216 `graph-semantics`; Q217 `resident-inference` (repaired, see Open questions); Q218-Q220, Q223-Q228 `distillery-models`; Q222 `alembic-implementation`; Q221, Q229-Q233 Eidetic repo; Q234-Q239 `workspace-hygiene` (X2); Q240-Q249 `graphshell-host` |
| Q250-Q319 | 2123-2468 | personal sync, web tree, community hosts, web adapters, receipt tools, Cambium toolkit and hosts, native accessibility | Q250-Q252 `reservoir`; Q253 `cambium-presentation`; Q254-Q258, Q263-Q269, Q272-Q283 `graphshell-host`; Q259-Q262 `moot-protocols`; Q270-Q271 `browser-carrier`; Q284-Q285 `projection-grammar`; Q286-Q288 `graph-view-curation`; Q289-Q313, Q317-Q319 `cambium-presentation`, Q298, Q300, Q314-Q316 `diagnostics-and-accessibility` |
| Q320-Q350 | 2469-2614 | backstep, meristem, sprigging, mesquite, workshop, alembic, linked-data, engines, eidetic-search, esp | Q320-Q325 `cambium-presentation`; Q326 `diagnostics-and-accessibility`; Q327 `workspace-hygiene`; Q328-Q329 `appearance`; Q330 `alembic-implementation`; Q331-Q332 `graph-semantics`; Q333-Q334 `presentation-engines`; Q335-Q338 `resident-inference`; Q339-Q350 `distillery-models` |
| Q351-Q378 | 2615-2792 | graphshell client, commons, gemot, mien, moothold, script hosts, servitor, comms, chrome | Q351 `graphshell-host`; Q352-Q365 `moot-protocols`; Q366 `2026-09-16_lexical_capability_plan.md`; Q367-Q375 `graph-behaviors`; Q376 comms: no plan in the set; Q377-Q378 `illume-text-lexer` |
| Q379-Q423 | 2793-3017 | stickleback, luggage, registry, tabard, receipt tools, gazette, OPFS probe, pelt, signalman, scripts | Q379-Q381 `murm-runtime`; Q382 `distillery-models`; Q383-Q385 `family-update-resident`; Q386, Q391 `diagnostics-and-accessibility`; Q387-Q389 `graph-behaviors`; Q390 no plan in the set; Q392-Q393 `appearance`; Q394, Q422-Q423 `custody`; Q395-Q396 `cambium-presentation`; Q397-Q399 `moot-protocols`; Q400-Q404, Q420 `workspace-hygiene` (X2); Q405-Q416 `presentation-engines`; Q417-Q418 retinue repo; Q419 `workspace-hygiene`; Q421 `platform-qualification` |
| Q424-Q465 | 3018-3233 | retained Burn and CubeCL patches, resident and custody wrappers, browser release receipt | Q424-Q462 `distillery-models` (carried vendor patches); Q463 `resident-inference`; Q464 `custody`; Q465 `cambium-presentation` |

- Done when: every owner plan has taken its ids into a phase or recorded them as declined by ruling, the 212 candidates have an owner, and the register's unrepaired count is re-taken.
- Needs: the owner plans of this set
- Rulings: R2
- From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:7-8,144-3233`

### Q1. Q073, the exact-byte trace identity
The graphshell-client all-features exact-byte trace test fails on a clean main: reserialization adds 15 groups of default relation fields, growing the old trace from 10,391 to 11,741 bytes, and removing only those groups restores the bytes. Decoding compatibility is not byte identity. A versioned canonical-encoding and identity review is deferred; neither serializer defaults nor the golden were changed to turn the gate green.
- Done when: a versioned canonical-encoding and identity review decides the trace's bytes and the all-features exact-byte trace test passes on that decision.
- Needs: `scenograph-editor` and `graphshell-host`
- Rulings: R1, R3 (named as the pre-existing red)
- From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:43,70,133,186-196,579,3250,3310-3311`

### Q2. Q069 and its producers Q074 to Q079, finite geometry and wire validity
Structurally valid scene geometry can serialize into bytes that cannot reopen. Done when admission is consistently finite across snapshots, committed diffs, portable captures and traces, with successful reopening controls and downstream solver coverage; negative and degenerate geometry policy can stay separate from finite-value rejection. A one-field patch would leave the misleading portable-success claim. The producers: Timeline casts its span and numerator to f32 (a finite `[0, 1e100]` score gives NaN and a snapshot that cannot reopen); a host-named L-system grammar realizes as Hilbert; explicit L-system and Penrose depths admit 255 with no work budget; negative radial indices round into the focus and a maximum ring overflows; Sets places an impossible six-way member in one region without an unmet-placement report.
- Done when: as above, plus configurable work and output budgets with bounded refusal for the fractal depths (a broader compiler and host pass).
- Needs: `scenograph-editor` (admission), `projection-grammar` (solvers)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:179-184,202-245`

### Q3. R4's personae item
Ledger a personae change so `from_seed` takes the seed as `&[u8; 32]` rather than by value (item 13 of `research/2026-10-03_upstream_candidates_ledger.md`), recorded in the dramatis ledger first, with no upstream issue or PR. Not done.
- Done when: `from_seed` takes `&[u8; 32]` and the change is recorded in the dramatis ledger.
- Needs: the dramatis repository
- Rulings: R4
- From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:90-106,3279-3297`

### Q4. Runtime qualification the audit never ran
Actual GPU, custody, browser, model, broker and wallet runs were not made; the probes used synthetic doubles and reproduce defects without proving product behavior. The log also records upstream adoptions and headed or GPU scopes as still open for several subsystems.
- Done when: the boundaries the audit probed with synthetic doubles have actual runs recorded by their owner plans, or their owners record them as unrun.
- Needs: `platform-qualification` for headed runs
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:1071,2219,2846,3204-3214,3225`

## Open questions

- **Q112, Q113 and Q217 are repaired**: the register lists them as documented (Armillary pool admission and worker retirement, inference actor loss), while the family promotion plan records Q112 and Q113 repaired in Armillary `92d5b15` on 2026-10-21 and Q217's two public counterexamples repaired in the bounded receiving lane (its own earlier lines still call Q112 and Q113 open). Re-count the 441. From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:7-8,3219`; `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:270-272,504-512,551-554,733-775`.
- **Who triages the 212 candidates?** They are unconfirmed follow-ups with no owner or schedule. From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:7-8,3219`.
- **Which ownership migration goes first?** The audit's repair direction is to finish unfinished ownership migrations one operation at a time: make graph, pane, session or device identity explicit at dispatch; let durable owners decide admission and revision; keep projections and caches tied to those identities; share pure validation and result classification where semantics match; preserve deliberately different privacy, authorization and transport policies. Removing a copied owner decision beats splitting a large file. It names no first operation. From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:3221-3223`.
- **Why was the 2026-10-16 castellan residue run green?** The harness was red under 1.99.0 and 1.99.1 on a clean checkout and the green run is unexplained. From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:3279,3297`.
- **Is `SceneInputError.path` a documented contract?** R5 left it a free string that nothing matches on; a reading, not ruled. From: `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:109-124`.

## Findings

- 2026-10-17: the registers hold findings Q001 to Q465 (24 repaired), authority entries A001 to A1536, 212 unconfirmed candidates and 23 ruled-out concerns; each finding carries its source, evidence, status, verification limits and repair disposition (audit 3219).
- 2026-10-17: the recurring strains are unfinished ownership migration (graph and session history, current projections, saved and live content, custody lifetimes, transport admission, host presentation each have an owner while callers repeat parts of its decisions) and misleading success (numeric validation that skips or accepts NaN, benchmark proof checking one output, ignored-test filters executing zero tests, fresh files standing in for valid receipts, cleanup attempts reported as completed closure) (audit 3221-3225).
- 2026-10-18: the two main reds (two scenomise backdrop tests, castellan's residue harness) are closed; Q073 stays (audit 3300-3311).

## Progress

- 2026-10-28: cut from the full-source audit plan; nothing new landed.
