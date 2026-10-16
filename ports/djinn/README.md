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

A GPU ask preempts the GPU trainer through owner reclaim. The offer is
withdrawn while inference waits or runs; inference loads device tensors for
each ask and releases them before returning the GPU. CPU inference retains
its loaded model. Library asks now retain exact token checkpoints in an
append-only encrypted log. Shutdown pauses unfinished asks and joins the actor;
local interrupted asks resume after reopen. Lock refuses the route, pauses at a
token boundary, clears materialized runs and the data key, and unlock rehydrates
and resumes lock-paused work. Explicit pauses remain paused. A GPU pause drops
its KV cache and device tensors before releasing the foreground guard.

The route also takes `distillery.inference/pause-v1` and
`distillery.inference/resume-v1` with the cancel payload's `{"run":"name"}`.
`distillery.inference/ask-sampled-v1` accepts `ask`, `max_tokens`, `temperature`,
`top_p` and `seed`; the owner cap still applies. Current journal replication
admits a shared persona signer. Connecting private asks to separately rooted,
keyed personal-graph devices remains an I3 authority boundary in the
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
