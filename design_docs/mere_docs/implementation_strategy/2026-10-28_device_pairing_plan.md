# Pairing by key: the fleet resident, SSH by key, host certificates, one ceremony and the other network

**Status:** 2026-10-28: cut from `2026-10-02_device_pairing_by_key_plan.md`, archived in `archive_docs/2026-10-28/`. Landed: D1 (the
paired-device directory route and CLI) except its real-machine lease-move receipt; D1b's mere fix (M1) and its fork fix (F1,
shipped in the 0.7.5 repin); `connected` following the gossip overlay while subscribed and open connections off it. D2 was
resumed by Mark on 2026-10-12 and is partly qualified on s-pc, Windows, the M4, q-pc and the ThinkPad. Open: the D1 and D1b
real-machine items, the overlay restart gap (held for iroh-gossip's next release, ruling 72), the fleet rollout of D2
with its Linux launcher, D3 (SSH by key), D4 (host certificates approved by the master), D5 (one ceremony) and D6 (the other
network's relay).

**For:** Mark's machines finding, reaching and trusting each other by device identity, not by address. Opened on 2026-10-01
when the leases on his `/22` mesh Wi-Fi had moved again and finding machines took a sweep for port 22: "these are not
static ip addresses. Is there a coordination strategy we can employ to pair devices easily?" Pairing a device becomes one
ceremony, SSH reaches a device by key, host-key prompts retire, and a self-hosted relay covers the second network.
Not here:
- DNS-SD (the Djinn update resident's F3): it is unbuilt and would not help SSH, since service records are link-local claims
  rather than identity; `2026-10-28_family_update_resident_plan.md`;
- the custody rules the principal, session and key flows must compose with, and the Secret Service design that waits on D2's
  launcher: `2026-10-28_custody_plan.md`; the LAN SSH and presence receipts do not complete D3 to D6;
- the Djinn self-update launcher: `2026-10-28_family_update_resident_plan.md` phase D;
- alternative overlays (Tailscale with Headscale, Nebula): considered on 2026-10-12 and not installed in this slice; D3
  (Djinn and Iroh) is the stack-owned transport slice;
- upstream p2panda, iroh and iroh-gossip issues: the upstream candidates ledger, per ruling 46 below.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

The archived plan holds rulings 1 to 74; those below bind an open phase, and the archived copy holds the rest. Numbers are the
archived plan's, and each `Source:` range is a line range in
`archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md` (the rulings run `:453-1025`).
- **Identity and reach (D3).** 2: device X resolves to the personal-graph node id, the key djinn already refreshes hints for.
  3: `ssh <device>` reaches it by a `ProxyCommand` through the local resident. 11: where device X is now lives in a Djinn
  app-door route and CLI (`device-directory-v1`, `djinn-devices`). 12: the Windows laptop is an SSH target in scope, which
  needs Windows' OpenSSH Server (an administrator step, Mark's). Source: `:463-471,515-522`.
- **What `connected` means (D3 is the first off-overlay user).** 31: for a peer on the overlay the gossip neighbour state is
  authoritative and iroh's path counts only for peers not on it. 32: "on the overlay" means this node has subscribed to the
  topic. 47, 48: peers off the overlay are tracked from iroh connection events and read connected while at least one
  connection to them is open, whoever opened it. 49: an open connection never makes a subscribed-overlay peer read
  connected. 39, 50: the card and `djinn-devices` word the reason ("no gossip neighbour; a path is still active" on the
  overlay, "no open connection; a path is still active" off it). Source: `:640-655,712-717,802-834`.
- **Host certificates (D4).** 5: finish host certificates now, not deferred. 6: each machine has its own CA and the master
  decides which may sign host certificates (reading, not ruled: OpenSSH cannot chain CAs, so the approval is a
  `SignedDelegationCertificate` per machine CA, and a client-side step checks each approval and its revocation and writes
  one `@cert-authority` line per approved CA). 10: the Windows laptop's vault master is the root. Source: `:478-491,510-513`.
- **One ceremony and the relay (D5, D6).** 8: one ticket and SAS recording everything. 4: a self-hosted iroh-relay; where it
  runs is open. Source: `:473-476,500-503`.
- **The fleet (D2).** 9: extend Djinn with a systemd user unit and a launchd plist, beside the Windows installer. 15: on
  Windows, side by side with the legacy `graphshell-device-host` on its own pipe until the resident's agent signs a real
  SSH login, then Mark runs Djinn's installer, which retires the legacy task; rolling back re-enables that task. Vault lock
  17 and 22: the standalone `personae-agent` and its three installers retire after D2 and macOS joins with D2; the
  reassessment starts from the vault lock's state and the order is Mark's to set. Source: `:505-508,535-542`; `:1706-1714`.
- **First contact and the overlay (D1, D1b).** 14: D1 is proven locally and its real-machine receipt follows D2. 16: fix
  mDNS-only first contact (done as M1 and F1). 26: mere's per-interface mDNS fork stays out until diagnosed. 36: ruling 31
  stands and the overlay restart gap is fixed in its own lane; 64: the fix is a patch to iroh-gossip; 65: change A1 only
  (clear a peer's pending entry in `on_join` when it is already an active neighbour); 66, 70, 71: if a fork is made it is a
  tagged fork repo, `mere-iroh-gossip-0.101.0`, based on upstream `main`; 72: wait for upstream, so no fork is made, ruling
  36's fix and ruling 67's resident flag (`--seed-node-after`) are held, and at iroh-gossip's next release PR 159 and issue
  172 are rechecked. 46: upstream projects get a plain-English note only after a release has passed a recorded issue by,
  and nothing is posted without Mark's read; 21: nothing goes to p2panda now. Source: `:529-533,544-557,611-616,684-696,782-800,951-1005`.
- **Repin hygiene.** 24, 29: `decode_cbor` is lenient upstream, so every call that feeds a hash, signature, content address
  or wire validation uses `decode_cbor_strict`, the four plaintext-after-authentication sites included. 63: three
  knot-desktop Collapse tests failing on Knot's `main` are left to Collapse. Source: `:598-603,628-630,941-949`.
- **Standing permission.** Mark allowed `known_hosts` entries to be updated, each key added only after its fingerprint
  matched the machine's recorded key (given 2026-10-01). Source: `:1022-1025`.

## Phases

### D1. The directory (remaining receipt)
A Djinn app-door route lists each paired device (node id, label, whether connected, current path, last hint) and a CLI reads
it; proven locally with two instances on one machine.
- Done when: after D2, the same holds across real machines when a lease moves.
- Needs: D2
- Rulings: 2, 11, 14
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:1057-1066`

### D1b. First contact and restarts
Two paired residents that share a network find and connect with no ticket, and keep doing so across restarts. The restart half
is open: after a crash-restart the survivor's iroh-gossip can hold a stale pending neighbour request and never answer the
restarted side's `Join`, in either restart order (ruling 36's assessment); the fix is held for iroh-gossip's next release.
- Done when: at every restart, in either order, the restarted side gets a gossip neighbour and data moves both ways within one
  poll, `connected` following the overlay (ruling 31); the same holds between two real machines.
- Needs: iroh-gossip's next release (PR 159, issue 172 rechecked); D2 for real machines
- Rulings: 16, 26, 31, 36, 46, 64, 65, 66, 67, 70, 71, 72
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:1001-1005,1068-1092`

### D2. A resident on every machine
Djinn runs as a systemd user unit on Fedora and Kubuntu and a launchd agent on macOS beside its Windows installer; on macOS
LAN discovery needs a signed app carrying the local-network usage declaration (dialling out does not). Open at the last
checkpoint: a fresh graphical login or reboot is unqualified for s-pc; Windows managed-daemon reboot persistence; Mac
enrollment and rollout; the M4's Djinn migration (it still uses its legacy `personae-agent`, which holds its SSH identity)
and q-pc's legacy personal-sync store (an incompatible CBOR operation store under p2panda 0.7.1, neither recovered nor
migrated); the source commit of the cached Windows native binary is not established; fleet master approval of machine CAs
(D4). Client-mesh residue, not a phase: native client connection checks for the other peers, the s-pc session lacking
`send_message_to_thread` (an unsolicited push into the O-PC chat is unqualified), and the ThinkPad's O-PC client error whose
SSH-shell script still needs its native administrator step and an actual reconnection.
- Done when: each machine runs the resident at login with its personal sync paired with the others; the M4's agent is the
  current one, with Mark's own SSH into each machine unaffected throughout; on Windows, Djinn runs beside the legacy host on
  its own pipe until its agent signs a real SSH login, and then Mark switches with Djinn's installer.
- Needs: the vault lock's state (custody plan); Mark's ordering
- Rulings: 9, 15, vault lock 17, 22
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:20,50-53,59-60,92-94,142-143,168,178,198,220-226,263-264,274,322,363-372,1094-1108`

### D3. SSH by key
`ssh <device>` runs a `ProxyCommand` helper that asks the local resident to dial the device's node id on its own ALPN; the
target's resident admits paired devices only and forwards to its local `sshd`. The Windows OpenSSH Server was installed
and its host key pinned during the 2026-10-13 D2 checkpoint.
- Done when: `ssh <alias>` reaches each machine after its address changed, with no sweep and no new prompt; an unpaired peer,
  and a peer after unpairing, is refused; a forged host key still fails; each machine is reached as a target, the Windows
  laptop included.
- Needs: D2; the off-overlay liveness rules above
- Rulings: 2, 3, 12, 47, 48, 49
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:50-53,96-104,1110-1119`

### D4. Host certificates approved by the master
Each machine's own CA signs that machine's host certificate through a new mint-and-install path (installing it is a root or
administrator step on each host, Mark's). The Windows laptop's master issues an approval for each machine CA.
- Done when: each machine's CA signs its host certificate; the master issues an approval for each machine CA, and revoking one
  removes its trust everywhere within one refresh; each client's `known_hosts` carries one `@cert-authority` line per
  approved CA, written from checked approvals; a host's first contact shows no prompt, and a host key not certified by an
  approved CA is refused.
- Needs: D2; the master vault on the Windows laptop
- Rulings: 5, 6, 10
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:1121-1131`

### D5. One pairing ceremony
One ticket, typed code and SAS records a new device for personal sync, Knot and SSH trust (its machine CA submitted for the
master's approval); unpairing removes all three. Pairing took six steps and two 128-hex copy-pastes for personal sync, a
separate Knot `--pair-writer`, and a third SSH ceremony; pandect's wallet-grant pairing (ticket, typed code, six-digit SAS)
has no caller.
- Done when: pairing a fresh device takes one ceremony, its step count measured against today's six plus two separate
  ceremonies; a wrong code fails (a negative control).
- Needs: D4 for the SSH half
- Rulings: 8
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:444-449,1133-1139`

### D6. The other network
A self-hosted iroh-relay both networks can reach, set on every resident. Where it runs is a fork for Mark at this phase's start.
- Done when: the M4 on its wired-only network is reached by key through the relay; a control isolates the relay as what carried
  the dial (direct paths blocked), which is also the reachability plan's open R1 receipt.
- Needs: D3
- Rulings: 4
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:1141-1146`

### R. The 0.7.5 repin's remaining consumers
Turnstone, Cleromancy and Isometry move at their own next mere repins (Isometry with its strict-CBOR site); Retinue's desktop
workspace needs the `p2panda-core` patch row; the second Retinue copy (0.2.0, for `knot-site`) goes when Knot repins `knot-site`.
- Done when: Turnstone, Cleromancy and Isometry are on the repin, the desktop workspace carries the patch row, and the second
  Retinue copy is gone.
- Needs: each owner's next repin
- Rulings: 24, 29
- From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:1604-1607,1678-1682`

## Open questions

- Is there a Linux launcher now? The Secret Service plan said Djinn had only a Windows installer; this plan's checkpoint
  records a systemd user unit on s-pc and a Linux installer that refused to replace the active resident, and the custodian
  ships its own system service. Which is D2's? From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:55-62,146-153`
- Retire or keep the interim install scripts (`install-agent-windows.ps1`, `install-agent-linux.sh`, `install-agent-macos.sh`
  in personae, whose Windows header still says "Until then")? Vault lock ruling 17 retires the standalone agent and its
  installers after D2. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1143-1147`
- Per-interface mDNS: the fix for multi-homed Windows hosts has not been in effect (the patch is unused in the lock and the
  live crate has no per-interface sockets), and ruling 26 keeps it out until diagnosed. Has the diagnosis been done? From:
  `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:611-616,1264-1272`
- Do the three knot-desktop Collapse tests still fail on Knot's `main`? From: `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:941-949`
- Do the lane-reported code-comment drifts still hold: `ports/djinn/src/pairing.rs:46-48` (the host never writes settings
  back, while `personal_sync.rs` writes hints), `peer_id.rs:11-14` (a `PeerID` derives from the master, while hosts pass
  per-graph seeds), and `djinn --pairing-facts` computing the group pre-key and omitting it from its output? From:
  `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:1044-1049`

## Findings

- 2026-10-02: four unrelated device identities existed: the wallet `DeviceRoster` entry, the personal-graph node id that
  `PairedDevice` keys on, Knot's `PairedWriter` key, and SSH's device id (a hash of the hostname). Host certificates were never
  built: `SshCertAuthority::mint_host_cert` has no caller outside tests and `personae-vault enroll-host` writes one
  `cert-authority` line into `authorized_keys` and prints that host-key prompts still apply. Each vault's CA derives from its
  own profile master, a fresh vault generates its master at random, and there is no master export or import. Source:
  `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:407-430`.
- 2026-10-03: iroh keeps a closed connection's paths active until the peer's remote actor idles out, 60 s after its last
  queued message, and mere's own `remote_info` polls keep resetting that timer; an endpoint connection hook sees every
  connection and its `closed()` fires within seconds of a kill. Source: same file `:767-821`.
- 2026-10-04: a restart fails exactly when the survivor holds a stale pending entry for the restarted side (it predicted every
  traced restart); upstream `main` does not fix it and no release does. Source: same file `:951-966,980-984`.
- 2026-10-02: mere's per-interface mDNS patch for `iroh-mdns-address-lookup` is `[[patch.unused]]` in the lock, the live
  crate being crates.io 0.5.0, so H10's fix has not been in effect. Source: same file `:1264-1272`.
- 2026-10-13: the Linux installer refused to replace an active resident and the Windows installer refused an unelevated
  caller; a fresh graphical login or reboot is not qualified for s-pc. Source: same file `:59-60,146-153`.

## Progress

- 2026-10-28: cut from the device pairing plan; nothing new landed.
