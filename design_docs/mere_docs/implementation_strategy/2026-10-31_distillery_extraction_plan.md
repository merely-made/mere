# Distillery extraction plan

**Date:** 2026-10-31
**Status (2026-11-03):** P1–P3 landed for Distillery, compute mesh, probes and
inference patches. The renamed repository preserves the receiving history;
all seven actual consumer repositories pass their locked receiving checks.
The primary `distillery-receive` checkout is preserved. Athanor and Alembic
remain held under D16, outside this completed cutover.
**Receipts:** donor/source and manifest comparison, standalone tests and feature
checks, Cargo source identities, every actual consumer check, and final pushed
commits. Retain at most 100 MB in `Code/testing/esp/distillery-extraction-20261103`;
use the stable ESP and consumer targets, with no isolated Cargo home.
**Scope:** Distillery, its job and lease layer `mere-mesh`, and the workshop halves of
Alembic and Athanor leave Mere for the ESP repository, which is renamed `distillery`.
Distillery becomes a producer: it keeps its own surfaces and speaks only the published
contract crates. The graph halves stay in Mere as `pandect` modules. Every consumer of
`esp.git` moves to `distillery.git` in the same pass.

## Origin

Mark, 2026-10-31: "extract distillery from mere into ESP, as planned. Remove old
directories. Rename the esp repo distillery." No dated plan recorded the move; the
nearest record is isocosm case 1044 ("esp, Distillery and the other ports being
extracted from mere"). The same session refined stack.1 (house-style D12): extraction
waits for a planned second host, not an arrived one.

## Decisions

- **D1. The move (Mark, 2026-10-31):** "commit and extract distillery from mere into ESP,
  as planned. Remove old directories. Rename the esp repo distillery."

  Links: draws on house-style D12 · summary Distillery leaves Mere for the ESP repository, which becomes `distillery`; donor directories are removed.

- **D2. The back edge (Mark, 2026-10-31):** "Cut first", over Knot-style lockstep.
  Distillery imported eleven Mere crates; the repository it joins stays free of
  `mere.git`, as ESP is today.

  Links: summary Cut Distillery's Mere imports before moving it; the receiving repository takes no mere.git dependency.

- **D3. Contents (Mark, 2026-10-31):** "Port + alembic", over the port alone.

  Links: summary ports/distillery and crates/distillery/alembic move together.

- **D4. Consumers (Mark, 2026-10-31):** "Sweep all six now", over relying on GitHub's
  redirect. Mere, cleromancy, eidetic, knot-editor, mer3ly and turnstone name `esp.git`;
  Cargo treats the old and new URLs as different sources.

  Links: summary Every esp.git consumer moves to distillery.git in this pass, one source per graph.

- **D5. mere-mesh (Mark, 2026-10-31):** "Moves with Distillery", over staying in Mere or
  going to Armillary. It is Distillery's job and lease layer (15,107 lines, used in 15
  Distillery files) and depends on nothing in Mere; Djinn and Graphshell take it from
  `distillery.git`.

  Links: summary mere-mesh moves into the distillery repository.

- **D6. Djinn holds no endpoints (Mark, 2026-10-31):** "I'm not sure djinn should be an
  endpoint like that. Isn't it execution? Not state? It would be like luggage becoming
  host". Then "Moot-style port" named `mere-distillery`; superseded by D7.

  Links: summary Djinn supervises and serves no endpoint; the adapter-port answer is superseded by D7.

- **D7. Distillery is a producer (Mark, 2026-10-31):** "Why don't we just make Distillery
  not an app itself, but a surface producer for mere, right? And then GraphShell is
  actually a useful thing, because then it's like, it is the way, it is the host." Then
  "Producer", over the adapter port. Chronicle, `inference.ask` and the installed
  surface stay in Distillery, which takes the contract crates from crates.io; Mere
  patches them back to its paths.

  Links: amends distillery-extraction D6 · summary Distillery keeps its surfaces as a producer over the published contract crates; no mere-distillery port.

- **D8. Leaf and branch (Mark, 2026-10-31):** "Both must hold", over owning a graph alone
  or hosting alone, after seeing every repository classified. A branch owns a graph (links
  `mere`, `mere-kernel` or `pandect` for its own logic) and hosts others' surfaces; anything
  else is a leaf that produces surfaces, with or without a standalone frame. Refines
  stack.3.

  Links: amends stack.3 · summary An app is a branch only if it owns a graph and hosts others; otherwise it is a producer leaf.

- **D9. Any branch hosts (Mark, 2026-10-31):** "Any branch hosts", over Graphshell as the
  only host. Graphshell is the reference host that loads whatever is installed; Turnstone
  stays the flagship. Refines the 2026-08-22 suite census without reversing it.

  Links: amends stack.3 · summary Every branch can host producers; Graphshell is the reference host, Turnstone the flagship.

- **D10. Order (Mark, 2026-10-31):** "Extract, then survey". The other ports are classified
  against D8 in a dated brief after this extraction.

  Links: summary Finish Distillery first; survey the other ports against D8 after.

- **D11. Djinn is execution only (Mark, 2026-10-31):** "Execution only", over Djinn owning
  the resident's meres. This reopens the 2026-08-22 census line giving Djinn the store
  lifetimes. Recorded as an open fork, not acted on here.

  Links: summary Djinn should hold no meres; who owns store lifetimes is open.

- **D12. Graph halves (Mark, 2026-10-31):** "Modules in pandect", over a slim
  `mere-athanor` left in Mere. Alembic's `memory_levels` becomes `pandect::memory_levels`
  (pandect already depends on it); Athanor's `furnace` becomes a `pandect` module behind a
  feature, which Turnstone's recycle bin takes.

  Links: draws on house-style D2 · summary Graph halves become pandect modules; mere-alembic and mere-athanor leave Mere whole.

- **D13. Atomic file replacement (Mark, 2026-10-31):** "Move to muniment", over copying
  it into Distillery. `pandect::atomic_file` goes down into `muniment` (Eidetic), which
  pandect and Distillery both depend on.

  Links: draws on house-style D2 · summary Atomic file replacement lives in muniment; pandect and Distillery share it.

- **D14. Gate G1 (Mark, 2026-10-31):** "Publish both", over gating Distillery's surface.
  cambium 0.6.0 and sprigging 0.3.0 go to crates.io after a dry run, with the dependency
  chain reported first.

  Links: summary Publish cambium 0.6.0 and sprigging 0.3.0 so Distillery's surface builds standalone.

- **D15. Repairs to D8 (Mark, 2026-10-31):** asked for "a lil consistency critique first",
  then "Quite odd contradiction!", then all four repairs: "Frame ≠ host", "Endpoints are
  producer-side", "Graph core is contract tier", "Runners are a third kind". Before them:
  Cleromancy, Woodshed, Signalman and Knot "could own graphs but not be hosts", Tabard
  "doesn't own a graph"; "They all have widget/utility potential, too. But that doesn't
  make them hosts outright"; "i still would like to be able to build and install a
  'woodshed' app". Hosting is loading producers at runtime; an installable app is a frame
  with its producers fixed at build; an endpoint is what a producer exposes; Djinn, Murm,
  Luggage and Armillary are runners. Refines stack.3.

  Links: amends stack.3, distillery-extraction D8 · summary Producers, hosts and runners; frames are not hosts; endpoints are producer-side; the graph core is contract tier.

- **D16. Athanor and Alembic wait for their lane (Mark, 2026-10-31):** "Communicate with the
  active lane?", then "Plan note + prompt". Put to Mark on finding, after merging 201
  upstream commits, that the Codex lane's Alembic workshop (`codex/alembic-workshop-20261025`,
  merged 2026-10-29) added `workshop_destination` and `workshop_sealed_destination`, which
  write into a mere graph, and that its plan says "there is no source-home move or new crate".
  Distillery and `mere-mesh` move now; Athanor and Alembic stay in Mere until the lane answers
  the two questions recorded in the Alembic plan's Open questions. D12's pandect moves stand.

  Links: amends distillery-extraction D3 · summary Distillery and mere-mesh move now; Athanor and Alembic wait for the Alembic lane's answer.

- **D17. Republish the drifted contract crates (Mark, 2026-10-31):** "Bump and publish all
  11", over parking P2 or pinning `mere.git`. Put to Mark on finding that ten of the eleven
  contract crates in Distillery's graph no longer matched crates.io under their versions
  (`graphshell-endpoint` 0.0.2 there lacks `projection`). Bumped in Mere `e55850a1`.

  Links: summary Bump and republish the contract crates whose source drifted from crates.io.

- **D18. Slim Djinn before publishing it (Mark, 2026-10-31):** "Publish the djinn chain, but
  also, aren't we supposed to be moving those services so djinn isn't a god crate and can
  just be execution?", then "Slim, then publish", over publishing Djinn 0.0.3 as it stands.
  `graphshell-endpoint` 0.0.3 needs `djinn` 0.0.3, which links custody and identity services
  (`custody-client`, `profile`: chatelaine, dramatis, personae, insigne, eidetic) and the
  unpublished `peer-facts`. The door stays in Djinn as runner work (capability-door D1, D8);
  custody and identity services move to their owners, and the door's wire types become a
  small contract crate that `graphshell-endpoint` depends on in place of `djinn`. That gets
  its own plan; P2 waits for it.

  Links: amends distillery-extraction D17 · summary Slim Djinn to the door before publishing; producers depend on the door's wire contract, not on Djinn.

- **D19. The store owns its lifetimes (Mark, 2026-11-05):** "The store itself", over a
  long-running resident service that Djinn launches and over each application owning its own
  mere. Opening, durability and closing of every mere belong to the reservoir store (pandect's
  reservoir and the resident library), wherever it is hosted. A Djinn-launched resident and an
  application embedding the resident only execute it. Phones were the deciding case: a device
  that runs no Djinn embeds the resident, so lifetimes cannot belong to a background process
  only some devices run. Closes F1. Woodshed's reopened custody ruling followed the same day:
  its whole domain moves into a Woodshed mere
  (`woodshed/design_docs/2026-10-14_dsp_graphshell_hocket_survey_brief.md`, ruling 6).

  Links: closes distillery-extraction F1 · follows D11 · summary Store lifetimes belong to the reservoir store; Djinn and embedding hosts only execute it.

## Open forks

- **F1 (D11).** Closed by D19 (2026-11-05): the reservoir store owns its lifetimes.
- **F2.** Closed by D14.
- **F4 (D18).** Answered by Armillary's Djinn slimming plan
  (`armillary/design_docs/armillary_docs/implementation_strategy/2026-10-31_djinn_slimming_plan.md`):
  the custody half becomes a Dramatis crate (`armillary:djinn-slimming` D2), producers depend
  on a door contract crate in Armillary (D3), and the cut waits for capability-door I1 (D4).
  Closed 2026-11-02: the slimming's S4 and S5 landed. `graphshell-endpoint` 0.0.3 depends
  on `ostiary` and `castellan-client`, not `djinn`, and is published with
  `script-wire`, `script-witgen`, `chirograph` 0.0.4, `workbench`, `incipit`, `scenograph`,
  `scenomise`, `mere-surface-api` and `cambium` 0.6.1. Cambium waited on `genet-events`
  (Genet commit `43d13ce56` made it a dependency); Genet released it as 0.0.1 on 2026-11-02
  at Mark's instruction, with the API still unsettled.
- **F3 (D15).** Publishing the graph core (kernel and the graph persistence in pandect) as
  contract tier is its own plan. Until then Cleromancy, Knot and Woodshed compile in
  lockstep with Mere; this pass moves Cleromancy off `mere-alembic` and no further.

## Phases

### P1. Cut the Mere imports, in Mere

Move `alembic::memory_levels` into `pandect` and `athanor::furnace` into a `pandect`
feature; inline the one `pandect::write_bytes_with_backup` call. Drop the `recall` and
`furnace` features. Point Turnstone's recycle bin at the pandect feature.

*Done when:* `distillery`, `mere-athanor`, `mere-alembic` and `mere-mesh` have no normal
dependency on `mere`, `mere-kernel` or `pandect`; their only Mere-hosted dependencies are
the contract crates (`sceno`, `scenotime`, `chirograph`, `graphshell-endpoint`,
`mere-surface-api`, `scenograph`, `scenomise`, `incipit`, `cambium`); Djinn, Graphshell
and pandect tests pass.

### P2. Receive in the renamed repository

Filter Mere's history to the moved paths and merge it into ESP, laid out as
`crates/{distillery,mesh}` plus the probe workspaces (Athanor and Alembic wait, D16). Contract crates come
from crates.io (G1 for cambium and sprigging).

*Done when:* moved sources match the donor at the P1 commit, with recorded manifest
exceptions; the workspace builds and its tests pass standalone, with no `mere.git` source
in its lock; history is preserved.

### P3. Rename and cut over

Rename `merely-made/esp` to `merely-made/distillery`. Mere takes `distillery` and `mere-mesh` from `distillery.git` at one revision
(Athanor and Alembic remain in Mere under D16) and patches the
contract crates from crates.io to its paths. Sweep the six consumers named in D4. Remove
the donor directories.

*Done when:* no active manifest in the six named repositories or the additional
Isocosm consumer names `esp.git` or a removed path; each
lock resolves one source per package; Mere's touched crates and each consumer's check
pass; changes are committed in each repository.

## Prompt for the Alembic lane

> Read `design_docs/mere_docs/implementation_strategy/2026-10-31_distillery_extraction_plan.md`
> (D7, D12, D15, D16) and the last item under Open questions in
> `2026-10-28_alembic_implementation_plan.md`. Answer its two questions in that plan: whether
> `workshop_destination` and `workshop_sealed_destination` can become a `pandect` feature, and
> whether the graph-free workshop core leaves Mere with Distillery after W3. Note that
> `alembic::memory_levels` is now `pandect::memory_levels` and Athanor's furnace is
> `pandect::furnace`. Stop and show Mark the answer before moving any code.

## Findings

- 2026-10-31: Distillery's Mere imports are concentrated in `chronicle.rs`,
  `chronicle_recipe.rs`, `inference.rs` (the `InferenceAskEndpoint`, lines 1669-2028),
  `surface.rs` and one call in `installed.rs`, all contract crates except that
  `pandect` call. `mere-mesh` is imported as `mesh` across 15 files.
- 2026-10-31: `pandect` depends on `alembic` with `recall`
  (`crates/system/pandect/Cargo.toml:29`); Turnstone depends on `mere-athanor` for its
  recycle bin (`turnstone/Cargo.toml:34`, `src/recycle.rs`).
- 2026-10-31: crates.io holds the local versions of sceno 0.0.6, scenotime 0.0.4,
  chirograph 0.0.2, graphshell-endpoint 0.0.2, mere-surface-api 0.1.0, scenograph 0.0.5,
  scenomise 0.0.6, incipit 0.2.0, hest 0.2.0, workbench 0.1.0. Equal version numbers do
  not prove equal source; P2's standalone build is the check.

## Progress

- 2026-10-31: F4 answered in Armillary's Djinn slimming plan (D1 to D4). The library
  slims first; `mere-resident`'s lanes move to their producers in the plan after it.
- 2026-10-31: assessment and rulings D1 to D12; no code moved.
- 2026-10-31: muniment took `atomic_file` (Eidetic `5b3752f1`, then `613799af` carrying
  upstream's `replace_bytes_if_unchanged`); sprigging 0.3.0 and cambium 0.6.0 published to
  crates.io; Cleromancy reads `pandect::memory_levels` (`8cc4f83`). Mere's branch
  `distillery-extraction` merged 201 upstream commits (Djinn moved to Armillary; the
  remaining resident is `ports/mere-resident`).
- 2026-10-31: P2 receiving. `git filter-repo` with default pruning silently dropped merge
  content (the received `wire.rs` lacked upstream's strict decode); with
  `--prune-degenerate never` the 964-commit history matches all 268 donor blobs at
  `0874522c`. Merged into ESP branch `distillery-receive` (local, unpushed) with a generated
  workspace. Its first resolve failed on `graphshell-endpoint` 0.0.2's missing `projection`,
  which found the drift behind D17.
- 2026-10-31: D17 wave. Mere `e55850a1` bumped the eleven; Knot `81b2c209` took them (its
  0.0.x requirements had pinned the old versions, so Mere's patch stopped applying and the
  lock carried two copies until Knot moved); Mere `7300f91f` locked them with one copy each;
  Cleromancy `f86874eb` follows. Published: sceno 0.0.7, scenotime 0.0.5, hest 0.2.1,
  chirograph 0.0.3. Stopped at `graphshell-endpoint` 0.0.3 (D18). Not yet published:
  graphshell-endpoint, scenograph, workbench, scenomise, mere-surface-api, cambium 0.6.1,
  incipit.
- 2026-11-02: F4 closed by the Djinn slimming's S5 (Armillary plan). Published the rest of
  D17's chain: script-wire 0.0.1, script-witgen 0.0.1 (their publish guard removed),
  chirograph 0.0.4 (Knot's requirement moved first, `62aff7b`; the wit feature needs
  `CarrierError::Lapsed`), graphshell-endpoint 0.0.3 (its `wit/world.wit` is a copy of
  `crates/script/wit/world.wit` that `tests/wit_copy.rs` holds equal), workbench 0.1.1,
  incipit 0.2.1, scenograph 0.0.6, scenomise 0.0.7, mere-surface-api 0.1.1. `dramatis` and
  `personae` rows follow Dramatis's 0.0.4 and 0.3.1. `genet-events` 0.0.1 (Genet `ad3627231`)
  and then `cambium` 0.6.1 followed (Mere `e14dd68cf`): D17's chain is published.
- 2026-10-31: P1 in part. `alembic::memory_levels` is `pandect::memory_levels`;
  `athanor::furnace` is `pandect::furnace` (feature `furnace`); Alembic and Athanor are
  workshop-only. Cleromancy reads `pandect::memory_levels` (local, verified through a
  `--config` patch). pandect's 21 moved and dependent tests and Alembic/Athanor's 23
  pass.

- 2026-11-03: P2 resumes. Dramatis now owns the reusable resident identity service;
  Distillery receiving follows the current cleanup direction. The original filtered
  history is merged with current ESP main. Standalone default tests, actor tests,
  and all-feature/all-target compilation pass. Probe intake also repairs old paths
  and updates unconstrained generation requests to the current ESP API. The
  browser probe owns its module constructor guard rather than depending on a
  whole unpublished Cambium host for that helper. Cubek's carried browser repair
  follows the probes; its historical receipts are retained unchanged.

- 2026-11-03: P2 landed in the renamed Distillery repository. The source cut
  removes the two Mere workspace members, their probes and inference patches;
  graph/host composition uses external `distillery` and `mere-mesh` dependencies.
  Mere patches published producer contracts to its own paths so runtime type
  identities stay singular. A separate, previously broken lexical wasm probe
  moves beside its codec and again agrees with the native pinned digest.
  Isocosm is an additional actual ESP consumer beyond D4's original six. Its
  independent roots are included in the receiving sweep. The original ESP
  receiving branch remains untouched; a collision worktree holds main.

- 2026-11-03: P3 receiving checks pass in Mere, Eidetic, Knot, Cleromancy,
  Isocosm, Mer3ly and Turnstone, including independent standalone roots.
  Turnstone's graph retirement calls now use `pandect::furnace`; custody and
  door callers use Dramatis and Ostiary directly. Its two recycle tests pass.
  Every active tracked Cargo manifest in those repositories and Distillery
  is free of the old ESP URL. Exact pushed consumer revisions and qualification
  limits are in the linked receipts. Further p2panda ownership work remains
  in the family promotion plan, including Stickleback's networking split.
