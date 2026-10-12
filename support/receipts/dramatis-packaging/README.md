# Dramatis Windows packaging receipt

**Status (2026-10-11): Windows debug packaging/Luggage path qualified.**

The native package uses cargo-packager 0.11.8, NSIS, the existing
`luggage-manifest` binary and a dedicated synthetic signing key. The exact
artifact/feed hashes, compiled input fingerprint, commands and results are
in `receipt.json`. `contents.json` records the extracted binary hash and build
information. Receipt logs, JSON and public keys preserve exact bytes across
Git checkouts. The public keys here are fixture keys, not production keys.

| Gate | Result |
| --- | --- |
| Pre-package locked build and version/source/lock checks | Passed, one compiler job |
| NSIS pack and artifact signature | Passed |
| Luggage manifest and detached manifest signature | Passed |
| Extracted binary version and source identity | Passed |
| Extracted native window with absent synthetic Djinn endpoint | Passed; capture visually reviewed |
| Actual artifact download and durable stage/reload | Passed |
| Unsigned/edited manifest, other valid publisher, bad artifact signature | All refused |
| Modified artifact and staged bytes | Both refused |
| Feed restoration after controls | Exact hashes match |
| Real-package integration test | 1 passed |
| App state / retained interaction | 4 / 7 passed |
| Dependency bans and portable metadata | Passed |

The installer carries the explicit Dramatis UI binary, notices, full license
and public `release-info.json`. The compatibility declaration requires an
existing user-scoped Djinn supporting the quarantine routes from Mere
`ec1e65ca` or a descendant, with `cxf`. No custody binary, fixture host,
identity file, signing key or shared-vault deletion hook is shipped. The
extracted app's pending view was inspected with an unavailable synthetic
endpoint, without accessing an actual vault.

Reproduce with `python support/dramatis_package_receipts.py`. The gate builds
from the reusable `C:/t/cargo-targets/mere` cache with one compiler job. It
generates disposable keys for its child signer, extracts only its own NSIS
package, exercises real Luggage guards, restores feed bytes and removes
the private signing keys, staging and extraction. `preliminary.zip` retains
the first verifier-feature failure and earlier bounded package runs.

Retained output is the marker-owned `C:/t/cargo-targets/mere/dramatis-packaging/dev/nsis/`
test-signed installer/feed for review, plus generated configuration/build
information in its parent. These are generated packaging output, owned by
the Dramatis lane. No isolated worktree, Cargo target or Cargo home was used.

The release recipe is `ports/dramatis/RELEASING.md`. This debug artifact does
not qualify an optimized build, actual installation, Authenticode,
Gatekeeper/notarization, automatic update controls or public publication.
The Mac signing option is host-configurable and awaits native qualification.

| Remote host | Status |
| --- | --- |
| ThinkPad / Fedora | Pending until reachable |
| Intel Mac | Pending until reachable |
| ARM Mac | Pending until reachable |

No remote job or monitor was created. Resume serially with an existing
approved target, a build-owner/memory check and one compiler job by default.
