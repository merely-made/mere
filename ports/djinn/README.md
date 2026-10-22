# Djinn

Djinn is Mere's local desktop resident. It composes the owner-held parts of a
personal device: Personae authority, the SSH agent, Graphshell's local browser
and application brokers, personal sync,
[Knot's](https://github.com/merely-made/knot-editor) source/sync/evidence custody,
per-profile Castellan custody, and the shared physical blob store.

Graphshell remains the local session and admission protocol. Knot remains the
Djot editor and document authority. Djinn owns their shared process lifetime
when they use persona-held state.

## Run

### Explicit Moot site endpoint

Feature `moot-site` adds a local public Gemini endpoint over supplied Gemot and
Eidetic owners:

```text
cargo run -p djinn --no-default-features --features moot-site --bin djinn-moot-site -- ENDPOINT_ROOT OWNERS_JSON
```

The versioned owners file names `moot_id`, `moot_directory`, `content_database`,
`retention_revision`, `availability` and `erasure`. Governance is reopened from
signed genesis. It contains no author signing key. This standalone caller uses
redb; an embedding supplies existing owner handles to `MootSiteService`.

Stdin accepts one JSON command per line: `configure` with a `SiteBindingV1` and
`GeminiListenerPolicyV1`, `import` with local signed-record and optional snapshot
paths, `start`, `status`, `stop`, or `shutdown`. Replies describe success, refusal
and endpoint status. Import is explicit local delivery, with signature and
content verification; admission and retention span separate owners and a later
failure does not roll back facts already admitted. Ordinary requests acquire
nothing. An empty or ambiguous selection, lost authority or withdrawal refuses
subsequent requests once the signed updates have been admitted locally.

The binding and TLS identity survive restart. Restart serving requires the
configured opt-in policy and a currently valid selection. `stop` durably clears
the running intent; shutdown/EOF joins listener and connection work while
preserving it. Armillary records actual task outcomes. Local disk I/O and bounded
snapshot decoding run inline and are not preempted by network deadlines.
Desktop publishing controls, the main resident broker's owner attachment,
remote acquisition, hosting promises and Scroll/Micron listeners remain later
consumer gates. The earlier `djinn-site` snapshot-handoff command stays available.

### Personal resident

```text
cargo run -p djinn --bin djinn -- --dir <personae-vault> --data-root <data-root> \
  --receipt-agent-endpoint <endpoint> --browser-endpoint <endpoint> --app-endpoint <endpoint>
```

The standard SSH agent endpoint and the default browser and app endpoints
belong to the installed resident, which the installer launches with
`--installed`. Any other run refuses them and names its own.

`--log-filter <directives>` sets the tracing filter (`info` by default, for
example `info,iroh_gossip=debug`). `--events-file <path>` appends one JSON line
per lifecycle event: `started`, `listening`, `ready`, `stopping` and `stopped`
with its reason. `djinn --resident-status` prints a running resident's status
from its app door, and `djinn --stop-resident` stops it the way Ctrl-C does;
both routes are owner-only.

`--diagnostics-config <path>` opts into an in-memory redacted lifecycle history.
The JSON file requires explicit ingress and retained record/byte/age bounds and
a sampling rate. For example:

```json
{"ingress_records":32,"ingress_bytes":65536,"ingress_age_secs":300,
 "retained_records":32,"retained_bytes":65536,"retained_age_secs":300,
 "sample_rate":1.0}
```

A zero in any ingress or retained bound disables that buffer; `sample_rate`
must be between zero and one. `djinn --resident-diagnostics` reads the bounded
copy through the existing owner-only status route. Absent configuration reports
disabled coverage. Only fixed lifecycle phases and explicit stopped outcomes
are retained; profile/credential/endpoint/error text is excluded before queue
admission. The event file remains separate. This history is neither a durable
journal nor a final stopped receipt after process exit, and it installs no
process-wide capture of other logs. Each reader has an independent cursor. Byte limits use admission accounting,
not JSON output size.

The default configuration continues to read the existing Graphshell
application directory. That is a compatibility bridge for selected profiles,
pairing records, and content-store migrations, not an additional resident.

## Installation and SSH

On Linux, `install-linux.sh` takes prebuilt Djinn and `personae-vault`
binaries. It installs a systemd user unit, enabled at login, without building
on the target or replacing its existing `ssh-agent`. The optional last two
arguments set the resident's memory ceiling and CPU quota:

```sh
bash ports/djinn/install-linux.sh /path/to/djinn /path/to/personae-vault 768M 50%
systemctl --user start --no-block djinn.service
```

The native desktop dialog creates or unlocks the vault. Passphrases are not
stored in the unit, environment, or installer. Start from the desktop user's
session so the systemd user manager has its display and session bus environment.
The dedicated agent is `$XDG_RUNTIME_DIR/djinn/agent.sock`; name that socket in
each peer's SSH `IdentityAgent` setting. The desktop's existing agent remains
available to other clients. An active resident or another owner's unit is
preserved; a stopped unit's stale socket is removed only after checking its
type, owner, inode and absence of a listener.

`personae-vault generate-ssh <comment>` creates an Ed25519 SSH key inside
Djinn and prints only its public half. The existing persona master remains
unchanged. The key uses session unlock, is held in the encrypted vault, and
becomes available to the live agent immediately. Register the public half on
the intended peer. `scripts/djinn-ssh-control.py` is a small owner-local custody
client for generation, public inspection and explicit lock checks; it handles
neither private keys nor passphrases.

On Windows, `install-ssh-server-windows.ps1` runs elevated to install the inbox
OpenSSH Server, initialize missing host keys, and enable `sshd` automatically.
It permits public-key authentication for one named local account and restricts
the firewall rule to one peer IPv4 address. Existing SSH configuration owned
elsewhere is refused for review. For an administrator account, pass the peer's
public Ed25519 key as `-AuthorizedKeyFile`; enrollment appends it to the
administrator key file and applies the Windows-required ACL. The script is
for an administrator login, as used by this lane. Inspect its public host-key
fingerprints locally before pinning the host at the peer.

Windows' native identity unlock uses desktop Windows Hello with a visible
parent window owned by Djinn. Its UI thread keeps that window responsive
while Castellan verifies presence; the parent closes when verification ends.
The SSH agent cannot unlock the vault over its wire. After a lock, use the
resident's native unlock action before retrying SSH.

These scripts establish SSH execution and custody. The Iroh SSH proxy,
cross-device CA approval, and fleet pairing ceremony remain work in the
device-pairing plan. Installing an SSH server does not establish those grants.

## Saved Gemini sites

`djinn-site` submits an ordinary saved Knot site to the running resident through
its owner-only application broker. Each selected Personae profile has one
publication, its own certificate, and independently retained content.

```text
djinn-site publish <site-directory> 1965 --resume
djinn-site status
djinn-site stop
djinn-site remove
```

The port is optional; zero asks the operating system for a free loopback port.
Status reports the bound address, snapshot digest and certificate fingerprint.
Browse `gemini://localhost:<reported-port>/` in Lagrange. The stock 1.21.1 client
passed with this hostname; its IP-literal certificate check refused the same
loopback certificate, so `127.0.0.1` is not an accepted Lagrange URL receipt.
`--resume` explicitly enables serving after a resident restart. Without it,
the saved publication starts stopped on the next run. The default listener is
restricted to `127.0.0.1`. The broker endpoint follows
`GRAPHSHELL_APP_ENDPOINT`, matching the resident's `--app-endpoint` when one is
selected.

Publishing selects saved bytes once. Later source edits become visible only
after another successful publish. Stop releases the listener and retains the
publication. Remove forgets the serving record and releases its storage lease;
status reports retained custody if cleanup needs retrying. The profile's TLS
identity remains available for its next publication. Missing or mismatched
identity files are refused during serving restoration.

The initial caller is a command-line utility. Knot's desktop controls, explicit
certificate rotation, additional protocols, public binding and governed moot
hosting remain separate integration work. Exporting Tabard's Lagrange palette
does not change Gemini page content or this listener's policy.

## Paired devices

`djinn-devices` asks the running resident where each paired device is now,
through the same owner-only application broker. For every device paired in
this profile's personal sync it prints the node id, label, root, pairing id and
time added; whether the device is connected now; every address the transport
holds for it, the live ones marked active; and the last saved dial hint,
decoded into its direct addresses and relay.

```text
djinn-devices
djinn-devices --json
```

The directory is read-only and lists paired devices only. The broker endpoint
follows `GRAPHSHELL_APP_ENDPOINT`, as `djinn-site` does.

## Resident inference

Build with `--features inference` for a CPU model-library lane, or
`--features inference-gpu` for the first discrete GPU. GPU selection is
probed and refused when that adapter class is unavailable. Owner settings
name the exact imported row and output limit:

```json
{"inference":{"model":{"library":{"row":"blake3:<row hash>","device":"gpu","max_tokens":256}}}}
```

The lane requires a `distillery` block and shares its persona's model library.
`esp`'s `import_inference_model` example imports a local checkpoint and declared
template into that library. Its printed settings select CPU; change `device`
to `gpu` for the GPU build. The `inference.ask` route is granted to Turnstone.

Passing a streaming-options JSON file as the import example's sixth argument
stores weights as immutable per-layer artifacts. The same library setting
selects that row on CPU or GPU. The resident reads and verifies one layer at a
time through its existing library owner, releases the library lock before
tensor work, and uses bounded query blocks during prefill. Its execution loader
has a `/layer-stream-v1` suffix, so checkpoints cannot silently resume through
another loader. The native I6 decoder rows qualify a model exceeding GPU
memory and bounded long prefill; the plan records those measurements and the
resident composition regressions separately.

An indexed GPU row can also carry an explicit `weight_cache` block:

```json
{"host_bytes":3221225472,"device_layer_bytes":4294967296}
```

These illustrative numbers are owner limits, not detected free memory. The
optional block belongs inside `library`; absence preserves uncached streaming.
`host_bytes` bounds the retained capacity of verified immutable encoded layer
buffers (at most 128 entries). `device_layer_bytes` bounds additional retained
dense f32 layer-tensor payload during a generation. The owner must separately
reserve shared weights, KV, one transient streamed layer, execution workspace,
allocator overhead and other applications. These limits are not RSS or total
VRAM bounds. Nonzero cache settings are refused for CPU library execution;
device-layer caching currently refuses MoE and reduced-precision rows.

Caches admit a first-fit subset instead of evicting every layer on a cyclic
sweep. Remaining layers still stream. Host hits share their verified allocation;
source errors are not cached and tighter byte limits are rechecked. Device
weights stay inside the inference foreground guard and are dropped before that
guard is released; host buffers belong to the lane and can serve later asks.
Both caches are ephemeral. They change placement and repeated work, while the
original format, prompt, token sampling, checkpoint/session identity and model
content references remain the same. Debug cache counters report logical reads,
hits and payload/capacity bounds without recording prompt or answer text.
Performance qualification also needs an optimized tensor/backend build: an
unoptimized development backend can spend most of its time constructing and
submitting kernels even when the caches hit. Cache counters establish reuse;
measure generation separately from artifact verification, import and prefill.

A GPU ask preempts the GPU trainer through owner reclaim. The offer is
withdrawn while inference waits or runs; inference loads device tensors for
each ask and releases them before returning the GPU. CPU inference retains
its loaded model, with only shared weights resident for layered rows. Library
asks now retain exact token checkpoints in an
append-only encrypted log. Shutdown pauses unfinished asks and joins the actor;
local interrupted asks resume after reopen. Lock refuses the route, pauses at a
token boundary, clears materialized runs and their device-local data key, and
unlock rehydrates and resumes lock-paused work while the device retains a
current execution seat. Unlock also reconciles operations refused while locked.
Explicit pauses remain paused. A GPU pause drops
its KV cache and device tensors before releasing the foreground guard.

A qualified `esp.layer-weights/v2` row selects its declared f16, bf16, Q8 or
Q4 format through the same library setting. CPU rows use Flex for half precision;
GPU rows retain the trainer reclaim path. Imported format candidates must be
measured against bf16 before publication. The measured loader must match the
selected device's loader, and a format change refuses old checkpoints.

The optional `inference-stages` feature also composes automatic execution from
settings. It requires the live keyed personal graph and a layer-indexed library
row. For example, the `model` field can be:

```json
{"automatic":{"row":"blake3:<qualified-row-hash>","device":"gpu","max_tokens":128,"stages":{
  "local_resident_bytes":2147483648,"worker_layer_bytes":536870912,
  "max_peers":8,"max_hidden_bytes":16777216,
  "discovery_timeout_ms":2000,"discovery_every_ms":1000,
  "lease_wait_ms":10000,"call_timeout_ms":60000,"poll_every_ms":25,
  "lease_duration_ms":600000,"heartbeat_ms":10000
}}}
```

These numbers illustrate explicit owner limits. `local_resident_bytes` admits
weights and full-context KV for whole-model residence; zero skips that rung.
`worker_layer_bytes` is the offered layer/KV budget after reserving embedding/head,
RoPE, activations, allocator and backend space. Zero offers no worker. Neither
budget is a process RSS or VRAM limit. The device's existing mesh lending rules
still apply; an explicit resource allowlist must permit its device-addressed
`esp.inference.stage.<mesh-author-hex>/v1` resource and the restart class.

An ordinary ask first tries local residence within that budget, then discovers
current keyed readers offering the exact row, format and backend, posts real
stage leases, and falls back to local layer streaming if placement is unavailable
or leaves mid-answer. Discovery exchanges attested compute endpoints over the
existing personal connection, then uses Distillery's existing endpoint for
stage work. Compute joins a graph-scoped mesh across distinct profile roots.
Ask journals, device IDs and existing history retain their profile-scoped paths.
No prompt enters discovery, reservation, the mesh post or its fixed receipt.

`ResidentDistillery::enable_inference_stages` and the explicit staged actor API
remain available to embedding hosts. Both paths use the existing library and GPU
owner, exact-session guards and current graph seats through `PersonalGraphSeals`.
The vault holder refuses new work and waits for catalog, KV and tensors to drain
on lock. Historical readability does not authorize new stage work. Install the
holder before admitting application sessions. Native I8 rows use independently
rooted software devices/libraries on one physical computer; physical fleet
capacity and inter-computer latency require separate receipts.

The route also takes `distillery.inference/pause-v1` and
`distillery.inference/resume-v1` with the cancel payload's `{"run":"name"}`.
`distillery.inference/ask-sampled-v1` accepts `ask`, `max_tokens`, `temperature`,
`top_p` and `seed`; the owner cap still applies. Every keyed personal-graph
device can read received asks and resume one with the exact model session.
Inference borrows personal sync's existing key group and endpoint; devices
keep separate identity roots. Shared asks refuse until the device is keyed,
then become available through live key agreement. With personal sync off,
asks stay locally durable. Existing local history is imported when keyed.
Revocation excludes future epochs while preserving already received history.
The native/embedded composition can call `configure_inference_recall` with an
already-owned retained corpus and recall limits. Turnstone then uses
`distillery.inference/ask-recall-v1` with `{"ask":{"run":"name","question":"..."},"scope":[]}`.
An empty scope means the composed corpus. The route returns supplied passages,
source-version staleness, validated citations and their answer spans alongside
the run. Citation bases distinguish model markers from exact-quote annotations.
The journal retains exact source versions and selectors through restart and
personal-graph continuation. The desktop binary does not discover or reopen
Turnstone's app-owned browsing database. Turnstone publishes retained text over
the admitted route, with an owner namespace per browsing session, then asks
against that publication. Only supplied passages enter the sealed run journal.
The optional `inference.recall` object sets `chunk_chars`, `max_passages`,
`min_quote_chars`, `max_context_chars` and `max_corpus_chars`. When this object is
omitted, RecallConfig defaults apply. Inline publications above 128 KiB are
refused. The Turnstone native client is qualified through I5, including streaming,
pause/resume, cancellation and exact source reopening after both processes
restart with the origin offline. That gate uses a test-owned native resident;
the installed Djinn binary retains its separate gate.
See the
[resident inference plan](../../design_docs/mere_docs/implementation_strategy/2026-10-15_resident_inference_and_recall_plan.md).

## Publishing

`0.0.2` is the source version of this workspace resident, not a crates.io
release. Knot Editor is pinned from its own public repository; the Graphshell
composition still uses workspace-only dependencies, so `cargo package` correctly
refuses it. A public Djinn release needs an installable package boundary and a
staged release of the Mere dependencies it exposes.

## Security boundary

Djinn holds durable authority but does not manufacture public services. Its
default personal-sync policy can use local discovery when the owner has
configured it; relays remain owner-selected transport configuration.

The following deployments are intentionally absent until a forcing consumer
defines their policy and acceptance receipt:

- a public Knot publisher;
- a Misfin receiver;
- a Gemot community host;
- a dedicated relay;
- a Secret Service daemon.

Castellan record and freshness keys are separately derived from the unlocked
Personae identity and opened per Djinn profile. Starting Secret Service also
requires a concrete persona selection and allowed-caller policy, so Djinn does
not guess either from a profile name.

## Lifecycle

Djinn uses Distillery's `lifecycle` module (the `mere-resident` crate until
2026-09-23) for the small rule shared with Distillery: close
resources in a concrete order, attempt every close, and retain every failure.
It does not share product policy, configuration, or service APIs with
Distillery.

## License

MPL-2.0
