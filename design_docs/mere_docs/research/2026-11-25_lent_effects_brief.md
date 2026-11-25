# Lent effects brief: how a host lends playback and capture to a guest

**Status:** 2026-11-25: brief, unreviewed. Nothing is proposed to code; no ruling is recorded.
The WIT below is a sketch to argue with, not a contract.

**Question (Mark, 2026-11-25).** "Being able to do stuff in Turnstone with data from Redshank ...
creating an audio note on that podcast in the manner that you would in Redshank, or Woodshed, being
able to maybe a progression that you saved ... being able to play through the notes." And of the
investment it takes: "it means that the other app is essentially linking Woodshed, right? And I
don't know how hot swappable that is, if that can be done at runtime."

**Short answer.** Presentation and editing already cross at runtime. Sound and recording do not:
every guest contract is pure, and the one place where another app's listening runs inside a host
is linked at compile time. This brief is about that gap only.

## What exists

Read 2026-11-25 at Mere `abdca5845`, Turnstone `c97fc8504`, Woodshed `09873b1b1`, Genet
`db1dbb5b8`, Graphshell `37cc8df6f`.

- **Guests are pure.** `block-guest` imports only `log` and `caps`
  (`graphshell/crates/graphshell-endpoint/wit/world.wit`). A view is a `sceno` scene and a
  `vector-sheet` the host paints. Woodshed's notation is such a guest
  (`woodshed/crates/woodshed-notation`, feature `block-guest`), and it ran signed in two hosts
  (application experience plan, N2). It can show and edit a progression. It cannot sound one.
- **Redshank already runs inside Turnstone, compiled in.** Turnstone names four Redshank crates as
  git dependencies and calls itself Redshank's second host (`turnstone/Cargo.toml`,
  `turnstone/src/redshank_host.rs`). The tile is Redshank's own compact dock, admitted through the
  contributed-surface seam. Turnstone owns one listening model, one store and "the ONE
  Firewheel/CPAL output authority the tile shares".
- **Capture is refused there.** `redshank_host/redshank_host_command.rs`: "Turnstone has no
  microphone adapter, the projection keeps `voice_capture_available` false, and a voice command
  that arrives anyway is refused in words." So the audio note on a podcast, from Turnstone, is the
  one part of Mark's example that does not work today. Text notes do.
- **The vocabularies are already written, as Rust traits.**
  - Playback: Genet's `components/media/player/controller.rs` has `PlaybackCommand` (`Load`,
    `Play`, `Pause`, `Stop`, `Seek`, `SetRate`), `MediaSource` (`Local`, `Http`, `HostBlob`) and a
    `PlaybackSink` whose doc says "the host-owned sink supplies the presentation clock used for
    persisted timed targets".
  - Capture: `redshank-model`'s `AudioCaptureHost` has `begin_capture`, `finish_capture` returning
    a `NoteBody`, and `cancel_capture`. A voice body is a mono PCM WAV blob in content-addressed
    storage (`woodshed/ports/redshank/README.md`).
  - Commands a mounted surface issues: `redshank-surfaces`' `CompactCommand`, which already
    includes `BeginVoiceNote`, `FinishVoiceNote` and `PlayVoiceNote`.
- **Woodshed's sound is its own.** `woodshed-audio` renders chords and walks a `Song` inside
  Woodshed's runtime. Nothing outside Woodshed can ask for it.

## Rulings this has to keep

- "One audio authority coordinates playback" (`woodshed/design_docs/2026-11-03_listening_annotation_port_plan.md`).
- Redshank's seams are shared "with explicit playback/microphone capability admission", and
  "product execution, scheduling, media ownership and audio devices stay in their hosts"
  (`woodshed/design_docs/2026-11-03_audio_ports_rehome_plan.md`).
- No injected web APIs; "unimported means unreachable" (`mere:script`).
- A capability is promoted to WIT only with a native and a component consumer (capability door
  plan, K1), in the form settings took (D22): typed records, a typed `denied`, a gate asked on
  every call, an application world that imports it.
- "A type or relationship label never grants permission to fetch a resource or run code"
  (`crates/graph/ontology/application-views.md`).

## The shape proposed

A guest never opens a device. It either describes a sound for the host to make, or commands a
transport the host owns, or asks the host to record and gets back a reference. Three grades, each
a larger grant than the last.

### 1. Described sound (no new capability)

The guest stays pure and returns data; the host's audio authority performs it. For a progression
this is a schedule: a tempo and a list of timed events (bar, beat, duration, pitches). It is one
more result kind beside `block-adapter`'s `attrs` and `text`.

What it costs: the sound is the host's instrument, not Woodshed's. Returning rendered PCM instead
would keep Woodshed's sound, but the contract caps a payload at 1 MiB, about twelve seconds of
16-bit mono at 44.1 kHz.

### 2. Lent transport

An interface the host grants, mirroring Genet's controller. The guest commands; the host owns the
decoder, the device and the clock, and the guest never sees a sample.

```wit
interface playback {
    variant failure { denied(string), unavailable(string), refused(string) }
    record snapshot { position-ms: u64, duration-ms: option<u64>, playing: bool, rate-percent: u16 }
    resource transport {
        play: func() -> result<_, failure>;
        pause: func() -> result<_, failure>;
        seek: func(position-ms: u64) -> result<_, failure>;
        set-rate: func(percent: u16) -> result<_, failure>;
        snapshot: func() -> result<snapshot, failure>;
    }
    /// `source` is a resource identity the host resolves under the caller's grant.
    open: func(source: string) -> result<transport, failure>;
}
```

The source is a resource identity, not a URL. A guest that could name any URL would hold a
network capability under another name.

### 3. Lent capture

The same three calls as `AudioCaptureHost`, with one change that matters: `finish` returns a
reference, not the audio.

```wit
interface capture {
    variant failure { denied(string), unavailable(string), refused(string) }
    record take {
        body: string,                 // content address of the stored recording
        duration-ms: u64,
        /// The host transport's position when recording began, if one was playing.
        anchor-ms: option<u64>,
    }
    begin: func() -> result<_, failure>;
    finish: func() -> result<take, failure>;
    cancel: func() -> result<_, failure>;
}
```

- **The host stores the body** and the guest records an annotation that points at it. A guest can
  make a voice note without ever holding the recording. Reading it back is a separate grant.
- **The anchor comes from the host's clock.** Redshank freezes a note's target when capture begins.
  When the host owns the transport, only the host can say where playback was.
- **A grant to ask is not a grant to record.** Each take starts from the user's action, and the
  host shows that it is recording and can stop it.

### Where it would live

In `mere:script`, beside `settings`, each with a crate of the `settings-wit` kind: a server, a
client over `script-wire`, a testkit. The same interface then binds to an in-process component and
to a provider process over the door, as the projection session does (capability door C0).

A block adapter's claim would name the grades it asks for, as a pack names `requested_scopes`
today. An offer to open a resource can then say what it will ask to do before it is approved.

## What it does not resolve

- **Real-time audio.** Woodshed's live input, the looper and Hocket's recorder are sample-accurate
  and callback-safe. A command contract cannot carry that; they stay inside their own hosts.
- **Audio into a guest.** Analysis (onsets, estimators) needs a stream of samples. `script-witgen`
  refuses streams, and it is a heavier grant than any of the three above.
- **Two apps on one machine.** "One audio authority" holds per host. A standalone Redshank and a
  Turnstone tile each own an output and the operating system mixes them.
- **Arbitration.** When two guests each hold a transport, the host pauses, ducks or refuses. Which,
  and who is told, is undecided.
- **Mobile and browser lifecycle.** Background audio, autoplay rules and the browser's own
  microphone permission belong to each platform and are not covered here.

## Options for Mark

1. **Which case goes first.**
   - The Redshank voice note from Turnstone (grade 3). It is refused today, both ends exist as
     Rust, and it is the case in the question.
   - A Woodshed progression sounding in the block pane (grade 1). No new capability, and it
     extends the guest that already runs in two hosts.
   - Both, since they share nothing but the authority.
2. **What a described sound is.** Timed events the host plays; rendered audio within the payload
   cap; or events now with rendered audio left open.
3. **Who holds a recording.** The host stores it and returns a reference; or the guest receives
   the bytes; or capture is offered only to a provider process that already owns its storage.
4. **Whether Turnstone's Redshank stays compiled in.** Leave it linked and add the contract beside
   it; or move the dock behind the contract once the contract has its two consumers.

## Open questions

- Does `open` take the same resource identity a projection session uses, so one grant covers
  showing an episode and playing it?
- Is the voice body's content address in the host's store, the guest domain's mere, or the
  persona's reservoir?
- Does a transport outlive the pane that opened it, the way a tile's playback does today?
