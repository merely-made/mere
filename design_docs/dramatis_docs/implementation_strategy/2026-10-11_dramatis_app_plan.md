# Dramatis persona selection and quarantine review

**Status (2026-10-11):** first Windows app slice complete and qualified.
Djinn owns custody and consent; concurrent main work is retained. Persona
creation, contacts, credential fill and CXF export remain open. The Windows
debug packaging pipeline is qualified; optimized releases, OS install trust,
publication and Linux/macOS host qualification remain open.

**Security continuation (2026-10-21):** the [custody hardening plan](2026-10-21_custody_isolation_and_security_hardening_plan.md)
keeps this app's public client and resident-owned quarantine consent, while
qualifying app-name admission, signing approval and process isolation separately.
The historical functional/native receipts do not establish an authenticated
application principal or a trusted signing-consent channel.

## Phases and done-conditions

- [x] Map the existing picker, public roster, custody route and sealed item
  authority. Isolate Graphshell changes from concurrent primary edits.
- [x] Add secret-free quarantine cards and per-item resident actions, with
  explicit profile/namespace ownership, exact reviewed metadata and native
  Djinn consent. Locked, stale, denied, unsupported and wrong-persona acts
  must leave items sealed and unchanged.
- [x] Ship a runnable Cambium window with persona selection, reconnect,
  native unlock, lock, cards and reasoned accept/delete controls. The client
  must never open custody or render a passphrase field.
- [x] Qualify real absent/Locked and unlocked wire flows, resident consent
  denial, acceptance, deletion, stale revision and cross-persona refusal.
  A deliberately broken consent or quarantine gate must fail the controls;
  restore source exactly before recording green receipts.
- [x] Run the native window and inspect retained layout, keyboard focus and
  resize behavior on synthetic fixtures. Record only exercised host gates.
- [x] Commit, push and integrate the isolated lane, preserving other edits;
  update the owning CXF P5 status and retain reproducible evidence.

## Findings (2026-10-11)

The app is `ports/dramatis` in Mere, package `dramatis-app`, binary
`dramatis-ui`. This keeps Cambium and Graphshell client dependencies outside
Dramatis's portable identity workspace, which Mere already pins. There is
one existing Djinn process owning Castellan authorities for its existing credential stores; the app
requests public state and acts through the owner-local custody door.
`AppId` is a label, not a security credential. Resident consent remains the
boundary for review mutations even from an owner-local caller.

The existing public roster is in the Dramatis facade. The persona picker
provides shared row conventions. The client transport already reads roster
and status while Locked and requests profile switches when Unlocked. Djinn
already has native unlock dialogs and a lock-bound credential authority.
The app now lists quarantined items and requests their state changes through Djinn.

CXF destinations contain two distinct identifiers: an identity `ProfileId`
and a sealed item namespace `PersonaId`. The earlier importer checked both
but did not record their association. The review lane binds the explicitly
chosen import destination inside sealed resident records. A namespace cannot
be reassigned to another profile, and the view never guesses ownership from
a display name or a derived UUID. Older unbound namespaces appear in a separate assignment section. The owner
chooses a persona, then Djinn confirms association of the exact namespace and
rechecks its whole metadata revision before binding it. Assignment does not
accept quarantined items. Profile selection changes Djinn's chosen profile, consistent
with the existing custody API, rather than inventing a separate app identity.

A store's sealing persona remains fixed across profile selection, lock/unlock
and restart. The resident catalog discovers existing profile-scoped roots
through the persisted roster. Cards and mutation requests identify both the
original store and the explicitly assigned destination profile; selecting a
persona does not re-derive another store's keys or hide older imports after
restart. No empty store is created for a persona by discovery.

Cards contain public Chatelaine metadata only. Listing never reads a payload.
The revision digest names exactly the metadata Djinn shows before asking
for consent. Djinn asks on its native surface without holding the vault
mutex, then reacquires profile authority and checks the item again before
changing sealed records. Locking, switching or concurrent item edits cannot
turn approval for one card into authority for another.

Unsupported OTP counts, unknown or refused credentials, passkeys and files
remain quarantined. Other accepts authenticate and validate their held
payloads before marking the item accepted; the view cannot claim support
that custody lacks. Deletion applies to one exact quarantined item. Consent
cancel or an unavailable native surface preserves it. Broad credential fill,
contacts, persona creation, CXF export and packaging remain follow-on work.

## Progress

2026-10-11: isolated `mere-dramatis-review` from the dirty Graphshell primary.
Added the first public review protocol, sealed namespace/profile bindings,
resident-native confirmation seam and Cambium app skeleton. Verification is
pending; source drafts do not establish runtime or visual acceptance.

2026-10-11: regression testing exposed credential keys being re-derived from
the newly selected profile on unlock. Restored holders now name their original
sealing profile. The first 15 wire tests pass after the fix; multi-store,
startup discovery, negative consent and native receipts are still running.

2026-10-11: qualification complete. The custody suite passes 16 tests, startup
store discovery passes one, Castellan passes 303, app state passes four, and
retained interaction passes seven. Real app wire runs pass for absent,
Locked and unlocked Djinn. The consent control passes without the synthetic
bypass, fails as expected with it enabled (exit 101), and passes all 16
custody tests after byte-exact source restoration. The flag is absent from
restored production source.

Five native GPU captures were visually reviewed: absent, Locked and unlocked,
with wide and narrow Locked/unlocked windows. The retained host harness
exercises pointer activation, Enter, keyboard order, scroll preservation,
resize and zoom. Native Windows consent was exercised separately: No keeps
three cards; Yes deletes only the selected card. The receipt also verifies
No is the native default button. Native screenshots caught and corrected the
root background and Windows' different default-button indexing; the harness
caught disabled controls in Tab order and missing focus reveal.

Evidence is [the Windows receipt](../../../support/receipts/dramatis-app/README.md),
including source hashes, commands, logs, screenshots and archived failed
controls. Production Djinn checks, portable metadata verification and
`cargo deny --offline check bans` pass. These are synthetic vault receipts,
not qualification of an actual user's vault or other platforms. Documentation
audit baseline findings are recorded separately in that receipt.

The focused lane integrates concurrent documentation changes without
changing their source. Its temporary worktree and synthetic runtime roots
are removed after integration. Builds reuse `C:/t/cargo-targets/mere`.

## Packaging through Hocket/Luggage (2026-10-11)

The app declares cargo-packager metadata and uses `ports/dramatis/package.py`
to build with a single compiler job, package the explicit UI binary, sign its
artifact and assemble/sign a Luggage feed. The shared target is reused without
cleaning another lane's cache. `--version` and `--build-info` bind each binary
to its package version, application source, workspace manifests and lockfile;
the pre-packaging hook rejects stale inputs or a concurrent source/lock change.

The package contains public release information, the Merely brand icon and
license notices. Djinn remains a shared resident dependency; its quarantine
routes require Mere `ec1e65ca` or a descendant with `cxf`. That source floor is
declared in the package because the resident's package version spans both
older and newer route sets. The installer/uninstaller owns the Dramatis UI,
with no custody binary or vault-data removal hook.

The Windows debug NSIS pipeline passes: pack/sign, extraction, the extracted
binary's version/source identity and an absent-Djinn native GPU capture.
Luggage verifies the actual artifact and signed manifest and restores durable
staging. Unsigned or edited manifests, a different valid publisher key,
bad artifact signatures and altered artifact/staged bytes are refused.
Every feed byte is restored after controls. The gate passes one real-package
test plus the four state and seven retained interaction tests; dependency
bans and portable metadata also pass.

Evidence: [packaging receipt](../../../support/receipts/dramatis-packaging/README.md).
The artifact is debug-built and signed with a synthetic key, which is removed
with the extraction after qualification. It proves the packaging and Luggage
path, not an optimized, installed, OS-signed or published release. Automatic
update controls and resident installation are separate work.

Remote qualification is explicitly pending on Fedora/ThinkPad, Intel macOS
and ARM macOS until those hosts are reachable. Resume one native host at a
time, inspect existing build owners/free memory, reuse its approved target,
and default to one compiler job. The Mac signing identity is a host setting;
its option and the non-Windows formats remain unqualified on Windows.
