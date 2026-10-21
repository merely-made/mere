# One graph, one owner, one invited guest

The `graphshell-coop` native surface hosts one explicitly selected Pandect graph
session. The holder keeps the authoritative graph and journal. A guest visits
through Graphshell's admitted projection protocol over the existing p2panda
QUIC carrier. Addresses and titles are shared; this slice does not fetch or
transfer the bodies behind those addresses.

The graph store can contain other sessions. The invitation names one session,
one guest's attested transport key, and reader/editor permission. The endpoint
exposes no session list, archive, invitation, undo, or filesystem operation.
Every edit checks its graph, current epoch and revision before the holder
submits a transaction. The wire revision follows the stored graph journal
position; a separate epoch advances on each owner launch. This pair identifies
projection progress, not immutable content. The journal author comes from the admitted guest's
root binding, with `graphshell.coop` as the application. It never comes from a
payload field. A successful save means the holder's transaction acknowledged
storage; a failed transaction leaves graph, history and revision unchanged.

Both people use their chosen Djinn persona. Graphshell opens no vault and has
no fallback identity. Djinn releases only the existing scoped endpoint key;
certificate and revocation signatures remain in custody. Contact files contain
public attested keys. Invitations contain recipient-bound grants and are kept
outside this public evidence packet.

## Running the surface

Build the current workspace with its lockfile:

```sh
cargo build --locked -p graphshell --no-default-features --features coop-window --bin graphshell-coop
```

Each person runs their own unlocked Djinn. The first-party app endpoint is
`GRAPHSHELL_APP_ENDPOINT` when an explicit endpoint is needed. The holder
chooses a session from their store; there is no implicit latest-session fallback:

```sh
graphshell-coop list --store /absolute/path/graphs.redb
graphshell-coop owner --store /absolute/path/graphs.redb --graph SESSION_UUID --handoff /absolute/path/owner-contact
```

`new-graph --store /absolute/path/graphs.redb` creates an empty journaled graph
and prints its id. Use it when starting a new shared activity. The holder's
window edits that selected graph and exports a current, authority-free
`contact.json`. It keeps the store open exclusively for its lifetime.

The guest exports their public contact through their own custodian:

```sh
graphshell-coop contact --handoff /absolute/path/guest-contact
```

The holder chooses that `identity.json` in **Guest contact file**, chooses an
invitation output path and presses **Invite editor** or **Invite reader**.
Invitations expire after 24 hours. Retrying the same invitation export keeps
its original grant and expiry. A different guest or access level requires
ending the existing access first.

The guest receives the invitation and holder contact, then opens:

```sh
graphshell-coop guest --invitation /absolute/path/invitation.json --contact /absolute/path/contact.json --handoff /absolute/path/guest-session
```

Both windows show the selected graph, saved activity and save/refusal status.
**Select object** chooses the target for **Save title**. The holder's **End
guest access** stores a signed revocation before reporting success. Existing
sessions stop serving the guest; reconnect and cold reopen recheck that stored
revocation. Guest projections are memory-only and disappear on an access or
connection failure. Previously observed information cannot be recalled from a
recipient's independent copies.

After a holder restart, distribute the current `contact.json` again. It carries
fresh connection information and no additional authority. The original
invitation remains bound to the same holder, graph and guest. This deliberately
uses explicit contact exchange; public web rendezvous is a separate capability.

## Acceptance

`run_native.py` launches two independent passphrase-encrypted Djinn profiles and
real native Graphshell processes. Its scripts dispatch ordinary native controls
and capture successfully presented frames. It checks a private sibling graph,
owner/guest edits, guest process return, title revision, revocation, both resident
and application shutdown, holder cold reopen, and refusal of the old invitation.
A final visit to the sibling checks that its nodes, activity and revision are
unchanged. Software tests separately probe forged identity, graph/session/target
scope, injected storage failures, stale retries and the real admission carrier.

```sh
cargo test --locked -p graphshell --no-default-features --features coop --lib coop::
cargo build --locked -p djinn --features passphrase-fd --bin djinn
python3 ports/graphshell/docs/receipts/coop_20261020/run_native.py \
  --binary /absolute/path/to/frozen/graphshell-coop \
  --djinn /absolute/path/to/frozen/djinn \
  --output /absolute/path/to/a/new/private-run-directory
```

The receipt is bounded to the recorded stack, macOS renderer, same-host QUIC
route and semantic control dispatch. It does not establish internet traversal,
browser/PWA hosting, physical keyboard/IME or OS assistive activation, document
body sharing, guest offline editing, or power-loss durability. The reusable
co-op owner and endpoint accept the existing transactional backend boundary;
this native launcher uses a Redb store. Existing browser IndexedDB and resident
stores are not automatically exposed or migrated by launching it.

## Qualified result, 2026-10-20

The current-stack `native-05` run passes all seven phases and 18 individually
reviewed native captures. Seven distinct native processes and four custodian
launches cover two independently held persona roots. The graph grows from one
to three sources, the returning guest revises its source, both applications and
custodians stop, and the owner cold-opens the same three sources with their
saved attribution and revoked grant. A fourth owner source saves after restart;
the old guest invitation is refused as `Delegation(Revoked)`. The private sibling
keeps its original object, activity and stored journal position.

The runtime source is `639751fe904ca0b7a868737c43c6d68b1b715fc2`, integrating
current main's shared DOM semantics and resident diagnostics. `build-inputs.json`
records 3,205 repository source hashes, both executable hashes and the frozen
runner hashes. Separate application and resident dependency trees record their
actual feature graphs. The Git supplier audit checks 19 pinned code checkouts.
The full selected Graphshell library gate passes 337 tests with six existing
ignores, including eight co-op tests. The focused custody policy regression also
passes. Port boundaries, changed-file formatting and Clippy pass; the package
formatter's existing differences in 12 untouched files are retained separately.
No full-workspace, WAN or browser/PWA acceptance is claimed.

Publication also incorporates main's subsequent Tabard example repair. That
example is absent from both executables' compiler inputs. Cargo normalized one
dependency-array order in `Cargo.lock`; its parsed dependency resolution is
unchanged. Fresh locked, offline builds after integration produce byte-identical
Graphshell and Djinn executables to `native-05`. `publication-inputs.json`
records the final source hashes and that equivalence, while the original frozen
`build-inputs.json` remains intact. The verifier checks both source snapshots,
the lockfile equivalence and the recorded compiler inputs.

The earlier passing `native-04` cohort remains intact in
`prior-cohort/native04`, qualified against implementation commit `b48fd8b40`.
The diagnostic ledger records three earlier rejected attempts and the fixes.

Verify this packet and its exact committed source snapshot:

```sh
python3 ports/graphshell/docs/receipts/coop_20261020/verify.py --commit HEAD
python3 ports/graphshell/docs/receipts/coop_20261020/verify.py \
  --packet ports/graphshell/docs/receipts/coop_20261020/prior-cohort/native04 \
  --commit b48fd8b40
```

Vaults, stores, keys, live grants and executables stay private. The public packet
contains synthetic addresses, public test identities, process observations and
reviewed presentations only.
