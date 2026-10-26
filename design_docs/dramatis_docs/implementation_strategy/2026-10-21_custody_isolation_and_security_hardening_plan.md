# Custody Isolation and Security Hardening Plan

**Date:** 2026-10-21 (America/New_York).
**Status (2026-10-22):** H1 production migration remains gated. Native11 passed 38/38 synthetic checks in the separate non-admin runtime account; the measured same-user signing attack failed. Native10 remains a failing counterexample for the signing account. Production Castellan, consent/policy and lifecycle integration remain open; no real consumer has migrated.
**Audit base:** Mere 0261e2602b5b13fcf59dced3ae408c3441c38a71, freshly fetched
origin/main; Dramatis 7d498c27df8076dacccb74ba4a05f863f9f48c76, the exact Git
revision in Mere's lockfile. This frozen-source assessment does not qualify
later main, the installed fleet, or every build feature.
**Direction:** OS-launched custody, identity-restricted sealing, and a shared
worker confinement baseline; isolate custody first and require enforcement
evidence before consumer migration.

This is the canonical security continuation of the family-promotion plan's
[custody design scope](../../mere_docs/implementation_strategy/2026-10-19_family_promotion_plan.md#separate-design-pass-custody-process-boundary).
Promotion owns extraction and receiving cohorts; this plan owns containment,
authorization and security acceptance. Existing identity and storage formats
must survive the transition.

## 1. Evidence and threat model

Every claim uses one of these evidence classes:

- **Source-confirmed:** behavior or a missing check is visible in the frozen
  source. This does not establish an installed exploit.
- **Executed:** a retained synthetic diagnostic exercises the identified
  production components. Reproducing unsafe behavior is not a passing security gate.
- **Needs testing:** an attack chain, OS property, deployment condition or
  broader guarantee not established here. Historical receipts retain their
  original source, feature and platform limits.

The assets are persona masters, private wallet epochs/device seeds, credentials
and OTP seeds, wrapping material, authority to sign/delegate, application keys,
private content, and trustworthy consent.

| Adversary | Required boundary and present limit |
|---|---|
| Untrusted document, model, peer or protocol bytes | A flaw in a general worker must not expose roots. No such exploit was demonstrated here; those components can share Djinn's address space today. |
| Compromised native app under the owner's OS account | Cannot obtain the custodian's sealing key or unlocked memory, replace its executable/launch policy, impersonate an admitted principal, or approve itself. Requires both §5.2 isolation layers; worker sandboxing alone does not cover this adversary. User-authorized disclosure, a compromised signing/update authority, administrator and kernel are outside this guarantee. |
| Compromised Djinn broker/worker | Must not extract roots, mint unrestricted client authority or answer custody consent. A socket to an unrestricted child would preserve these problems. |
| Disk image, crash dump or suspended image | At-rest sealing and memory/dump policy are separate protections. No fresh disk-theft, swap, dump or hibernation experiment ran. |
| Administrator/kernel, compromised OS or replacement of trusted custody | Outside the guarantee of this split unless a particular OS deployment demonstrably constrains it. Do not promise universal same-user-malware resistance. |

Scope includes Djinn/Castellan, Graphshell local admission/client/framing,
Dramatis's native app and pinned portable contracts, root/epoch derivation,
SSH approval, locking, import/review, optional resident compute, service launch
and related plans. This is not a full crypto audit, dependency-advisory campaign,
network penetration test or fleet qualification. Diagnostics use fresh synthetic
vaults and explicit temporary sockets: no personal vaults, peers, service changes
or native dialogs.

## 2. Current boundary map

| Boundary | Implementation and authority |
|---|---|
| Dramatis window → custody | The current execute_at flow opens label dramatis, requests public state, selection and resident-owned authentication/review. It creates no vault or passphrase file. A window gesture is not independently authenticated to custody. |
| App/browser relay → local door | Unix sockets use 0600; Windows checks the client's user SID. An app name selects a route but does not identify a binary. Browser extension allowlisting is a separate gate and does not stop arbitrary same-user native code. |
| Door → root authority | The broker calls Keeper, which holds PersonaeHost and credential authorities. Custody bypasses ordinary carrier admission so locked/public calls can answer. The door is trusted today. |
| Custody → general resident | Castellan links into Djinn alongside sync, Knot, sites and optional model/trainer services. cargo-deny constrains source dependencies, not memory access. |
| Root → key consumer | Release and ReleaseEpochKeys return real key bytes. Some door/transport derivations intentionally persist while locked; other consumers cooperate with watches to forget keys. |
| Caller → signing approval | Pending UUIDs and actions are projected. IdentityEndpoint forwards an advertised signing intent to PersonaeHost and the approval broker. Native-only metadata is not proof of a human act. |
| Vault lock → holders | ResidentLock locks the vault, synchronously calls registered holders and broadcasts a two-state value. Unregistered copies and out-of-process clients are outside synchronous completion. |
| Custody → OS authentication/storage | Native prompts currently belong to Djinn. Windows AutoOs uses DPAPI; passphrase formats use existing Argon2id/AEAD. External native password helpers are part of the authentication trust chain. |

Primary source anchors:

- [admission](../../../crates/graphshell/graphshell-endpoint/src/door/app_admission.rs),
  [local transport](../../../crates/graphshell/graphshell-endpoint/src/door/local_endpoint.rs),
  [app client](../../../crates/graphshell/graphshell-endpoint/src/door/app_client.rs),
  [framing](../../../crates/graphshell/graphshell-endpoint/src/door/frame.rs);
- [broker](../../../ports/djinn/src/door/app_broker.rs),
  [keeper](../../../ports/djinn/src/keeper.rs),
  [custody policy/dispatcher](../../../ports/djinn/src/custody.rs),
  [payload operations](../../../ports/djinn/src/custody/payload.rs);
- [authority/lock](../../../ports/djinn/castellan/src/authority.rs),
  [approval broker](../../../ports/djinn/castellan/src/custody/broker.rs),
  [SSH agent](../../../ports/djinn/castellan/src/custody/agent.rs),
  [identity intents](../../../ports/djinn/src/door/identity_endpoint.rs),
  [approval projection](../../../ports/djinn/src/door/identity_projection.rs);
- [resident startup](../../../ports/djinn/src/bin/djinn.rs),
  [Distillery adapter](../../../ports/djinn/src/resident_distillery.rs),
  [Distillery authority](../../../ports/distillery/src/installed.rs),
  [Dramatis client](../../../ports/dramatis/src/lib.rs).

## 3. Findings, 2026-10-21

Priorities assume the adversaries in §1; they are not CVSS scores.
The receipt distinguishes the exact executed path from broader untested consequences.

| ID | Priority | Evidence | Finding and practical limit |
|---|---|---|---|
| C01 | High | Source-confirmed | Root custody shares Djinn's address space with general services. A crate or repository split does not contain arbitrary code execution there. No network/content exploit was demonstrated. |
| C02 | High | Source-confirmed; W1 | Any same-user client reaching the app socket can claim an allowed AppId. Comparisons against dramatis, turnstone or personae-vault do not identify those executables. Unknown-label refusal is a routing control. |
| C03 | High | Source-confirmed; W1 | Default release policy is mainly a global salt/source allowlist. An admitted dramatis label can request Woodshed's sealing salt; AsProfile opens a named profile without an app-to-profile grant. Epoch release accepts persona UUID/derivation rules without a general per-principal persona grant. Exact radio/payload checks exist, but several rely on the same unverified app label. |
| C04 | High | Source-confirmed; W2 | Projected signing-approval intents resolve pending broker requests without independent native confirmation. An admitted identity client can act as approver. Native-only metadata, a known request UUID and scene freshness do not prove human consent. Quarantine review has a different, stronger confirmation path. |
| C05 | High | Source-confirmed; W1 | A released key can be copied and used after lock. A notification cannot revoke bytes. Mixed release intentionally supports continuous consumers; safe use depends on consumer trust and the authority conveyed by each key. |
| C06 | Medium | Source-confirmed; W4 | WatchLock exposes current Locked/Unlocked state rather than a monotonic generation. A delayed observer after a complete lock/unlock interval cannot learn the intervening lock. CustodyIdentity forgets keys on observed lock/loss; the actual continuously running watcher's scheduling race needs another experiment. |
| C07 | High for restart guarantees | Source-confirmed; W3 | ResidentLock logs a persisted-marker write failure and still returns success. Memory is locked but the restart barrier may be absent. The marker uses a direct write without explicit file/directory sync. Ordinary I/O failure is distinct from untested power-loss durability or an actual unattended restart. |
| C08 | Medium | Source-confirmed | macOS has no native screen-lock/suspend source here and idle() returns None. The shared sampler still treats unknown idle as idle and locks after its timeout. The gap is native event coverage; claiming explicit lock only would be incorrect. Installed timing needs testing. |
| C09 | High | Source-confirmed design; attack needs testing | connect_local and the hello/challenge do not authenticate a pinned custodian to the client. Separately, terminal request_unlock sends a passphrase over the control route: native-only unlock is not universal. A complete fake-server/control-route experiment is required before claiming credential capture. |
| C10 | Medium–High by policy | Source-confirmed | The SSH agent does not populate authenticated process/requester facts. Cache keys include optional fields, but absent facts cannot distinguish processes. Verified host/session facts help where supplied; destination constraints are not stored as key policy. Session-tier keys intentionally skip per-request approval. Approval behavior across lock requires explicit policy/tests. |
| C11 | Medium | Source-confirmed missing budgets; exhaustion needs testing | Frames cap at 1 MiB, but the local accept loops spawn per-connection tasks without a shared admission budget or handshake/read deadline. CXF commands read caller-named files before parsing. Per-frame bounds do not bound aggregate connections, pending prompts or file reads. No exhaustion or arbitrary-file exfiltration was demonstrated. |
| C12 | Platform-dependent | Source-confirmed exclusions; deployment needs testing | The pinned threat statement excludes in-process code, same-user DPAPI unwrapping, OS-permitted unlocked memory inspection, swap/hibernation and dumps. Linux's installer does set LimitCORE=0 and NoNewPrivileges=true. Those controls do not establish all-platform memory/process protection. |
| C13 | Medium | Source-confirmed allocation paths; residue needs testing | Generic frame read/write uses ordinary Vec buffers and serde_json serialization for secret-bearing custody responses. Base64 helpers also create ordinary encoded/decoded allocations; PayloadBytes redacts Debug but has no clearing Drop. ReleasedKey/ReleasedEpochKey clear their seed fields, but that does not clear serialized copies. No heap/stack recovery experiment for these IPC paths ran. |

C03's operation inventory also includes ChooseProfile, CreateProfile,
IssueStationGrant and RevokeDevice. The dispatcher lacks a complete
authenticated-principal policy for these administrative acts. Public output
does not make an act unprivileged; a confirmed boolean on a different interface
is not trusted presence.

### Controls to preserve

- Empty/master salts and unknown derivations refuse. Retained payloads add
  profile/master/AAD and content-hash checks under current-profile authority.
- SSH checks lock again after waiting for approval; per-use without a broker
  refuses. RSA SHA-1 and unsupported key forms refuse.
- Quarantine consent belongs to the resident; exact revision/profile ownership
  are rechecked after the prompt. Cancellation and unsupported forms stay held.
- Passphrase replacement requires current credentials and checks lock at commit.
  Wrapped-root and portable cold-reopen receipts exist.
- Credential holders share lock state; restore failure relocks the vault.
  Ordinary clients treat absent/Locked as pending rather than invent keys or
  fall back to unsealed writes.
- OTP's admitted-session adapter binds exact credential, authenticated
  participant, transcript, expiry and revocation. This narrower pattern is a
  reference, not a gate automatically enforced on every custody call.
- Secret Service remains parked. Its optional implementation/disposable-bus
  tests do not mean Djinn has become the installed keyring daemon.

## 4. Diagnostics and experiments

The [receipt directory](../../../support/receipts/custody-security-20261021/)
retains synthetic diagnostic source, a reproduction driver, output, and frozen
source hashes. W1–W4 intentionally assert CURRENT weaknesses. They must fail at
the identified unsafe assertion after repair while authorized controls succeed.

**Diagnostic execution (macOS):** all four witnesses executed successfully against the frozen production code. This confirms the described weaknesses; it does not mark the security boundary safe. Exact commands, controls and hashes are retained in the qualification record.

Existing controls were run independently: 26 Djinn integration tests passed
(custody route/payload, quarantine review, passphrase change and locked doors;
two headed fixture hosts ignored), 41 endpoint door tests passed, and seven
lock-trigger policy/host tests passed. These controls used Homebrew Rust/Cargo
1.99.0; the witness reproduction additionally selects the repository's installed
1.99.1 compiler explicitly. The earlier ordinary custody test failure came from
a 112-byte macOS Unix socket pathname; a direct bind reproduced the path-length
error and the unchanged tests passed with TMPDIR=/tmp. This is not a new
authorization failure or an unfiltered full-suite claim. Port-boundary and
cargo-deny bans checks passed; neither establishes runtime containment.

| Witness | Actual path | Controls and scope |
|---|---|---|
| W1 | Real local listener/broker/custody client and synthetic sealed storage | Unknown app and empty/master salt refuse. The test executable claims dramatis, obtains Woodshed's key in two profiles, then keeps a usable copy after lock while a new release refuses. Same user/one machine; no personal credentials. |
| W2 | Real per-use SSH agent, identity IPC route, intent handler and approval broker | Signing is pending before the client's intent; Keeper has no native UI. A signature verifies after the ordinary client approves. Agent request uses the actual Rust Session implementation; this is not an external OpenSSH process attack. |
| W3 | Vault lock and marker persistence | A regular file prevents marker-directory creation. Lock reports success and memory locks, without a persistent marker. No power loss or restarted unlock is simulated. |
| W4 | Custody WatchLock dispatcher and actual lock/unlock | Current Locked state is observable. A delayed observer after Locked → Unlocked times out with seen=Unlocked. Direct dispatcher test; no claim about an actually scheduled remote watcher. |

| Test | Required experiment and evidence |
|---|---|
| T01 | Fake custodian on a fresh private endpoint: real status/control client, synthetic canary passphrase, endpoint override and expected endpoint. Require rejection before secret transmission. |
| T02 | Deterministically hold root/epoch/payload/signature delivery across lock, profile switch and revocation. Record linearization; no post-barrier secret/proof delivery. Disconnect before/after commitment. |
| T03 | Suspend an actual CustodyIdentity watcher across lock/unlock and restart. Stale generation must force reauthorization even if final state is Unlocked. |
| T04 | SSH pending/cache approval across lock, process change, expiry, policy change, host rebinding and forwarding. Proposed default: invalidate on lock/generation change. |
| T05 | Wrong OS user, app, profile/persona, operation, namespace, target and UI role over each real OS transport. Include malicious proxy/forwarded label and fake server. |
| T06 | Actual macOS screen lock, sleep, lid close where applicable, idle/unavailable-idle, logout, reboot and crash restart. Keep Linux/Windows evidence independent; physical sleep is attended. |
| T07 | Same-user debugger/memory/root-file access, executable/helper replacement, inherited descriptors, injected environment and dump policy on each supported installed build. Report prevention versus exclusion. |
| T08 | Slow/truncated/oversized/replayed frames, connection/prompt floods, huge CXF/files and stalled holders. Require responsive lock/status under explicit byte/time/concurrency limits. |
| T09 | Crash at every marker/passphrase replacement boundary; reboot/power-loss where required; old backups and freshness-ledger rollback. Separate old-passphrase revocation from access to an old encrypted backup. |
| T10 | Separate custodian and malicious-worker fixture: cannot read roots, approve itself, impersonate another client or restart into unattended authority. One root signature and one scoped continuous consumer first. |

### 4.1 H1 macOS synthetic fixture, 2026-10-22

The executable fixture lives in
[testing/macos-custody-h1](../../../testing/macos-custody-h1/).
It uses separately signed controller, custodian, App Sandbox XPC worker and
unrelated-app binaries, native NSXPC peer requirements and signed launch
constraints. A fresh random Ed25519 seed is sealed using CryptoKit AES-GCM and a
32-byte data protection Keychain item. Each run has unique state paths, account
and Mach service names. The dedicated App ID is
com.merelymade.djinn.h1.custodian; the emitted code entitlements contain only its
own Keychain group, with no production group sharing.

This first fixture qualifies OS primitives. It does not yet host the real Rust
Castellan vault/approval/marker implementation, qualify its buffer hygiene, or
close production C04/C07. Those receiving gates remain in H1 after the OS gates.
The existing Developer ID private key was used for approved fixture signing;
no private key was exported, no signing-key ACL was changed and no production
vault, consumer or service was migrated. The user approved the dedicated App ID
and Developer ID profile. Profile/binary hashes and consent scope belong to the
H1 receipt, rather than modifying W1–W4's historical evidence.

| Result | Evidence and exact limit |
|---|---|
| **Executed: worker confinement** | The real App Sandbox XPC worker is denied vault canary, symlink, blob and loopback connection with EPERM. The unrelated same-UID app opens those files/connects, while the worker consumes selected input and reads/writes its own container. The controller deliberately opens a non-CLOEXEC canary descriptor; the launchd-created worker finds neither that descriptor's bytes nor its path. |
| **Executed: launch and peer controls** | Custodian reports parent PID 1 and an authenticated instance. Direct and hostile-environment child launches receive SIGKILL. A separate hardened launchd probe ignores a test DYLD library; the unhardened positive control loads it. A fake server knows the correct public run ID and can capture synthetic input with pinning removed; the pinned client sends no secret. Native10 additionally activates the protected bundle through SMAppService; launchctl records ServiceManagement ownership, the expected parent bundle and a spawn constraint. |
| **Executed: ordinary-app key/memory denial** | With successful owner Keychain access/signing as control, same-team/different-app and ad-hoc apps cannot retrieve the synthetic item or call custody. The worker and unrelated app cannot obtain the live custodian's task port. An unconfined self-task-port control succeeds; App Sandbox also denies the worker's self task_for_pid call. This is one debugger primitive, not exhaustive memory/dump/side-channel resistance. |
| **Executed: sealed identity continuity** | After fixing the fixture defects below, a new custodian process reopens the same synthetic encrypted seed and verifies signatures under the same public key. This is process restart, not physical logout/reboot, storage rollback or crash-durability qualification. |
| **H1-F1 — confirmed failure: signing authority shares the desktop account** | The unrelated signed app invokes /usr/bin/codesign unattended with the existing Developer ID key, signs another fixture app with the custodian's App ID/profile and minimal group entitlement, retrieves the wrapping key and decrypts the synthetic vault. Its recovered root public key matches the legitimate custodian before and after restart. No key export or approval is needed for this measured path. **Same-user resistance fails in the signing account on this host.** |
| **Executed: protected installation** | The user ran the narrow digest-pinned installer. Native10 verifies installed artifact hashes, strict signatures and root-owned non-writable ancestors. The worker cannot open the custodian binary for writing (EPERM); the unrelated same-user app cannot open its binary, config or LaunchAgent plist for writing or create a file in the install directory (EACCES). SMAppService registration reports enabled and launchctl identifies ServiceManagement ownership. These are measured writes/launches, not an installation inference. |

The protected native10 receiving result passes 34 of its 35 explicit checks;
the remaining failed gate is same-user signing-authority isolation. The earlier
local native07 result passed 28/30, with protected installation still absent. The evaluator also requires functioning fake-server/injection controls,
correct owner key use, stable sealed identity and a real live target before
counting corresponding denials. Startup failure, ENOENT, a missing owner
entitlement, timeout or arbitrary signing failure cannot manufacture a pass.

Retained failed iterations are part of the evidence. Before provisioning, the
legitimate custodian returned -34018, so attacker Keychain errors did not count
as isolation. The first provisioned run exposed two fixture defects: listener
admission raced initial item creation, and clearing aliased rawRepresentation
Data changed the first CryptoKit key. Initialization now precedes admission and
serialization uses an explicitly allocated byte copy. The cold-reopen check
rejected the mismatched first identity; it was not waived. These defects are in
the new native fixture, not evidence of a production Castellan regression.

The native09 administrator attempt also failed closed before activation:
codesign's -R option interpreted the inline requirement as a filename because
the installer omitted its leading equals sign. The failed staging directory
was removed and no installed bundle remained. Native10 uses one shared command
builder for preparation and installation. Before requesting administrator
authentication, preparation verifies all three exact bundle identities and
confirms that a wrong identity receives a requirement-mismatch rejection.
The native09 failure and correction are retained separately; its frozen
manifest and installer source have not been rewritten.

Receiving logs, source cuts and exact fixture metadata are retained in
support/receipts/custody-h1-macos-20261022. Native10's synthetic item deletion
and SMAppService unregister succeeded; its custody, fake and hostile-probe jobs
are no longer registered. The root-owned fixture bundle remains as evidence. Profile material and signed binaries
stay outside versioned source; receipts retain hashes and bounded public facts.

### 4.2 Separate-account receiving gate passed, 2026-10-22

The user authorized the next experiment in a separate macOS account without
access to the signing key. Native11 ran successfully in the new local standard
account djinnh1 (UID 503), separate from the signing account (UID 501). Its fresh signed bundle has its own run ID and state paths under
that account's home. The root-owned installer publishes only public fixture
inputs and receiving tools, never writes a user's home and never executes a
fixture as root. The account creates its own synthetic vault during its normal
graphical login; no real vault, signing key or consumer moved accounts.

The generated setup commands create the account with an interactive password
prompt and install a digest-pinned public receiving package. The root installer
refuses an admin receiver, the builder's UID or an unexpected home. Instructions
are also generated in the installed protected directory, so the receiver does
not need to open the signing account's home. Its unprivileged bootstrap verifies
the exact manifest/artifacts before copying the synthetic inputs.

Preparation verified all three exact installed-role requirements and the wrong
identity control. A disposable copy of the exact unsigned attacker payload was
successfully signed/verified on the signing workstation. The live receiving
preflight rejected the builder account. All 42 evaluator/installer/context tests
pass, including missing GUI session, sudo session, admin membership, changed
payload control and absent-file counterexamples. Those preparation controls
remain separate from the subsequent receiving result.

The receiving run passed **38/38 mandatory checks**. The unrelated app's
codesign subprocess returned exit 1 with "The specified item could not be found
in the keychain", without timing out. The same attacker payload had signed and
verified successfully on the signing workstation. Opening that account's
default Keychain file was denied with EACCES. Direct custody-key lookup returned
-34018, the app could not call custody or obtain its task port, and no attacker
key/root was recovered. The forged-identity stage was not launched because its
signing step failed. The legitimate custodian accessed its 32-byte key,
signed successfully, and a new PID/instance reopened the same sealed root under
the same public key. Worker confinement, protected-file denials, independent
SMAppService launch and fake-peer controls also passed.

Review verified all 45 root-owned public payload artifact hashes, eight frozen
source hashes, account/run identity, native stdout versus parsed values, and
recomputed the evaluation from the frozen runner. The exported JSON is owned
by UID 503; its SHA-256 is
98933bc94f478e3c6d818e16fdad377b4898e727f6901efb0268eb37550bffca.
The receiving process reported successful exact-item deletion and SMAppService
unregister. Both recorded custodian PIDs are absent. The builder session's
cross-account launchd domain query is denied, so its service-query errors are
not counted as independent proof that jobs were unregistered.

The receiving receipt is
support/receipts/custody-h1-macos-20261022/native11-receiving-qualification.json;
the earlier separate-account-preparation.json remains a historical preparation
receipt. H1-F1's measured runtime-account gate passes for UID 503. The signing
account's native10 escape remains reproducible evidence; running real consumers
there with the same available signing key would fail the deployment gate.
Production deployment must preserve this separation and repeat qualification
with the actual custodian/consumer packages. Rust Castellan integration, scoped
IPC/consent, C04/C07 and lifecycle qualification remain open. A passing native
primitive fixture authorizes no real-consumer migration.

This closes the measured signer-oracle path for UID 503, not every possible
same-user attack. Copied/relaunched signed bundles, altered bundle resources or
configuration, explicit cross-account Keychain API variants and signed-update
replacement are not established by these 38 probes. Bind production startup,
configuration and authorization to the protected service and test those
variants before extending the claim. Full memory/residue, crash durability,
physical sleep/logout/reboot and resource-exhaustion limits also remain open.

## 5. Architecture decision: isolate custody first

A separate **custodian process** is the only production owner of Castellan
custody. This role does not imply a new repo or a second store. Preserve the
vault, wallet, credential formats and derivation identities inside any added
local storage envelope. Each direct opening has one replacement path; no
in-process fallback when IPC fails.

~~~mermaid
flowchart LR
  D[Dramatis public UI] --> I[Authenticated local custody IPC]
  A[Admitted application] --> I
  O[OS service manager and protected install] --> C[Independent custodian]
  J[Djinn client and confined workers] --> I
  I --> C
  C --> P[Custodian policy and operation gates]
  U[Trusted consent and OS presence] --> P
  P --> V[Vault, wallet, credentials and root signing]
  P --> R[Scoped result or reviewed key release]
~~~

A compromised broker must not inherit all application permissions. Custody must
verify independently established principal/session facts rather than a forwarded
app name. Authentication, operation permission and human consent are three
separate decisions.

### 5.1 Ownership and dependency cut

Keep vault bootstrap, private wallet state, credential/freshness keys, root
operations, SSH policy and native authentication/consent inside custody.
Keep rendering, model loading/execution, sync protocols and document processing
outside. Allow only narrow bounded local/SSH adapters; no general network
listener, shell command or arbitrary file-reading API in custody.

| Current source | Migration requirement |
|---|---|
| Djinn startup and startup_vault | Custodian alone obtains unlock credentials and opens root storage; supervisor receives public status/scoped session. |
| Keeper and resident credential catalog | Every profile's credential/freshness keys stay inside; return public snapshots rather than references/providers. |
| resident_distillery::open_mesh | Remove the second vault open. InstalledAuthority already accepts an identity provider and declares two release salts. Supply reviewed derived material or operations. Its consumed startup authority is not evidence that a second root vault stays live forever. |
| resident_knot, epoch_keys and retained payload | Read epochs inside custody. Prefer seal/unseal/sign operations; explicitly review continuous writer/vault-key exceptions, preserving bytes and AAD. |
| Inference journal/library and staged holders | Derive inside custody; prompts, tensors and models stay outside. Close private-work admission immediately and bound drain. No GPU/model dependency in custodian. |
| Door keys and transport authors | Inventory exact derived keys intentionally usable while locked, their authority and identity/rotation consequences. Preserve peer identity unless an explicit migration changes it. |
| VaultCommand, CXF and SSH import | Replace string commands/paths with typed bounded operations or deliberately selected handles. Isolate hostile parsing where practical without disclosing a raw credential corpus to general workers. |

Narrow cargo-deny's production wrapper to the custodian; general Djinn cannot
normally link Castellan. Validate the actual normal dependency graph and compile
negative consumers. Keep public wire/policy data free of custody implementations.
Receipt hosts must remain synthetic and absent from release packaging.

### 5.2 Enforcement and independent launch, decision 2026-10-22

Choose two separate controls. **Worker confinement** prevents a compromised
parser/compute/ingress worker from opening vault or blob-root files or inspecting
other processes. **Custodian protection** prevents an unrelated, unsandboxed
application in the desktop account from using the storage key, reading unlocked
custodian memory, or substituting its launch. A sandbox applied to our workers
cannot restrict that unrelated application.

The production defaults are below. Start implementation on macOS, the current
audit host; Linux and Windows use their stated defaults when those deployment
modes are enabled. A missing mechanism fails activation for the protected mode;
it does not silently select an unconfined sibling or weaker sealing. H1-F1 adds
a separate deployment gate: the desktop account must not possess unattended
authority to mint the trusted custodian identity.

| Platform | Worker confinement choice | Custodian launch, storage and process protection choice |
|---|---|---|
| macOS | Individually signed **App Sandbox XPC workers**, with a common minimal entitlement baseline and role-specific additions. No vault/blob-root bookmarks, shared data container or custody keychain group. Hand over selected bytes/handles only. | **launchd-managed per-user LaunchAgent**, registered with SMAppService, in an administrator-owned signed bundle. Require the expected launchd parent with signed launch constraints and pin the agent in the plist SpawnConstraint (protected mode requires macOS 14+); enable Hardened Runtime and library validation, omit get-task-allow and injection/debug exceptions. Seal local private custody files with a random envelope key stored in the **data protection Keychain**, using kSecUseDataProtectionKeychain and the custodian's unique, provisioning-authorized keychain access group; no group sharing with Djinn, workers or the general Dramatis window. Independently verify signed XPC peers. |
| Linux | A common worker launcher applies **Landlock + seccomp + no_new_privs before hostile input**, closes ungranted descriptors, and constrains process inspection. Use systemd system-service ProtectSystem=strict, ProtectHome=yes, NoNewPrivileges=yes and DevicePolicy=closed with explicit role additions; offline parser/compute roles use PrivateNetwork=yes and deny socket creation beyond the granted IPC. Probe the required Landlock ABI/access rights and refuse protected activation if absent. | A **systemd system service under a dedicated non-login custodian UID**, with root-owned unit, executable and configuration; no desktop-user-writable overrides. Custodian files and a local envelope key live in service-owned directories outside the desktop home, inaccessible to the desktop UID. Keep existing passphrase AEAD inside that envelope; set non-dumpable, LimitCORE=0 and restrict proc/ptrace access. This separate OS identity supplies the same-desktop-user boundary; a user unit under the desktop UID does not. |
| Windows | **AppContainer workers** with individual container identities, a shared default-deny capability baseline, explicit file/pipe ACL grants and a bounded Job Object. Parser has no network capability; device/GPU and network grants are role-specific. A Job Object alone is resource control, not the access boundary. | **SCM-managed service under a dedicated virtual service account/service SID**, outside the interactive user's token. Protect service configuration, executable, vault and key ACLs from that user. Use a persisted **CNG wrapping key with a DACL restricted to the custodian service identity**; the software KSP is the baseline, TPM-backed storage an optional stronger hardware property with separate tests. Restrict pipe and process access; verify the server and admitted container/service principals. |

These are mechanism choices, not interchangeable labels. Apple documents
[per-target App Sandbox/XPC isolation](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/EnablingAppSandbox.html),
[launch-environment constraints](https://developer.apple.com/videos/play/wwdc2023/10266/),
[SMAppService installation](https://developer.apple.com/documentation/ServiceManagement/updating-your-app-package-installer-to-use-the-new-service-management-api),
and [data protection Keychain access groups](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains).
Use the custodian's application-specific group and a distribution provisioning
profile, not an unrestricted macOS app-group name; Apple's
[distribution signing guidance](https://developer.apple.com/documentation/xcode/creating-distribution-signed-code-for-the-mac/)
explains restricted entitlement authorization. Ordinary legacy Keychain ACLs
with expandable trusted-app prompts are not this selected policy. Secure Enclave
backing may add non-exportability, but is not required to preserve existing
portable signing identities, and is not proof of caller identity or consent.

Linux [Landlock](https://docs.kernel.org/userspace-api/landlock.html) restricts
ambient filesystem rights; seccomp constrains syscalls, not path authorization.
Require Landlock ABI 6 or newer for the baseline file/device rights and
abstract-socket/signal scopes, and install it in the single-threaded launcher
before libraries create threads. Private mount namespaces expose only the
manifest's IPC paths; do not assume Landlock ABI 6 restricts pathname sockets.
Already-open descriptors, Unix sockets and GPU/device ioctls require explicit
tests; newer ABI features cannot be assumed from a kernel version string. [systemd execution controls](https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html)
complement the dedicated service identity. Linux's baseline key-file ACL is
access separation, not hardware sealing or independent disk-theft protection;
passphrase/disk encryption still matter. TPM enrollment is a separate optional
decision, not a prerequisite disguised as completed work.

Windows [AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer),
[service configuration ACLs](https://learn.microsoft.com/en-us/windows/win32/services/service-security-and-access-rights)
and [CNG key security descriptors](https://learn.microsoft.com/en-us/windows/win32/api/ncrypt/nf-ncrypt-ncryptsetproperty)
are distinct controls. [DPAPI](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata)
under the interactive user is not application-bound; machine-wide DPAPI is not
the service boundary either. TPM presence alone does not authorize a caller.

**Cost and rejected alternatives:** protected installation/update requires an
administrator-owned bundle or system-service configuration on these desktops.
There is a macOS 14+ floor and a Linux ABI probe; portable unprivileged runs do
not inherit the protected-mode claim. Independent services add lifecycle work,
but one shared worker launcher/protocol bounds it. Same-UID unsandboxed siblings,
a desktop-user systemd unit and a Djinn-spawned custodian fail the threat model.
App Sandbox is the macOS worker default rather than a custom sandbox-exec profile;
ordinary DPAPI and a hardware key without caller ACLs fail the storage boundary.

**Signing-authority gate, H1-F1:** the current Mac is both a signing workstation
and the custody test host. The explicit profile authorizes the custodian App ID;
Apple's profile contains a team wildcard for permissible Keychain groups, while
the signed binary selects its single group. An unrelated app that can use the
signing key can mint that signed entitlement. Even an exact-group profile would
not stop minting the same custodian identity. Keychain behavior here follows its
signed-identity model; this is a deployment trust-boundary failure.

Choose separate signing and custody runtime accounts/devices for a protected
deployment. Build/sign in an account or machine unavailable to the custody
desktop account, install verified artifacts through the protected update path,
and run the same unrelated-app signing-oracle test on the receiving host.
A signing key in a file, Keychain or hardware token is acceptable only if that
receiving account cannot use it unattended to authorize replacement code.
Moving/revoking the user's existing certificate or changing its ACL is outside
this synthetic slice; preserve current release workflows until that separation
is deliberately installed. A developer account that retains this authority is
an explicit development mode, and cannot pass H1's same-user guarantee.

**Launch rule:** Djinn connects to custody; it never spawns it, chooses its
executable/configuration/key location, supplies its environment, or inherits its
secret handles. Service-manager activation may be requested through the fixed
installed service identity. No caller-selected command, DYLD_*/LD_* setting,
DLL search path or writable plugin/configuration is accepted. OS-managed
restarts retain lock policy. Updates are a separate trusted, signed installation
path; Djinn has no update/reconfiguration authority. A launchd/systemd/SCM label
alone proves none of this: binary ownership, effective launch constraints,
configuration ACLs and client-side peer verification are part of H1.

Worker activation also uses fixed installed manifests and a trusted launch
adapter; Djinn can request a role, not executable paths, sandbox exemptions or
arbitrary inherited handles. The adapter reports readiness only after
confinement is active. Its small policy surface is shared across worker roles.

**Principal rule:** macOS uses signed connection requirements on both ends;
Windows uses admitted service/container identities with connection-bound token
checks. Linux SO_PEERCRED identifies the dedicated service UID, not arbitrary
applications sharing the desktop UID. Grant unattended app authority only to an
independently identifiable installed service/container principal. Ordinary
same-UID/unpackaged applications remain untrusted requesters requiring
custody-owned approval for sensitive operations. PID, pathname, AppId, a shared
readable token or a forwarded label cannot grant another application's scope.

**Storage rule:** local sealing must cover wallet, credential/epoch material,
wrappers and every usable automatic-unlock or recovery copy, not just the main
vault file. Preserve internal formats, identities and an explicitly authorized
portable backup/recovery route. Migration is transactional: verify the sealed
reopen, then retire live unsealed copies; test old backups as retained authority.
A downgrade, missing Keychain item, key ACL failure or service outage leaves
custody locked. Sealing does not protect released keys, disclosed plaintext,
a compromised custodian/update signer, or a user deliberately authorizing an
attacker. Same-user kill/denial of service is not a confidentiality guarantee.

**H1 enforcement evidence:** run a malicious confined worker and a separate
unsandboxed desktop-user application. Try direct/symlink/root-file access,
inherited directory/file handles, process-memory inspection, key retrieval/use,
service replacement/reconfiguration, hostile environment/arguments, fake server
and self-approval. Workers must be denied vault/blob-root reads even if bytes
are encrypted. The unrelated macOS app may be able to copy ciphertext; its
required denial is envelope-key use, plaintext recovery and custody authority.
Linux/Windows also require service-file ACL denial to the desktop account.
Include an allowed operation control and record effective entitlements,
accounts, ACLs and kernel features. On macOS test both wrong-team
and same-team/different-app identities, signed updates, logout/cold reopen and
Keychain denial. GPU driver access, trusted presence/UI behavior and raw/encoded
memory residue have their own receiving tests. These are T05/T07/T10 additions;
the earlier synthetic W1–W4 do not establish OS enforcement.

### 5.3 Deny-by-default operations and independent consent

Policy must bind authenticated principal, allowed profile/persona, operation,
key/credential namespace, target, session generation and consent requirement.
Recheck at commitment/delivery. AsProfile and a persona UUID request scope;
they never grant it. Replace the global releasable-salt permission with
per-principal grants.

| Operation | Required contract |
|---|---|
| Public status/roster | Explicit disclosure rules, secret-free while locked where supported. |
| Signing/attestation/station/delegation | Typed purpose/audience, allowed namespace/target and bounded payload; proof/result only. No general signing oracle behind an app label. |
| Continuous key release | Exception per consumer, exact derivation/profile/authority/lifetime and rotation limits. Prefer a remote operation. Raw-key expiry cannot be enforced on a malicious holder. |
| Seal/unseal | App/profile/content scope and generation checked at delivery. Roots stay inside; plaintext returned to an app is then in its trust boundary. |
| Create/switch/revoke/remove/review/import/change passphrase | Administration grants, custody-owned confirmation as appropriate, exact selection/revision, lock generation and bounded filesystem authority. Preserve current authentication/quarantine controls. |
| Approval decision | Only a separately authorized trusted-consent channel may resolve the exact pending operation. A general projected-intent client cannot approve merely by knowing its UUID. |
| Unlock | Native custody prompt by default. If terminal unlock remains, separately authorize and mutually authenticate its secret channel and bound/zeroize buffers; document the exception. |

Trusted prompts identify the actual principal (or explicitly unknown), persona,
credential, operation and proved target. Caller text is not authenticated fact.
Define request state on expiry, lock, profile switch, client/UI death and
disconnect. Never accept confirmed=true as a substitute for trusted consent.

### 5.4 Lock, revocation and failure

Bind operations, leases and approvals to a monotonic generation under a unique
custodian start/session. A later Unlocked state must not revive old authority.
Close admission as lock begins; invalidate approvals, drain/cancel private
operations and recheck before delivering secrets/proofs. Define separately
already committed public effects.

Report three facts separately: memory locked, consumers drained, restart barrier
durable. Marker persistence failure preserves the memory lock and returns
explicit degradation/failure. Use existing atomic storage facilities with a
stated filesystem durability contract and crash controls.

Use generation-aware invalidation/reconnection for honest clients; forget keys
on broker loss/stale generation. That does not erase malicious key copies.
Use server-side operation checks, epoch/key rotation and peer enforcement where
supported, and state historical-readability/offline-peer limits. Do not change
transport identities on every ordinary lock without reviewing peer contracts.

Custodian loss means identity pending. Restart discards approval authority and
follows durable lock policy; no fallback or unattended reopen on IPC failure.
A worker must not replace the custodian/helper or impersonate it at launch.
The independent launch and key-access policy in §5.2 applies before any
production consumer migrates.

### 5.5 Budgets and evidence

Bound connections, requests/pending decisions per principal, queue bytes, decoded
fields, file imports, expensive derivations, handshake/read/decision deadlines
and holder drain. Reserve lock/status capacity. A hung worker cannot prevent
closing admission or receiving a truthful outcome indefinitely.

Do not blindly retry a mutation after an ambiguous disconnect. Use stable
request identity and an operation-specific retained outcome/idempotency
contract. Do not claim exactly-once behavior merely because a wire request has an ID.

Retain bounded public security events: principal category, operation, scoped
opaque identifier, generation, consent/refusal outcome. Exclude seeds,
passphrases, OTPs, raw imports, private content, request payloads and arbitrary
error text. Redacted lifecycle diagnostics are not a durable security audit log
or proof that a UI gesture happened.

Secret-bearing framing must use clearing buffers on success, error, cancellation
and reallocation paths, reusing existing zeroizing serialization facilities where
appropriate. Account for base64/JSON intermediates, terminal unlock payloads and
returned private plaintext separately from Debug redaction. Extend T07 with a
tracking-allocator positive control for raw and encoded canaries; do not infer
IPC hygiene from the older vault-only residue receipts. Kernel/OS/helper copies
remain separate qualification limits.

### 5.6 First-slice IPC contract

This is an illustrative protocol contract, not compile-ready API code. Keep
public wire data separate from the current Graphshell endpoint implementation
where that dependency would pull rendering or general resident code into custody.
Reuse maintained OS peer verification, framing/serialization and cryptographic
owners; do not invent a new authentication algorithm for this split.

| Message or fact | Required owner and binding |
|---|---|
| Peer principal | Derived by the custodian from the accepted OS connection, never from a hello label. The client independently verifies the custodian before sending confidential input. |
| Session opened | Version, fresh server instance ID, current lock generation, principal's granted profile/persona scopes and finite session limits. Client nonce is correlation/replay context, not identity proof. |
| Operation request | Stable request ID, expected server instance/generation, typed operation, explicit requested profile/persona, resource/key scope and bounded arguments. The connection supplies principal. |
| Consent petition | Custodian-generated operation digest and immutable public summary, request ID, generation, authenticated principal and expiry; no secret payload. |
| Consent decision | Only the trusted consent channel, bound to that digest/request/generation and one decision. A generic UI/projected-intent response cannot mint this role. |
| Operation outcome | Request ID, generation, typed success/refusal/degraded/unknown outcome and public or explicitly permitted secret result. A disconnect is not success. |
| Lock/revocation | Monotonic invalidation fact and explicit memory/drain/durability result, with reconnect forcing current admission. A state flip to Unlocked cannot clear evidence that the generation advanced. |

For H1, select an existing scoped Graphshell signing/delegation consumer and
Distillery's declared mesh-author consumer. The signature path validates the
actual purpose/audience and performs the root-derived operation inside custody;
the continuous consumer receives only its reviewed exact derivations. This
tests both operation-only and mixed-release migration without a generic root
provider or a broker-wide permission. Scope the synthetic fixtures to the same
two operations and demonstrate a denied adjacent namespace/profile.

Initial **fixture** limits are proposed explicitly: 1 MiB maximum frame, eight
connections globally/two per principal, sixteen queued operations/eight MiB
aggregate queued bytes, one active consent prompt, five-second handshake,
thirty-second noninteractive deadline, 120-second consent expiry and five-second
drain budget. Close private admission immediately on lock; a drain over budget
reports incomplete/degraded rather than lying about completion. Measure these
limits before selecting production settings, and keep a reserved lock/status
path outside the ordinary queue. Never truncate an operation to meet a budget.

Native message boxes are not a secure attention sequence. Include desktop
automation/accessibility authority in the deployment threat model; where user
presence is required, the custodian must validate the appropriate OS presence
result rather than infer presence from a button or a signed UI process alone.

### 5.7 Splits out of Djinn that improve containment

Use **three authority tiers**, with a common worker sandbox and launch/IPC
implementation. The roles below are capability manifests and scheduling/lifetime
choices, not six independently designed service protocols. One visible resident
can coordinate them without owning their authority.

| Tier / role | Minimum authority | Containment requirement |
|---|---|---|
| **1. Independent Castellan custodian** | Vault/wallet/credentials, root operations, grant policy and trusted authentication. No model, document or peer execution. | §5.2 protected launch, storage key and process; typed custody operations with independent principal/scope/consent checks. Djinn is only a client. |
| **2. Private data and blob owner** | Exact profile/space stores, domain decryption/writer material, blob leases/GC and document admission. | A separately protected data service, not a broad mount in the supervisor. Own one physical content store; authorize every object, enumeration, lease and write. Its process/account/container rights and storage keys exclude worker principals. |
| **3. Confined workers: ingress** | Peer/browser sessions, exact admitted transport identities and scoped byte/storage operations. | Shared confinement baseline; network capability for this role only. No vault/blob-root paths or general corpus/credential access; graph plaintext/signing exceptions stay domain-scoped. |
| **3. Confined workers: compute** | Selected model artifacts, admitted job inputs and scoped journals/transport authority. | Inference and training share one compute/library/GPU owner. Device access is an explicit sandbox addition; private prompts/passages remain sensitive in that worker. |
| **3. Confined workers: publication** | Verified selected immutable publication closure, listener and that publication's TLS key. | No arbitrary blob hashes, private-store enumeration or authoring credentials. Current Gemini listener binds loopback. |
| **3. Confined workers: import** | One bounded selected input, private result channel, short lifetime. | No network, persistent store, general custody client or arbitrary paths. Separate instance for secret import material; do not reuse the compute process. |
| **Djinn coordination client** | Public status, resource scheduling and requests to fixed installed worker roles. | No vault opening, custody-launch/update authority, raw unlock credential or universal application grant. It cannot widen a worker manifest. |

Tier 2 uses the same protected-install/OS-launch rules with a distinct macOS
data-service Keychain group, Linux data-service UID or Windows data-service SID.
It receives only reviewed derived data authority from custody; workers and Djinn
receive neither its storage-key group nor its service identity.

**Order follows present exposure:** custody and the shared confinement launcher
first; enabled peer/browser ingress next; enabled compute next; loopback
publication after those. Move hostile credential parsing out before exposing it
inside the new custodian. The private-data/blob service must exist before any
worker is claimed to be isolated from the general corpus; it can initially expose
only the operations needed by the first worker. No optional worker starts merely
because it has a role in this table.

This order assumes current production capabilities, not a future sync feature.
The default Cargo features include personal-sync and cxf. Djinn invokes
personal_sync::start; settings resolve sync on when a profile or CLI configures
a graph, and PersonalSyncHost binds a real P2pandaOverlayHost and seeds paired
peers. No default public relay is configured. Browser/device and transfer parsing
are additional ingress. Thus configured peer ingress precedes loopback serving;
a deployment proven to have sync/transfer/browser ingress disabled can skip that
role. Current fleet settings and connected peers were not inspected.

Source anchors: [feature defaults](../../../ports/djinn/Cargo.toml),
[startup](../../../ports/djinn/src/bin/djinn.rs),
[sync activation](../../../ports/djinn/src/personal_sync.rs),
[profile/CLI resolution](../../../ports/djinn/src/settings.rs),
[transport binding](../../../ports/djinn/src/sync/personal_sync_host.rs),
[browser/device broker](../../../ports/djinn/src/door/device_broker.rs) and
[loopback publication](../../../ports/djinn/src/resident_site.rs).

**Shared worker baseline:** deny vault/blob-root access, other-process memory,
credential/keychain access, ambient custody authority, launch-policy modification
and unneeded network/devices. Start from no input handles and add only the
manifest's exact channels/resources. Restrict descendants and descriptor/handle
inheritance; a sandbox that still inherits an open vault handle fails. Role
additions are explicit, reviewed grants; a network entitlement is not a promise
of per-destination filtering. Network-scoped policy needs its own enforced gate.
Compute GPU access and online sync are additions to test, not reasons to disable
the baseline.

Use two shared protocol families: narrow typed custody operations (§5.6), and a
bounded worker/data-operation envelope with version negotiation, request IDs,
instance/generation binding, resource capabilities, cancellation/deadlines and
explicit reconnect/unknown-outcome semantics. Role payloads remain typed; no
arbitrary command execution, executable paths or all-purpose file I/O RPC.
Reuse one launcher, framing/budget implementation and adversarial fixture suite.
Private-data authorization is service policy, not delegated to a worker-provided
path or hash. This retains per-role authority without six lifecycle/versioning
frameworks. The cost is a small trusted launch adapter plus platform packaging,
not just moving modules into crates.

Two ownership constraints survive every cut:

- **GPU/library:** inference currently borrows Distillery's library and shares
  its reclaim gate. Keep one resource owner for both; do not duplicate model
  writers or bypass drain/reclaim when moving the process.
- **Blob store:** ResidentBlobCustody currently exposes the whole underlying
  BlobStore as well as its scoped authorizer. Neither object crosses to a worker.
  Replace that access with scoped operations/selected bytes, with one physical
  writer/lease/GC owner. Sharing ciphertext/deduplication does not grant hash
  enumeration, arbitrary reads, decryption or another lane's lease removal.
  Distillery's separate job-artifact store remains a separately scoped resource.

For sync, split transport from domain admission/decryption where the protocol
allows opaque signed/encrypted bytes. Where current semantics require graph
plaintext/signing, isolate the admitted domain and state that authority. Preserve
real transfer/replication behavior and identity; do not assume all sync can become
ciphertext-only without tracing those calls.

Import results are untrusted typed candidates. Custody rechecks size/type,
destination/profile, source identity, quarantine and the exact approved action
before commit. Dramatis's ordinary window stays a public-state/request client;
trusted prompts use custody's authenticated OS-session helper and presence path.
Keep the small SSH signing adapter with custody initially, with its parser and
policy separately reviewed. Neither general rendering nor a projected intent
can mint approval.

Every receiving role runs the shared hostile-worker suite plus its scoped
allowed-operation control: file/handle/memory/credential denial, adjacent-domain
and arbitrary-blob denial, service replacement, self-approval and crash/lock/
reconnect. Test necessary plaintext/key/device/network exceptions separately.
The initial custody-only split closes the root boundary; corpus confidentiality
requires tier 2 and the worker grants as well.

## 6. Feature phases and done-conditions

No calendar estimate. C04 and C07 are immediate security repair candidates:
their fixes can precede a complete split and must be included in H1 acceptance.

### H1 — Enforced custody boundary and adversarial vertical slice

**Current slice:** native11 passes 38/38 synthetic native controls in the
separate non-admin GUI account. The signing-authority attack fails there;
native10 remains a failed control for the signing account. Production
Castellan, authority/consumer integration and lifecycle gates remain open.

- [x] Demonstrate signing-authority separation in the synthetic receiving
  account (H1-F1, native11). The exact attacker payload succeeds in the signing
  account and cannot use the identity in UID 503. Preserve that deployment
  separation and requalify the actual production packages before migration.

- [x] Exercise the selected macOS primitives in a protected synthetic native
  fixture against sandboxed and unrelated same-user attackers (native10/11).
  Native11 passes the measured OS gates; production H1 remains incomplete.

- [ ] Implement §5.2's macOS launchd/SMAppService identity, protected install,
  Hardened Runtime/environment constraints, custodian-only data protection
  Keychain envelope and signed XPC peers. Equivalent Linux/Windows modes use
  their selected mechanisms before activation there.
- [ ] Build the shared confined-worker launcher/manifest and hostile-worker
  fixture; activate sandboxing before parsing. Test malicious workers and a
  separate unsandboxed same-desktop-user app against files, handles, key access,
  memory and launch replacement. Include key denial, fake server, hostile
  launch environment and signed-update/cold-reopen controls (T05/T07/T10).
- [ ] Inventory root openings and dependency graph; migrate a synthetic vault
  transactionally, covering all usable custody copies and recovery behavior.
  No general worker links custody or opens its root storage.
- [ ] Implement one root signature and one scoped continuous consumer with
  authenticated IPC, principal/profile policy, independent consent, generation
  binding and bounded framing.
- [ ] Close general-client signing approval (C04) and false durable-lock success
  (C07), retaining authorized controls. Isolate hostile import parsing before
  that parser is admitted to the production custody path.

**Done:** the installed OS denies worker and unrelated desktop-app extraction,
replacement and self-approval, while authorized operations and sealed cold reopen
succeed. W1/W2's unsafe assertions fail after repair. A socket-only implementation
cannot pass H1. A failed platform mechanism blocks that protected deployment.

### H2 — Complete authority and consumer migration

- [ ] Cover every call, named profile/persona, epoch, radio/grant/revoke and
  terminal command with the operation table.
- [ ] Migrate SSH, credential catalogs, Knot, payloads, inference, Distillery and
  transport/door exceptions; remove direct/fallback openings.
- [ ] Create the tier-2 private-data/blob owner and only the scoped operations
  needed by its first worker. Run the shared worker baseline for enabled ingress,
  then compute, then loopback publication, with domain-specific authority tests.
  Keep inference/training under one GPU/library owner; remove whole BlobStore
  handoffs. Corpus-isolation claims wait for this boundary.
- [ ] Qualify trusted authentication helpers and any explicit terminal-unlock
  exception (T01/T02/T04).
- [ ] Publish each raw-key exception and peer/rotation consequence.

**Done:** one authority; roots/epochs remain inside. Golden identity/derivation,
stored content and cold-reopen checks preserve data and public keys. No broad
implicit grant replaces the old global allowlist.

### H3 — Lifecycle and remaining platform acceptance

- [ ] Generation-aware invalidation, bounded drain and durable marker outcomes;
  execute T02/T03/T09.
- [ ] Native macOS lock/suspend/idle sources and truthful coverage reporting;
  requalify actual events separately from Linux/Windows (T06).
- [ ] Re-run H1 enforcement on each additional supported platform and signed
  update/recovery path. Verify exact service accounts/entitlements, key ACLs,
  helper/UI presence and process/device restrictions (T05/T07). Platform
  enforcement is required at activation, not deferred until this phase.

**Done:** separate OS receipts establish advertised guarantees. Missing platform
restrictions remain unqualified; software tests are not physical-sleep,
debugger-resistance or power-loss evidence.

### H4 — Hostile input and independent review

- [ ] Enforce/measure budgets and responsive lock/status (T08).
- [ ] Requalify import, quarantine, authentication, replacement, OTP exact-session
  release and enabled epoch/history behaviors. Secret Service remains parked.
- [ ] Dependency/advisory and independent protocol/platform review on the final
  bounded custodian, exact features and signed artifacts.

**Done:** every finding/test has evidence, an explicit accepted limit or an
unqualified deployment. Source review, ordinary suites and this plan alone do
not justify calling the product hardened.

## 7. Related-plan reconciliation

| Existing owner | Preserved facts and correction/dependency |
|---|---|
| [Vault lock](2026-10-05_vault_lock_plan.md) | Preserve L1–L4 receipts. New scope is generations, release limits, persistence failure, in-flight completion and platform coverage; do not revive the old claim that the vault never locks. |
| [Dramatis app](2026-10-11_dramatis_app_plan.md) | Public client and resident quarantine consent remain. App label is not authentication; passphrase-change receipts do not qualify caller isolation or signing consent. |
| [Secret Service](2026-10-08_secret_service_plan.md) | SS8 stays parked; no keyring/activation takeover. A future adapter needs the same custody authorization and OS gates. |
| [Family promotion](../../mere_docs/implementation_strategy/2026-10-19_family_promotion_plan.md) | Custody design is now owned here. Shared shutdown/observations help implementation but do not establish containment. Repository moves stay independent. |
| [Resident services](../../mere_docs/implementation_strategy/2026-08-22_djinn_family_resident_services_plan.md) and [consolidation](../../mere_docs/implementation_strategy/2026-08-20_device_resident_consolidation_plan.md) | One user-facing resident coordinates three authority tiers. The OS launches custody independently; worker roles share confinement/IPC infrastructure. Sites and general tools receive no roots or whole-store access. |
| [Device pairing](../../mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md) | Preserve LAN SSH/presence receipts; D3–D6 are not completed by them. Principal/session/key rules must compose with eventual transport/CA/pairing ceremony. |
| [Resident inference](../../mere_docs/implementation_strategy/2026-10-15_resident_inference_and_recall_plan.md) | Preserve bounded journal/stage lock behavior. Models/tensors/remote execution stay outside roots; generation and drain are explicit dependencies. |
| [Protocol architecture](../../mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md#37-isolation-and-capability-scoping-stated-threat-model) | Its level-0 process trust remains the current limit. Tokens inside a compromised native address space cannot supply containment. This request starts stronger isolation without waiting for another protocol mod. |
| Pinned Dramatis tier/repo/threat/CXF/SSH-CA plans | Preserve portable type ownership, mixed-release identities, quarantine and crypto. Qualify absolute lock/native-only wording against C05–C09. DR-B moved wallet custody; the threat statement's older pending-wallet-move note is not a current extraction task. |

External documents were read from the exact pinned Dramatis revision, not an
assumed sibling checkout. Their paths/hashes are in the receipt inventory.
Vendor-cache files are not edited. The canonical external threat statement and
tier changes need owning-repo follow-up at implementation, not a competing
copy of those documents here.

## 8. Progress

- 2026-10-21: User requested a security-boundary audit/plan starting with custody
  isolation, then explicitly broadened related-plan review. Fetched Mere main,
  preserved the primary checkout and created an isolated audit worktree. Reviewed
  pinned Dramatis contracts and historical qualification limits. Recorded source
  findings and diagnostic witnesses separately from remaining experiments.
  No production hardening, fleet change or publication is claimed.
- 2026-10-21, service-split follow-up: the user explicitly accepted splitting
  responsibilities out of Djinn where it improves security. Added §5.7's ranked
  process roles, minimum permissions, storage/GPU ownership constraints and
  per-split hostile-worker acceptance; superseded in topology/ranking by the
  review below.

- 2026-10-22, H1 implementation: added the synthetic native macOS fixture and
  receipt evaluator, registered the user-approved dedicated App ID/profile, and
  ran real sandbox/XPC/launch/Keychain tests. Native07 passes 28/30 checks, with
  same-user signing authority demonstrably recovering the synthetic vault root.
  Native10 subsequently passed protected installation/SMAppService and all
  other fixture controls (34/35), while the same signing-authority escape
  remained successful. Failed pre-provisioning, fixture-bug and native09 installer
  iterations are retained; no real consumer migrated.
- 2026-10-22, separate-account follow-up: prepared native11 for the dedicated
  standard local account djinnh1, with hash-pinned public transfer, protected
  receiving tools and 38 mandatory receiving controls. All 42 preparation tests
  pass; account setup/login and receiving execution require user OS interaction.
  Native11 subsequently passed 38/38 checks in UID 503: the same-user signer
  attack failed while authorized signing/reopen succeeded. Production migration
  remains gated; native10 continues to demonstrate the signing-account escape.
- 2026-10-22, architecture review: chose explicit worker confinement, independent
  OS launch and custody key-access defaults for macOS/Linux/Windows (§5.2);
  separated hostile-worker and unrelated-desktop-app guarantees. Replaced six
  bespoke service cuts with three authority tiers/shared worker infrastructure,
  moved configured ingress ahead of loopback serving, and made enforcement part
  of H1. Re-read activation/transport/settings source; no installed fleet or OS
  containment experiment was run. Official mechanism references and review
  evidence are in review-confinement-20261022.json beside the original receipt.

Documentation verification retains inherited failures rather than calling the
whole tree clean: the full D2 audit has existing missing judgments and a tracked
snapshot digest mismatch. The new plan has its own supplemental judgment;
before/after checks must add no errors. Relative-link/index audit deltas and
the existing baseline are retained in the same receipt. The immutable snapshot
and unrelated documents are not rewritten to manufacture a passing result.

