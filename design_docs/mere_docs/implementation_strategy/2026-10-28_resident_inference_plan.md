# Resident inference and recall

**Status:** 2026-10-28: cut from `2026-10-15_resident_inference_and_recall_plan.md` and `2026-08-12_search_surface_wiring_plan.md`, archived in `archive_docs/2026-10-28/`. Inference: I1 to I8 and I10 are done to their native gates (I10 qualified 2026-10-20), I9 is in progress (dense variants, Qwen2 routed and shared experts, a 57B streamed row and a Mistral row pass; largest-per-family and broader-family gates are open), I11 is open with no progress, and the physical-fleet and installed-binary receipts remain separate at every phase. Recall and search: capture, omnibar recall, frecency, page text, page identity, the in-tree BM25 engine and the candidate index are landed; W3's reports have no progress entry, W4's canvas wiring is open, and W5 was overtaken by the 2026-09-08 retirement of the hashed vector lane.

**For:** Djinn's resident hosting `esp::infer` so that a question asked of the persona's retained material is reliably answered with citations to the exact source versions used, on the largest model the reachable hardware can hold, and the recall and search surfaces beside it. Opened by Mark's ruling 9 of 2026-10-15: "I just want people to be able to use their hardware, share it, and even task it for things they'd have to pay for otherwise… i would rather be a peer than a client", with "fast is nice but not the point", and by the 2026-09-07 recall rulings on the lighter recall brief.
Not here: the embedding compatibility envelope, run-record schema, assessment and tag workflows (Eidetic development plan P4 and P6, Eidetic repository); Burn changes beyond what esp's decoder needs and tolerant mesh verification, `2026-10-28_distillery_models_plan.md` (M4); sharing with other people's devices (the compute mesh and resource banking briefs; this plan must not preclude it); HTTP-backed providers (host processes, per the device resident consolidation plan's 2026-09-20 ruling); Turnstone's complete research workflow, answer card, headed receiving receipts and R4 final-source UI checks (Turnstone repository); the Luggage and Djinn update resident, `2026-10-28_family_update_resident_plan.md`.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **Host (rulings 1, 3, 8).** The inference actor lives in djinn's resident, with recall over retained material as the forcing consumer; Turnstone and later Graphshell render answers as clients of a resident route and link no esp. The lane borrows the Distillery lane's model-library handle and refuses to open without it; lifting the library to a resident-level owner is reconsidered when a third user appears. The first client is Turnstone. The embedded resident must be able to compose the same lane. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:80-97,104-111,143-148`.
- **One GPU holder, ask preempts trainer (ruling 4).** An ask is owner foreground work and reclaims a running trainer job through its lease's owner reclaim; the trainer ends as a clean cancellation. `HostFacts.gpu` must not claim the GPU while inference holds it. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:112-120`.
- **Purpose and measure (rulings 9, 12 to 14).** Success is whether an ask completes on hardware people own, not tokens per second; every phase records how long an answer took and whether it completed. Nothing assumes a datacenter, a paid endpoint or one strong machine. There is no fixed target model: an ask runs on the fastest rung the hardware supports (whole model in one device, layer ranges across devices, layers streamed from disk) and falls back when a device leaves. Stages are leased mesh work over iroh with their own small protocol, never an unaccounted side channel; Reticulum is not a work transport for this lane. Support goes beyond the Llama family in the order that unlocks the largest openly licensed models; a model whose licence does not permit the use is not added to a fleet default. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:149-165,194-218,533-555`.
- **Durable asks (rulings 10, 11, 15).** The checkpoint is the rendered prompt, the exact session (model, template, weight format), sampling settings and seed, and generated token ids; no KV cache is saved, and a resume re-prefills and records where and on which device it resumed. On lock the route refuses and a running ask pauses at its next token boundary, resuming on unlock. Every keyed personal-graph device can read received asks; continuing needs the exact model session and an explicit Resume, concurrent continuations stay a visible fork, and inference never silently publishes plaintext. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:166-193,219-239`.
- **Templates and recall (rulings 5 to 7).** Templates are the small declared format `esp.prompt-template/v1`; SmolLM2-360M-Instruct proves the pipeline, not the size target; recall is lexical first, and the vector lane returns only behind Eidetic's envelope as sparse vectors or at most 384 dense dimensions. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:121-142`.
- **Stop rules.** Stop and record if the lane cannot stay behind a feature without changing djinn's default build; a phase would need an HTTP fetch or a paid or hosted endpoint; citations cannot name source versions without changing Eidetic P4's schema; the route would need pushed token payloads rather than revision bells; answers would write into the graph other than as derived facets; a resume cannot be validated against its session; or stage work would run outside mesh leases. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:835-848`.
- **I9 budgets.** New retained I9 receipts across repositories have a 100,000,000-byte total cap, and the forcing model rows share a 375,000,000,000-byte aggregate model-data budget; ordinary Cargo outputs stay in the stable Mere target and no extra executable copies are frozen. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:688-705`.
- **Recall rulings (2026-09-07, 2026-09-08).** Page text enters through the host, never the search crate, attached to a fingerprint-keyed page record (blake3 exact plus a near-duplicate hash) rather than the trace event; consent gating stays Capture C4's; frecency is a third ranking beside the lexical one. The hashed vector lane was retired from recall (Mark: retire first, measure, then revisit esp's representation if a vector lane is wanted again); the local tokenizer is swapped for Genet's UAX #29 segmentation once it is founded there. Non-goals: the moot consume half of `SearchIndexSpec`, a crawl-driven corpus (`mere-crawl`, parked) and new embedding backends. From: `archive_docs/2026-10-28/2026-08-12_search_surface_wiring_plan.md:56-97,120-126,653-690`.

## Phases

### I9. Architecture coverage
The Llama block is completed and Qwen2 routed and shared experts run with expert-granular verified reads. Dynamic or YaRN RoPE, partial rotation, other activations, undeclared weights and other families still refuse (Phi fused projections and partial rotary, Gemma norms and soft-capping, DeepSeek MLA, Falcon parallel blocks, gpt-oss attention sinks and MXFP4, Mixtral and OLMo2 per R12). MoE expert weight formats and differentiable expert adapters need their own qualification. The 57B streamed row matches its references on one physical computer, which appears to meet the MoE clause; largest-per-family is not met.
- Done when: each added family has a reference-matched row and a template artifact; the largest model of each family whose licence permits the use, and which the fleet can reach, has completed an ask by some rung; and at least one mixture-of-experts model larger than any single device's memory has completed an ask with streamed experts.
- Needs: a fleet that can reach the largest models; the budgets above.
- Rulings: 12, 13, 14
- From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:530-560,673-752`

### I11. Batch embedding on another device
Re-embedding the corpus through `esp.remote.burn/v1` on the GPU device, using the BERT path already proven on stable. Plain or Fusion server profiles only (remote autotune does not honour a lease-bound close; see the distillery plan).
- Done when: a two-device run embeds the corpus on the GPU device under a lease, the poster's index is bit-identical or within a recorded tolerance of a local run, reclaim mid-batch recovers, and the disclosure ruling is recorded (retained text is local-only today).
- Needs: Distillery `remote` in djinn; transport access in `bind_resident`; `2026-10-28_distillery_models_plan.md` (remote autotune reclaim).
- Rulings: 13
- From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:556-560,824-831`

### IP. Physical fleet, installed binary and headed rows
At every phase the native software gate is separate from: a physical second computer and physical-fleet capacity and latency; the installed Djinn binary running these asks; new headed scale or browser rows; cold-disk throughput and isolated per-process VRAM high-water for I6; larger quantized models and Metal bf16 for I7; and a heap-residue audit, which is not claimed. Djinn does not build on the Kubuntu Surface (Genet's font stack needs `pkg-config` and fontconfig development files, and its fontique backend lacks fontconfig-sys runtime loading); Distillery's lane builds there.
- Done when: not stated; each class is its own receipt.
- Needs: physical devices; an installed Djinn.
- Rulings: 9
- From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:670,922-925,1106-1107,1179,1312,1394-1397,1468-1469,1541-1543,1603-1604,1622-1623,1698,1721-1722`

### IR. Retained-text publication and producers
Inline retained-text publication is capped at 128 KiB and refuses rather than truncates; a chunked larger-corpus protocol is open. Automatic discovery of Turnstone's app-owned browsing store and concrete capture, clip and Fleece producer adapters beyond the shared contract are not built; broader source kinds and platforms for embedding recall are separately scoped; installed-binary and paraphrase-entailment claims lie outside the I4 receipt.
- Done when: not stated; a chunked larger-corpus protocol was left outside the slice that set the cap.
- Needs: Turnstone's producers (Turnstone repository).
- Rulings: 7
- From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:1221-1223,1247-1250,1274-1276,1313-1316,1784-1785`

### IQ. Answer quality boundary
The stock SmolLM2 note answer repeats the requested body and adds references to other sources; identity and range validation and a valid model-authored marker do not certify semantic support. Numeric-marker requests are unreliable with the stock small model and the automatic quote threshold (32 characters) is unchanged. The rapid-click behaviour of the fused UI recipe is not signed off.
- Done when: not stated; owed.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:1827-1830,1871-1878`

### W3. Reports
The trail and steward surface renders `top_domains` and `visits_histogram` from the fast-field columns, with no re-index. Small; may ride another slice. E5's remainder (W3's reports and the corridor strip) is the same item.
- Done when: not stated; the sequence rule is that each slice lands with its own receipt against a real store, not fixtures only.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-12_search_surface_wiring_plan.md:56-58,13`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1370-1377`

### W4. Canvas semantic search
Wire `canvas::canvas_search` and `canvas::field_bridge` into the canvas live surface so a query becomes a similarity field through quint, with `esp::embed::persistence` saving the `VectorIndex` via Eidetic. Start on the lexical embedding provider (deterministic, no Burn), `bert` behind its existing feature. `SemanticSearch` now reads sparse or dense stores behind one facade. Open beside it: a caller for Pictograph's `set_content_affinity`, which has none in Mere or Turnstone; the BERT `semantic-embeddings` provider upgrade; the off-thread embedding actor for raw-body text.
- Done when: not stated beyond the wiring; each slice lands with its own receipt against a real store, not fixtures only.
- Needs: a lexical provider; Eidetic persistence.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-12_search_surface_wiring_plan.md:12-13,98-106,714-736`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1009-1012`

### W5. The fusion fork
`fuse()` merges the lexical ranking with a vector ranking; the hashed vector lane was retired from recall (Turnstone fuses a lexical and a behavioural lane and has no direct esp dependency), so the slice no longer matches the tree. Open beside it: vector acceleration routing (no crate enables `index-burn`, so affinity, recall and canvas search do not route to the accelerators), HNSW Path B with its mere-side twin P4, GPU top-k returning `[Q,k]`, and a per-shape warm-on-first-use pass. If a vector lane is wanted again, esp's sparse representation (about 100 bytes per page) or 512 dimensions, and a semantic embedder.
- Done when: not stated; depends on the question below.
- Needs: the W5 decision; a semantic embedder.
- Rulings: 2026-09-08 retirement
- From: `archive_docs/2026-10-28/2026-08-12_search_surface_wiring_plan.md:107-119,683-685`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:887-890,1399-1404`

### WF. Recall follow-ons named and not built
Page body storage has no size cap and blobs are never collected (`forget` drops the slot only); `consented_to_keep` is a named no-op (Capture C4's). Frecency: adaptive input history (a picked-row signal and store write path), a settings knob for lane weight, a producer for `dwell_ms`, Turnstone tuning of `FrecencyConfig`; head agreement with Firefox is weak and the dwell bonus is inert on the measured profile. Engine: CJK segmentation per ideograph, stemming off, esp sharing the tokenizer, incremental updates (tombstone plus vacuum), Turnstone still calling plain `rebuild`, sharing the tokenization pass between the candidate index and BM25 build, and swapping to Genet's UAX #29 segmentation. `import::history_to_traces` supersedes the `eidetic-recall` example's `transition_of`, which should call through; `visited_at` is second-granular. Turnstone has never had its rustfmt sweep and its clippy is unreachable under the scratch patch until the family re-pin. The old `recall.phrase_order` and `recall.phrase_influence` settings are dead and never read.
- Done when: not stated; each named item owed or declined.
- Needs: the Genet segmentation component; a family re-pin.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-12_search_surface_wiring_plan.md:95-97,464-467,502-505,537-538,573-578,598-617,652-655,664-665`

## Open questions

- **What becomes of W5 now that the vector lane is retired from recall:** retire it, or keep it parked for a future semantic embedder. From: `archive_docs/2026-10-28/2026-08-12_search_surface_wiring_plan.md:117-119`
- **Is Path A of the mere-side index-burn lift closed** (routing is each caller's one-line check), or does its P3 stay open until callers route. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1399-1402`
- **Who owns the geist's fused query**: the deferred phases plan or this plan's recall surface. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1370-1377`
- **When the model library lifts to a resident-level owner** (ruling 3 reconsiders it when a third user appears). From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:109-110`
- **Other people's devices through trust rings and bounties** are "later" and no owner is named; the plan must not preclude them. From: `archive_docs/2026-10-28/2026-10-15_resident_inference_and_recall_plan.md:161-164,1698,1721-1722`

## Findings

- 2026-10-28: prefill and step-by-step decoding are not guaranteed bit-identical and backends can differ in floating point, so a resumed answer records where it resumed and on which device (`esp::infer`).
- 2026-10-28: BM25 recall already scores 15/15 on the fixed corpus, and the hashed vector lane cost 685 MB at corpus scale, which is why ruling 7 caps any returning lane at sparse or at most 384 dense dimensions.
- 2026-10-28: `mere-embed` is a re-export shim plus three mere-coupled modules (persistence, field bridge, canvas search) built and awaiting wiring; its census "retire" branch is closed. The Eidetic reorg moved those modules into the crates that use them.

## Progress

- 2026-10-28: cut from the resident inference and recall plan and the search surface wiring plan; nothing new landed.
