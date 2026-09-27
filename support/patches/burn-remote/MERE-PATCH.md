# Mere burn-remote patch

Source: crates.io `burn-remote 0.22.0-pre.4`, upstream Burn commit
`d3c8d7c615e5a1b739d9997de8a88dcd4dea638c`.
License: MIT OR Apache-2.0, unchanged from upstream.

The released server authorizes an Iroh session only at admission. Removing a
session from its manager does not stop the live duplex pump, whose task sender
can still accept work. Mere requires targeted close before owner revocation.

Each session is reserved before application authorization and carries a close
signal observed by the pump. This fences the authorization-to-worker-binding
interval. `IrohRemoteProtocol::sessions` exposes reserved or active session IDs
and opaque credentials; `close_session` targets one pump. Client close uses the
same teardown. The public exports include `ServedSession` and `SessionId`.

Rebased on 2026-09-27 from the preserved pre.3 delta at `610a32c5`. Seven
source deltas apply unchanged to pristine pre.4. The two server exports are
inserted alongside pre.4's new `ServerLogging` export. Both manifests retain
pre.4's `tracing-subscriber` server feature and add `tokio/macros` for Mere's
pump selection. The normalized manifest restores exact pre.4 test dependencies,
an empty workspace, and its self patch. No pre.3 source file replaces an
upstream pre.4 file wholesale.

This rebase alone is not lifecycle acceptance. Full two-peer reclaim/recovery
and headed checks remain gates in the migration plan. Remove the patch when an
upstream release exposes equivalent session control and passes those receipts.
