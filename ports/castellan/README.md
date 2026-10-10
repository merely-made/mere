# castellan

**Castellan**, the credential-keeper port of the Mere platform.

A castellan holds a keep in trust for its lord: custody without ownership, and
the office of the gate. This port is that keeper for your credentials. It
holds secret custody inside djinn (release, signing, presentation), answering
participant-gate petitions over an agent-style channel. Secret-free views live
in graphshell. Applications call djinn's custody route; only djinn links
castellan, as enforced by Mere's `deny.toml`.

The vocabulary it keeps, per the dramatis tier model:

- **chatelaine**: the secrets. Passwords, 2FA seeds, tokens, foreign key
  material. Never presented, only exercised.
- **insigne**: the proofs. Graded presentations of identity a persona hands
  out, from a bare handle to signed cross-attestations. Made to be shown; what
  lands in someone else's gaz.

The boundaries are the point: not [personae](https://crates.io/crates/personae)
(the faces and vault substrate castellan serves), and not
[gaz](https://crates.io/crates/gaz) or gazette (which keep and find the other
players; castellan guards and presents you). Castellan issues a persona's
public presentation and gazette announces it (ruled 2026-09-30), so the port
that holds secrets never grows a public listener.

Lives in the [mere](https://github.com/merely-made/mere) workspace at
`ports/castellan`.

## State (2026-08-21)

**2026-10-10, DR-C:** `personae-vault` is now a graphshell client. Install it
with `cargo install --path ports/graphshell --bin personae-vault` from Mere.
It accepts `--app-endpoint` and `--profile`; vault location and unlocking belong
to djinn. Command implementations run against djinn's already-open storage,
within its lock boundary. Private slots never cross this terminal route.
`profiles` reads the public roster even while Locked; identity-bound commands
report pending when djinn is absent or Locked.

Implemented:

- `otp` — RFC 6238/4226 codes and `otpauth://` URIs, plus persona-scoped OTP
  items sealed through Personae's record store. `OtpReleaseGate` returns a
  redacted-debug `OtpCodeTile` only after a participant-bound petition receives
  an explicit approval; its time facts leave ring geometry to the host.
  `OtpAdmittedSession` consumes Notochord admission for one exact credential, derives
  the participant from the signed transcript, rechecks expiry and revocation at
  approval and delivery, and exposes the tile only beside the original carrier.
  It leaves byte encoding to the composing host's existing protocol. Steam
  Guard is a separate, explicit code style with Valve's five-character
  alphabet and base64 `shared_secret` import. It does not reinterpret an
  `otpauth://` extension as Steam.
- `resident` — one process-wide owner for Castellan's sealed records. The
  resident retains an exclusive OS file lock, shares composite Secret Service
  transactions across independent views, and checks a separately rooted keyed
  freshness ledger before releasing restored HOTP state.
- feature `secret-service` — the Freedesktop Secret Service 0.2 object tree on
  Linux. The resident owns `org.freedesktop.secrets` without replacement,
  implements the recommended `plain` transfer session, binds sessions to D-Bus
  callers, and delegates every operation to a host policy over bus credentials
  and `/proc` executable identity. A `secret-tool` store/lookup/clear receipt
  runs under a disposable session bus.
- The Reticulum station derivation that began here is `personae::reticulum`
  now, and the station grant policy `pandect::station_grant` (DR-C, D17).
- feature `keeper` — the two halves made real, moved home from graphshell
  where they first grew: `view` (the secret-free read model) and `authority`
  (`PersonaeHost`, the resident keeper that holds the vault, serves the SSH
  agent, and brokers approvals).

For private mutable slots, `PersonaeHost::sealed_backend(persona, backend)`
loads the wallet epoch and supplies Pandect's `WalletSealedBackend`. An absent
carry root or epoch returns `PermissionDenied`. There is no cleartext fallback.
The host still chooses the Muniment backend, codec, and persona-to-scope mapping.
The Gaz receipt at `tests/sealed_contacts.rs` uses JSON and postcard books,
real redb reopening, and retained-proof rechecking. Keys stay visible, epoch
history must be supplied for old records, and replay of an authenticated value
at its original key is outside this adapter's freshness guarantee.

The address-intake receipt at `tests/sealed_webfinger_intake.rs` also composes
Gazette's supplied WebFinger adapter with this backend. JSON/postcard reopen and
replay preserve private names, notes, Kin, address trust/usage and root/device
artifacts. Fresh claims start Unverified; a new contact requires the host's
LocalId. The receipt proves native sealed composition and persona isolation;
live resolver transport and application contact UI remain separate work.

Graphshell composes all three and re-exports them at its pre-founding paths,
so it is the first host rather than the owner. The intent wire strings keep
their `castellan.*` values for now; renaming the wire vocabulary is
a separate decision. CXF import remains follow-on work; its policy was ruled
on 2026-10-01. The everyday credentials are stored and SSH keys go through
the native SSH import. Identity documents, payment cards, passkeys, files and
unknown types are quarantined: sealed, never exercised, and accepted one at a
time by the user. The secret-free item taxonomy moves to chatelaine. The file freshness
ledger detects rollback of the credential-record root only when its separate
root was not restored with it; stronger platform monotonic storage remains a
host deployment choice. See the keeper founding plan and the credential port
brief in mere's `design_docs`.

## License

MPL-2.0
