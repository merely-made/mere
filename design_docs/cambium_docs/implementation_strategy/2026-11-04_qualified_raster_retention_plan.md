# Qualified raster retention and native frame receiving

**Status:** 2026-11-05. R2–R7 retain their fixed-cut qualification. R8
is complete. All four shared owners and Isocosm `da56ce39` are
published, with current-cohort native, consumer build and C4 evidence.
Fresh-query sharing, target pooling, complete CPU/GPU attribution and
physical 60 Hz remain open. R4 and conditional C5 retain their separate
scope.

## Direction and ownership

Mark, 2026-11-04: "We should work towards this. Radical change is no issue;
prototypes and all. Refactor/simplification? Missing technical component?
And the wing… what’s relevant to the sim and the rest?" This authorizes
investigation and implementation toward the 16.7 ms host frame budget; it does
not certify another platform, physical refresh, or a supplier cohort update.

Genet owns checked GPU admission, shader quality observation and device-loss
attribution. Its pinned native batch API already supplies nonblocking
submission, worker completion and guarded accepted publication. Cambium
Rootstock owns retained output, the native frame lifecycle, input and capture
publication. Netrender owns ordinary paint content semantics and renderer
policy. Isometer and product adapters retain scene/subject interpretation.
Cosmograph owns simulation authority, clocks, matter, scheduling and replay.
A rendering optimization must preserve those ownership boundaries.

## Evidence and alternatives

The wing's paired paused fixtures have no scene redraws, producer staging or
layout mutation, yet the complete checked raster call has an approximately
20ms region outside the inner renderer timing. Source inspection reaches
Genet's synchronous `device.poll(Wait)` quality loop. A fresh native sample
places most main-thread observations there; its presentation watchdog fails,
so it is attribution evidence, not a successful headed performance result.

| Option | Decision | Consequence |
| --- | --- | --- |
| Faster component iteration or another ECS | Separate experiment | Cannot eliminate an unchanged app-frame raster/quality wait. Native store work remains useful for data access and simulation. |
| Remove or weaken GPU qualification | Rejected | Would change the checked-rendering contract and hide failed output. |
| Retain the last qualified complete raster | First implementation | Eliminates raster/quality readback only for exact unchanged admitted content. Still validates current bindings and presentation. |
| Adopt Genet's pending/accepted native frames | Implemented and locally qualified in R3 | Keeps input serviceable while qualification is pending, with immutable frame facts and atomic producer/capture publication. Bounded CPU/picture evidence does not certify physical display throughput. |
| Compose a separate retained HUD and viewport directly | Conditional larger refactor | May avoid a second full raster, but must preserve painter order, transforms, clipping, opacity and pixel coverage. |

## Phases

### R1. Attribute and preserve

**Status, 2026-11-04:** complete for the bounded source/native sample; the
early fresh headed replays refused acquisition while macOS reported
occlusion. Later old/new runs both present successfully; the early
availability failure is not attributed to retention.

Preserve the current dirty checkouts and frozen wing fixture. Inspect pinned
and current owner source separately, obtain a native sample and account for
producer/render/quality/acquisition scope. Failed or unpresented runs remain
failed evidence. No timings are inferred as GPU duration.

Done when: the missing wall-time region has a source-backed attribution and
all preserved input/fixture identities are recorded in the receipt.

### R2. Retain qualified native output

**Status, 2026-11-04:** implemented in isolated owner worktrees. Rootstock,
paint, native admission and policy gates pass; optimized offscreen timing is
recorded. Both backends complete matched headed receiving with unchanged
saved worlds. This closes R2 locally on the declared fixed SDK, not the
changed-picture target or supplier adoption. The baseline performs a
checked raster on each request and its actual-GPU control fails as expected.

Compare the complete emitted ordinary paint payload after translation and
physical producer-binding validation. Publication sequence/layout epoch alone
are insufficient. Equality includes commands, font collection/index/bytes,
inline image dimensions/bytes and resolved producer registration, device,
content-stamp and size identities. The target key also includes host, context,
physical extent, scale, clear color and actual device identity. Reuse is
refused for renderer policies that independently animate pixels, including
the dirty-tile diagnostic overlay. Policy inspection refuses lock contention
without waiting. Portable/custom paths without this proof
continue to render normally. Retained fragment references also refuse reuse
until their owner supplies a revision witness.

Retain one output per host/driver, not an unbounded global cache. A hit reuses
a previously qualified raster under Genet's native-frame/loss admission. Its
new `try_reuse_native_raster` seam performs no raster, submission or device
poll and refuses a busy batch before cloning output. The host still acquires, composes and presents before
committing a fresh stamp. Cache misses retain existing checked raster/error
semantics. Refused current bindings cannot become hits; device loss,
restoration and disposal invalidate retained resources. Captures name the
actual presented output. Reused frames report no new inner raster work.

Done when: a genuine repeated-raster control fails before the fix and passes
after it; exact-payload hits avoid a raster, while changed paint/resources,
producer lifetime, size/scale/clear and device/context produce fresh results;
actual GPU outputs match cold references; loss/refusal and capture controls
remain true. Matched host observations retain p95/p99, CPU-budget counts and
all unavailable qualification dimensions.

### R3. One native pending/accepted frame lifecycle

**Status, 2026-11-04:** locally qualified on the declared fixed SDK. The
genuine R2 receiving control fails before the change and passes with R3;
actual-GPU cancellation, shader refusal, input/capture, two-host availability
and immutable product-fact controls pass. Both retained and component headed
workloads complete with the same native save and exact scene viewport.
Per-picture UI tails improve, while the consistent frame budget and native
wall tail remain follow-on work.

Expose the already-owned shared Core capability at the native Surface seam
and receive Genet batches in Rootstock. Consolidate the legacy host and owned
driver publication paths. Producer raster callbacks must receive the active submission capability;
a nested legacy checked raster is refused by the shared Core. A prepared
frame must retain its source facts,
producer geometry/resource lifetimes, generation/extent, capture requests and
publication callbacks while pending. Coalesce newer requests with bounded
backpressure; preserve the previous accepted image. Input, hit testing and
capture retain accepted A's source, camera, geometry, site and scale while B
is staged. Close/resize/loss and source replacement revoke stale staging.
Completion collects an existing packet without application hooks or another
OS redraw. Other hosts wait on a one-shot availability hint registered under
Genet's admission mutex; readiness is not a reservation. Unavailable surfaces
release work without a redraw loop; reconfiguration can owe one retry.

Done when: host collection never waits on the GPU; pending input/resize/close
and stale-generation controls pass; an accepted stamp cannot describe a
different product snapshot; actual shader refusal never publishes output;
changed-frame latency and completion throughput are measured separately.
These conditions pass for the bounded local fixture; they do not close the
broader frame-time or physical-display target.

### R4. Wing and shared-stack receiving

Mesocosm receives the shared mechanism in R3. VTT and Eponym still require
explicit transactional source/producer opt-in, accepted-fact snapshots and
their own declared fixtures; legacy sources are not implicitly staged.
Qualify other Cambium consumers independently. Keep rendering demand separate from native
simulation pacing; reuse Clock/Pacer/Ticker under each product's policy.
Attention/foreground handbacks already affect native resolution and must
remain admitted and replayable. Didgeridoo can later bind immutable native
queries and explicit intents to these source/lifetime contracts; VM fuel alone
does not supply dependency or publication semantics.

Done when: each receiving product has a declared fixture and source cohort;
paused, active and interacting cases retain native save/replay results and
show bounded scheduling. Broader old/new hardware and platform qualification
belongs to the presentation qualification owner; no local result claims it.

### R5. Accessibility work and native wall tails

#### R5a. Publish accepted accessibility trees; drain actions independently

**Status, 2026-11-04:** implemented and locally qualified. Genuine redundant-
publication and idle-event-route controls fail before their changes and pass
afterward; final owner/receiving and matched headed gates pass. Rootstock owns publication
and typed action admission, the Winit host owns turn sequencing, and the
native accessibility adapter owns its published routes and OS bridge.

Separate full semantic projection/tree publication from queued action
mapping/draining. Submission or an empty collection must not rebuild and
republish an already accepted tree solely because it is another host turn.
Publish only a newly admitted displayed semantic state, preserving the
accepted tree and route namespace while a newer picture is pending. Preserve
explicit synchronous callers and legacy/custom host routes under their
existing contracts; the owner chooses the smallest compatible API seam.

Action delivery is independent of whether another picture is due. Drain
requests through the publication they could have observed, before replacing
its namespace; retain typed click/focus/value distinctions, contributor and
host lifetime checks, request ordering, and ordinary native admission. A
stale or removed A target must not route into B under a reused ID. Draining
can create later model/render demand without publishing B early. A drain
alone must not claim a newly submitted tree or erase accepted publication.

Retain startup's install-before-show/root-only bootstrap, adapter-failure
visibility fallback, and later pending-tree preservation. Hidden, disposed,
resized, source-replaced, cancelled and shader-refused candidates must retain
their existing refusal behavior. Existing input/IME, captures and two-host
Core-availability controls remain required.

Done when: a genuine pre-change control executes and exposes redundant
publication; post-change controls prove accepted A stays published while B
is pending, actions drain without a fresh tree/frame, stale A routes refuse,
and one accepted B updates its tree/routes coherently. Bootstrap and ordinary
synchronous consumers still pass. Record full-tree publication and action
work separately and keep their total in all-attempt/per-picture accounting.
No speedup or physical accessibility qualification follows from source alone.

#### R5b. Attribute the native wall tail and repeat the comparison

**Status, 2026-11-04:** bounded attribution and comparison complete locally.
The worker qualification interval carries the measured long tail in this
fixture; notification delivery is sub-millisecond. AT portions are lower,
while increased acquisition waiting offsets them in paired profiled totals.
This is not an overall speedup, GPU-duration or causal attribution claim.
Genet owns worker qualification/queue spans; Rootstock owns preparation,
collection, acquisition and publication spans; the product retains the
immutable scene/subject and native save/replay controls.

Use the same identified pending packet to distinguish preparation CPU,
submission-to-worker delay, checked qualification/queue wall time,
completion-to-collection scheduling and acquisition/publication. Report
unavailable spans explicitly. Do not add overlapping spans or call elapsed
qualification/queue time GPU duration. Keep shader quality, device loss,
exclusive admission and bounded completion/availability semantics intact.
No polling loop or timed redraw is a substitute for the owned notification.

Freeze the qualified R3-07 source/executable as the before arm and the
admitted R5 source/executable as the after arm. Reuse the fixed SDK, native
paused bench, six workload phases, declared retained/component backends and
scenario from R3, with matched environment/session where available. Run both
before/after pairs and retain every failed, unpresented, missing-profile and
pending attempt. Pair recorded profiled portions for each logical picture,
including AT within those spans; accepted-profile totals alone are
insufficient. Pre-admission, sealing and outer bookkeeping lie outside those
clocks. Standalone/coalesced action-only costs are unavailable unless measured
separately; the profile is not complete callback or whole-event-loop work. Report medians, p95/p99, strict 16.7 ms and exact-60-Hz budget
counts, callback cadence, native wall latency, counts/uploads, native
save/subject/part agreement and decoded full/crop pixel differences.

Done when: the actual controls and all four bounded headed runs execute,
source/binary identities and native save/replay match, and raw-derived
accounting exposes the AT change and remaining wall tail without discarded
attempts. The comparison can establish a bounded improvement or no
improvement; consistent 60 Hz and physical input/display remain separate
acceptance targets. Initial QuerySnapshot deep-copy/COW work is measured
and proposed independently, not silently included in the AT change.

### R6. Worker, preseal and acquisition attribution

**Status, 2026-11-05:** implemented and locally qualified in isolated owner
and fixed-SDK receiving lanes. Final gates and all four matched observations
pass. Worker wall data identifies bounded completion waiting and preparation
has producer CPU cost. Complete CPU decomposition and actual GPU duration
remain unavailable; no causal speedup or consistent60Hz claim follows.

Netrender owns the shared same-thread POSIX CPU clock and provider quality
observations; Genet carries intrinsic batch/Core-bound worker reports.
Rootstock owns synchronous preparation, observer/input capture, encoding,
sealing, collection and acquire/clear/compose/present call scopes. Mesquite
retains raw availability and attempts. Simulation authority is unchanged.

Fresh owned Winit Cores enable CPU/quality diagnostics after boot; refusal
logs without failing boot. Borrowed/shared Cores are not reconfigured, so
worker data may be unavailable. Only the additional optional GPU feature
request is default-off (`CAMBIUM_NATIVE_GPU_TIMING=1`); custom options may
already enable timestamps, and actual device features remain authority.
Only the exact SDK-selected netrender-vello provider276b466 joins the source
overlays, preserving quality/error/FIFO behavior and its original submit.

Final source is frozen as r6-03 with eight overlays and sixteen selected
receiving identity replacements, zero new registry/foreign Git/common records
and104 unchanged approved owner inputs. Provider10, shared clock3, Genet55,
Rootstock252 (five unchanged filters), Winit25, Mesquite51 and Mesocosm78
(six ignored), plus explicit staging GPU1 and WorldSave1, pass. Exact clock
and inventory reruns overlap the named suites. The receipt records failed
admission/compile/filter attempts separately from passing qualification.

R5's broad worker-ready marker remains after report assembly/copies at
completion construction, before channel publication. R6's CPU/wall total
ends earlier through the GPU metadata snapshot, excluding report finalization.
Their wall difference bounds bookkeeping and clock overhead, not CPU/GPU
execution. UI CPU clocks stay within one synchronous submit/collect call;
no CPU stamp crosses pending time. Wall-minus-CPU is a residual containing
scheduling, locks and possible driver waits, not pure driver or GPU time.

Both candidates validate266/266 worker wall partitions, but only17/16 full
worker CPU sums. Derived other CPU is rejected249/250 times; UI unscoped CPU
remainders are also frequently rejected. Whole observed intervals and named
spans remain valid, with raw rejected counters retained and null valid values.
There is no complete UI/worker CPU decomposition. Nested timer overhead is
an explanation to test, not an isolated experimental cause; never clamp it.

The15 slow packets per backend spend about27.4/27.7ms wall in original bounded
Device.poll(Wait) calls but about0.33/0.31ms on the current thread. Mapped
quality inspection costs about33µs CPU. Acquisition averages about10.5ms wall
versus57µs CPU. The sole enumerated Vega56 Metal adapter lacks timestamp
support; GPU duration stays null in every candidate packet. The implemented
existing-encoder bracket excludes direct queue uploads and later presentation;
its positive feature branch is unqualified on this hardware. No wgpu/core
Metal-observer cohort expansion or canary substitute is adopted.

Fresh paired UI medians remain about16.2ms. Recorded sums change−0.15% retained
and−1.95% components in this single ABBA observation, with material desktop
load and an isolated baseline tail. This is not a causal optimization result.
All four native saves and all eight scene crops equal frozen R5. Retained
full images are exact; components differ at two one-channel levels outside
the crop. C4 has no physical native bodies and cannot qualify embodied ecology.

The next measured CPU target is the producer/Isometer adapter: maximum
producer spans reach17.976/18.744ms CPU but include target setup, configuration,
draw/submit and query capture. R7's completed attribution below separates those calls; its measured
query/draw result guides a follow-on after current-main integration. Consistent60Hz, complete
CPU accounting, actual GPU observability, R4 receivers and broader physical/
hardware/platform qualification remain open. Preserve all rules/quality/loss
checks and the immutable R5/R6 evidence. Detailed gates, scope and raw results
are in the linked qualification receipt.

### R7. Producer resource, draw and query attribution

**Status, 2026-11-05:** implemented and locally qualified on the fixed R6
supplier cohort. New diagnostic/resource controls and four serial matched
observations pass. The direct call data separates drawing, fresh query
capture, explicit setup and retained cloning. No COW or pooling is
implemented; broader CPU remainders, GPU duration and physical 60Hz remain
unqualified. R8 current-main integration is active separately.

Mesocosm owns product configuration and candidate interpretation; Isometer
owns generic scene/resource/query operations. Optional Mesquite
`Product::cost_details` transports product-local cumulative `main_scene`/
`previews` data into raw `Sample.product_work`, with a default of `None`
and the original observation wrapper preserved. No Rootstock registry or
producer API expands. Stable cost epochs, immutable attempt snapshots and
the phase predecessor make repeated payload, replacement, reset and
disappearance explicit. Missing observations never become measured zero.

Native candidate B retains a distinct writable target while accepted A keeps
its displayed image, site/camera/input and immutable query facts. Fresh
capture and a retained query `Arc` clone are different operations. Counters
describe attempts, including refusals/discards, and do not publish B early.
Instrumentation calls each existing operation once without an additional
draw, submission, capture or allocation request. Dense-payload byte counts
do not claim total heap/GPU allocation or bandwidth.

The fixture has one registered viewport producer containing ten voxel
bodies and no physical native members. Both candidates show 13 phase fresh
draws/captures, 253 retained clones and 266 publications. There are 13 phase
target textures/26 views and zero phase Section creations; lifetime has one
Section and 19 fresh draws/targets, including six calls outside the phases.
Do not confuse these13 fresh draws with 15 provider slow-worker packets.

Fresh query capture averages about 3ms CPU and drawing about 7ms. Explicit
setup totals about 2.3ms across 26 phase calls, about 0.18ms per fresh attempt;
retained query clones average less than 1µs. Thirteen snapshots copy roughly
109MB of dense terrain payload despite zero phase ground rebinds. The latter
does not prove map immutability, and drawing still contains internal resource
work. New direct stages have no unavailable/rejected CPU calls, while the
broader UI/worker derived CPU decomposition remains materially incomplete.

Mesquite 54, Isometer 73, Mesocosm 82 (six ignored), explicit native staging 1
and WorldSave 1 pass. Every serial run exits 0; native saves and all scene
crops match frozen R6. External compiler/desktop/GPU contention is explicitly
admitted for this performance launch and retained in the receipt. Recorded
paired UI sums do not improve, and no causal elapsed-time or consistent 60Hz
claim follows. Actual GPU duration remains unavailable on the Vega56 adapter.

The measured CPU target favors immutable query sharing over target pooling,
after R8 integration. Any follow-on must admit actual map/revision changes
before mutable COW access, preserve per-frame camera/stamp facts and protect
accepted/in-flight ownership. Merely cloning the current `Arc` or calling
`make_mut` before every refresh does not remove initial capture cost. Deeper
draw attribution is still needed before selecting its resource changes.
No such optimization is part of R7. Detailed timings, counts, identities and
qualification limits live once in the linked receipt.

### R8. Integrate the qualified scene/raster source onto main

**Status, 2026-11-05:** complete. Mere `ad6aaf60` and Isocosm
`da56ce39` are verified on their remote main branches after separate
owner and receiving gates. The shared owner check covers 366 native tests,
seven changed examples and both standalone wasm roots, including the
affected current-main Graphshell recheck. Isocosm consumes the qualified
Mere code commit `ad6aaf60`. The documentation follow-up is based on
`73dfe50a`.

The current Isocosm cohort passes 298 native tests and locked release
checks for VTT, Eponym, Wing and Parry. Four headed C4 runs and four native
replays agree on saved worlds. All eight decoded scene crops match the
fresh within-cohort oracle.

The native cut includes the restored 14-line `Runtime::synchronize_view`
method, which adopts the accepted product section before presentation.
The port contains 61 Rust paths. Resolver receipts identify the later
consumer lock generations separately from the frozen C4 inputs.

Physical refresh, GPU duration, wider platform qualification and
ecological persistence require separate evidence. Explicit VTT and Eponym
staging belongs to R4. C5 Didgeridoo remains conditional. The next planned experiment shares fresh query data and
attributes more of the draw cost before choosing further resource changes.

Preserve dirty canonical checkouts and every qualified baseline. Inventory
exact source paths, current fetched main revisions, canonical working-byte
overlaps and dependency declarations before mutation. Use clean isolated
integration worktrees at those exact revisions, then reconcile task hunks
against current refactors; do not replace whole older files or broadly stage
canonical dirties. Rootstock public-hook compatibility updates belong to the
same source closure. Keep unrelated canonical ecology work outside it.

Provider diagnostics must follow the owned dependency line actually selected
by Netrender, preserving its changes. The qualified provider revision and
current dependency branch are distinct from the unrelated upstream-style
fork main. Record that choice explicitly; do not merge old research source
into the unrelated line just because the user's goal names main.

Integrate prerequisites in source order: selected provider hooks, Netrender
content/policy/CPU support, Genet admission/lifecycle/attribution, Mere host/AT/
producer/telemetry receiving, then committed-source Cosmograph-host APIs and
Isometer/Mesocosm consumers. The C1 `NativeRevision`/replication prerequisite
must be ported with current-main behavior preserved and full Cosmograph-host
unit gates; it does not authorize biological changes. Use bounded
exact dependency commits, source-aware lock audits and compatibility fixes.
Requalify owner and receiving/native/save behavior for the actual integrated
source/lock tree. Frozen R7 results remain evidence for their own cohort,
not proof of the new-main port or its performance.

Done when: scoped source commits and appropriate gates are recorded, main
integration/publication state is verified against fresh ancestry, consumer
pins and remaining qualification are explicit, and unrelated canonical bytes
remain protected. Direct-main integration may run from isolated validated
worktrees while dirty canonical HEADs stay preserved. No forced fast-forward
of those checkouts, broad staging, destructive reset or self-attribution is
part of the plan. Report local source, remote main and consuming-pin adoption
separately. R4 and conditional C5 remain unchanged.

## Findings

- 2026-11-04: native tile dirtiness proves geometry-cache invalidation, not
  final image equality. A same-size external texture update can keep tile
  hashes while changing pixels. Complete resource binding is required.
- 2026-11-04: at the attribution cut, the pinned and then-current Rootstock
  native presenter synchronously checked each new raster. The Genet
  batch/worker API already existed in the selected pin; R3 receives that
  capability rather than introducing a renderer engine.
- 2026-11-04: the initial native host and owned WgpuDriver had separate output
  lifetimes. R2 shares one retention mechanism; the locally qualified R3
  lifecycle consolidates pending publication and driver capture semantics.

- 2026-11-04: `presentation/native/wake.rs`, Winit's `redraw_turn.rs` and
  Genet's `native_frame/availability.rs` separate application demand,
  completion collection and shared-Core availability. The failed first
  headed receiving run exhausted its cost ledger through redraw polling;
  the corrected bounded runs retain all attempts and complete.
- 2026-11-04: paired per-picture UI accounting must include submission and
  collection accessibility work. The additional submission pass costs about
  1.6 ms mean per picture in the bounded fixture. Separating accepted AT
  publication from queued action draining is the next bounded component
  experiment. R5a now qualifies that separation with bootstrap/action/frozen-
  geometry controls preserved; its component saving is not an overall speedup.
- 2026-11-04: native preparation-to-publication wall p99 remains about 50 ms.
  R5b's new-only markers place the long interval in checked worker
  qualification rather than notification delivery in this fixture; R6 retains
  deeper worker/preseal/acquisition attribution. This is not GPU duration. Isometer's initial `QuerySnapshot` capture still
  deep-copies body/map data; measure that path before an owner-level
  immutable copy-on-write resource seam. Neither result selects an ECS
  migration or changes authoritative native storage.

- 2026-11-04: R6 planning verified that Genet's existing worker combines
  polling, error scopes and quality observation, while Netrender's named
  renderer spans use `Instant`. The shared CPU seam must remain same-thread
  and call-scoped. Default host options request no optional GPU feature;
  explicit feature support tests do not prove product-device enablement.
  [Apple's thread-info vocabulary](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/mach/thread_info.h)
  separates runtime fields from CPU percentage, and the pinned
  [wgpu encoder contract](https://docs.rs/wgpu/30.0.1/wgpu/struct.CommandEncoder.html#method.write_timestamp)
  requires operation-specific enabled timestamp features.

- 2026-11-05: `borrowed_section/producer.rs::encode_frame` currently combines
  target creation, configuration, draw/submit and `Accepted::capture`.
  `producer/staging.rs::stage_candidate` separates the displayed target from
  the fresh candidate and restores accepted facts after encode. Attribution
  must distinguish that deliberate separation from ordinary size-based reuse.

- 2026-11-05: R7 direct call observations isolate substantial fresh query
  capture beside dominant draw CPU; explicit setup is smaller and retained
  query cloning is cheap. Zero ground rebind is not map-immutability proof.
  The bounded result favors a future query-sharing experiment over target
  pooling, after R8 integration; drawing still contains internal resources.
- 2026-11-05: current-main source refactors and committed-source C1 host APIs
  are prerequisites of receiving. Candidate path inventory is provisional;
  frozen SDK acceptance cannot substitute for integrated dependency/type gates.

## Progress

- 2026-11-04: preserved canonical diffs, traced the approximately 20 ms residual,
  retained a fresh native sample and began the owner-level content witness and
  bounded qualified-output retention implementation in isolated worktrees.

- 2026-11-04: the expected pre-fix GPU failure is retained. Rootstock passes
  193 tests (three existing ignores), paint API 14, native admission three,
  Mesquite 42 and the focused renderer policy one. All 32 optimized
  unchanged-output controls retain the checked texture. The actual-pixel,
  native-batch, device-destruction and surface-unavailable controls pass;
  all four R2 headed trials then pass with byte-identical saved worlds.
  Unchanged work improves while that cut's changed-picture tails justify
  the subsequent R3 lifecycle. Heap, GPU duration and physical input/display
  remain unmeasured.

- 2026-11-04: R3 began with the initial source-only cut and preservation of
  eight owner/receiving and canonical checkouts. Producer staging, immutable
  input geometry, Mesocosm accepted facts and the host lifecycle have distinct
  owners. Unavailable surfaces release the Core and refuse bound captures.
  The final audit records canonical Netrender's separate clean advance;
  neither the fixed SDK nor isolated owner inputs adopt it.

- 2026-11-04: the genuine frozen R2 receiving control fails; the same
  public-API control passes against R3 with acceptance pending. Integration
  corrected a module-name collision. An interrupted debug suite is retained
  as partial output; optimized tests and explicit native controls provide
  the subsequent passing evidence. Completion notices are distinct from
  application wakes; accepted scale and coalesced layout demand are retained.

- 2026-11-04: pre-scheduling owner/product gates pass, but the first headed
  receiving run fails its cost ledger through pending redraw polling despite
  exact native save and scene viewport. Completion-driven collection,
  coalesced demand, per-window draining and one-shot Core availability provide
  the subsequently qualified correction. Unavailable surfaces create no
  retry loop; reconfiguration may owe one retry.

- 2026-11-04: R3 is locally qualified after the scheduling/availability
  correction. Final owner/product, accessibility and save/replay controls
  pass, and all four matched headed before/after workloads complete. Both
  scene backends retain exact native saves and scene crops; full captures
  differ at one UI pixel each. Corrected paired UI-work tails improve but
  remain above the target. The receipt names all executed generations and
  limits; R4 and consistent 60 Hz remain open.

- 2026-11-04: opened R5a's bounded accessibility publication/action-drain
  separation and R5b's native wall-tail attribution with a fresh matched
  retained/component rerun. Planning verified the combined adapter seam and
  preserved root-only bootstrap/action routing requirements. R2/R3 remain
  locally qualified. That planning cut preceded the implemented source and
  executed results recorded below.

- 2026-11-04: R5a/R5b are locally qualified after genuine before controls,
  final owner/receiving gates and four same-session ABBA runs. Accepted AT
  actions drain before new frame preparation; the native timing partition
  remains bound to the accepted packet. AT's profiled portion decreases,
  while acquisition waiting offsets the saving; no overall profiled-duration
  improvement is claimed. R6, physical/hardware qualification and R4 remain
  open. Native saves and scene pixels retain their declared acceptance.

- 2026-11-04: opened R6 diagnostics against frozen R5; planning preceded
  implementation and qualification. The exact provider extension preserved
  the SDK revision and other suppliers.
- 2026-11-05: final r6-03 gates and four ABBA observations pass. The original
  broad worker-ready boundary is restored, while new CPU scope is narrower.
  All worker walls validate; most derived CPU remainders are rejected and
  retained, so complete CPU decomposition remains unavailable. GPU timestamp
  capability is absent on this adapter. Wait/acquisition residue is off-CPU;
  producer CPU cost motivates finer R7 attribution before optimization.

- 2026-11-05: opened R7 producer/resource/query attribution against frozen
  R6. Preserved the nine-repository initial cut and planned execution/status
  controls before any COW/pool decision. Gates and measurements were pending at that planning cut.

- 2026-11-05: main integration is authorized after R7 qualification. R8
  inventories the bounded prerequisite chain and ports it onto exact current
  main targets in clean integration worktrees; canonical dirty checkouts and
  frozen R7 evidence remain protected. New-main qualification is separate.

- 2026-11-05: final R7 diagnostic/resource/save gates and all four serial
  observations pass under explicitly admitted external build contention.
  Native saves and scene crops remain exact; no elapsed-time speedup, full
  CPU decomposition or physical 60 Hz is established. No COW/pool implemented.
- 2026-11-05: R8 source ports are staged in clean current-main trees, with
  exact refactor/provenance patches and canonical preservation. Committed-
  source host prerequisites, dependency locks and integration gates remain
  active; source, remote main and consuming pins will be reported separately.

- 2026-11-05: the current-main Mere owner cut on 81e passes 366 native tests,
  seven exact changed-example checks and both actual standalone wasm roots,
  with original Smoke lock absence restored. The later nonoverlapping main
  advance has a separately recorded affected Graphshell check. The owner
  publication is verified at `ad6aaf60`.

- 2026-11-05: Isocosm `da56ce39` is verified on remote main after 298
  native tests, four locked consumer builds, four C4 runs and replays,
  and eight decoded scene crop checks. The shared and product receipts
  bind the separate published inputs and observed results. R8 is complete.

## Frozen R2–R7 working source

These isolated lanes and the copied, hashed Mesocosm SDK record the frozen
R2–R7 experiments. Those experimental cuts supplied local qualification;
they did not constitute supplier publication or current-main adoption. Their
source and measurement limits remain fixed. R8 uses the separate current-main
integration trees and exact published prerequisites described above; its
executed owner qualification is recorded separately in the receipt.

| Owner | Local source lane |
| --- | --- |
| Mere / Rootstock and Mesquite | `Code/worktrees/mere-qualified-output` |
| Netrender / ordinary paint and policy | `Code/worktrees/netrender-qualified-output` |
| Exact selected Vello quality provider | `Code/worktrees/vello-native-worker-attribution` |
| Genet / native reuse, batch and availability admission | `Code/worktrees/genet-qualified-output` |
| Fixed SDK Mesocosm receiving | `Code/worktrees/isocosm-scene-composition-validation` |

## Receipts

[The qualification receipt](../testing/receipts/2026-11-04_qualified_raster_retention.md)
records exact artifacts, controls, source overlays and limitations. Raw evidence
is retained under `Code/output/mesocosm-scene-composition/2026-11-04/raster-attribution/` and the sibling `native-frame-receiving/`, `native-a11y-tails/` and
`native-worker-attribution/` and `producer-preparation/` directories.
