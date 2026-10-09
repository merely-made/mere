# Secret Service Plan

**Date**: 2026-10-08
**Status (2026-10-09)**: parked (ruling SS8). Apps keep the platform's
keyring; djinn serves no Secret Service for now, and SS5 to SS7 stay as
the design for when an app needs djinn's lock. Before that, this was a
plan. Scoped by the
[vault lock plan](2026-10-05_vault_lock_plan.md)'s ruling 87, which amends
ruling 69's "later items". Building waits on the
[device pairing plan](../../mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md)'s
D2, which gives djinn a Linux launcher and retires the `personae-agent`
that keeps its passphrase in gnome-keyring. §3's five forks are ruled
(SS1 to SS7). djinn coexists with gnome-keyring under a bus name of its
own (SS5).
**Scope**: djinn's resident serves castellan's Secret Service 0.2 on the
user's real Linux session bus, in place of gnome-keyring's secrets
component, with the vault lock's semantics (rulings 10, 67 and 68).

**Related**:
- [vault lock plan](2026-10-05_vault_lock_plan.md): rulings 10 (a Locked
  collection, and `Unlock` returns a Prompt), 66 (nothing exists before the
  first unlock), 67 (the Prompt shows the native unlock), 68 (a client
  `Lock` locks the whole vault), 69 and 87.
- [chatelaine and CXF plan](2026-10-01_chatelaine_cxf_plan.md): rulings
  17 and 20 (the store is a view of chatelaine items; a clean break for
  castellan's own old Secret Service records).
- [dramatis repo plan](2026-10-06_dramatis_repo_plan.md): moves
  `secret_service/` out of castellan later. *Corrected 2026-10-09:* D2 keeps
  the service in castellan; D21 and D30 moved only its metadata, to
  chatelaine (DR-A's A6).
- [standards survey brief](../../2026-08-24_standards_survey_brief.md),
  lines 280-286 (a check of which process holds the name) and 723, 909
  (the client-side portal, marked SKIP).

---

## 1. What exists (assessed 2026-10-08)

- **castellan's adapter** (`ports/castellan/src/secret_service/`) is built
  and proven under a disposable bus (`tests/secret_service_linux.rs`,
  vault lock plan `aac67a85`). It runs behind castellan's `secret-service`
  feature.
- **djinn doesn't enable that feature.** It takes castellan with `keeper`
  only (`ports/djinn/Cargo.toml:53`).
- **The host's two seams are traits with test implementations only:**
  - `SecretServiceAccessPolicy`, which decides which bus callers may act
    (`dbus/state.rs:79`);
  - `SecretServiceVault`, which carries the lock, its watch and the native
    prompt (`dbus/vault.rs:11-25`). Its `prompt_unlock` blocks.
- **`serve()` requests `org.freedesktop.secrets` without replacing**
  (`dbus/mod.rs:124-125`). zbus 5.19 defaults to "do not queue", so it
  fails while gnome-keyring holds the name.
- **Nothing exists before the first unlock** (ruling 66), and the Linux
  resident starts locked (ruling 42). At logon, a client would find no
  service, and D-Bus activation would start gnome-keyring.
- **djinn has no Linux launcher.** `install-windows.ps1` is its only
  installer.
- **`org.freedesktop.impl.portal.Secret`** is a different role: the
  backend that gives Flatpak apps a per-app master secret, chosen by
  `portals.conf`. Nothing in the stack covers it.
- **Not verified on the ThinkPad** (general knowledge, to check in S2):
  - gnome-keyring is D-Bus-activated through
    `org.freedesktop.secrets.service`;
  - PAM unlocks it at logon;
  - whether it allows its name to be replaced.

## 2. Phases

- **S1 — djinn serves it under a test name.** Done when:
  - [ ] djinn enables castellan's `secret-service` on Linux;
  - [ ] a production `SecretServiceAccessPolicy` binds callers as §3
        rules;
  - [ ] a `SecretServiceVault` on djinn's lock coordinator and native
        prompt, with `prompt_unlock` on a blocking thread;
  - [ ] the service starts after the first unlock and survives lock and
        unlock;
  - [ ] a receipt on the real session bus under a name of its own, with
        `secret-tool` against it, and gnome-keyring left untouched.
- **S2 — its own name, and routing** (SS5 to SS7; amended 2026-10-09
  from "the name"). Done when:
  - [ ] djinn serves under a bus name of its own, and gnome-keyring keeps
        `org.freedesktop.secrets` untouched;
  - [ ] routing an app (a `SECRET_SERVICE_BUS_NAME` override in its
        `.desktop` file) is a reversible step, and moves that app's items
        across with a report (SS6);
  - [ ] each item records its creating executable, shown in the snapshot
        (SS7);
  - [ ] the check of which process holds `org.freedesktop.secrets` reports
        it.
- **S3 — the launcher** (with pairing D2). Done when:
  - [ ] a djinn user unit starts the resident at logon, and the service
        before or after the first unlock behaves as §3 rules.
- **S4 — the receipt.** Done when:
  - [ ] on the ThinkPad, `secret-tool` and a GNOME app store, read, lock
        and unlock through djinn, and a client `Lock` locks the whole
        vault (ruling 68);
  - [ ] the vault threat statement and the tier architecture name the
        service.

## 3. Forks for Mark (unasked)

1. **How the name is taken:**
   - the owner disables gnome-keyring's secrets component, and djinn takes
     the free name;
   - djinn requests it with replace-existing, which works only if
     gnome-keyring allows replacement;
   - djinn becomes the D-Bus activation target.
2. **Before the first unlock:**
   - no service, accepting that activation races;
   - a "locked" service with an empty snapshot, so clients get ruling 67's
     Prompt.
3. **gnome-keyring's existing items:**
   - migrate them through the client API;
   - a clean break, like chatelaine ruling 20;
   - leave them in gnome-keyring, read-only, outside the stack.
4. **The Flatpak portal backend:** out of scope; djinn implements it too;
   gnome-keyring keeps it.
5. **Caller binding** in the access policy: by bus credentials and the
   `/proc` executable path, as the standards survey names; or allow every
   same-user caller, as gnome-keyring does.

### Rulings

Asked 2026-10-08, with the ThinkPad's state read first:
- gnome-keyring holds the name through D-Bus activation
  (`/usr/share/dbus-1/services/org.freedesktop.secrets.service` runs
  `gnome-keyring-daemon --components=secrets`), plus the
  `gnome-keyring-secrets.desktop` autostart entry;
- PAM unlocks it at login (`gdm-password`);
- `portals.conf` names it for `org.freedesktop.impl.portal.Secret`;
- its `login` collection holds 21 items (counted, not read), and 10
  Flatpak apps are installed.

**Ruling SS1** *(fork 1).* Options: djinn owns the activation (a
user-level service file that outranks the system one, with gnome-keyring's
secrets autostart disabled, reversibly); the owner disables gnome-keyring
by hand; request the name with replacement. Mark: **"wait, wait. this
sounds antisocial. let's look and see the options for coexisting. 1
otherwise, if there are none."** Follows: survey the ways djinn can
coexist with gnome-keyring, and put them back as the next round; owning
the activation is the fallback if none works.

**Ruling SS2** *(fork 2).* Options: serve a locked, empty service from the
start, so a client's `Unlock` gets ruling 67's Prompt; no service until
the first unlock. Mark: **"Serve locked, empty (Recommended)"**.

**Ruling SS3** *(fork 3).* Options: migrate the 21 items through the
client API while gnome-keyring is unlocked, then retire it; a clean break;
leave them read-only in gnome-keyring. Mark: **"Migrate, then retire
(Recommended)"**. *Reading, not ruled:* "retire" assumed djinn replaces
gnome-keyring. If SS1's survey finds a way to coexist, this comes back
with it.

**Ruling SS4** *(fork 4).* Options: out of scope for now, recorded as a
gap and revisited after S4; djinn implements the portal too; gnome-keyring
keeps it permanently. Mark: **"Out of scope for now (Recommended)"**.

**The coexistence survey** (2026-10-09, an Opus research lane, primary
sources cited in its report; the two load-bearing claims checked on the
ThinkPad):
- No project runs two providers behind `org.freedesktop.secrets`.
  KeePassXC ("Only one secret service provider can be enabled at a
  time"), KWallet 6, pass-secret-service, secretsd and oo7-daemon each take
  the name or hand it over. gnome-keyring requests the name with no flags
  and queues behind another owner.
- libsecret reads `SECRET_SERVICE_BUS_NAME` for its default service, and
  the ThinkPad's libsecret 0.21.8 carries the string (checked). The
  variable is undocumented. Python `secretstorage`, Rust `secret-service`
  and `oo7` hardcode the name.
- Fedora 45 plans to replace gnome-keyring with oo7-daemon, migrating
  its data one way. The ThinkPad is on Fedora 44 and has no oo7 installed
  (checked).
- One page the lane read held text telling an agent to run `glab`
  commands. The lane ignored it.

**Ruling SS5** *(answers SS1).* Options: djinn serves the same API under
its own bus name, and apps opt in through `SECRET_SERVICE_BUS_NAME`;
djinn keeps a collection inside gnome-keyring as a client; the fallback,
djinn owning the activation. Mark: **"Wait. We can just use the platforms’
keyrings? Hmm. Would all platforms support that in their own special ways?
1 is acceptable, but if 2 creates lots of work where we could just do 1… I
suppose 1 is ok"**. Answered at the time:
- each platform has one, differently: Windows' DPAPI already roots djinn's
  vault; the macOS Keychain comes with D2; on Linux the keyring is
  whichever provider the desktop ships;
- option 2 is new client code per platform, and gnome-keyring's lock would
  govern those items, not djinn's;
- option 1 reuses castellan's built server under another name.

Follows: djinn coexists under its own name. gnome-keyring is untouched,
routed libsecret apps get djinn's lock and prompt, and hardcoded clients
stay on gnome-keyring. §2's S2 changes from displacing gnome-keyring to
routing apps.

**Ruling SS6** *(amends SS3).* Options: when an app is routed to djinn,
its items move across (matched by attributes) with a report, and the rest
stay; leave them all; SS3 stands. Mark: **"Move per app when routed
(Recommended)"**.

**Ruling SS7** *(fork 5).* Options: any same-user caller may act, and each
item records the executable that created it, shown in the snapshot;
per-executable binding; same user only. Mark: **"Same user, record
executable (Recommended)"**.

**Ruling SS8** *(asked after Mark's "What’s wrong with ceding control to
the native os?").* Answered first: nothing in itself. It is simpler,
maintained, unlocked by PAM, and reached by every app, and djinn already
cedes its root to DPAPI on Windows. What it costs is the lock's promise:
the ThinkPad's `login` collection read `Locked = false` after that night's
`loginctl lock-session` and suspend (checked 2026-10-09). Options: cede
apps' secrets to the OS and shrink the plan to nothing; SS5 stands; cede
now and keep SS5 on the shelf. Mark: **"Cede now, keep SS5 on the
shelf"**. Follows: djinn serves no Secret Service, and its own items
(chatelaine's keychain, SSH keys, OTP) stay in its vault under invariant
13.

## 4. Findings

*2026-10-08, from the vault lock plan's L4 scoping (read, not run):* the
facts in §1.

## 5. Progress

*2026-10-08:* plan written (ruling 87).
