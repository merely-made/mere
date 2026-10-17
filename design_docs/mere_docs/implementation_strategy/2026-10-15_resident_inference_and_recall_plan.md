# Resident Inference and Recall Plan

**Date:** 2026-10-15
**Status (2026-10-16):** in progress. I1 to I4 software gates done (§7):
strict decoder, declared templates, CPU and GPU model rows, native trainer
owner reclaim, durable checkpoints, restart, lock recovery and keyed
personal-graph continuation pass on O-PC. I4's native gate adds BM25 recall,
exact retained passage evidence and version-checked citations with the pinned
SmolLM2 model. I5's Turnstone producer handoff and headed answer card are next.
The new device
boundary uses two separately rooted resident compositions on one computer;
a physical second computer and installed-binary receipt remain separate.
Rulings 1 to 15 recorded (§1). Rulings 9 to 14 widened the plan from a first answering host to
reliable answers from models as large as the available hardware can hold,
across devices; ruling 11 supersedes ruling 2.

**Purpose (ruling 9):** people use the hardware they already have, share it,
and task it for things they would otherwise pay for. A question asked is
reliably answered, in minutes when the hardware allows and hours when it does
not. Speed is welcome but is not the measure; the frontier is not the point.
The person asking is a peer of the machines answering, not a client of a
service.

**Scope:** Djinn's resident hosts `esp::infer`. Its first consumer asks
questions over the persona's retained material and answers with citations to
the exact source versions it used. An ask is a durable job: it survives
interruption, pauses and resumes, and can continue on another device. It runs
the largest model the reachable hardware can hold: in one device's memory,
split by layer ranges across devices, or streamed layer by layer from disk.
This is the [Distillery models plan](2026-10-06_distillery_models_plan.md)'s
M2 (closed by I5) and the inference-surface half of the
[Eidetic development plan](../../eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md)'s
P4, built once in one host.

**Owns:** the djinn inference lane and its route; the bridge from the actor
to the resident; prompt templates as data; recall-to-prompt assembly with
citation handles; durable ask checkpoints; the layer-streaming provider; the
decoder's weight formats and architecture coverage; the layer-stage protocol
between devices; and the answer card's first headed receipt.

**Does not own:** the embedding compatibility envelope and the run-record
schema (Eidetic P4 owns both; this plan is their first consumer); Burn or
burn-remote changes beyond what esp's decoder needs upstreamed; tolerant
mesh verification (Distillery M4); sharing with other people's devices
(the compute mesh brief's rings and the resource banking brief's outer
shell; this plan must not preclude it); and any HTTP-backed provider (the
device resident consolidation plan's 2026-09-20 ruling keeps HTTP in host
processes).

**Related:**
- [Distillery models plan](2026-10-06_distillery_models_plan.md), M2 to M4
- [Eidetic development plan](../../eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md), P4 and decision 3
- [Device resident consolidation plan](2026-08-20_device_resident_consolidation_plan.md), §1, §3, §4
- [Vault lock plan](../../dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md), rulings 24, 43, 48, 66, 83
- [Graphshell reference host plan](2026-07-27_graphshell_reference_host_plan.md), §2 and §5 (inference output is a derived facet)
- [Search surface wiring plan](2026-08-12_search_surface_wiring_plan.md), fixed-corpus recall receipts and the 2026-09-08 vector-lane retirement
- [Inference provider plan](../../archive_docs/2026-10-06_completed_plans/2026-07-05_inference_provider_plan.md) (archived), meerkat `>ask` receipt and its follow-ups
- [Personal compute mesh brief](../../archive_docs/2026-06-04_resource_coordination_merge/2026-06-03_compute_mesh_brief.md), trust rings (§1), owner reclaim (§4) and the inference ladder (§5)

---

## 1. Rulings

**Ruling 1 (2026-10-15).** *Where does the inference actor live, and what
forces it?* Options: a Turnstone surface; a Graphshell surface; a djinn
resident service. Mark: **"Host the inference actor in djinn's resident;
plan M2 with recall over retained material as the forcing consumer."**

Follows:
- The resident already owns the persona's stores and network runtimes
  (consolidation plan §1), so retained material, the model library and any
  later mesh routing are composed in one process rather than across a
  process boundary.
- Turnstone, and later Graphshell, render answers as clients of a resident
  route. Neither links esp.
- This answers the "Distillery inference host" part of the Eidetic plan's
  decision 3. Its engine-pair and browser-executor parts stay open there.
- The embedded (mobile, sandboxed) resident must be able to compose the same
  lane (consolidation plan §1; reservoir plan V5), so nothing here may assume
  the desktop process shape.

**Ruling 2 (2026-10-15; superseded by ruling 11).** *Lock posture?*
Options: close on lock like Knot; keep running like Distillery (vault lock
ruling 24); close and unload the model. Mark: **"Close on lock like Knot
(Recommended)"**. Superseded the same day: with a 15-minute idle lock,
cancelling on lock would end every long ask.

**Ruling 3 (2026-10-15).** *Who owns the model library?* Options: borrow the
Distillery lane's handle; lift the library into a resident-level owner both
lanes borrow. Mark: **"Borrow the Distillery lane's handle (Recommended)"**.
Follows: one writer stays (consolidation invariant 1); the inference lane
requires the Distillery lane and refuses to open without it. Lifting the
library is reconsidered when a third user appears, the consolidation plan's
own trigger for growing shared lifecycle.

**Ruling 4 (2026-10-15).** *One GPU, two lanes?* Options: one holder at a
time with inference preempting the trainer; share and accept contention;
inference falls back to CPU while the trainer holds the GPU. Mark: **"One
holder at a time; inference preempts the trainer (Recommended)"**. Follows:
an ask is owner foreground work. The resident keeps one GPU holder; an ask
reclaims a running trainer job through its lease's owner reclaim (compute
mesh brief §4), and the trainer's job ends as a clean cancellation, not a
lapse. `HostFacts.gpu` must stop claiming the GPU while inference holds it.

**Ruling 5 (2026-10-15; role narrowed by ruling 12).** *First answering
model?* Options: SmolLM2-360M-Instruct; SmolLM2-1.7B-Instruct; Llama 3.2 1B
Instruct; TinyLlama-Chat. Mark: **"SmolLM2-360M-Instruct (Recommended)"**.
Follows: it is the same family as the proven 135M browser and native rows,
Apache-2.0, and small enough for CPU rows. Under ruling 12 it is the model
that proves the pipeline (I1 to I5), not the size target.

**Ruling 6 (2026-10-15).** *Template format?* Options: a small declared
format; a Jinja subset running Hugging Face `chat_template` strings; code per
model. Mark: **"A small declared format (Recommended)"**. Follows:
`esp.prompt-template/v1` carries per-role prefix and suffix strings, BOS and
EOS handling, a context slot and stop strings. It is total, hashable and
reviewable. Code per model was already excluded by M2's done-condition.

**Ruling 7 (2026-10-15).** *Recall lanes and budget?* Options: lexical
first, vector after the envelope; both lanes from the first recall phase.
Mark: **"Lexical first, vector after the envelope (Recommended)"**. Follows:
I4 uses BM25 only, which already scores 15/15 on the fixed corpus. The
vector lane returns in I10 only behind Eidetic's envelope, as sparse vectors
or at most 384 dense dimensions, so it cannot repeat the 685 MB that retired
it from Turnstone.

**Ruling 8 (2026-10-15).** *First client?* Options: Turnstone; Graphshell's
GUI. Mark: **"Turnstone (Recommended)"**. Follows: Turnstone already talks to
djinn routes, owns recall and is where retained browsing lives. The route is
granted to `turnstone`; Graphshell gets its own grant when it renders
answers.

**Ruling 9 (2026-10-15).** *What is this for?* Mark: **"I just want people to
be able to use their hardware, share it, and even task it for things they'd
have to pay for otherwise… the frontier isn't the point, accessibility and
personal/communal infrastructure as an alternative to paying for things is
the point… i would rather be a peer than a client."** And: **"this is a pop a
question and reliably get an answer even if it takes a while kinda method.
fast is nice but not the point."** Follows:
- Success is measured by whether an ask completes on hardware people own,
  not by tokens per second. Every phase records how long an answer took and
  whether it completed.
- Nothing here assumes a datacenter, a paid endpoint or a single strong
  machine.
- The same machinery must later serve other people's devices through the
  compute mesh brief's trust rings: permissions inside a trust domain, the
  banking brief's bounties at its edge. Stage work therefore runs under mesh
  leases from the start, never under an unaccounted side channel.

**Ruling 10 (2026-10-15).** *Must an ask survive interruption?* Mark:
**"surviving interruptions would make this less babysitting and more a usable
tool."** Follows:
- An ask is a durable job. Its checkpoint is the rendered prompt, the exact
  session (model, template, weight format), the sampling settings and seed,
  and the token ids generated so far. No KV cache is saved: any holder of
  the same session rebuilds it by prefilling the prompt plus those tokens.
- The checkpoint is small and append-only, so it replicates as an ordinary
  log over p2panda and iroh. An ask can pause when a device sleeps, wait out
  a relay, or continue on another device.
- Reticulum is not a work transport for this lane.
- Resume re-prefills the prompt plus the generated tokens, and seeded
  sampling skips the recorded number of random draws. Prefill and
  step-by-step decoding are not guaranteed bit-identical (§2), and devices or
  backends can differ in floating point, so a resumed answer records where
  it resumed and on which device; any later divergence from an
  uninterrupted run is then attributable.

**Ruling 11 (2026-10-15; supersedes ruling 2).** *Lock posture for long
asks?* Options: pause on lock and resume on unlock; cancel on lock. Mark:
**"nice, yeah, i'm alright with that"** (pause on lock). Follows: on lock the
route refuses, and a running ask stops at its next token boundary, writes its
checkpoint and pauses; on unlock it resumes. The loaded model may stay
resident, because weights are not secrets. Prompts, retained text and
answers are persona data, which is what vault lock rulings 43 and 48 and
Dramatis invariant 13 (ruling 83) protect. Djinn's lock arm gains an
inference branch beside Knot's.

**Ruling 12 (2026-10-15).** *How big a model?* Mark: **"big is as big as can
be."** Follows: there is no fixed target model. Each ask runs on the fastest
rung the reachable hardware supports and falls back when a device leaves:
1. the whole model in one device's memory;
2. layer ranges held in several devices' memory (ruling 13);
3. layers streamed from disk on one device.

Each scale row records the model, weight format, devices, rung, seconds per
token, peak memory and whether the ask completed.

**Ruling 13 (2026-10-15).** *Split a model across devices?* Mark: **"hell
yeah, that was why i brought it up."** Follows: the compute mesh brief put
sharding last because of its per-token network cost; with ruling 9's
priorities that cost is acceptable and pooled memory is what matters. A
stage holds a contiguous layer range and its KV cache; only the hidden state
for the current token moves between stages, about 16 KB per token for a
70B-class model in bf16. Stages are leased mesh work over iroh with their own
small protocol, not burn-remote's per-operation tensor traffic.

**Ruling 14 (2026-10-15).** *Only the Llama family?* Mark: **"i would hope we
can also support more than the llama family."** Follows: architecture
coverage is a phase (I9). §2 records what the decoder runs today and what
blocks each other family; families are added in the order that unlocks the
largest openly licensed models.

**Ruling 15 (2026-10-15).** *Which devices may read and continue private
asks?* Mark: **"Proceed with I3 using every keyed personal graph device as
the boundary."** Follows:
- The personal graph's existing group distributes the data epochs. Inference
  does not create another group or distribute a persona's private identity.
- An incoming writer attests its distinct root under an inference-specific,
  graph-specific derivation and must belong to the personal graph key group for that data epoch.
  The encrypted envelope binds the inference domain and signed log address.
  Transport reachability and pairing without a data key confer no read access.
- Every keyed personal device can read received asks. Continuing one requires
  the exact model session and an explicit Resume on that device. Concurrent
  continuations remain a visible fork.
- Existing group revocation semantics apply: a removed reader retains old
  readable bytes, while an epoch turn excludes it from subsequent private
  asks. Local historical reads do not reapply current writer admission.
- With personal sync off, asks remain locally durable. With personal sync on,
  shared asks refuse until this device is keyed. Live key agreement enables
  them without restarting; inference never silently publishes plaintext.

---

## 2. Findings carried in

Each names its evidence; a later phase may correct one.

- **The actor is self-contained.** `spawn_inference_actor(wake, build)`
  (`crates/intel/esp/src/infer/actor.rs:68`) runs one named OS thread,
  builds the provider on that thread, takes one request at a time in FIFO
  order, and reports through a blocking `std::sync::mpsc::Receiver` plus a
  `Wake` callback after every update. Cancel is honoured at the next emitted
  fragment; prefill is not interruptible (`actor.rs:7-21`).
- **Templating is above the seam and absent.** `GenerationRequest` says chat
  templating happens above it (`infer/provider.rs:52`). No manifest carries a
  template; only hashes exist (`ModelSession.prompt_template_hash`,
  `BoundModelSession::prepare`, `infer/session.rs:41`, `:250-266`). Meerkat's
  receipt found TinyLlama-Chat decodes a raw question badly without its chat
  template.
- **Exact sessions do not reach the actor.** `BoundModelSession` does not
  implement `InferenceProvider`, and `InferCommand::Generate` carries a plain
  `GenerationRequest` (`infer/session.rs:198-290`, `actor.rs:37-63`).
- **Model bytes come from Eidetic.** `ModelLibrary::resolve_components`
  (`crates/eidetic/eidetic-core/src/models/library.rs:117`) then
  `DecoderProvider::from_bytes` (`infer/decoder/provider.rs:168`).
  `load_wgpu_provider` binds `DiscreteGpu(0)` without probing
  (`infer/decoder/mod.rs:91-105`); `probe_gpu_adapter` exists
  (`gpu_probe.rs:207`).
- **The decoder runs one architecture.** A pre-norm Llama block: RMSNorm,
  bias-free grouped-query attention, split-half RoPE over the full head
  with only `rope_theta`, and a SwiGLU MLP. Real-checkpoint receipts exist
  for TinyLlama (`crates/intel/esp/tests/tinyllama_real.rs`) and
  SmolLM2-135M only. Paths below are under `crates/intel/esp/src/infer/decoder/`.
- **Unsupported configs can load and answer wrongly.** `config.rs` never
  reads `model_type` or `architectures` and ignores unknown fields
  (`config.rs:9-45`); `head_dim` is always `hidden / heads` (`:85-87`).
  Refusals come only from missing fields or tensors, shape or dtype
  mismatches. So Qwen2 and 2.5 load without their q/k/v biases, Qwen3 8B and
  14B load without QK-norm, Gemma 2B runs GELU as SiLU, Llama 3.1 and later
  ignore `rope_scaling`, and Mistral v0.1 ignores its sliding window past
  4,096 tokens: each produces wrong output with no error. *Fixed
  2026-10-15:* I2's strictness step now refuses these by name (§7).
- **Coverage by family** (a model family "runs" only with a
  reference-matched row; "Today" was the state before the 2026-10-15
  strictness step, after which every row but the first is refused by name):

  | Family | Today | Gap |
  |---|---|---|
  | TinyLlama, SmolLM2 | runs | none |
  | Llama 1, 2, 3 | math runs | sharded weights; Llama 3's extra stop token |
  | Llama 3.1, 3.2, 3.3 | wrong output | `rope_scaling` (llama3); sharding above 1B |
  | Mistral 7B | wrong past 4,096 tokens | sliding window; sharding |
  | Mistral NeMo, Small | refused | explicit `head_dim` |
  | Qwen2, 2.5 | wrong output | q/k/v bias |
  | Qwen3 dense | refused or wrong | explicit `head_dim`; per-head QK-norm |
  | OLMo2 | refused | post-sublayer norms; QK-norm |
  | Phi-3, 3.5, 4 | refused | fused qkv and gate-up; sliding window; `longrope`; partial rotary |
  | Gemma 1 | wrong (2B) or refused | GELU; (1+w) norm; embedding scaling |
  | Gemma 2, 3 | refused | Gemma 1's gaps plus extra norms, soft-capping, alternating sliding window, QK-norm |
  | Mixtral, Qwen3-MoE | refused | mixture-of-experts routing |
  | gpt-oss | refused | odd `head_dim`, attention sinks, biases, MoE, MXFP4 weights |
  | DeepSeek V2, V3 | refused | MLA attention, MoE with shared experts, FP8 weights |
  | Falcon (original) | refused | fused qkv, parallel blocks, LayerNorm, GELU |

- **Weights are one file, all resident, all f32.** Only a single
  `model.safetensors` is read, with no sharded-index support
  (`loader.rs:38-44`); F32, BF16 and F16 are decoded into `Vec<f32>`
  (`tensors.rs:21-39`), so a bf16 checkpoint costs 4 bytes per parameter
  resident and about 6 at load. There are no quantized formats
  (`tensors.rs:14`, `:36`; `provider.rs:154`).
- **The layer code is already per-layer.** `LoadedDecoderLayer`,
  `DecoderLayer::from_loaded`, `DecoderLayer::forward_cached` and
  `LayerKvCache` are public (`layer.rs:55-112`, `attention.rs:103-109`).
  The tensor loader and the embedding, norm and linear builders are
  crate-private, and there is no layer-range forward: `forward_cached` runs
  every layer on one device (`model.rs:82-126`). The hidden state between
  layers is a plain `[batch, seq, hidden]` tensor.
- **Long prompts cost quadratic memory.** Attention builds the full
  `[batch, heads, seq, total]` score tensor (`attention.rs:242-269`), and
  prefill runs the output head on every position, allocating
  `seq × vocab` f32 logits (`model.rs:125`). A multi-token block over a warm
  cache is unsupported (`attention.rs:218-223`), so resume re-prefills the
  whole token list.
- **Resume determinism is untested at scale.** Greedy decoding is
  deterministic on one backend, but prefill and step-by-step decoding are not
  guaranteed bit-identical; only equal tokens on a tiny model are tested
  (`generate.rs:291-302`). Seeded sampling draws exactly one random number
  per token, but the generator's position is private (`sample.rs:40`, `:105`).
- **Tokenizer and stops.** `tokenizer.json` only; stop tokens come only from
  `config.json`'s `eos_token_id` (`provider.rs:177-178`, `:283`).
- **Burn 0.22 can hold smaller weights, unevenly by backend.** Its
  quantization schemes cover 8-, 4- and 2-bit integers and FP8/FP4 values with
  per-tensor or per-block scales (cubecl-common `quant/scheme.rs`).
  - The cubecl backends (wgpu, CUDA, ROCm) multiply quantized weights inside
    the matmul kernel; wgpu has F16 everywhere and BF16 on Metal only.
  - burn-ndarray has no F16 or BF16 and only symmetric Q8, which it widens
    to f32 for every matmul.
  - burn-flex is a CPU backend with F16 and BF16; esp does not enable it.
  - burn-store can memory-map safetensors and load lazily; esp does not use
    it.
  - No Burn crate imports GGUF, GPTQ, AWQ or bitsandbytes files.
- **Mixture-of-experts suits slow, big asks.** Only the routed experts run
  for each token, so a streamed MoE model reads a fraction of its weights per
  token. Public figures: Mixtral 8x22B is about 141B parameters with about
  39B active; Qwen3-235B-A22B about 22B active; gpt-oss-120b about 5B active;
  DeepSeek V3 671B with about 37B active.
- **Djinn has a lane pattern.** A settings field on `OwnerSettings`, a field
  on `DjinnResident` with reverse-order unwind, a route registered in the
  catalog and granted per app, a `close_all` entry, and a branch in the lock
  arm (`ports/djinn/src/settings.rs:110-125`, `src/resident.rs:43-156`,
  `src/bin/djinn.rs:1231-1238`, `:1379-1422`). Heavy lanes are behind djinn
  features and refuse settings the build cannot honour
  (`src/resident_distillery.rs:141-172`).
- **The app endpoint does not stream payloads.** Endpoint methods are
  synchronous; a source bumps a revision, `poll_notice` rings, and the client
  resumes or snapshots. Frames are capped at 1 MiB; Chronicle polls at
  250 ms (`ports/graphshell/src/native/endpoint_catalog.rs:97-127`,
  `browser_carrier.rs:49`, `resident_distillery.rs:76-80`).
- **The vault lock says nothing about inference.** Knot closes on lock and
  reopens on unlock (rulings 43, 48); Distillery keeps running (ruling 24);
  nothing is served before the first unlock (ruling 66). The idle lock
  defaults to 15 minutes (ruling 20).
- **The model library has one writer.** It is a redb owned by the Distillery
  lane and shared by handle (`resident_distillery.rs:407`, `:608-637`;
  consolidation invariant 1).
- **Nothing arbitrates the GPU.** A `trainer-gpu` lane and an inference
  provider could both bind the discrete adapter. Meerkat measured contention
  with a renderer roughly doubling per-token cost.
- **Mesh jobs are leased and reclaimable.** `JobSpec` carries a
  `CheckpointClass` and `LeaseTerms`; owner reclaim is proven on stable for
  remote sessions (Distillery plan, 2026-10-15). Portable checkpoints are
  deferred to Distillery M4, but an ask's checkpoint (ruling 10) is only
  token ids and settings, not model state.
- **Lexical recall is ready; semantic recall is not.** `eidetic-search`
  BM25 with n-gram RRF scored 15/15 Ranking@1 on the 28-record fixed corpus;
  MiniLM alone scored 12/15 at about 195 MB and 22 ms per query on CPU
  (search surface wiring plan, :231-247, :278-327). The vector lane was
  retired from Turnstone recall on 2026-09-08 at 685 MB for a dense
  4,096-dimension index (:658-685).
- **Recall keys are URLs.** Fusion and the `eidetic-recall` example key hits
  by URL string (`crates/intel/eidetic-search/src/fusion.rs:32-74`). Retained
  material has version-bearing identities that citations need:
  `ResourceCaptureRef` (content hash), `PageTextRef`, `RetainedClipV1`, and
  Fleece's `ContentRef{source_id, node_id, version}` with selectors
  (`eidetic-core/src/browsing/captures.rs:35`, `browsing/text.rs:42`,
  `clip_profile.rs:93`, `crates/fleece/src/traversal.rs:18`).
- **No embedding envelope exists.** A persisted `VectorIndex` records only
  dimensions, metric and entries, and `with_index` checks only dimensions
  (`esp/src/embed/persistence.rs:40`, `search.rs:272`). The nearest envelope is
  `esp.meaning-model/v1` for e5-base-v2 (`embed/meaning_model.json`), used by
  no index.
- **Remote offload is tensor-level.** `esp.remote.burn/v1` runs the model on
  the poster against a remote Burn device; MiniLM is proven on stable, a
  decoder over remote is not, and remote autotune fails owner reclaim
  (Distillery plan, 2026-10-15). Djinn builds Distillery without `remote`, and
  `bind_resident`'s `host_config` closure receives no transport, so a
  courier or `RemoteSessionService` cannot be composed through it
  (`ports/distillery/src/installed.rs:315-345`).
- **The hardware in reach.** O-PC: 31 GiB RAM, an 8 GiB RTX 4060 Laptop and
  1 TB free disk. The Kubuntu Surface: 5.8 GiB RAM, integrated Radeon. The
  ThinkPad, the M4 iMac and the Intel iMac (Radeon Pro Vega 56) are not yet
  measured here. A 70B-class model is about 140 GB in bf16 and about 35 GB at
  4 bits, so on this fleet it runs by rung 3, or by rung 2 once enough
  memory is pooled.

---

## 3. Shape

```text
Turnstone answer card                      (client, I5)
   │  route `inference.ask`: Ask / Pause / Resume / Cancel intents;
   │  snapshot = state, progress, partial text, citations; notices on change
   ▼
djinn inference lane                       (resident, I1)
   ├─ recall: retained sources → passages keyed by source version + selector  (I4, I10)
   ├─ assembly: question + passages → template render (template is data)      (I2, I4)
   ├─ durable ask: checkpoint log (prompt, session, seed, token ids)           (I3)
   ├─ actor bridge: Wake → tokio Notify; ids; session-aware provider          (I1, I2)
   └─ run record in Eidetic: inputs, settings, outputs, status, citations     (I4)
        │
        ▼
execution ladder (ruling 12), chosen per ask, falling back when a device leaves
   1. whole model in one device's memory      (I2, I7)
   2. layer stages across devices over iroh   (I8)
   3. layers streamed from disk on one device (I6)
```

The lane is one more resident lane. It owns no store of its own: it borrows
the Distillery lane's model library handle (ruling 3), reads retained
material through the persona's existing stores, writes run records through
Eidetic's schema, and replicates ask checkpoints through the persona's
existing p2panda log machinery.

---

## 4. Components

**R1. Lane and features.** `InferenceLaneSettings { model, template, device:
cpu|gpu, limits }` on `OwnerSettings`; absent means off. Djinn features
`inference` (`esp/actor`, `esp/decoder`) and `inference-gpu`
(`esp/decoder-wgpu`), so the default build stays light. Settings naming a
device the build lacks are refused by name, and so is the lane without
the Distillery lane, whose model library it borrows (ruling 3). Djinn's lock
arm refuses the route and pauses running asks at a token boundary, keeping
the loaded model (ruling 11). The resident holds one GPU claim: an ask on the
GPU reclaims a running trainer job through its lease, and `HostFacts.gpu`
stops advertising the GPU while inference holds it (ruling 4). The lane
closes through `lifecycle::close_all` beside Distillery and before blob
custody, checkpointing running asks and joining the actor thread.

**R2. Actor bridge.** The `Wake` callback signals a `tokio::sync::Notify`;
a lane task drains the receiver with `try_recv`, folds updates by
correlation id, and bumps the route's revision. A thin
`SessionProvider` implements `InferenceProvider` over a `BoundModelSession`,
so the exact `ModelSession` id and template hash reach every run record
without changing the actor. Device choice goes through
`probe_gpu_adapter` instead of `load_wgpu_provider`'s fixed adapter.

**R3. Templates as data.** A prompt template is an Eidetic artifact, its
bytes hashed into `ModelSession.prompt_template_hash` (format: ruling 6). A
model row names its template; changing either changes the session id. The
renderer is small and total: unknown fields are errors, not silent drops.

**R4. Recall and citations.** The corpus unit is a passage of retained
material keyed by its source version and a selector: page text by
`PageTextRef` blob, captures by `ResourceCaptureRef` content hash, clips by
clip id, Fleece content by `ContentRef`. Passages are produced by a
deterministic chunker whose version is part of the index spec. I4 uses
lexical BM25 only (`DocumentIndex<K>` keyed by passage handles, not URLs).
I10 adds a vector lane behind Eidetic's envelope (ruling 7). A source whose
current version differs from the indexed one is reported stale, never cited.

**R5. Route `inference.ask`.** Granted to `turnstone` first (ruling 8).
Intents: `Ask { question, scope }` returns `Accepted` with a run id; `Pause`,
`Resume` and `Cancel { run }`. The snapshot carries state, progress (tokens
so far, seconds per token, the rung and devices in use), accumulated text,
retrieved passages and citations. Fragments are coalesced: the revision bumps
at most every 100 ms and on state changes, keeping notice traffic bounded and
the 1 MiB frame cap far away. Sessions get per-session adapters over one
shared source (reservoir plan R2 shape), so one client's reads do not
consume another's notices.

**R6. Run records.** Per Eidetic P4: input refs (question blob, the passage
handles actually placed in the prompt), settings (session id, template hash,
weight format, request parameters), output ref, the devices and rungs used,
and a terminal status that keeps `finished`, `cancelled` (with partial text)
and `failed` distinct from `paused`. Answers are derived facets with
producer, model version, input identity, time and lineage, and never
overwrite a user assertion (Graphshell plan §2, §5).

**R7. Answer card.** Turnstone renders the ask as a job: progress, partial
text, citation chips that open the cited source version, pause, resume and
cancel, and reopening a past ask from its record. Turnstone needs a mere
repin to reach the route constant.

**R8. Durable asks.** The checkpoint of ruling 10 is written at every
committed token, or every few tokens when a token is fast, as an
append-only per-ask log. Resume validates the session id, rebuilds the KV
cache by prefill, and continues sampling from the recorded seed and
position. A paused ask is resumable by any device that holds the same
session; the record names each device change.

**R9. Layer streaming.** A provider that keeps only the embedding, the final
norm and head, the KV cache and one layer's weights in memory. For each
step it reads a layer from disk, runs the current tokens through it,
releases it, and moves on. Prefill runs the whole prompt through each layer
in one pass. Memory is about one layer plus the KV cache; time per token is
bounded by disk reads of the whole model, or of the routed experts only for
a mixture-of-experts model. It needs, from §2: sharded safetensors with
their index; memory-mapped, per-layer loading (burn-store's lazy loader, or
the crate-private tensor loader made public); a layer-range forward;
attention that bounds the score tensor during prefill (chunked over query
positions); and the output head on the last position only.

**R10. Weight formats.** Hold half-precision weights at half precision
rather than widening them to f32: on CPU through burn-flex (F16 and BF16),
on GPU through wgpu (F16 everywhere, BF16 on Metal); burn-ndarray has
neither. Then 8-bit and 4-bit block formats with per-block scales, which the
cubecl backends multiply in-kernel and burn-ndarray only stores (Q8). No
Burn crate imports GGUF or GPTQ, so quantization happens once at import from
safetensors, and the quantized layers are stored as model-library
artifacts for streaming. Each format's deviation from bf16 is measured on a
fixed prompt set before it is selectable. A format is part of the session
id, so a resume never mixes formats silently.

**R11. Layer stages.** A stage protocol over iroh: a coordinator assigns
contiguous layer ranges to devices by available memory; each stage keeps its
range's weights and KV cache resident; per token the hidden state moves
stage to stage and the last stage returns logits or the sampled token.
Stages run as leased mesh jobs, so owner reclaim, admission and later trust
rings apply. When a stage leaves, the ask checkpoints, the coordinator
reassigns or falls back to streaming, and the ask resumes.

**R12. Architecture coverage.** First the decoder becomes strict: it reads
`model_type` and `architectures`, refuses unknown ones by name, and refuses
every config feature it does not implement (`rope_scaling`, explicit
`head_dim` that differs from `hidden / heads`, `attention_bias` or bias
tensors, `sliding_window`, QK-norm tensors, `hidden_act` other than SiLU,
soft-capping, experts) instead of loading and answering wrongly. Families
are then added behind the same provider seam, each with a reference-matched
row (token ids equal to an independent implementation at temperature 0)
before it is selectable, in this order:
1. *Completing the Llama block:* `rope_scaling` (llama3, linear, yarn),
   explicit `head_dim`, q/k/v bias, sliding window, per-head QK-norm and
   extra stop tokens. This reaches Llama 3.1 to 3.3, Qwen2 and 2.5, Qwen3
   dense, Mistral 7B, NeMo and Small, and, with post-sublayer norms, OLMo2.
2. *Mixture of experts:* routing with top-k experts and shared experts,
   streaming only the routed experts (R9). This reaches Mixtral and
   Qwen3-MoE, then gpt-oss with attention sinks and MXFP4 weights.
3. *The rest, as hardware and interest allow:* Phi's fused projections and
   partial rotary, Gemma's norms, scaling and soft-capping, DeepSeek's MLA
   attention, and Falcon's parallel blocks.

Each row records its licence; a model whose licence does not permit the
use is not added to a fleet default.

**R13. Batch embedding on another device (later).** Re-embedding the corpus
through `esp.remote.burn/v1` on the GPU device, using the BERT path already
proven on stable. Needs Distillery `remote` in djinn, transport access in
`bind_resident`, plain or Fusion server profiles only, and a disclosure
ruling because retained text is local-only today.

---

## 5. Phases

Each phase stops at its done-conditions; none has a date. I1 to I5 prove the
pipeline with SmolLM2-360M-Instruct (ruling 5) and close Distillery M2. I6 to
I9 grow the size and breadth of models an ask can use (rulings 12 to 14).
I10 and I11 widen recall.

### I1. Lane, route and stub

R1, R2 and R5 with `StubInferenceProvider`.

Done when: an installed resident with the lane on answers `Ask` through the
route with streamed stub text; `Cancel` mid-stream yields a `cancelled` run
with partial text and no later fragments; enabling the lane without the
Distillery lane is refused by name (ruling 3); shutdown joins the actor; a
default-feature djinn build is unchanged.

### I2. A real model and its template

**Status (2026-10-15): done.** Receipts are in §7.

R3 and the real provider path: resolve a manifest from the model library,
build a `DecoderProvider` (or session-bound provider) on the probed device,
render the template.

Done when: a native test generates from SmolLM2-360M-Instruct with its
`esp.prompt-template/v1` artifact (ruling 6), records the exact session id
and template hash, and matches an independent reference token-for-token at
temperature 0; changing the template changes the session id; GPU and CPU
rows both pass where hardware allows; an ask on the GPU while a trainer job
holds it reclaims the job, which ends as a clean cancellation (ruling 4); and
the decoder refuses by name each config feature listed in R12's strictness
step, with a test per feature, so no other family can load and answer
wrongly.

### I3. Durable asks

R8, and the lock behaviour of ruling 11.

Done when: an ask killed mid-answer (resident stopped) resumes after restart
and finishes, and on the same backend its tokens match an uninterrupted run
or the first differing position is recorded and traced to prefill-versus-step
numerics; locking pauses a running ask at a token boundary and unlocking
resumes it the same way; seeded sampling resumes at the recorded draw; a
paused ask resumes on a second device holding the same session, and its
record names the device change and any token where outputs first differ
from a single-device run; the checkpoint log replicates over the persona's
existing transport.

### I4. Recall with citations over retained material

R4 (lexical), R6.

Done when: over a fixed retained corpus with version-bearing sources, a
question returns an answer whose citations name source versions and
selectors that were in the prompt; a changed source is reported stale and
not cited; the run record reopens after restart with its status; a stub
provider does not satisfy this phase.

### I5. The answer card, headed

R7.

Done when: Turnstone asks through the route against the real model and
retained corpus, shows progress and the streamed answer, pauses and resumes
one ask, cancels another, and reopens a past ask's record, headed. This
closes Distillery M2 and the inference-surface half of Eidetic P4.

### I6. Larger than memory: layer streaming

R9.

Done when: on one device, a sharded model whose weights exceed that
device's free memory completes an ask whose tokens match the same model run
in memory on a larger device (or an independent reference) at temperature 0;
prefill of a several-thousand-token prompt stays within a stated memory
bound; the row records peak memory, disk bytes read per token and seconds
per token; the ask survives a pause and resume mid-stream.

### I7. Weight formats

R10.

Done when: half-precision weights stay half precision in memory on a CPU
row (burn-flex) and a GPU row; 8-bit and 4-bit rows, quantized at import and
stored in the model library, each record their deviation from bf16 on a
fixed prompt set and their memory and streaming cost; selecting a format
changes the session id; a resume refuses a format change.

### I8. Layer stages across devices

R11.

Done when: two or more devices over iroh hold disjoint layer ranges of one
model and complete an ask whose tokens match a single-device run of the same
session on the same backend class; one stage leaving mid-answer causes a
checkpoint and a completed answer by reassignment or streaming fallback; the
row records per-token time and bytes moved between stages; stage work runs
under mesh leases.

### I9. Architecture coverage

R12, in its order: the Llama block completed, then mixture of experts, then
the rest.

Done when: each added family has a reference-matched row and a template
artifact; the largest model of each family whose licence permits the use,
and which the fleet can reach, has completed an ask by some rung; and at
least one mixture-of-experts model larger than any single device's memory
has completed an ask with streamed experts.

### I10. The envelope and a vector lane

R4's vector lane behind Eidetic's embedding envelope.

Done when: restart accepts an index whose envelope matches and rejects a
same-dimension index from another space; fused recall is measured against
lexical-only on a held-out paraphrase corpus; resident memory stays inside
ruling 7's budget (sparse, or at most 384 dense dimensions).

### I11. Batch embedding on another device

R13.

Done when: a two-device run embeds the corpus on the GPU device under a
lease, the poster's index is bit-identical or within a recorded tolerance
of a local run, reclaim mid-batch recovers, and the disclosure ruling is
recorded.

---

## 6. Stop rules

Stop and record the reason if any of these happen:
- the lane cannot stay behind a feature without changing djinn's default
  build;
- a phase would need the resident to fetch over HTTP, or would need a paid
  or hosted endpoint to complete an ask;
- citations cannot name source versions without changing Eidetic P4's
  schema (raise it there first);
- the route would need token payloads pushed rather than revision bells;
- answers would write into the graph other than as derived facets;
- a resume cannot be validated against its session (refuse it rather than
  continue on a different model, template or format);
- stage work would run outside mesh leases.

## 7. Progress

- **2026-10-15.** Founded from ruling 1. Research covered `esp::infer`,
  Distillery and mesh plumbing, djinn's resident and app endpoint, and the
  retained-material and recall state; §2 records it. Nothing started.
- **2026-10-15.** Mark took the recommendation on seven open questions
  (rulings 2 to 8).
- **2026-10-15.** Mark widened the purpose (rulings 9 to 14): reliable
  answers over speed, durable asks, pause on lock, models as large as the
  hardware allows, layer stages across devices, and families beyond Llama.
  Phases were renumbered: durable asks became I3, recall I4, the answer card
  I5 (closing M2), and the envelope and batch embedding I10 and I11; layer
  streaming, weight formats, stages and coverage are I6 to I9.
- **2026-10-15.** A decoder audit (§2) found one supported architecture,
  several families that load and answer wrongly, single-file f32-resident
  weights and quadratic prefill memory. R9, R10 and R12 and the done-conditions
  of I2, I3, I6, I7 and I9 now carry what it found; making the decoder refuse
  what it cannot run is part of I2.
- **2026-10-15, I2 strictness step landed.** The decoder now refuses, with a
  typed `InferError::Unsupported { feature, detail }` naming the config field
  or tensor: a `model_type` other than `llama` or an `architectures` entry
  other than `LlamaForCausalLM` (absent fields are allowed, for synthetic
  configs); `text_config` nesting; non-null `rope_scaling`;
  `partial_rotary_factor` other than 1; an explicit `head_dim` that differs
  from hidden / heads; `attention_bias` or `mlp_bias` true; a `sliding_window`
  shorter than the context unless `use_sliding_window` is false;
  `hidden_act` or `hidden_activation` other than SiLU; logit soft-capping;
  any expert count. The loader refuses any tensor it would not read (biases,
  QK-norm, experts, extra norms, layers beyond the config), tolerating only
  RoPE `inv_freq` buffers. Nineteen new unit tests cover the refusals and
  the real Llama-block fields; esp's 83 inference tests, its Eidetic corridor
  and trainer-forcing tests, and Distillery's trainer mesh-job test pass on
  CPU. The real SmolLM2-135M-Instruct config and all 272 of its tensors
  satisfy the rules (checked from its header on O-PC); the real TinyLlama
  tests were not run here (no checkpoint on this machine). Mistral
  7B v0.2 and v0.3, which would compute correctly, are refused too until a
  reference-matched row adds `mistral`.
- **2026-10-15, I1 done.** The lane core is
  `ports/distillery/src/inference.rs` behind Distillery's new `inference`
  feature, beside Chronicle's endpoint, so an embedded resident can compose
  it; djinn wires it in `ports/djinn/src/resident_inference.rs` behind its own
  `inference` feature.
  - *Lane.* `InferenceLane` owns the esp actor and a run table. A plain bridge
    thread folds the actor's updates into the table, instead of R2's
    `Wake`-to-`Notify` bridge: it needs no async executor, which keeps the
    embedded form open. Updates for an ended run are dropped, so nothing
    reaches a cancelled run. `close` cancels unfinished runs, then joins the
    actor and the bridge.
  - *Route.* `inference.ask` serves one card whose media is the run table
    (`distillery.inference-runs/v1`) and rings when the table moves; its
    100 ms notice poll is the fragment cadence. `Ask` carries a caller-named
    run, because an intent result carries no data; `Cancel` names the run.
    Duplicate and unknown runs, empty fields and other intents are rejected.
  - *djinn.* `OwnerSettings.inference` (`{"model": "stub"}` in I1, skipped when
    absent so older builds can still read the file) is refused before anything
    opens when the build lacks the feature or there is no Distillery lane
    (ruling 3). The lane closes first at shutdown and is granted to
    `turnstone` alone (ruling 8).
  - *Distillery's esp edge.* `decoder-lora` moved from the optional `esp`
    dependency onto `trainer` and `flora`, so `inference` adds only esp's actor;
    the trainer and FLoRA features still compile.
  - *Receipts.* Distillery: six lane tests (streaming to a finished answer, a
    gated provider cancelled mid-stream keeping its partial text and nothing
    after, refusals, close cancelling queued runs, the route's ask, stream
    and ring), 20 of 20 repeated runs clean on the Kubuntu Surface. djinn on
    O-PC (Windows, Rust 1.98.1): settings tests with and without the feature;
    `tests/inference_route.rs` opens a real resident with the Distillery and
    inference lanes, serves the real app broker on a named pipe, asks and
    reads the answer as Turnstone, refuses three other app labels, and shuts
    down cleanly; the default-feature djinn binaries build; clippy is clean on
    the changed files. The installed `djinn` binary's own grant was compiled,
    not run.
  - *Low-end note (ruling 9).* djinn does not build on the Kubuntu Surface
    as it stands: genet's font stack needs `pkg-config` and the fontconfig
    development package, and its fontique backend does not support
    fontconfig-sys's runtime-loading mode. Distillery's lane builds there.
  - *Not in I1.* Pause on lock (ruling 11) is I3.

- **2026-10-15, I2 CPU model/template continuation.** The branch continuation
  adds `esp::infer::prompt`: strict `esp.prompt-template/v1` data with literal
  role delimiters, BOS, EOS, a context slot and stop strings. The checked-in
  `crates/intel/esp/templates/smollm2-instruct-v1.json` follows
  [the upstream tokenizer template](https://huggingface.co/HuggingFaceTB/SmolLM2-360M-Instruct/blob/a10cc1512eabd3dde888204e902eca88bddb4951/tokenizer_config.json),
  including its default system instruction. Tokenizer postprocessing is disabled
  for declared templates so it cannot silently insert a second BOS. Stop matching
  holds a possible stop prefix across fragments, flushes an incomplete prefix at
  a normal token limit, and does not flush it on cancellation.
  - *Library and session.* An `esp.inference-model/v1` row names the existing
    `ModelManifest` and a typed template artifact. `infer::library` resolves both
    with `NoFetcher`, validates the schema and exact bytes, and builds a
    `SessionProvider<DecoderProvider>` with an immutable `ModelSession`.
    `InferenceLane` resolves under Distillery's existing library handle, releases
    the store lock, then loads on its actor thread. It exposes the exact session
    id and template hash in the table and each run. A fallible actor constructor
    exposes load failure before any ask and fails later asks with that error;
    it never announces a ready model or substitutes the stub.
  - *Composition.* Djinn's opt-in `inference` now enables Distillery's separate
    `inference-model` feature; bare Distillery `inference` remains actor-only.
    Settings preserve `{"model":"stub"}` and accept
    `{"model":{"library":{"row":"blake3:<64 hex digits>","device":"cpu","max_tokens":256}}}`.
    All three library fields must be written; unknown fields, malformed row
    references, zero limits and GPU selection are refused. Distillery opens the
    persona library for inference even without a trainer, and retains its sole
    handle. Existing trainer library overrides are reused through that handle.
    An opening failure unwinds Distillery, Knot and blob custody in order.
  - *Import.* `cargo run -p esp --release --features decoder,model-session
    --example import_inference_model -- LIBRARY_REDB MODEL_DIR TEMPLATE_JSON
    MODEL_ID LICENSE` imports local files and prints the inference settings
    block. Close any resident holding that library before importing; the tool
    opens the same redb file and does not bypass its writer exclusion. The
    resident performs neither download nor HTTP fetching.
  - *Independent basis.* `crates/intel/esp/tests/reference_smollm2.py` runs local Transformers
    4.57.6 / CPU Torch 2.14.1+cpu in f32 with eager attention, greedy decoding
    and 16 generated tokens on three questions. The committed
    `crates/intel/esp/tests/smollm2_360m_reference.json` pins revision
    `a10cc1512eabd3dde888204e902eca88bddb4951`, SHA-256 for the model/config/
    tokenizer files, rendered prompts, input/output token ids and elapsed time.
    `crates/intel/esp/tests/smollm2_library.rs` verifies those artifact hashes before loading
    through `ModelLibrary`, checks prompt equality and every generated token,
    and reports the exact ESP session/template identity and completion time.
  - *CPU receipt.* `support/receipts/resident_inference_cpu.json` records the
    passing real-checkpoint library row on O-PC (Windows, AMD Ryzen 9 7940HS,
    Rust 1.98.1). All 39 greedy token ids match the independent reference
    (7/16/16 across the three prompts). The first answer reaches EOS; the other
    two reach the configured 16-token cap. Measured generation calls take
    15.74/33.43/36.17 seconds; the test repeats each through the actor-compatible
    provider path and finishes in 172.27 seconds. Loading the resolved library
    row takes 1.07 seconds. A live Windows peak-working-set sample is
    3,626,852,352 bytes; this test includes an in-memory `MemoryBackend` library,
    not the resident's redb store. The receipt includes exact session/template
    identities, the tested parent, source byte hashes and the test-binary hash.
    Reproduction: set `ESP_SMOLLM2_DIR` to the pinned local snapshot, optionally
    set `ESP_INFERENCE_RECEIPT` to an output JSON path, then run `cargo test
    --locked -p esp --release --features decoder,model-session,actor --test
    smollm2_library -- --ignored --nocapture`. The independent script was rerun
    against the pinned files and reproduced all artifact hashes, rendered/input
    prompts and output tokens.
  - *Other checks.* ESP release: 160 unit tests, three template tests, two
    library/session tests and the existing Eidetic corridor pass (166 checks).
    Djinn with `inference`: five settings/validation unit tests and all three
    real app-broker tests pass, including a synthetic library-backed decoder
    with no trainer, exact session/template identity in the served run table,
    Turnstone-only access and clean shutdown. The real 360M row and the real
    app-broker row are separate receipts. At that CPU checkpoint, default-feature, Distillery
    recheck and import-example checks did not start: a separate `cargo metadata`
    on the primary checkout held the shared package-cache lock for over fifteen
    minutes. Only this lane's queued checks were stopped; the other process was
    preserved. Unrun commands: `cargo test --locked -p distillery --features
    inference-model --lib inference`; `cargo test --locked -p djinn --lib
    resident_inference`; `cargo test --locked -p esp --no-default-features --lib`;
    `cargo build --locked -p esp --release --features decoder,model-session
    --example import_inference_model`. Formatting and diff checks pass. The
    documentation audit reports no new changed-source/plan findings; existing
    repository-wide findings remain. No installed resident binary was run.
  - *Open at that CPU checkpoint:* the GPU row and owner-reclaim handshake.
    The continuation below closes them. This CPU receipt does not claim an
    installed-binary run or change I3's durable asks and lock posture.

- **2026-10-15, I2 GPU loading and trainer reclaim qualified.**
  `distillery/inference-gpu` and `djinn/inference-gpu` add a probed first-discrete
  adapter path. GPU settings require that feature; an unavailable adapter is
  refused. The actor retains resolved host buffers, loads device tensors for
  each ask, and drops and synchronizes them before returning the GPU. CPU
  inference retains its loaded provider. Both use the same declared template
  and exact model-library row; generation borrows no store lock.
  - *One holder.* `distillery::gpu::GpuAccess` withdraws the GPU offer as soon
    as an ask requests it. The mesh host initiates existing lease-owner reclaim
    for GPU work. The trainer's guard lives inside its blocking computation,
    through device synchronization, so an async task detached after lease loss
    cannot falsely free the device. Shutdown also waits for that actual guard.
    Both trainer arms check owner stop signals between tensor operations. A
    checkpoint request cancels these non-resuming trainer arms cleanly; it
    never claims a saved checkpoint. This does not implement I3's durable asks.
  - *Real model.* The pinned SmolLM2-360M-Instruct row passes all 39 independent
    greedy token comparisons (7/16/16), including a second generation through
    the provider seam for each question, on O-PC's RTX 4060 Laptop GPU, Vulkan,
    driver 610.88, 8188 MiB total GPU memory. Load was 2.53 s and measured first
    generations were 4.29/2.10/1.69 s; this is a correctness receipt, not a
    benchmark. Session `blake3:e98a2d6d857bbeea8f431367885d4a5ddee51d3751f69e9ade6943c19314f925`
    binds the same template hash as the CPU row. Peak GPU memory was not
    measured. The first prompt reaches EOS; the other two reach the 16-token
    test cap. Exact identities, source and binary hashes, and native reclaim
    rows are in [the GPU receipt](../../../support/receipts/resident_inference_gpu.json).
  - *Native reclaim.* Three Djinn resident tests use the real GPU trainer
    resource over a tiny model: finite-difference and autodiff jobs posted as
    restartable, plus a job posted as checkpointable. Each observes a live
    lease, withdrawn `HostFacts.gpu`, denied background GPU access, a clean
    `Step::Reclaimed { reason: ForegroundActivity }`, a finished GPU ask, a
    restored offer, and drained GPU ownership after shutdown. No lease-loss or
    failed-release result substitutes for owner reclaim. These call the native
    in-process lane; the real SmolLM2 row and the app-broker receipt remain
    separate harnesses. No installed resident binary or headed application ran.
  - *Checks.* ESP: 169 release unit tests, six template/library/corridor tests,
    and 82 no-default-feature tests pass. Distillery: 34 unit tests, including
    close while waiting for GPU ownership, and the real two-host supervised
    reclaim test pass. Djinn: five inference/GPU settings tests and four
    default-feature settings tests pass; the three real app-broker tests also
    pass in the GPU-capable build. The import example now builds with
    its redb dev feature declared. All four commands blocked at the CPU
    checkpoint have completed. The two device-gate tests pass 20 repeated
    runs; a generated copy that ignores occupied ownership fails its intended
    detached-worker assertion. The checkout was never mutated for that
    negative control. Formatting and documentation checks retain the existing
    repository baseline; this continuation adds no new audit findings.
  - *Next:* I3's durable ask checkpoints and lock pause/resume. I2 is closed;
    the installed-binary and headed answer-card gates are still separate.

- **2026-10-15, I3 durable generation and local recovery.** ESP's portable
  checkpoint carries the rendered request, exact session/template, prompt IDs,
  generated IDs, actual seed, committed draw count, accumulated text and completion
  marker. No KV tensors are serialized. The decoder pre-fills those IDs on resume
  and advances splitmix64 by the recorded number of draws. A host-supplied commit
  barrier runs before each checkpoint becomes visible; a failed write prevents
  another token. Unsupported providers refuse durable generation. The legacy
  stub and the actor-only Distillery feature remain ephemeral fixtures.
  - *Retained evidence.* Eidetic owns `models::inference::InferenceRunRecord`:
    question/output/checkpoint references, actual request settings, exact session,
    status, device visits and explicit token-comparison evidence. Distillery's
    encrypted journal carries those records and their schema reference beside
    checkpoint deltas. State transitions also materialize sealed native Eidetic
    manifests and opaque question/output/checkpoint blobs through `save_typed_sealed`;
    retained reads use `load_typed_sealed` with `NoFetcher`. Remote log replay can
    rebuild the same typed projection. Passage refs are empty until I4 supplies
    retained passages. Paused, cancelled partial, failed and finished remain
    distinct; a token-complete run ends only after provider synchronization.
  - *Lifecycle and route.* Library asks are retained in `inference.redb` below
    the existing Distillery mesh root. A local interrupted or shutdown-paused ask
    resumes at reopen; a user-paused ask waits for Resume. Pause/Resume intents
    use the run-name payload, and a separate sampled Ask intent records explicit
    temperature, top-p, seed and owner-capped token count. A synchronous vault
    lock holder refuses route access, waits for the actor boundary, clears run
    materialization and registered endpoint caches, and drops the data key.
    Unlock derives that key again, reloads retained state and resumes only
    lock-paused asks. GPU pauses discard KV/device tensors and synchronize before
    releasing the guard; keeping weights allocated is permitted, not required,
    by ruling 11. Shutdown joins inference and its sync lane before Distillery.
  - *Log mechanics.* `PersonaRunJournal` signs per-device, per-ask logs through
    Stickleback's existing muniment store and policy-before-insert processor.
    XChaCha20-Poly1305 seals checkpoint contents under a separate persona-derived
    data salt, binding the signed log address as associated data. Immutable
    prompt bytes occur at creation; ordinary token entries append only new IDs
    and text. Explicit device/state visits carry snapshots. Replay retains
    current prefixes rather than every historical prefix in memory. Concurrent
    continuation tips are refused as a visible fork. An uninterrupted comparison
    stores the reference token-ID hash and first differing position; an absent
    comparison hash means unmeasured, not matched.
  - *Qualification boundary at `9a4418ce` (historical; superseded by the
    personal-graph binding below).* The existing Distillery endpoint carried the
    journal lane; a native two-host test uses independent transport endpoints
    and the same journal persona key. It proves real p2panda catch-up, continuation
    on the second host, device/position provenance and equal seeded IDs. It does
    not prove a second physical computer, or grant separately rooted paired
    personal-graph devices access. Personal sync already pairs such roots and
    supplies its own key group. At that checkpoint, whether private asks followed
    that keyed group or only shared persona keys was pending; that implementation
    preserved shared-persona admission and left I3 open. Ruling 15 now resolves
    the boundary. No installed
    resident or heap-residue audit is claimed by these tests.
  - *Checks.* ESP passes 160 release unit tests, both durable decoder tests
    (every pause boundary under greedy and seeded top-p sampling, failed-commit
    barrier and bad draw-position refusal), and 82 bare-feature tests. The pinned
    real SmolLM2 GPU row matches all 39 independent greedy IDs and all 39 resumed
    IDs after a three-token pause, with no differing position on this backend.
    Distillery passes 36 unit tests and eight durability test entries, including
    the child-process harness; these cover seven scenarios. The seven local
    entries pass 20 repeated runs, excluding only the network scenario. A real
    child process is killed while held after its committed third token, then its
    redb log reopens and finishes at the recorded seed/draw position. The native
    two-host LogSync test proves catch-up and an explicit transfer; cryptographic
    negative controls refuse wrong persona signers, wrong keys, modified bodies
    and unsealed reads, and concurrent continuation tips remain a visible fork.
    Djinn passes all three real app-broker tests, now including lock/refusal and
    sealed retained-answer reopen, all three GPU trainer-reclaim regressions, and
    four default-feature settings tests. Formatting and the documentation audit
    add no findings beyond the recorded repository baseline. Exact source and
    binary hashes, commands and qualified limits are in
    [the durable-ask receipt](../../../support/receipts/resident_inference_durable.json).

- **2026-10-16, I3 keyed personal-graph binding (ruling 15), software gates
  qualified.** The personal-sync host supplies a live protection handle to its
  existing data keyring and membership. Inference uses a distinct sibling topic on
  that host's endpoint, with additive peer tags kept when later peers pair.
  No new key group or transport endpoint is opened for inference. Writer roots
  remain distinct; graph/domain-specific public attestations bind each signer.
  Private envelopes bind their complete signed address before insertion.
  The header carries a compact 129-byte versioned public attestation, within
  the existing p2panda header bound. Local appends round-trip the real header
  codec before insertion, so an oversized extension cannot poison the log.
  Incoming group records are schema-checked and must have an already retained
  parent from the same run before insertion. Out-of-order children are refused
  and can be offered again, keeping a catch-up interrupted by a crash reopenable.
  - *Composition.* Djinn prepares the resident first and starts inference after
    personal sync opens, before registering its lock holder and route. The
    inference owner retains the personal host through actor and journal drain.
    A resident without personal sync keeps its local journal without joining
    the former shared-persona replication lane.
  - *Storage and lock.* Graph journals live in `inference-personal-<graph>.redb`
    beside `inference.redb`. On first keyed read, existing local records are
    re-sealed into the group without overwriting an existing shared head.
    A different local ask using an existing run name is a visible collision;
    its local source is retained instead of silently selecting shared history.
    Local Eidetic materializations keep a device-local persona-derived key.
    Lock gates admission and materialization, clears heads, and also locks any
    pending local import; unlock restores local access and reopens the sync
    session to reconcile its retained frontier. Key-group revisions and missing
    causal parents also request reconciliation. Locked checkpoint bodies are
    refused rather than queued as decrypted data. The personal-sync owner
    continues to own group epochs and key agreement.
  - *Native evidence.* Two independently rooted vaults and resident compositions
    use real personal-graph key agreement and p2panda endpoints. The tiny CPU
    model continues on B after A's committed three-token pause; all eight seeded
    token IDs equal the independent reference. Sealed native Eidetic evidence
    records both visits and the measured comparison. An unkeyed device, an
    outside writer, wrong graph/domain/address, plaintext fallback and a
    cross-run parent are refused. A keyed reader can continue without browser
    graph write permission. Unlock catches up asks refused while locked;
    rotation excludes the retired reader from the new epoch, while retained
    history still loads and reaches a late reader. A local-name collision keeps
    its source. Both residents drain their shared custody and their personal
    stores reopen. This complete scenario passes 20 repeated native runs.
  - *Checks.* Distillery passes 37 unit tests and eight durability entries,
    including killed-child recovery and the legacy same-persona network fixture.
    Djinn passes six inference settings/attestation tests, four default-feature
    settings tests, the four native inference-route tests and all three GPU
    trainer-reclaim regressions. Graphshell passes 15 personal-sync host tests,
    including additive sibling topics on pairing and unpairing. Formatting and
    the documentation audit add no findings beyond the recorded baseline.
  - *Evidence boundary.* The new gate runs two real endpoints with separate
    identity roots on one O-PC and uses a tiny CPU model. It does not claim an
    installed-binary run, a second physical computer, or a heap-residue audit.
    The earlier real SmolLM2 and killed-child checkpoint receipts remain
    independently qualified by their own source revisions; ESP's decoder and
    provider are unchanged in this continuation.
  - *Verification record.* [The personal-graph receipt](../../../support/receipts/resident_inference_personal.json)
    records the gate results, repeated comparisons and source/binary hashes.
    I3's software gate is closed. I4 recall with retained passage citations is next.


- **2026-10-16, I4 retained lexical recall, native trained-model gate qualified.**
  Distillery borrows a source-owned corpus. `PageTextCorpus` reads already-open
  Eidetic page-text storage; capture, clip and Fleece producers use the same
  source identity/current-version contract. The host never reopens an application's
  private database or fetches a current URL as a substitute for a retained version.
  The fixed corpus gate uses two version-bearing page-text sources.
  - *Inputs and ranking.* Configurable Unicode chunk sizes and corpus/context/
    passage limits bound a versioned deterministic chunker. The existing BM25
    `DocumentIndex` uses passage content references as keys. Each passage binds
    the source identity/version, canonical text hash, Unicode range, chunker and
    exact text. `ask-recall-v1` records its source scope; the declared template
    renders selected handles and text in the context field before generation.
  - *Citations and stale sources.* Source owners revalidate current versions.
    Stale or unavailable hits are reported and excluded. With no usable passages,
    the ask is retained as failed before a token or checkpoint is generated.
    The answer view distinguishes model-written numeric markers from host-made
    annotations of a complete verbatim supplied-passage match. Both name the
    exact passage artifact and answer span, and both disappear from the current
    citation list when their source becomes stale or unavailable. Partial matches
    and invented handles do not establish support. The configurable minimum quote
    length defaults to 32 non-padding characters; zero disables host annotations.
    Source changes advance the route revision so clients receive a notice; each
    view uses one source-head observation for its revision and citation checks.
    This does not assert semantic entailment of generated paraphrases. Historical inputs remain inspectable.
  - *Durability and compatibility.* The journal saves supplied passages as native
    sealed Eidetic artifacts and fills `InferenceRunRecord.supplied_passages`.
    Original rendered prompts and passage evidence continue through I3's log.
    Every context-bearing entry carries authenticated `required_semantics`:
    old readers' deny-unknown envelope refuses it rather than answering a queued
    question without its context. Existing ordinary asks retain their encoding.
    Vault lock drops the derived recall index with the run materialization.
  - *Composition boundary.* The native/embedded host lends its existing source
    adapter through `configure_inference_recall`. Desktop Turnstone's producer
    handoff, repin and headed answer card are I5 integration. This gate does not
    claim automatic discovery of its app-owned browsing store, a headed run, or
    concrete capture/clip/Fleece producer adapters beyond the shared contract.

  - *Trained-model receipt.* The pinned SmolLM2-360M-Instruct library row answers
    through a real Turnstone-labelled app connection to an in-process Djinn,
    using its existing discrete GPU provider. BM25 selects one of two retained
    observatory notes. The model returns the Pelican note's 81-character answer
    verbatim; the service attaches a `verbatim_passage` citation over answer
    positions 0..81. This is explicitly a host exact-quote annotation, not a claim
    that the model generated a numeric citation marker. The prompt checkpoint
    contains that passage's exact version and selector and excludes the other
    note. Updating the source withdraws the citation; a subsequent scoped ask
    remains a failed, zero-token run without a checkpoint. Lock refuses the route.
    Restart reopens the finished answer and the identical sealed passage artifact
    through `NoFetcher`, while keeping its stale citation suppressed.
  - *Verification.* The new recall, downgrade-refusal and exact-quote controls
    pass with the 48 Distillery unit tests and eight durability entries. The five
    resident inference-route tests pass, including the real model and I3's
    separately rooted personal-graph handoff. ESP passes 160 release unit tests,
    two checkpoint tests, two library-session tests and five template tests.
    All three GPU trainer-reclaim regressions pass, as do six inference settings/
    attestation tests, four default-feature settings tests, 35 actor-only tests
    and four retained page-text store tests. Formatting and the documentation
    audit add no findings beyond the existing baseline. [The I4 receipt](../../../support/receipts/resident_inference_recall.json)
    retains the exact native output, gate results and source/binary hashes. This is
    a native software gate; installed-binary, headed application and broad
    paraphrase-entailment claims remain outside this receipt. Two standalone
    probe request literals also accept the new empty context field. The Wasm
    browser probe passes `cargo check` with its existing lock dependency edges
    refreshed and package versions unchanged. The native fixture build is separately blocked by fresh lock resolution through
    `burn-tch`/`tch` to yanked `zip 0.6` versions; the resident gate uses the
    existing workspace lock.

- **2026-10-16, I5 integration started.** The inference branch incorporates
  published Mere `9b6da81d`, preserving I1-I4 while adopting the application
  door in `graphshell-endpoint`, serving/catalog support in `graphshell-host`,
  and personal-sync ownership in Djinn. The private sibling sealing handle
  follows its key-owning host into `djinn::sync`; it is not an application-door
  capability. Turnstone's clean main incorporates its published updates while
  preserving its local committed change. Existing I1-I4 receipts retain their
  recorded source cuts; integrated compilation and I5 headed acceptance remain
  pending. I5 must supply retained source versions from Turnstone's existing
  trail owner through the admitted app route, keep source namespaces distinct,
  and consume a decoder-free wire view. It must not reopen the trail database
  in Djinn or introduce ESP into Turnstone.

- **2026-10-16, I5 producer protocol.** A decoder-free `inference-wire` feature
  exposes the existing run/control data to Turnstone. The admitted inference
  route accepts a bounded retained-text publication per live source-owner
  session, and asks against that namespace. Hashes are checked before indexing;
  an endpoint cannot overwrite another live publisher. Disconnect and lock
  withdraw publications, while supplied passages remain sealed evidence.
  Recall limits belong to Djinn settings. Inline publication JSON has a 128 KiB
  cap, with refusal instead of truncation; a chunked larger-corpus protocol is
  outside this slice. Published checkpoints require `recall/published-v1` so
  older readers cannot silently lose source ownership. Current integrated
  checks pass 50 Distillery unit tests and eight durability tests; Djinn's GPU
  library, binary and native route test compile. These are producer-protocol
  checks. The Turnstone consumer and headed acceptance are still pending.


- **2026-10-16, Turnstone graph-scene supplier integration candidate.**
  Turnstone's published fold consumer locks Mere
  `65b97b0bfdf1eef5ba3b4ecdccba73f47a049d03`, whose decoder-free
  `distillery/inference-wire` feature is on the published inference line and
  absent from the separate current `main` line. The isolated
  `codex/turnstone-scene-supplier` branch merges published main
  `d1e1d7aaed1b2b3f99fceca6fa7301ab09f5aa46` into that exact prior consumer
  basis. This brings Partition, Sets, Provenance, Mosaic, attention and seed
  presentation, and Pandect's scene-state facets together with Turnstone's
  existing admitted inference protocol. Git merged source and the workspace
  lock without conflicts. The newer inference continuation through
  `1f4809918154b4c4232ed00d8b837b196f10fe99` is outside this candidate, and its
  active branch and working copies are untouched.
  `cargo metadata --locked --offline --no-deps --format-version 1` passes;
  this is manifest validation, not compilation or behavior acceptance.
  Turnstone owns the graph-scene integration, focused tests, native primary
  and lens presentation, and fresh-process reopening gate before publication
  of its consumer update. Earlier inference receipts retain their exact
  original suppliers; this merge does not requalify those gates or mark the
  candidate as Mere main.


- **2026-10-16, Turnstone canonical-tag scene correction.** The receiving
  consumer's public `Canvas::tag_node` fixture exposed a Sets adapter read
  through legacy Surface tags, while the admitted write stores canonical
  Resource tag assertions. `scene_axes::tag_sets` now reads the kernel's
  `node_content_tags` contract, retaining its kernel-owned legacy fallback.
  A regression inserts tags through `GraphDelta::InsertNodeTag`, verifies
  that raw Surface tags remain empty, and checks the canonical membership
  axis. The bounded-scene fixture likewise uses the public tag mutation
  rather than filling the legacy Surface column. Provenance source writes
  and axes are unchanged: its receiving assertion needed to consume the
  buffered placement through a normal Canvas frame before reading positions.
  Three `scene_axes` unit tests pass under the candidate workspace lock,
  followed by all five bounded-scene tests in the same compiled test binary.
  The focused build uses an existing dependency cache with explicit source
  paths; it does not borrow an unbuilt consumer result. This remains the
  isolated supplier candidate; Turnstone's exact-source receiving tests and
  headed acceptance own its publication gate.
