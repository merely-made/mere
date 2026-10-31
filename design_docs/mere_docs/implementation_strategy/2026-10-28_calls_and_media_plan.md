# Calls and Media Plan

**Status:** 2026-10-28: cut from `2026-07-27_commons_calls_plan.md` and `2026-05-26_net_media_plan.md`, archived in `archive_docs/2026-10-28/`. The call grammar and state fold (A0) is landed and promoted at `crates/moot/commons`; the admitted control session, audio, real IP call, presence, radio complement and group-call decision (A1-A6) are not started. The media organ is plan-only: no `net-media` crate exists, and WebRTC data-channel carriage already moved to the browser carrier plan.

**For:** the first live product on the Commons spine, a two-person audio call admitted by Notochord inside one Commons with Turnstone as the fixed first consumer, and the media organ that any later call, `<video>`, `<audio>` or WebRTC track needs: demux, audio and video decode on a three-tier policy that keeps assemblers out of the default build, and media tracks on a native peer-connection stack. Opened by the Commons profile (2026-07-27) and the net-media plan (2026-05-26, rescoped 2026-08-25 and 2026-09-20).
Not here: browser and native WebRTC data-channel carriage, public rendezvous and the iroh relay lane (browser carrier plan); Turnstone's place-port T5 receipts and its call surface (Turnstone repository); the player contract, which genet owns; Redshank's Symphonia and Firewheel backend while it still lives in its port; retained Commons chat and clips (Commons); the retinue radio lane and low-power radio (low-power managed network plan).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- A call is an admitted live session inside one Commons: the Commons supplies membership, identity, invitation context and settings; Notochord admits each live service session; the call service owns signaling, media, presence and carrier policy after admission. Turnstone owns call actions, device choices, consent, interruption settings and the live surface; Graphshell may project call state but is not the host. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:19-40`
- The first product is two-person audio: invite, ring, accept, decline, leave, end; mute and push-to-talk; participant and connection state; user-configurable interruption policy; automatic carrier choice with an honest degraded state. Group calls, video, screen sharing, telephony gateways and recording are later products and the first slice must not abstract for them. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:26-40`
- Ownership: Commons membership and retained invitations are signed immutable facts under the Commons data profile; Notochord `SessionHello` proves caller and action before call bytes are read; `mere-transport` and Retinue supply carrier truth only through `AcceptedSession`; call state is ephemeral with sequence numbers, expiry and explicit terminal states; a recording or voice note is a message attachment, never live media history. Presence is soft state, never authority and never a retained member fact. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:42-60`
- Call media uses fresh per-call keys; Commons data epochs authorize and address the invitation but never become a shared media key. Joining, leaving, revocation and reconnect each have explicit key transitions. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:56-60`
- Control frames carry a random call id, the Commons space id, a monotonic sender sequence, the admitted participant identity and an expiry or terminal marker; duplicate and stale frames are ignored; a terminal frame dominates later non-terminal frames from the same participant; wall-clock order never settles concurrent state. `declined`, `missed`, `cancelled`, `failed` and `ended` are distinct terminal reasons. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:62-92`
- An invitation is a retained encrypted Commons fact with an expiry; accepting an old one does not resurrect a call and the live service still admits both peers. A sparse terminal receipt (participants, start and end times, reason, no keys or packet telemetry) is retained only when the user enables call history. Profile names: `commons.call.v1`, ALPN `mere/commons-call/v1`, path `/services/commons-call`, Notochord domain `mere.commons.call` with action `connect`, capabilities `invite`, `join`, `end` evaluated after admission, traffic class `Interactive`. These names stay in the call consumer; Notochord stays vocabulary-neutral. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:94-116`
- Carrier policy is user-configurable per device and Commons and reads measured loss, latency, jitter and bitrate, never transport names alone: an admitted IP bearer for live audio; reconnect on another admitted IP path; Reticulum or direct PHY for invitation, presence, terminal state and voice notes within their byte budget; otherwise "messages only" or "voice notes only". A direct-PHY receipt proves canonical Commons operations cross the radio, not live audio, and the current LoRa profile rules out an ordinary duplex call. A carrier change never changes call identity or authority; each new stream runs Notochord admission and proves the current call resume secret. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:118-141`
- Interruption is evaluated locally before ringing (allowed callers or roles, quiet hours and focus, ring, silent or auto-decline, second-call interruption, metered carriers, devices). Defaults live in settings, not the protocol; remote priority is a request local policy may ignore; accepting always needs a local gesture unless the user configured auto-answer for that caller and device. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:143-157`
- Security and privacy stops: the principal comes from `AdmittedSession`; a Commons grant for another service cannot join a call; revocation and delegation expiry are rechecked during long calls; media keys are forward-secret and never written to a Commons operation, checkpoint, crash report or quality log; the microphone starts only after local permission and visible acceptance; recording is a separate visible consent state and is not in the first product; quality reports are bounded, coarse and short-lived. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:159-173`
- Stop rules: no call types in Stickleback, Chartulary, Notochord or `mere-transport`; retained invitation sync is not "presence"; no radio-voice claim from a message transfer; no group media before a two-machine A3 receipt; no retained media or fine-grained quality history as a diagnostics side effect; no A1 before the place-port T5 headed receipt; neither Graphshell nor a Mere probe is the first call product. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:286-296`
- Media decode tiers: hardware decode first (`cros-codecs` for VA-API and V4L2, Vulkan Video via ash or wgpu; zero assembler); pure-Rust no-asm software (`rav1d` with `--no-default-features`, safe and slow, the fallback for untrusted bitstreams); asm-accelerated software opt-in behind a feature the media crate owns in isolation. The default `cargo build` pulls no assembler; this is permanent architecture, not a stopgap. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:80-106`
- Media scope: DRM, EME and Widevine are out by design; net-media decodes open and royalty-free formats; fetching media bytes is netfetcher's, and net-media takes bytes or streams and produces frames. Mere owns net-media and genet consumes decoded frames through a seam shaped like its `ImageLoader`; the JS media and WebRTC APIs bind through the host, never by linking net-media directly. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:33-48, 127-143`
- Media ownership split (ruled 2026-09-20, Mark): genet owns the player contract (`servo-media-player`'s controller, which Redshank consumes); this plan owns decoders and backends. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:19-25`
- The codec registry mirrors the engine-neutral organ pattern: a `codec-api` crate with neutral `Packet`, `Frame`, `Decoder`, `Encoder`, `Demuxer` traits and a `CodecRegistry` and no codec dependencies, per-codec crates, tier gating, and OxideAV's `core` as a reference and not a dependency. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:108-125`

## Phases

### A1. Admitted control session
Accept `mere/commons-call/v1`, convert `AcceptedSession` through the existing audited adapter, then call `notochord::admit_session` before decoding any call control. Seams: `murm/native/crates/transport/src/accepted.rs`, `repos/dramatis/crates/notochord/src/io.rs`, `turnstone/src/call.rs` beside Turnstone's place port. It must not found a second generic call host to keep the proof inside Mere.
- Done when: Turnstone's render-free call port completes invite, accept, leave and end over Memory and p2panda carriers; foreign-service grants are refused; denial emits no call frame.
- Needs: the place-port plan's T5 headed receipt (receipted as T5a-T5c at the strength of a founder-held document on a loopback place; see the open question); the noun question below, decided before A1 writes its first frame.
- Rulings: stop rules above.
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:208-237, 296-322`

### A2. Loopback audio
Choose the audio-device and codec dependencies only after a small duplex probe reports capture format, playback format, end-to-end latency, jitter behaviour and clean device loss. Keep codec frames outside Commons operations.
- Done when: two local processes exchange intelligible mono audio; mute stops capture; device removal ends or recovers without a stuck microphone; media bytes are encrypted with fresh call keys.
- Needs: A1.
- Rulings: A0 selects no audio device or codec dependency; that stays this probe's decision.
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:239-247`

### A3. Real IP call
Run the A2 media protocol over the admitted p2panda and Iroh bearer, with bounded jitter buffering, packet-loss accounting, reconnect and resume-secret proof.
- Done when: two machines complete a call, survive one forced connection loss and reject a replayed resume attempt; the final quality receipt reports measured loss, latency, jitter, reconnect count and selected codec.
- Needs: A2.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:249-256`

### A4. Presence and interruption
Expiring presence, local ring policy, focus and quiet-hours integration, device selection and visible degraded states.
- Done when: an offline or silenced peer is represented truthfully; presence expiry cannot change membership; every automatic decline names the local policy reason without disclosing private settings to the caller.
- Needs: A3.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:258-265`

### A5. Radio complement
Carry retained invites, terminal facts and bounded voice-note attachments over Reticulum and direct PHY; keep live audio disabled on carriers that fail the configured bitrate and latency floor.
- Done when: the connected T114 and Heltec V4 exchange an invite and voice note with byte-identical Commons verification; the UI says "voice note" and never claims a live call.
- Needs: A4; the retinue radio lane (low-power managed network plan).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:267-275`

### A6. Group-call decision
Only after A0-A5, compare mesh forwarding, a selected forwarder and an external relay using measured two-person media costs and the actual membership model.
- Done when: a written choice names participant limit, trust model, failure mode, bandwidth cost and the proof required before group calls enter the profile.
- Needs: A0-A5.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:277-284`

### N1. Demux-safe parsing, audio decode and the registry skeleton
`mp4parse` and `matroska` container parsing, Symphonia audio decode and the `codec-api` registry skeleton: fully pure Rust, no asm, no GPU, the cheap and high-value first slice over untrusted container bytes.
- Done when: known fixtures parse and decode, and the demuxers are fuzzed.
- Needs: Q4 below (repository placement).
- Rulings: decode-tier policy; ownership split of 2026-09-20.
- From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:145-150`

### N2. AV1 software decode (no-asm tier)
`rav1d` with `--no-default-features` behind `codec-av1`, wired into the registry, with the frame seam to a consumer.
- Done when: end-to-end decode is proved on the safe tier.
- Needs: N1; the frame-seam shape (Q3).
- Rulings: decode-tier policy.
- From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:151-153`

### N3. Hardware decode seam
A `cros-codecs` or Vulkan Video adapter (tier 1), the GPU-texture frame path and the netrender composite, reusing the External-layer and `wgpu` interop from genet's viewer and `wgpu-scry` work.
- Done when: a hardware-decoded frame reaches the compositor as a GPU texture through the media seam.
- Needs: N2; the frame-seam shape (Q3).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:154-155, 127-143`

### N4. WebRTC media tracks
Bind media-track plumbing to the chosen native peer-connection stack and the host frame and audio seams. Admission and Graphshell carriage stay with the carrier plan.
- Done when: media tracks flow over the selected native stack into the host seams.
- Needs: Q2 (the native WebRTC implementation); the browser carrier plan's dependency receipt; A2 for the call consumer.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:156-157, 167-169`

### N5. Opt-in asm decode
The isolated asm-accelerated decode feature (tier 3), the one place NASM enters the build, and only for fast CPU decode.
- Done when: the asm feature builds in isolation and never leaks into the default workspace build.
- Needs: N2; Q1.
- Rulings: decode-tier policy.
- From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:158-159, 163-166`

### D. Held items
Items both plans defer with a trigger of their own: group calls, video, screen sharing, telephony gateways and recording as later products; a low-bitrate radio voice experiment as a separate push-to-talk profile with its own intelligibility and airtime receipts; a sparse terminal receipt for missed-call history when the user enables call history; a recording consent state; Redshank's Symphonia and Firewheel playback moving here when a second consumer forces it; WPT `media-source/` and `webcodecs/` conformance after manual playback and demuxer fuzzing; pure-Rust H.26x and AV2 as watch lines, tracked until web-compat H.26x matters and then preferring C bindings only if forced.
- Done when: each item's own trigger arrives; none starts earlier.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:39-40, 101-103, 135-137, 170-171, 277-281`; `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:19-25, 59-65, 192-193`

## Open questions

- Is a call owned by the right noun? The contract makes a call a session inside one Commons, which excludes two people who know each other calling from a Murm relationship with no moot between them. The alternative is a plain reusable calls service taking an admitted session and a participant set from either context (Commons keeps retained invitations and terminal history for place calls, Murm initiates direct calls, Notochord admits either, Turnstone hosts the surface). A0's grammar is already sans-I/O and carries no Commons state beyond the space id, so extraction is cheap before A1 has a wire. Decide before A1 writes its first frame; do not decide by starting A1. From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:298-315`
- Who answers the noun question after Turnstone's place-port plan made Gemot the community surface (murmurs, moots, coop) on 2026-09-13: Gemot's lane, as owner of the community surface, or this plan, before A1? From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:317-322`
- Does T5's loopback-strength step 5 (a founder-held document on a loopback place) satisfy A1's gate, leaving A1 waiting only on the noun question, or does A1 wait for a step-5 receipt beyond a loopback place? From: `archive_docs/2026-10-28/2026-07-27_commons_calls_plan.md:226-233`
- Q1: does `rav1d --no-default-features` truly drop NASM, and how is the asm feature isolated so it never leaks into the default workspace build (a separate crate and explicit feature, like the boa fork's pattern)? From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:163-166`
- Q2: which native WebRTC implementation? Consume the browser carrier's dependency receipt if its selected stack exposes a sound media-track seam, otherwise compare current alternatives without coupling the two plans. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:167-169`
- Q3: what is the media frame seam's shape for genet, a CPU `Frame` or a GPU texture handle? Reuse the External-layer and `wgpu` interop rather than inventing. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:170-172`
- Q4: does net-media get its own repository (`Code/repos/net-media/`, sibling to netfetcher, the likely answer) or live as a crate under an existing repository? From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:173-175`
- Q5: is there a shared `Stream` or `Bytes` seam with netfetcher, or are they composed at the call site (likely composed; do not over-unify)? From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:176-178`
- Q6: codecs, hardware decode and the native peer-connection stack are native only, with the Wasm and PWA path using the browser's own `<video>` and WebRTC, as in netfetcher; is that platform split confirmed? From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:179-182`

## Findings

- 2026-05 (re-verify at build time): AV1 via `rav1d` and `rav1e` is the only ship-today pure-Rust codec bet; pure-Rust H.264 and H.265 are young; the hand-written asm versus compiler gap is permanent, so the tier policy is steady-state architecture. Demux (`mp4parse`, `matroska`, Symphonia's demuxers) and audio decode (Symphonia) are solved in pure Rust. `woodshed-audio` is a musician's-tool infrastructure crate and not reusable as a codec stack; the relevant crates are Symphonia and `web-audio-api`-rs. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:50-78, 195-207`
- 2026-08-25: WebRTC data channels and media decode have different authority, portability and acceptance consumers, which is why the data-channel lane left this plan. From: `archive_docs/2026-10-28/2026-05-26_net_media_plan.md:195-197`

## Progress

- 2026-10-28: cut from `2026-07-27_commons_calls_plan.md` and `2026-05-26_net_media_plan.md`; nothing new landed.
