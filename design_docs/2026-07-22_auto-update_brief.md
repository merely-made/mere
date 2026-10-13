# Auto-Update Brief

**Current qualification (2026-10-12):** the
[pre-adoption comparison](#2026-10-12-pre-adoption-comparison) reviews TUF trust,
portable updates and recovery with disposable fixtures. Wider Luggage adoption
is held pending that evidence. Earlier verdicts below describe their dated
pressure tests; successful update cycles do not establish interrupted recovery.

**2026-07-22.** Cross-cutting research: a configurable auto-update capability
for every deployment surface in the family (desktop apps, radio firmware,
web/wasm, eventually mobile). Requirement from Mark: this is figured out
before any load-bearing deployment; it is a utility in its own right, wanted
alongside the identity vault
([plan](archive_docs/2026-10-06_completed_plans/2026-07-22_identity-vault-ssh-agent_plan.md)).

## Requirements

1. **Configurable, never a checkbox default.** Update policy is a real
   setting per
   [`feedback_configurability_over_opinionated_defaults`](<user-home>\.claude\projects\c--Users-mark--Code\memory\feedback_configurability_over_opinionated_defaults.md):
   off / check-and-notify / download-then-ask / fully automatic; channel
   selection (stable/beta/dev); check cadence; metered-connection behavior.
2. **Honest status.** Real states surfaced to the user (idle, checking,
   available, downloading, staged, restart-pending, applied, failed with
   reason), never a placebo spinner
   ([`feedback_real_sync_feedback`](<user-home>\.claude\projects\c--Users-mark--Code\memory\feedback_real_sync_feedback.md)).
3. **Signed.** Every artifact verified against a release key before apply.
   Release signing keys are company/offline keys, deliberately separate from
   personae user identity.
4. **Safe apply.** Power-fail-safe where the platform allows it; rollback or
   at minimum a working previous version on failure.
5. **Respect the install channel.** A distro-packaged or store-installed
   build must detect that and disable self-update (the package manager owns
   it).

## Prior art by surface

### Desktop (hocket, turnstone, isometry, woodshed, mere, strophe)

- **Velopack** (MIT, active as of July 2026): installer + auto-update
  framework, Win/macOS/Linux, delta packages, background apply. Core is
  Rust with a first-class [`velopack` crate](https://crates.io/crates/velopack).
  Successor lineage of Squirrel.Windows/Clowd.Squirrel. Opinionated: its
  installer owns the install layout (per-user dir on Windows, no UAC).
- **axoupdater** (MIT/Apache): standalone updater or library, consumes
  GitHub Releases (cargo-dist receipt format). Still maintained (0.10.x,
  2026); axo the company wound down and astral maintains a cargo-dist fork
  for uv/ruff, so treat the pipeline as community-carried.
- **self_update** crate: simple in-process GitHub-Releases binary
  replacement. Smallest possible mechanism, no deltas, no installer story.
- **tauri-plugin-updater**: minisign signatures + update manifest JSON;
  Tauri-coupled, but its manifest + minisign shape is worth borrowing.
- **Sparkle** (macOS, ObjC): the mac gold standard; appcast feed, EdDSA
  signatures, delta updates. Design reference, not a dependency.
- **Omaha / Keystone** (Chrome): background service, silent updates, staged
  percentage rollouts, differential compression (courgette/zucchini).
  Design reference for channels + rollout, far beyond our v1 needs.
- **TUF** (The Update Framework): the threat-model framework (rollback,
  freeze, mix-and-match, key-compromise recovery).
  [`tough`](https://crates.io/crates/tough) (AWS, powers Bottlerocket) is the
  maintained Rust client. Adopt the threat model vocabulary now, the full
  framework only if/when we self-host a repository.

Platform mechanics to design around: Windows cannot replace a running exe
(rename dance, apply-on-restart); macOS is an .app bundle swap with
codesign/notarization and translocation to respect; Linux raw-binary swap is
easy but coexists with Flatpak/AppImage/distro channels that own their own
update stories (AppImage has AppImageUpdate/zsync).

### Embedded / radio firmware (Merely hardware: T114 nRF52840, V4 boards; Tulle/Tucket/Sennet)

- Meshtastic practice today: nRF52 updates via UF2 drag-and-drop onto the
  Adafruit bootloader's USB drive (double-tap reset), or BLE DFU from the
  phone app (documented as riskier); ESP32 via WebSerial web flasher or
  esptool, with no app OTA path.
- **embassy-boot**: Rust bootloader, A/B active/DFU partitions,
  power-fail-safe swap, trial boot + rollback, optional ed25519 signature
  verification (dalek or salty), nRF flavor exists. The pick if/when we own
  firmware images. **MCUboot** is the C incumbent (Zephyr/nRF Connect).
- FCC/region-lock constraint from the firmware plan: shipped devices default
  to signed, region-locked images, and the update channel must be
  region-aware; the GPLv3 firmware + bootstrap-console posture means the
  user-flashable path must remain open. Auto-update here means "the app
  offers and stages firmware for a connected radio," never silent flash.

### Web / wasm (genet browser targets, isometry-web)

Deploy is the update. The work is PWA service-worker lifecycle (update
detection, skipWaiting/clients.claim) plus hashed-asset cache busting.
Configurability surface: prompt-to-reload vs auto-reload.

### Mobile (future, note only)

Store-mediated. Play has an in-app updates API; iOS has none. Nothing to
build until a mobile target exists.

## Recommendation

Two layers, so "configurable" is shared and mechanisms are per-surface:

1. **A small shared policy layer** (family crate, name TBD): the policy
   enum, channel, cadence, metered handling, and the honest status state
   machine from Requirements 1-2, with a `Transport` seam per surface.
   This is the part every app and the firmware-offering UI share.
2. **Per-surface transports behind it.** Desktop v1 candidates, in
   preference order to probe:
   - **Velopack via its Rust crate** (full story: installer + deltas +
     staged apply on all three hosts), accepting its install-layout
     opinions; or
   - **GitHub Releases + minisign-style ed25519 + our own staging/swap**
     (self_update-shaped, smallest surface, no installer), if Velopack's
     opinions fight the family's needs.
   Firmware transport is embassy-boot-shaped and lives with the firmware
   plan, behind the same policy layer.

**Concrete next step (the pressure test):** package hocket with Velopack on
all three hosts (all three are available, including SSH to the Linux
laptop) and run a real update cycle v0.1 to v0.2. Done condition: an
installed hocket updates itself through a signed release with the policy
setting honored and honest status shown. What that probe teaches decides
Velopack-vs-composed before anything load-bearing ships.

Signing for v1 is detached ed25519 (minisign format) over release artifacts,
key held offline, verify before apply. TUF/tough enters when releases move
off GitHub onto Merely-hosted (or P2P) distribution. P2P distribution of
update artifacts over iroh/retinue (content-addressed, blake3, fits eidetic)
is noted as a fit and deferred.

**Two signatures, two jobs (clarified 2026-07-24).** These are complementary
layers and conflating them is the easy mistake:

- *OS install trust* — Authenticode on Windows, Developer ID + notarization on
  macOS. This is what stops SmartScreen and Gatekeeper warning users at
  install. It is per-platform, costs money, and Velopack drives it.
- *Update-artifact authenticity* — our own detached ed25519 over the release
  bytes, verified before apply. Platform-independent, free, and the thing that
  makes a compromised feed insufficient to ship us a binary.

Per-host status:

| Host | OS install trust | Status |
| --- | --- | --- |
| macOS | Developer ID Application + notarytool | **Available** — Mark holds Apple developer credentials (2026-07-24) |
| Windows | Authenticode: signtool cert, or Azure Trusted Signing (`vpk --azureTrustedSignFile`) | Needs a cert; Apple credentials do not cover it. Azure Trusted Signing is the subscription alternative to buying an EV cert |
| Linux | No OS gate | Our ed25519 signature is the whole story |

So the macOS install-trust question is answered; the Windows one is a
purchasing decision, and the ed25519 layer remains wanted on every host
regardless of either.

## Findings — Velopack pressure test (2026-07-24)

Ran the mechanism end to end on Windows, on a throwaway minimal Rust app
(`velo-demo`) rather than hocket first, to isolate the update flow from
hocket's heavy genet build (and because there was concurrent build churn in
the tree). A real v0.1 to v0.2 self-update cycle succeeded: an installed
v0.1 checked a local `FileSource` feed, found v0.2, downloaded, applied, and
restarted; the installed `current` binary then reported 0.2.0 and the
velopack log recorded "Installation completed successfully." Demo uninstalled
cleanly afterward (no stray shortcuts or registry keys).

What this settled:

- **The Rust runtime crate is genuinely clean.** `velopack = "1.2.0"`
  (matches the `vpk` CLI version). `VelopackApp::build().run()` first in
  `main`, then `UpdateManager::new(source, None, None)` +
  `check_for_updates()` -> `download_updates()` -> `apply_updates_and_restart(&*updates)`.
  About 50 lines for the whole flow. `sources::FileSource` gives a local
  feed with no server (good for tests and for LAN/mesh distribution later);
  `HttpSource`/`GithubSource`/`GitlabSource`/`GiteaSource` also exist. API is
  C#-flavored (PascalCase: `UpdateCheck::UpdateAvailable`/`NoUpdateAvailable`,
  `updates.TargetFullRelease.Version`).
- **Packaging needs a .NET runtime, but not an SDK, and not an install.**
  `vpk` ships only as a `.nupkg` now, but a nupkg is a zip and the tool
  inside is a framework-dependent .NET app; extracting `tools/net8.0/any/vpk.dll`
  and running `dotnet vpk.dll ...` works against the .NET 8 runtime already
  on the machine. `vpk pack -u <id> -v <ver> -p <dir> -e <exe> -o <feed>`
  produced Setup.exe + full nupkg + portable zip + RELEASES in under a second.
  So the packaging-side .NET coupling is real but light: CI needs a .NET
  runtime (GitHub Actions has it), not a Rust-hostile toolchain.
- **Signing is Authenticode-centric.** Every pack warned "No signing
  parameters provided, N file(s) will not be signed." Velopack's integrity
  is the RELEASES/nupkg SHA; *authenticity* on Windows is Authenticode, which
  needs a code-signing cert. This is the one genuine divergence from this
  brief's cert-free ed25519/minisign v1 plan. Options if we adopt Velopack:
  accept Authenticode on Windows (cert cost + friction), or layer an ed25519
  detached-signature check in the app before `apply` via a download hook.
  Not blocking, but it's the decision the signing section must revisit.
- **Install layout is opinionated (as expected) and reasonable.** Per-user
  `%LOCALAPPDATA%\<id>\{current,packages}`, silent install, no UAC, desktop +
  start-menu shortcuts by default, uninstall registry key, clean uninstall
  via `Update.exe --uninstall`. Fine for our apps; nothing fought us.

**Verdict so far: Velopack is the recommended desktop transport.** It
delivered a working installer + feed + apply + restart in minutes, its runtime
crate is clean enough to sit behind the shared policy layer unchanged, and the
only real friction (Authenticode signing, light .NET-in-CI) is manageable. The
composed path stays the documented fallback if the Authenticode requirement or
the install-layout opinions become a problem. This verdict is desktop-only and
mechanism-only; it does not yet cover hocket specifically or the other two
hosts.

## 2026-07-24 REVERSAL: luggage (Rust everywhere) supersedes the Velopack verdict

Mark's requirement sharpened while setting up the Mac leg: the *whole*
pipeline should be Rust, including the packing machine (Velopack's runtime is
Rust, but `vpk` needs a .NET runtime wherever releases are packed). Ruled:

- **Fork `cargo-packager-updater` as `luggage`, homed in mere**
  (`crates/system/luggage`; provenance and divergences in its README).
  Packing uses upstream's `cargo-packager` CLI unchanged (`cargo install`able,
  MIT/Apache, NSIS/MSI/DMG/AppImage) — so no packer fork, and minisign
  artifact signing is native to the pipeline, which settles this brief's
  artifact-authenticity layer by construction.
- **T1 (landed 2026-07-24):** pluggable `Feed`s — HTTP (upstream templating
  intact), local directory holding `luggage.json`, and `github:owner/repo` —
  plus an optional per-platform **BLAKE3 digest in the manifest**, verified
  before the signature. The digest is the content-addressing seam for T3.
- **T2 (partially landed):** verified artifact staging survives restart and
  rechecks BLAKE3 before apply. The self-sufficient signed offer and portable
  staged-swap mechanics remain open in the
  [Djinn resident plan](mere_docs/implementation_strategy/2026-08-22_djinn_family_resident_services_plan.md)
  A2.
- **T3 (planned, design constraint carried from day one):** update
  distribution as signed manifests + content-addressed artifacts over
  iroh-blobs (mere-transport's `BlobStore` already speaks the ALPN), so
  chunk-level dedup between versions gives delta efficiency without patch
  files; retinue/mesh carries manifest announcements only (LoRa bandwidth),
  IP lanes carry bytes. Rollback/freeze defense: monotonic versions + signed
  timestamp in the manifest.

**Finding 2026-07-24 (from hocket's H4 run): the signature covers the
artifact, not the manifest.** Demonstrated live — a manifest claiming
version 0.3.0 while serving a genuinely-signed 0.2.0 artifact is accepted,
since digest and signature both check out against the bytes served. A feed
controller without the key therefore cannot ship arbitrary code (modified
bytes are refused, also demonstrated) but *can* lie about the version and
replay any previously signed artifact: a downgrade attack. This makes T3's
"monotonic versions + signed timestamp" a correctness requirement rather
than a nicety, and it means the manifest itself must be signed. Recorded in
luggage's README; until then feed integrity (HTTPS/GitHub over an untrusted
share) is load-bearing.

**Closed since that finding:** Luggage now verifies `luggage.json.sig` against
the configured publisher key and requires a signed manifest by default. The
remaining stage gap is different: the persisted stage does not yet carry and
reverify the complete signed offer in a fresh process, which the Djinn plan A2
owns.

**Finding 2026-08-26: a live native session creates the first release-reference
consumer outside the updater.** An invitation must say which signed native and
browser release it represents without making its rendezvous host, byte source,
or Personae key into release authority. Luggage therefore owns a compact
`ReleaseRefV1` made from the exact signed-manifest BLAKE3 plus publisher-key
identity, and a Wasm-clean verifier over the shared release envelope. This
pulls reference and verification out of T3 now. General P2P distribution stays
deferred until the browser carrier closes its direct, TURN, admission, and
reconnect receipts.

The signed content manifest does not carry feed or artifact locations. A
separate disposable `ReleaseOfferV1` supplies URLs or peer locators for a
`ReleaseRefV1`; moving a release to another mirror cannot change its identity.
The current v1 manifest combines those roles through each platform's `url`, so
the split is a versioned format change with a compatibility reader.

Velopack stays wired in hocket as the selectable A/B alternative
(`HOCKET_UPDATE_TRANSPORT=velopack`) and retires if luggage's H4 cycles hold
up; its delta packages remain the one capability luggage does not replicate
(T3 addresses the same need differently).

Still open before this is load-bearing:

- Apply the same to **hocket** (real GUI app: the `VelopackApp::build().run()`
  call must precede winit/genet init; restart-on-update with a live audio
  engine needs a look). Deferred past the current concurrent build churn.
- The **other two hosts** (macOS .app bundle + notarization, Linux vs
  distro/AppImage channels) — needs the Mac and Linux machines.
- Resolve **signing** (Authenticode vs layered ed25519) per the note above.
- Test **delta** packages (`vpk delta`) for the large hocket/genet binary.
- Build the **shared policy layer** (the brief's layer 1) so the honest
  status state machine and the configurable policy sit above Velopack rather
  than being called ad hoc.

## Non-goals now

Omaha-class update servers, staged percentage rollouts, mobile, and general P2P
distribution. Content-addressed release references and carrier-neutral
verification are current because the browser/native session now consumes them;
peer distribution remains chosen later rather than drifted into.

## Sources

- [Velopack repo](https://github.com/velopack/velopack), [velopack crate](https://crates.io/crates/velopack), [Rust getting started](https://docs.velopack.io/getting-started/rust)
- [axoupdater](https://github.com/axodotdev/axoupdater), [astral cargo-dist fork discussion](https://github.com/posit-dev/air/issues/297), [uv self update](https://github.com/astral-sh/uv/pull/2228)
- [tough (TUF client)](https://crates.io/crates/tough), [rust-tuf](https://github.com/theupdateframework/rust-tuf)
- [embassy-boot](https://docs.embassy.dev/embassy-boot), [embassy-boot-nrf](https://crates.io/crates/embassy-boot-nrf), [Drogue firmware-updates writeup](https://blog.drogue.io/firmware-updates-part-1/)
- [Meshtastic nRF52 OTA](https://meshtastic.org/docs/getting-started/flashing-firmware/nrf52/ota/), [UF2 drag-and-drop](https://meshtastic.org/docs/getting-started/flashing-firmware/nrf52/drag-n-drop/)

## 2026-10-12 pre-adoption comparison

**Status:** bounded comparison complete; Windows Luggage/Velopack and TUF
library probes recorded. App adoption, publishing and remote
qualification remain unchanged. User request: compare trust and portable
recovery before extending Luggage across the family, preserving concurrent work.

### Scope and reproducibility

Measured Luggage 0.1.0 from Mere `28d43dd5` (the Luggage sources remained
unchanged through concurrent Mere commits), cargo-packager-updater's current
published base 0.2.3, Velopack SDK/helper/packer 1.2.161, and tough 0.24.0.
Hocket still pins Velopack 1.2.0; the new fixture uses current matching tools
without changing Hocket. Upstream release and source evidence was checked on
2026-10-12; a recently pushed repository is not proof of updater maintenance.

The [independent fixture](../support/luggage-comparison/README.md) has its own
lock and uses the approved shared Cargo target, one compiler job, no product
runtime edits, no resident installation and no real vault. Its public TUF
inputs carry pinned source and SHA-256 provenance. Durable evidence lives in
[comparison receipts](../support/receipts/luggage-comparison/README.md).

### Trust: TUF versus the current signed feed

The current signed manifest successfully refuses an unsigned or edited feed,
and download authenticates its artifact. The probe nevertheless accepted a
signed 0.3 offer after observing 0.5 while the installed version remained 0.1.
A publication date from 2000 was also accepted: a date is not enforced expiry.
The comparator checks the installed version, not durable previously accepted
metadata. These observations do not imply a signer can be replaced remotely.

tough 0.24.0 refused the expired upstream repository with a timestamp-expiry
error, refused an edited timestamp, accepted valid root rotation from 1 to 2,
and refused an old signed root placed at the next root URL. Deliberately
setting expiration to `Unsafe` made the expired repository load: the negative
control succeeded. This is native library evidence using local upstream
fixtures. Product delegation, cross-boot metadata rollback and Merely's
artifact/apply integration remain unqualified.

| Property | Current Luggage | TUF candidate |
| --- | --- | --- |
| Authenticated bytes | Minisign manifest and artifact; optional BLAKE3, required for staging | Signed metadata authenticates target hashes and lengths |
| Freshness | Publication date has no expiry policy | Expiring timestamp/snapshot/targets metadata |
| Metadata rollback | No persistent observation history | Version history, requiring a persistent client datastore |
| Key rotation | One configured publisher key; no rotation protocol | Sequential roots approved by old and new root thresholds |
| Product scope | Product-qualified feeds are deployment convention | Delegated target paths can restrict product signing authority |
| Installation recovery | Application responsibility | Application responsibility |

These are separate jobs. TUF protects repository metadata and target delivery;
it does not choose the correct Djinn compatibility floor, migrate app data or
restore an interrupted installation. Metadata rollback protection also does
not replace app-version/channel policy: an authorized publisher can describe
an older application in newly signed metadata. Sources:
[TUF specification](https://theupdateframework.github.io/specification/latest/),
[tough datastore and expiry API](https://docs.rs/tough/0.24.0/tough/struct.RepositoryLoader.html).

Operational costs are real: provision a trusted root, separate root/targets
from frequently renewed timestamp metadata, retain rotation history, manage
expiry and protect the client's persistent trust state. Expired metadata
should stop an update check, not prevent an already installed local-first app
from opening. Offline distribution needs timely signed metadata; globally
disabling expiry would remove freeze protection. tough 0.24.0 also brings
AWS-LC native crypto compilation, so a Rust API does not mean a pure-Rust
dependency graph. Its
[past root-version advisory](https://github.com/awslabs/tough/security/advisories/GHSA-5vmp-m5v2-hx47)
was fixed in 0.20.0; maintained dependencies and conformance remain necessary.

**Assessment:** TUF is a better candidate foundation for evolving repository
trust than growing a home-made expiry/rotation/rollback protocol. Keep
Luggage's product release semantics and exact release reference above it.
Initially, an existing signed `luggage.json` can be an opaque TUF target;
this costs two signing systems but avoids silently redefining existing
release identity. A later envelope/key-identity change needs an explicit
versioned design. This assessment does not authorize adoption.

### Portable apply and recovery

| Windows probe | Current Luggage | Velopack 1.2.161 |
| --- | --- | --- |
| Portable folder moved to a path containing a space | No Windows portable format/backend | Live check/download/apply passed |
| New process applies downloaded update with feed absent | Stage bytes reload, but apply still needs an `Update` object | Live local pending-package apply passed |
| User file outside replaceable app directory | Not exercised in this comparison | Preserved through update |
| Stage bytes edited alone | Digest refusal passed | Not measured separately |
| Stage edited together with unsigned validation metadata | Fresh-process `take_verified` accepted changed bytes and digest | A harmless asset edited inside a valid pending ZIP was applied |
| Apply failure after backing up the old executable | NSIS/MSI owns installation; no new failure qualification | Injected failure removed both current binary and old backup |
| Delta package | No implemented delta backend | Produced 4,173-byte delta for a 22,196,517-byte full toy package |

The Velopack delta result is a format capability check. A largely unchanged
16 MiB random fixture payload makes it unsuitable as a prediction of Genet's
real update savings. Its first portable update lacked a cached base package,
so that apply does not qualify delta reconstruction. Neither fixture has OS
production signing; the staged-asset result is not a claimed Authenticode
bypass. It establishes that this tested pending-package handoff reopens a
locally changed archive without rejecting that asset change.

The Luggage probe changed the stage's version to 99.0, format to WiX and
destination to another fixture path. The stage reader accepted all three
while reporting verified bytes. No installer was executed. A reconstructed
signed offer and destination binding remain necessary before fresh-process
apply can be called authenticated. Both byte-only refusal and joint record
tamper were exercised with the default signed-manifest requirement enabled.

The Velopack replacement failure held the new extraction directory open
without delete-sharing. Extraction succeeded, the old `current` directory was
backed up, the replacement rename retried and failed, then the helper exited
1 after deleting the old backup. Both binary-presence checks were false.
The source confirms cleanup of both temporary paths before propagating the
apply error:
[Windows apply source, pinned release](https://github.com/velopack/velopack/blob/1.2.161/src/bins/src/commands/apply_windows_impl.rs).
This was an ordinary apply-error injection, not a power-cut simulation.

Luggage's Linux source renames the old AppImage away and writes the new bytes
at the live path, restoring only on a returned write error. Its macOS source
moves the old bundle aside and lacks a matching restoration path if the new
rename fails. Those are source-review findings, not executed remote failures.
Velopack's current macOS source prefers an atomic exchange and restores the
old bundle on ordinary fallback rename failure; its elevated path and Linux
replacement still need their own native failure receipts. See
[Velopack macOS](https://github.com/velopack/velopack/blob/1.2.161/src/bins/src/commands/apply_osx_impl.rs)
and [Linux](https://github.com/velopack/velopack/blob/1.2.161/src/bins/src/commands/apply_linux_impl.rs).

### Recommendation and gates before spreading

Keep Luggage as the family-facing release API. Keep cargo-packager for the
already qualified packaging slice. Velopack is presently the stronger
Windows portable candidate, but its tested recovery failure prevents a
general reliability recommendation. The Rust-only preference is one cost
criterion; it does not substitute for these apply guarantees.

Before adopting either apply backend more widely:

1. Reverify a self-sufficient signed offer in the applying process, bound to
   product, target, expected version/channel and permitted install root.
2. Recover the old runnable version after failed replacement; retain durable
   recovery information for termination between backup and replacement.
3. Repeat portable relocation, offline restart, staged tamper, concurrent
   apply and interruption probes against the selected backend and real
   application data. Package rollback and schema rollback are separate gates.
4. Qualify a bounded TUF repository with persistent rollback state, root
   rotation, expiry, product delegation and verified artifact retrieval before
   making it the production trust provider. Browser/Wasm verifier suitability
   requires a separate proof; the native tough probe does not establish it.
5. Keep Djinn custody/consent and shared-resident compatibility explicit;
   neither an app installer nor a successful TUF fetch owns those decisions.

Native Fedora/macOS failure and relocation qualification stays pending until
the machines are reachable. No remote work was dispatched by this comparison.
Successful historical H4 cycles and basic T2 stage persistence remain useful
evidence, but do not close these gates. The Hocket plan's statement that T3
landed cannot establish P2P/delta delivery; that same plan says its P2P lane
is unbuilt. The current Luggage code has no such backend.
