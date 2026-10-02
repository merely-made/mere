# Device Pairing by Key Plan

**Date**: 2026-10-02
**Status (2026-10-02)**: in progress. Assessed and ruled by Mark on 2026-10-01
and 2026-10-02 (rulings 1 to 18 below). D1 landed (`4963b489`), proven
locally; D1b, mDNS-only first contact, is next, then D2.
**Scope**: Mark's machines find, reach and trust each other by device
identity, not by address: the stack's own peers already do on one network;
SSH, the path Mark uses daily, does not. Pairing a device becomes one
ceremony. A self-hosted relay covers the second network.

**Related**:

- [reachability rungs and privacy lanes plan](2026-08-03_reachability_rungs_and_privacy_lanes_plan.md):
  the shape (Syncthing's: identity is the key, addresses are disposable
  hints), R0 local mDNS and R1 cached dial hints, both landed. This plan is
  its SSH and pairing rung.
- [SSH CA projection plan](../../dramatis_docs/implementation_strategy/2026-08-12_ssh_ca_projection_plan.md):
  personae's SSH certificate authority; its host-certificate half was never
  built (§4).
- [djinn family resident services plan](2026-08-22_djinn_family_resident_services_plan.md):
  the resident that runs the agent, personal sync and Knot, and would run the
  directory and the SSH-by-key service.
- [device-grant delegation reconciliation](../technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md):
  the delegation grammar that ruling 6's CA approval rides.

---

## 1. Why

Mark's machines take DHCP addresses on a `/22` mesh Wi-Fi, and the leases
move. On 2026-10-01 the ThinkPad had moved from `.28` to `.32` (and `.28` had
become an iOS device), Q-PC from `.44` to `.68`, and finding them took a
sweep of the `/22` for port 22 and a hand comparison of host-key
fingerprints against `known_hosts`. The ThinkPad does not advertise over
mDNS at all: avahi runs with no service files, and its own name did not
resolve. One iMac sometimes sits only on a separate wired network with no
route to the mesh. Mark: "these are not static ip addresses. Is there a
coordination strategy we can employ to pair devices easily?"

## 2. What exists (assessed 2026-10-01)

From a read-only research lane, with the load-bearing claims re-checked in
code:

- **Four unrelated device identities.** personae's wallet `DeviceRoster`
  (`crates/dramatis/personae/src/carry/mod.rs:350`, device id, Ed25519
  public key, label, mode, exposure, no address field); the personal-graph
  node id that djinn's `PairedDevice` keys on
  (`ports/djinn/src/settings.rs:468`), derived per graph; Knot's own
  `PairedWriter` key; and SSH's device id, a hash of the hostname
  (`crates/dramatis/personae/src/enroll.rs`).
- **R0 and R1 work for the stack's own peers only.** djinn refreshes each
  paired device's `last_endpoint` hint every 5 s while connected
  (`settings.rs:511`, `:673-688`). The live copy is in-process and the
  saved copy is an iroh ticket in a settings file: nothing outside the
  resident can ask "where is device X now?". `relay_urls` default empty
  (`settings.rs:462`), so off-LAN reach is unconfigured.
- **SSH uses none of it.** No code generates ssh config, a `ProxyCommand` or
  a `KnownHostsCommand`.
- **Host certificates were never built.** `SshCertAuthority::mint_host_cert`
  (`ssh_ca.rs:276`) has no caller outside tests (`ssh_ca.rs:646`,
  `tests/ssh_ca_live.rs:146`); `personae-vault enroll-host` writes one
  `cert-authority` line into the target's `authorized_keys` and prints
  "host-key prompts still apply" (`bin/personae-vault/certs.rs:185`).
- **The CA is per vault.** It derives from the profile master
  (`certs.rs:34`), and a fresh vault generates its master at random
  (`bootstrap.rs:135`); there is no master export or import.
- **The machines today** (read 2026-10-02 from each running agent's public
  identities with `ssh-add -L`, no vault unlocked):

  | Machine | personae agent | Offers a certificate | CA in use |
  |---|---|---|---|
  | Windows laptop | running | yes | `SHA256:bKHWIGTTJcAq9kdCdbiY0b/SpP/kkfp54REdZrpd/W8` |
  | M4 iMac | running | no: the bare key `mark-ik@imac-2026-08`, an agent older than certificates | none |
  | ThinkPad | not installed | — | none |
  | Q-PC | not installed | — | none |

- **Pairing today** takes six steps and two 128-hex copy-pastes for personal
  sync (`djinn --pairing-facts`, `--pair-node`), a separate Knot
  `--pair-writer`, and a third SSH ceremony that needs a working login and a
  TOFU prompt before `enroll-host`. pandect's wallet-grant pairing (ticket,
  typed code, 6-digit SAS, `crates/system/pandect/src/wallet_grant/pairing.rs`)
  has no caller.
- **DNS-SD (djinn F3) is unbuilt** and would not help SSH: service records
  are link-local claims, not identity.

## 3. Rulings

Mark's answers, from multiple-choice rounds; each is the option label quoted
verbatim unless marked as his free text.

**Ruling 1.** *Take up "pair devices by key, not address" now?* Options:
assess it as its own objective; host certificates only; a router stopgap now;
note it and move on. Mark: **"Assess it as its own objective
(Recommended)"**. Follows: this plan.

**Ruling 2.** *Which identity does "device X" resolve to?* Options: the
personal-graph node id; the wallet `DevicePublicKey`; a new device-presence
key. Mark: **"Personal-graph node id (Recommended)"**. Follows: the key
djinn already refreshes hints for is the one SSH dials.

**Ruling 3.** *How does SSH reach a device by key?* Options: a `ProxyCommand`
via the local resident; generated ssh config from cached addresses; stable
names only; trial iroh-ssh first. Mark: **"ProxyCommand via the local
resident (Recommended)"**. Follows: phase D3.

**Ruling 4.** *Which relay reaches the other network?* Options: a
self-hosted iroh-relay; n0's public relays; leave the other network out.
Mark: **"Self-hosted iroh-relay (Recommended)"**. Follows: phase D6; where
it runs is open.

**Ruling 5.** *How are host-key prompts retired?* Options: stable names now,
pinning next, certificates later; finish host certificates now; pinning now.
Mark: **"Finish host certificates now"**. Follows: phase D4 is not deferred.

**Ruling 6.** *Which CA shape?* Options: trust every machine's CA; one
designated CA machine; one shared master. Mark (free text): **"how about
each machine has it's own ca, and you can use your master to determine
which ones can sign host certificates (trusted)."** Follows: per-machine
CAs, approved by one master. *Reading, not ruled*: OpenSSH cannot chain
certificate authorities, so the master's approval is an insigne
`SignedDelegationCertificate` per machine CA ("may sign host
certificates"), and a client-side step checks each approval (and its
revocation) and writes one `@cert-authority` line per approved CA.

**Ruling 7.** *How is each machine's CA read as evidence?* Options: Mark runs
`personae-vault ca` on each; I run it over SSH. Mark: **"I run it over
SSH"**. Follows: §2's table. *Reading, not ruled*: `personae-vault ca` opens
the vault, which on macOS and Linux needs the passphrase (no OS unlock
backend there), and taking a passphrase from a keychain is not done; the
CAs were read instead from each running agent's certificate with `ssh-add
-L`, which unlocks nothing.

**Ruling 8.** *What should pairing a new device be?* Options: one ticket and
SAS recording everything; one ceremony for sync only; keep the separate
ceremonies. Mark: **"One ticket + SAS, recording everything
(Recommended)"**. Follows: phase D5.

**Ruling 9.** *How does a resident run on the ThinkPad and the iMacs?*
Options: extend djinn with a systemd unit and a launchd plist; a lighter
presence daemon. Mark: **"Extend djinn: systemd unit + launchd plist
(Recommended)"**. Follows: phase D2.

**Ruling 10.** *Whose master is the root that approves machine CAs?*
Options: the Windows laptop's vault master; a new dedicated root; the M4's
vault master. Mark: **"The Windows laptop's vault master
(Recommended)"**.

**Ruling 11.** *Where does "where is device X now?" live?* Options: a djinn
app-door route and CLI; reading the settings file directly. Mark: **"A
djinn app-door route + CLI (Recommended)"**. Follows: phase D1, on the
`djinn-site` pattern.

**Ruling 12.** *Is the Windows laptop an SSH target in scope?* Options: not
now; yes. Mark: **"Yes"**. Follows: it needs Windows' OpenSSH Server
feature (an administrator step), and D3 and D4 include it.

**Ruling 13.** *Where does the plan live?* Options: a new plan in
`mere_docs`; extend the reachability plan; `dramatis_docs`. Mark: **"A new
plan in mere_docs (Recommended)"**. Follows: this file, linked from the
reachability and SSH CA plans.

**Ruling 14.** *No machine runs djinn yet, so D1's directory has no paired
peers to list on real machines. How does D1 proceed?* Options: build D1 now
and prove it locally; do D2 first. Mark: **"Build D1 now, prove it locally
(Recommended)"**. Follows: D1 is proven with two djinn instances on one
machine; its real-machine receipt moves after D2.

**Ruling 15.** *On Windows, moving to djinn retires the legacy
`graphshell-device-host`, which serves Mark's SSH agent today. How?*
Options: side by side, then Mark switches; I switch it in D2; leave Windows
on the legacy host. Mark: **"Side by side, then you switch
(Recommended)"**. Follows: D2 tests djinn on its own pipe beside the legacy
host until its agent signs a real SSH login; then Mark runs djinn's
installer, which retires the legacy task; rolling back re-enables that
task.

**Ruling 16.** *Two residents on one machine never made first contact by
mDNS alone (each learned the other's address and neither dialled; a ticket
connected them in 3.45 s), while in August mDNS-only first contact worked
across two physical machines. What is first contact?* Options: the pairing
ceremony's ticket, measuring mDNS in D2; fix mDNS-only first contact before
D2. Mark: **"Fix mDNS-only first contact before D2"**. Follows: phase D1b.

**Ruling 17.** *Keep D1's names, route `device-directory-v1` and CLI
`djinn-devices`?* Options: keep them; `paired-devices-v1` and `djinn-peers`.
Mark: **"Keep device-directory-v1 / djinn-devices (Recommended)"**.

**Ruling 18.** *Which app label may read the directory?* Options: a new
`djinn` label only; also grant turnstone; reuse `knot-editor` as djinn-site
does. Mark: **"A new `djinn` label only (Recommended)"**.

Also given in the same conversation (2026-10-01, Mark: "You can edit known
hosts"): `known_hosts` entries may be updated, which was done for the
ThinkPad (`.32`) and Q-PC (`.68`, `q-pc.local`), each key added only after
its fingerprint matched the machine's recorded key.

## 4. Corrections to other documents

Found by the assessment, each checked against code; carried into those
documents the same day as dated corrections.

1. **SSH CA plan.** Its T2 and design text say `enroll-host` writes a host
   certificate and retires TOFU fleet-wide; the code writes only an
   `authorized_keys` `cert-authority` line, and nothing mints or installs a
   host certificate. Its premise that one master is the fleet authority does
   not hold either: each vault's master, and so its CA, is its own.
2. **Reachability plan, R1.** It places `PairedDevice` in Graphshell's
   `device_sync`; it now lives in djinn (`ports/djinn/src/settings.rs:468`).
3. **Reachability plan, R2.** It says announces already bind authenticated
   app data; the code sends none, deliberately, so an announce fits a
   255-byte LoRa frame (`crates/murm/transport/src/reticulum_transport/announce.rs:38-51`),
   which also constrains R2's proposed `EndpointAddr` payload.

Lane-reported and not re-checked here (code comments, recorded for the
phases that touch them): `ports/djinn/src/pairing.rs:46-48` says the host
never writes the settings file back, while `personal_sync.rs` writes hints;
`crates/murm/transport/src/peer_id.rs:11-14` says a `PeerID` derives from the
master, while hosts pass per-graph seeds; `djinn --pairing-facts` computes
the group pre-key and omits it from its output (`ports/djinn/src/bin/djinn.rs:405-439`).

## 5. Phases

Each phase lands verified, as the chatelaine plan's lanes did. Nothing here
changes personae's agent path in the same slice as anything else: it is how
Mark SSHes into his machines.

- **D1 — the directory.** A djinn app-door route lists each paired device:
  its node id, label, whether it is connected, its current path and its last
  hint; a CLI reads it (ruling 11). Proven locally first (ruling 14). Done
  when:
  - [x] with two djinn instances on one machine (separate profiles, paired to
        each other), the output matches the resident's peer directory, and a
        peer restarted on a new endpoint shows its new path within one poll;
  - [x] a stopped peer shows as not connected (a negative control);
  - [x] only the owner's processes are admitted to the route;
  - [ ] after D2, the same holds across real machines when a lease moves.

- **D1b — mDNS-only first contact (ruling 16).** Two paired residents that
  share a network find and connect to each other with no ticket, the way
  R0's August receipts did across two machines, including two residents on
  one machine. Diagnosed before it is fixed: the first evidence points at
  p2panda's gossip healer re-joining only when a topic's set of nodes
  changes, not when a known node gains an address. A fix inside the
  `mark-ik/p2panda` fork (pinned by tag, `mere-p2panda-net-0.7.4`) means a
  new tag and a workspace repin, so where the fix lives comes to Mark first.
  Done when:
  - [ ] the cause is shown in the code and reproduced, with the evidence
        recorded in §6;
  - [ ] two residents on one machine, paired with no ticket and with mDNS
        their only way to meet, connect, and keep doing so across restarts
        (a control without the fix fails the same run);
  - [ ] the same holds between two real machines, which D2 makes possible.

- **D2 — a resident on every machine.** djinn runs as a systemd user unit on
  Fedora and a launchd agent on macOS (ruling 9), beside its existing Windows
  installer; on macOS, LAN discovery additionally needs a signed app
  carrying the local-network usage declaration (reachability R0), while
  dialling out does not. Done when:
  - [ ] each of the four machines runs the resident at login, its personal
        sync paired with the others;
  - [ ] the M4's agent is the current one (its agent predates certificates),
        with Mark's own SSH into each machine unaffected throughout;
  - [ ] on Windows, djinn runs beside the legacy `graphshell-device-host` on
        its own pipe until its agent signs a real SSH login, and then Mark
        switches with djinn's installer (ruling 15).

- **D3 — SSH by key.** `ssh <device>` runs a `ProxyCommand` helper that asks
  the local resident to dial the device's node id on its own ALPN; the
  target's resident admits paired devices only and forwards to its local
  `sshd` (rulings 2, 3, 12). Done when:
  - [ ] `ssh <alias>` reaches each machine after its address changed, with
        no sweep and no new prompt;
  - [ ] an unpaired peer, and a peer after unpairing, is refused;
  - [ ] a forged host key still fails;
  - [ ] each machine is reached as a target, the Windows laptop included,
        once its OpenSSH Server is installed (an administrator step, Mark's).

- **D4 — host certificates, approved by the master (rulings 5, 6, 10).**
  Done when:
  - [ ] each machine's own CA signs that machine's host certificate, through
        a new mint-and-install path (installing it is a root or
        administrator step on each host, Mark's);
  - [ ] the Windows laptop's master issues an approval for each machine CA,
        and revoking one removes its trust everywhere within one refresh;
  - [ ] each client's `known_hosts` carries one `@cert-authority` line per
        approved CA, written from checked approvals;
  - [ ] a host's first contact shows no prompt, and a host key that is not
        certified by an approved CA is refused.

- **D5 — one pairing ceremony (ruling 8).** One ticket, typed code and SAS
  records a new device for personal sync, Knot and SSH trust (its machine CA
  submitted for the master's approval); unpairing removes all three. Done
  when:
  - [ ] pairing a fresh device takes one ceremony, its step count measured
        against today's six plus two separate ceremonies;
  - [ ] a wrong code fails (a negative control).

- **D6 — the other network (ruling 4).** A self-hosted iroh-relay both
  networks can reach, set on every resident. Where it runs is a fork for
  Mark at this phase's start. Done when:
  - [ ] the M4 on its wired-only network is reached by key through the relay;
  - [ ] a control isolates the relay as what carried the dial (direct paths
        blocked), which is also the reachability plan's open R1 receipt.

## 6. Findings

**2026-10-01.** The ThinkPad's lease moved and it does not advertise over
mDNS (absent from a `_ssh._tcp.local` browse while both iMacs answer); avahi
runs with no service files; firewalld's `FedoraWorkstation` zone opens UDP
1025-65535, which includes 5353, so its firewall is not the block. Q-PC's
lease moved too. Both were confirmed by host-key fingerprint before any
`known_hosts` change.

**2026-10-02: no machine runs djinn.** The Windows laptop's SSH agent is
served by the legacy `graphshell-device-host.exe` (built 2026-08-16, in
`AppData\Local\Graphshell\bin`, its log showing the session's signing
requests), the resident djinn's installer is written to retire
(`ports/djinn/install-windows.ps1:18`). The ThinkPad, M4 and Q-PC run no
djinn. A first check reported djinn running on the ThinkPad: `pgrep -f djinn`
had matched its own command line, which contained the word; `pgrep -a` and
`systemctl --user` showed nothing. Instruments that match on command lines
need a control that excludes themselves.

**2026-10-02: D1 landed.** Built as `e15b8fcb` (lane, Opus) and merged onto
`main` as `4963b489` after verification in the normal-depth worktree. The
transport gained `peer_paths` (every address the endpoint holds for a peer,
each marked active or not) and a ticket decoder, so nothing outside it parses
an iroh ticket; djinn serves `device-directory-v1`, a read-only route listing
each paired device (node id, label, root, pairing id, time added, connected,
reachable, current path, the saved hint decoded), granted to the `djinn`
label only; `djinn-devices [--json]` reads it. Verified: djinn 84 unit tests
plus its integration targets, `mere-transport` 49, graphshell's library 191,
the portable gate; and the two-instance live receipt, run again by me
(temporary profiles and pipes, the installed resident PID 53336 before and
after): first contact 2.2 s after the second resident started, given a
ticket; a restarted peer's new path in 56 ms and its saved hint 1.6 s later;
a stopped peer shown not connected after 73.5 s, the transport's path
timeout. The lane's refusal test admits the granted label and refuses
Turnstone, an unknown app and an old protocol hello; its three controls each
failed their target.

Findings from D1, for D1b and D2:

- **mDNS alone made no first contact between two residents on one machine**:
  each learned the other's address and neither dialled for 45 s; with a
  ticket they connected in 3.45 s. Ruling 16 makes this D1b.
- **Saved hints accumulate**: after a restart a hint holds the old and new
  ports, and includes the laptop's WSL/Hyper-V adapter address
  (`172.28.32.1`) and global IPv6 addresses; existing R1 behaviour.
- **A local djinn cannot share the standard agent pipe** with the installed
  resident: its listener accepts only `\\.\pipe\openssh-ssh-agent`, so local
  instances need `--receipt-agent-endpoint`; D2's side-by-side run on
  Windows (ruling 15) needs the same.
- **A third node on the LAN**, `9b662f09…`, sends transport info the current
  p2panda rejects. That id is the supplier in August's personal-sync
  receipts (reference host plan, reachability plan R1), so it is most likely
  the installed legacy resident's own sync identity on this laptop
  (*reading, not verified*).

## 7. Progress

**2026-10-02.** Assessed by a read-only lane (Sonnet) and ruled in three
rounds (rulings 1 to 13); the load-bearing claims re-checked in code; the
machines' CAs read from their agents. Nothing built. Next: Mark's go to
start D1.
