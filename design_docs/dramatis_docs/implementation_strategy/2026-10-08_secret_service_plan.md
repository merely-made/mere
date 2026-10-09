# Secret Service Plan

**Date**: 2026-10-08
**Status (2026-10-08)**: plan. Scoped by the
[vault lock plan](2026-10-05_vault_lock_plan.md)'s ruling 87, which amends
ruling 69's "later items". Building waits on the
[device pairing plan](../../mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md)'s
D2, which gives djinn a Linux launcher and retires the `personae-agent`
that keeps its passphrase in gnome-keyring. The forks in §3 are unasked.
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
  `secret_service/` out of castellan later.
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
- **S2 — the name.** Done when:
  - [ ] the check of which process holds `org.freedesktop.secrets`
        reports it, and djinn says why it does not serve;
  - [ ] gnome-keyring's secrets component is displaced as §3 rules, and
        the displacement reverses cleanly;
  - [ ] gnome-keyring's existing items are handled as §3 rules.
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

## 4. Findings

*2026-10-08, from the vault lock plan's L4 scoping (read, not run):* the
facts in §1.

## 5. Progress

*2026-10-08:* plan written (ruling 87).
