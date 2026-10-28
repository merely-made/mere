# Application-defined contextual navigation

**Status (2026-10-24): shared navigation and both portable applications published
and qualified on Windows, including independent Knot pane bookmarks.** Woodshed and Knot receive the
generic Cambium contract from the Turnstone GUI lane. Its
minimal browser compatibility migration is also received here; reading chrome
remains owned by that lane.

## Decisions

Mark requested prototypes in Woodshed and Knot using shared Cambium pieces,
preserving the context the person came from and keeping navigation separate
from playback and edits. Scope definitions and parent destinations belong to
each application. Nested or skipped scopes are valid.

## Phases and done-conditions

1. **Shared contract intake.** Receive the exact generic `ScopeNavigation<P, A>`
   source and generic controls. Applications supply identity, admission and
   restoration matching. Done when the shared tests pass and neither adapter
   requires a fixed Document/Neighborhood/Graph/Overmap ladder.
2. **Woodshed receiving.** Overview, Set and Card occurrence routes use shared
   history and controls. Done when two Sets containing the same musical material
   retain distinct entry routes, viewport/selection restoration leaves content
   edits intact, and navigation preserves practice/rehearsal ownership and state.
3. **Knot receiving.** Site, document and recursively nested heading routes use
   the same shared history. Done when skipped heading levels work, history
   restores live editor context without reloading source or edit history, and
   source changes refuse stale heading destinations before moving history.
4. **Qualification.** Compile both native app targets and run focused boundary
   regressions. Native visual interaction is a separate gate; automated source
   checks do not imply a headed or physical playback receipt. Record exact
   shared source and receiving locks, preserving the portable lockfiles.

## Findings

- 2026-10-23: Cambium's original `scope_navigation.rs` embeds browser names and
  scope ancestry. The GUI lane supplies generic place/anchor history while moving
  browser policy into its own adapter. Frozen handoff commit
  `7cb8e4e08d482c59b906b8597c8a098df4a0d6c5` was recovered directly from chat
  tool output and verified against SHA-256
  `4052594473f0cd1a936345b060b12b8582a76b7bb579d7ff8cbbb0a806f8e862`.
- 2026-10-23: The shared trail assumes its current location is editable.
  `scope_trail_with_location_edit` lets applications with no location editor
  render the current crumb as text while leaving ancestors actionable.
- 2026-10-23: Woodshed Set activation stops arrangement practice, and graph
  selection uses the rehearsal cursor. The prototype must separate presentation
  selection and playback ownership rather than repurpose those operations.
- 2026-10-23: Knot outline items use source byte spans. Restoration must validate
  the live source basis before applying a remembered heading or selection.

## Progress

- 2026-10-23: Received the shared source, added optional location editing, and
  resolved both app dependency graphs against local Mere without changing their
  portable locks. Application adapters and qualification are in progress.
- 2026-10-23: Received browser compatibility commit
  `6a49cf419d6cf03683747ae31a837192b473000f`, verified its patch SHA-256
  `038ca3659598eb036ae3fcfb6a541e7e95439f2a3d189c2b8c2afaa39f827bbd`, and applied
  it after a clean `git apply --check`. Browser scope types now live under Mere
  navigation, with Graphshell aliases and caller migration. Existing serialized
  routes remain readable; explicit contextual routes support repeated or skipped
  parents. This receipt does not include the GUI lane's other chrome changes.
- 2026-10-23: The Cambium library suite passed 308 tests. Two additional shared
  control regressions retain keyboard focus/destination hints and verify that a
  location-only trail has actionable ancestors without a false edit control;
  all eight scope-navigation tests pass. Graphshell's 49 `forme_` tests and four
  Mere browser-policy tests pass.
- 2026-10-23: The browser product compiles for `wasm32-unknown-unknown` with
  `product,presentation-webgpu`. The first receiving-lock attempt failed because
  spareval resolved incompatible digest generations; aligning md-5 and sha1 to
  0.10 in the local receiving lock passed. Portable browser locks are unchanged.
- 2026-10-23: Both native binaries build. Woodshed passes seven navigation and
  four Windows path/configuration regressions; Knot passes eleven focused
  behavior, topology and scenario checks. Native scenarios pass with 175
  presented frames in Woodshed and 32 in Knot, with reviewed captures. Woodshed
  preserves the running owner through a first-Card loop; Knot preserves fixture
  bytes through nested headings and temporal history. Native visual review also
  corrected Knot's menu row and breadcrumb sizing.

## Evidence and ownership

Lane brief: `support/family-promotion/contextual-navigation-brief.json`.
Total retained evidence is capped at 10 MB; use compact results and bounded
screenshots. Reuse `C:/t/cargo-targets/{mere,woodshed,knot-editor}`.
App details live in their own dated contextual-navigation plans. No edit history,
playback state or source-content snapshot belongs in the Cambium history.

Qualification record: `support/family-promotion/contextual-navigation/qualification.json`.
These are local receiving builds using ignored configuration/lock files; source
publication and portable consumer repins are not claimed. History is transient.
Knot's scroll restoration, a responsive-size matrix, physical audio quality and
Woodshed persistence remain outside this receipt. The mixed-Set run's existing
Card 2 audio-admission failure and earlier scenario/layout failures are retained.
Reused stable Cargo targets remain owned by their repositories. No isolated
Cargo home or worktree was created; isolated app profiles and receiving locks
are retained locally for reproducible reruns.

## 2026-10-24 continuation

The first prototype omitted persistent app navigation and explicit Knot scroll
bookmarks. Eidetic already supplies the relevant lineage and storage substrate;
the host also already supplies semantic scroll reveal. These are receiving
integration gaps, not requirements for another history store or renderer.

The authorized order is persistent navigation plus Knot scroll first, followed
by the separate native tone-admission repair. The follow-up brief is
`support/family-promotion/contextual-navigation-continuation-brief.json`; retained
evidence, including the prior prototype, is capped at 20 MB. Prior receipts stay
unchanged. New evidence lives under `contextual-navigation-continuation`.

The shared adapter now uses Chartulary/Stemma as visit/cursor/branch authority.
Mutable view bookmarks remain separate from immutable encounter context. Old
linear snapshots migrate explicitly; recognized malformed snapshots are refused
before supplier reconstruction. Render reads project directly from the validated
snapshot. Diverging after Back retains the previous branch instead of silently
truncating it. The previous 128-visit truncation is not retained; pruning and
retention policy remain host-owned, with a 100,000-visit restoration guard.

Woodshed receives the lineage in its existing persisted session, independently
of practice events. Knot receives durable source/site addresses through existing
atomic local preferences and rebinds only admitted open documents. Its semantic
reading anchors use the source revision and source or rendered-text byte, with
heading/block fallbacks for other previews, then queue the existing host reveal
after the matching tile has mounted. Runtime DOM
or document IDs are not durable identity.

The first continuation phase is qualified. Seventeen shared history tests,
nineteen input-routing tests and forty-nine browser compatibility tests pass;
the browser product also checks for wasm after the host scroll-query change.
Woodshed's eight navigation regressions, owner-switch regression, opaque session
roundtrip and fresh-handle encrypted filesystem roundtrip pass. Its native run
preserves the active rehearsal owner across navigation (186 presented frames).
The isolated native vault was locked, so encrypted cold-store tests provide the
save/reopen evidence rather than a live-custody native claim.

Knot's native Source/Preview run passes at deep text positions in a long paragraph
with no heading visible. A fresh process restores the active Preview position;
a second fresh process checks the exact saved text byte (2761). The fixture SHA-256
remains `bc93b9b441a3f14af104071f20635686b5757c7e994e5b84edfd49d6a16a1ec0`.
These runs presented 65, 6 and 12 frames respectively. They qualify the active
view's bookmark, not simultaneous persistence of every idle pane's viewport.
Focused app tests cover source revision refusal, stale same-path reopened tabs,
malformed-data preservation, heading-free Preview anchors and scroll observation.

Only after those gates passed did the separate audio implementation begin.
The audio repair preserves the bounded integer-cycle PCM fast path and adds a
4096-frame periodic table with fixed-point phase/interpolation when that path
cannot meet the existing one-ppm frequency limit. Timing and transport ownership
remain in the output processor. All 174 audio tests pass, including allocation
guards, measured waveform pitch, block invariance, device-frame completion and
silent tail. Native full-Set qualification is recorded in the continuation receipt.

The original qualification above remains an immutable receipt of the earlier
prototype. Its transient-history and missing-scroll limitations are superseded
only by the scoped continuation evidence. App portable locks remain unchanged;
Mere's two portable locks add the direct Cambium `chartulary` and `serde_json`
dependency names. Local receiving overrides remain unpublished.

Final consumer hardening includes the browser's immutable encounter contexts in
its owner-graph admission predicate. The strengthened native compatibility suite
passes 50 tests and the standalone browser product passes its wasm check. A
broader Knot pass also tightened retained outline-row source guards, disambiguated
outline selectors from the shared Down control, and retained source readability
with the compact navigation toolbar at high UI zoom.

The initial receipt inventory omitted some retained local receiving artifacts.
The combined evidence exceeds the continuation's 20 MB cap; the final record
reports the measured total and overrun. Existing receipts and source recovery
backups remain preserved under the workspace hygiene rules.

Final continuation qualification:
`support/family-promotion/contextual-navigation-continuation/qualification.json`.
Knot's full serial library suite passes 357 tests with one ignored diagnostic;
post-snapshot-sharing focused regressions and the rebuilt native return/reopen
checks also pass (64 and 11 frames, exact Preview byte 2761). Woodshed's audio
library passes all 174 tests, and the actual 44.1 kHz native mixed Set reaches
Card 2 Playing and then full completion with the runner stopped. All source
remains local and uncommitted on main. Existing stable targets are reused;
receiving locks, isolated app profiles and source recovery evidence remain
owned by this lane. No isolated Cargo home or worktree was created.

## 2026-10-24 portable publication and platform chrome

The publication intake starts from Mere main `003f9b350`, preserving the primary
checkout's unrelated staged work. The shared source passes locked offline gates:
17 scope-history tests, 19 input-routing tests, seven titlebar integration tests,
two platform-policy unit tests and 50 browser compatibility tests. The standalone
browser product also checks for wasm using its portable lock. That lock now uses
the same Genet revision as the workspace, `2b5c018`, with compatible digest
dependencies. The current Atom representation requires explicit string views in
18 existing test comparisons; those narrow annotations do not change controls.

Native frame policy remains in the winit host. `platform_window_frame()` suggests
an app frame on Windows and a host frame on macOS/Linux; applications can select
`HostOptions.window_frame` explicitly. `caption_controls_for_frame` supplies
custom captions only when the selected frame needs them, retaining macOS native
traffic lights even for a custom content frame. The portable titlebar component
continues to own slots and drag regions rather than product or OS policy.

Woodshed and Knot receive these defaults in their own chrome. Their tool and
location controls remain ordinary application content; a system-owned frame does
not receive an additional application identity/caption row. Windows retains the
shared client caption controls and Snap behavior. Native macOS/Linux presentation
is a separate qualification gate from the pure policy checks.

This source qualification is recorded at
`support/family-promotion/contextual-navigation-publication/shared-source-qualification.json`.
The subsequent custody reopen, publication revisions and app intake are recorded
by the publication lane. Its new-artifact cap is 30 MB and combined retained cap
is 55 MB; earlier receipts remain unchanged.

The shared implementation is published at `3e305d118`; its historical receipt
archive is `7632bbd32`, the Mere pin used by both published app baselines. Woodshed source
is published at `de7b8dc` and its receipt-link follow-up at `91b2dd5`. Its portable
Windows binary passed a fresh isolated unlocked-Keeper seed/reopen pair at
212/171 presented frames: the exact owning Set/Card route, stored music and
view reopened with playback stopped. Back restored the newer Overview at zoom
1.25 and the earlier visit at 1.00. All 174 audio tests pass on the portable graph.

Knot's portable baseline is published at `8ab5268`. The native caption probe
found an eight-pixel overflow that the headless layout did not reproduce;
the application now reserves the caption edge explicitly. Native rectangle
checks and a reviewed final Windows image confirm containment. Twelve focused
chrome tests include narrow viewports and a user-selected full-menu threshold
with the actual Show/Hide following Preview wording. The final image's scenario
setup failed after capture; separate successful zero-capture navigation and
full-menu/Preview runs qualify behavior. Failed receipts remain intact.

The independent-pane intake adds `ScopeNavigation::try_map_bookmarks` at
`9e91c7c95`. Live pane changes map only mutable visit and remembered-place
bookmarks; the complete immutable Stemma encounter lineage, branches and owner
cursor remain unchanged. Full `try_map` remains the admission/migration path.
All 19 shared scope-navigation tests pass with the current Genet `4532fe0`
supplier graph, including exact encounter preservation and failed-map atomicity.
The earlier silent intake attempts were Cargo materializing Genet's 187,911
tracked files. Their interruption records remain preserved; the completed
intake passes both locked online and locked offline gates.

Knot's independent-pane implementation is published at `1766bf2`, using Mere
`9e91c7c95` and Genet `4532fe0`. The final serial library gate passes 362 tests
with one existing diagnostic ignored. Source and Preview retain separate
semantic positions, explicit pane presence, and the last active view. Scrolling
an inactive pane does not move focus. Source and rendered-text digests guard
restoration; old single-position anchors migrate without retaining runtime tile
or DOM identity. Pin/close refreshes update mutable bookmarks, and restoration
preserves foreign pinned readers instead of opening a replacement over them.

The final Windows native binary passes seed, reopen/close, closed-pane restart,
and explicit Preview reopen at 72, 20, 7, and 30 presented frames. Preview byte
2427 restores exactly. The saved Source target 8133 remains inside the live
rendered viewport after the split; its first-visible sample normalizes to 8063
when text wraps differently. The visibility witness reads DOM/layout geometry
independently of saved navigation metadata and does not change text or selection.
Fixture bytes remain unchanged throughout those runs. The reviewed two-pane
image belongs to the immediately preceding binary; the final postimage is
qualified by the no-image native runs. Full tile-layout persistence and native
macOS/Linux visual qualification remain separate gates.

Current publication and retained-environment details:
`support/family-promotion/contextual-navigation-publication/qualification.json`.
The primary checkouts receive the published work with unrelated staged and
working changes preserved. The three temporary publication worktrees are
removed; private fixture/recovery artifacts remain in ignored repository-local
folders and stable Cargo targets remain reusable.

---

*Archived 2026-10-28.* Complete; its residue (what it named as unmet or deferred) now lives in `mere_docs/implementation_strategy/2026-10-28_platform_qualification_plan.md`, which cites this file by line. This copy is history: its status lines, receipts and claims are as they stood, not current.
