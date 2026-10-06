# Batch 36 — S14 pass, phase A, second seven (records for documents added since 2026-09-05)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-09-27_graphshell_tree_migration_inventory.md | current | yes | 42 | 39 | 2 | 1 |
| mere_docs/research/2026-09-16_lexical_grammar_resources_brief.md | current | yes | 3 | 3 | 0 | 0 |
| mere_docs/testing/2026-09-27_graphshell_controls_physics_receipt.md | current | n/a | 149 | 144 | 1 | 4 |
| mere_docs/testing/2026-09-27_graphshell_producer_receipt.md | current | n/a | 23 | 23 | 0 | 0 |
| mere_docs/testing/2026-09-29_apparatus_diagnostics_receipt.md | current | n/a | 32 | 30 | 0 | 2 |
| mere_docs/testing/2026-09-30_pre4_allocator_diagnosis.md | historical-unmarked | no | 18 | 13 | 4 | 1 |
| moothold_docs/research/2026-09-05_author_offline_publication_proof.md | current | n/a | 19 | 16 | 1 | 2 |
| **Totals** |  |  | **286** | **268** | **8** | **10** |

**Totals: 7 docs, 286 claims checked (268 holds, 8 stale, 10 unverifiable), 4 contradictions.**

Audit base: Mere `26060e88` (2026-10-05 21:58); the tree at the base was
clean. Evidence under `Code/testing/mere/` was read where the documents cite
it; sibling commits were read at their repositories' HEADs. `archive_docs/` is
excluded. No cargo command was run.

This batch belongs to the stack seams plan's S14 pass (rulings S14 and S33),
phase A. A read-only subagent (opus) drafted the seven records. Every stale
claim and contradiction below was re-checked in this session: commit
ancestry and dates (`6279ca31`, `e9d95554`, `9adc4415`, `bb52201c`,
`108647cb`, `88fd392f`, `861042c9`, `cec0b3a4` on first-parent main,
`a90a9d11`, `724b613d`, `b9e9078f`, `f41eee58`), `TreePage`'s fields in
`ports/graphshell/src/web_tree.rs` and the physics form in
`ports/graphshell/src/web_product.rs` at the base, the 2,000-node
`result.json`, the base `Cargo.lock`'s burn version, and the cited lines of
the burn plan, the one-tree plan and the Moot object plan. The "holds" counts
are the draft's. Nothing in the seven documents changes here.

## mere_docs/implementation_strategy/2026-09-27_graphshell_tree_migration_inventory.md

- disposition: current
- status line: "Status (2026-09-30): phase 4 in progress; saved-graph Title/Tags migration has native and headed reopening receipts. Remaining product migration and large-graph responsiveness are open." — accurate: yes
- claims checked: 42 — holds: 39, stale: 2, unverifiable: 1

### Stale claims

- Lines 34-39: the default `TreePage` "retains only its graph" and "the main
  browser page still owns the broader product state". At the base `TreePage`
  also holds `physics: physics::PhysicsPanel`, `session: remote::Session`,
  `sections`, `draft: remote::DraftControls`, `remote_seen`,
  `remote_link_seen` and `tools_open`: the Graph tools physics panel
  (`6279ca31`, 2026-10-01) and the Remote session section (`e9d95554`,
  2026-10-01; `9adc4415`, 2026-10-02) are on the tree.
- Lines 54-58: the old physics form "combines DOM reads, Canvas setters and
  DOM synchronization in `web_product.rs::apply_physics_from_form`". At the
  base that handler builds a typed `PhysicsChoice` and calls
  `canvas_physics::apply_physics` (and `apply_profile_from_form` calls
  `canvas_physics::apply_profile`): the split the paragraph asks for landed
  in `6279ca31`.

### Contradictions

- The controls receipt (its 2026-10-01 and 10-02 sections) and the one-tree
  plan's 2026-10-01 Progress record the physics panel and the remote session
  on the tree; this inventory's lines 34-39 and 54-58, migration steps 2 and
  3, and its 2026-10-01 annotation ("physics panel next") do not.

### Recommended action

- A dated annotation recording the physics panel (`6279ca31`) and the Remote
  session (`e9d95554`, `9adc4415`) on both tree routes; lines 34-39 to name
  what `TreePage` holds; lines 54-58 to past tense; steps 2 and 3 marked moved.
- Date line 105's scenario count ("40 at `bb52201c^`; 87 at `26060e88`").

### Notes

Line 105's "40 `.scn` files … at this inventory" matches the tree before
the inventory's own commit (`bb52201c^` has 40, `bb52201c` 43, the base 87):
ambiguous rather than wrong, so not counted. The draft checked at the base:
`CanvasCommand` and its callers; the `GraphshellApp<MemoryBackend>` fixture;
the `app=local` route and its generated-graph refusal; Cambium's controls,
`on_pointer`, `PointerPhase`, `on_wheel`, and `WheelEvent` without modifiers;
the web host's `mount`, `a11y`, `files` and `WebCapture`; Seiche's
`advance_elapsed`, `ElapsedStepConfig` (50 ms, 3 steps) and `TICK_DURATION`;
Pictograph's `frame_at`; Rootstock's `redraw_at` and `set_hidden`; the page
params, scenarios and saved-edit evidence (four `local_edit` tests, the
reopen UUIDs, `tree_visibility_initial` at 32,004.3 ms, the 403-test sum).
Unverifiable: the 12/8 runtime-test pass, which matches the tree's test
count but cites no log.

## mere_docs/research/2026-09-16_lexical_grammar_resources_brief.md

- disposition: current
- status line: "Status (2026-09-16): research complete. Its open questions were answered the same day and the resulting decisions are recorded in the [lexical capability plan](../implementation_strategy/2026-09-16_lexical_capability_plan.md); read the stacks and questions below as the options that were weighed, not as live questions. The chosen shape is closest to Stack 3 plus Stack 2's data as an optional add-on." — accurate: yes
- claims checked: 3 — holds: 3, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The status line's three claims about the tree hold: the linked plan exists,
it records the same-day decisions, and its chosen shape matches Stack 3 plus
Stack 2's data as an opt-in add-on. The body is about external datasets and
makes no claims about the tree. Whether `reference-data` is the plan's L2 is
the plan's question (batch 35), not this brief's.

## mere_docs/testing/2026-09-27_graphshell_controls_physics_receipt.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 149 — holds: 144, stale: 1, unverifiable: 4

### Stale claims

- Line 357: "The 2,000-node run is still in progress." Its raw receipt,
  `Code/testing/mere/scenarios/graphshell-web/tree_final_live_2000/result.json`,
  records state ok, 14 frames, interval p50 2217.3 ms and p95 2486.2 ms; the
  file is dated 2026-09-30 02:42, before the line's commit `108647cb`
  (2026-09-30 07:26). The receipt never records the result, and the one-tree
  plan's line 780 carries the same "still running".

### Contradictions

- Its 2026-10-01 and 10-02 sections contradict the migration inventory's
  lines 34-39 and 54-58 (that record).

### Recommended action

- Record the 2,000-node result under the fresh live diagnostic, with the same
  dev-build qualifications, or say the run was set aside; the same in the
  one-tree plan's line 780.

### Notes

The 2026-09-29 section says background-tab initialization has no headed
receipt and the 2026-09-30 section supplies one: dated sections in sequence,
not counted. The draft checked: 31 Mere commits on main; 12 Genet commits and
netrender `9607d16f`; the named symbols at the base; the five named tests and
fixtures; the 28 + 7 `bd3e8861` pins with `genet_web_smoke` at `5ae30cad`;
all 46 cited logs and 33 receipt directories, with 48 gate counts matching.
Unverifiable: bundle hashes `bae48b17…` (line 71), `a0fa9dbc…` (line 308) and
`8d7849a4…` (line 433), found in no log or JSON under `Code/testing/mere`, and
the 12/8 runtime-test pass. Line 229's
`scripts/elapsed-host-baseline-control.py` resolves under
`Code/testing/mere/scripts/`, not Mere's `scripts/`.

## mere_docs/testing/2026-09-27_graphshell_producer_receipt.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 23 — holds: 23, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The draft checked: `815279cf`, `95cd5f27` and `c190bcb6` on main; the pins at
`815279cf` (netrender `9607d16f`, 27 Genet `92b249af` rows); "Vello 0.10.1"
resolving as `netrender-vello` 0.10.1 (the `vello` package itself is the
fork at tag vello-0.10.0); `ProducerContext.core`, continuous rendering and
the five p3 scenarios; the bundle digest in `mere-p3-culling-build.json`;
254 + 2 culling tests; the frame table against `result.json`; 50 + 1
mesquite tests. The receipt calls its measurements historical, and its "no
main-branch build" scope still holds.

## mere_docs/testing/2026-09-29_apparatus_diagnostics_receipt.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 32 — holds: 30, stale: 0, unverifiable: 2

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The draft checked: `bd5912fb` on main and the named crates; the log counts
(`core-serde` 14, `host-integration` 75, `apparatus-core` 18, registry 135
and 36, `focused` 64, `containing-7bf` 108, `containing-offline` 210,
`command-menu` 7, `winit-keys` 5); the negative controls failing as
described; `CaptureProjectionLimits` defaults (64 KiB, 1 MiB, 128); the
registry queue's `Full` refusal; the Genet pins; the Turnstone, Woodshed and
Knot commits on their mains. Unverifiable: the 74-test all-feature gate and
the 2026-09-30 22-test gate, neither with a retained log.

## mere_docs/testing/2026-09-30_pre4_allocator_diagnosis.md

- disposition: historical-unmarked
- status line: "Date: 2026-09-30. Status: measured diagnostic; repair not selected or accepted." — accurate: no
- claims checked: 18 — holds: 13, stale: 4, unverifiable: 1

### Stale claims

- Line 3, "repair not selected or accepted": ruling 508 put the repair in
  burn-remote's close path (`88fd392f`, 2026-10-03), and its acceptance is
  recorded in `861042c9` (burn plan §13.32, receipt
  `ports/distillery/probe/receipts/2026-10-03_pre4_remote_minilm_repaired.json`).
- Line 52, "The proposed burn-remote repair is a pending patch-design fork":
  answered by ruling 508 (the burn plan's 2026-10-03 entry, line 67).
- Lines 78-80, the executable "must be rebuilt before an acceptance run" and
  the worktree "retained for the unresolved gate": the acceptance run passed
  on 2026-10-03 (`861042c9`).
- Line 81, "Main integration and downstream repins remain held": `cec0b3a4`
  (2026-10-05, "Merge burn-pre4-repin … S16, ruling 557") is on main's
  first-parent history, the base `Cargo.lock` resolves `burn` and
  `burn-remote` 0.22.0-pre.4, and `a90a9d11` records Knot's repin started.

### Contradictions

- The burn plan's 2026-10-03 entry (lines 61-67) and §13.32 contradict lines
  3, 52 and 78-81.

### Recommended action

- A dated status annotation: the repair selected by ruling 508 (`88fd392f`)
  and accepted 2026-10-03 (burn plan §13.32); pre.4 merged to main at
  `cec0b3a4` on 2026-10-05; the measurements remain pre-repair evidence. That
  makes the record historical-marked.

### Notes

The status is inline after "Date:" rather than a `Status:` line of its own.
The draft checked: `124fc42b` on main; isometry `052ee05` (ruling 411) on its
main; the three linked logs with exact SHA-256 matches; the completion-poll
log's 5,323,776 / 41,943,040 / 1.9222; the executable hash in
`binary-admission.json`. Unverifiable: the nine rejected corrupt inputs.

## moothold_docs/research/2026-09-05_author_offline_publication_proof.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 19 — holds: 16, stale: 1, unverifiable: 2

### Stale claims

- Line 58: the production owner of `PublicationRevisionV1` and
  `HostingCommitmentV1` "still need[s] a ruling". Ownership was ruled in the
  Moot object plan (lines 220-228, `724b613d`, 2026-09-06): Eidetic owns the
  signed publication revision, Gemot owns the hosting promise as a Standing
  fact. Both types are still unimplemented outside the example; the
  compatibility grammar and governed linkage may still be open.

### Contradictions

- The Moot object plan (lines 347-349) says "The concurrent proof artifact is
  not yet committed" and marks its link to this document `*(planned target)*`
  with a planned-link comment. This document was committed in `b9e9078f`
  (2026-09-06 10:39), after that annotation (`f41eee58`, 03:19). The doc audit
  reports it among its 14 `stale_historical_annotations` ("annotated relative
  link now resolves"); it exits 0 because `--fail-on-findings` is not the
  default.

### Recommended action

- A dated annotation on line 58: owners ruled 2026-09-06 (Moot object plan),
  not yet implemented.
- In the Moot object plan: drop the planned-link marker and the "not yet
  committed" sentence (phase C, with the audit's other 13 resolving
  annotations).

### Notes

The draft checked: the example `crates/moot/gemot/examples/author-offline-publication.rs`
and `MootEvent::Shared`; the personae, stickleback, muniment, errand,
gemini-protocol and transport crates and Gemot's Standing; the receipt JSON,
its seven identity hashes, PIDs, timestamps, port and statuses; the gap list
(lines 60-62) still matching the plan. Unverifiable: the 132/133 test gates
and the build timings, without a cited log.
