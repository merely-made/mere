# Remote mere route admission

**Status (2026-10-15):** the shape the transport agnosticism note's rulings T4,
T5 and T7 ask the implementing pass to write down before serving the routes
remotely. Implemented with this note in Mere; the Djinn binary does not start
the host yet (§7, Progress).

**Origin.** The [transport agnosticism note](2026-10-15_transport_agnosticism_note.md)
§7 and §9: Djinn runs a dialable projection host on a dedicated resident key
(T7); the persona's delegation names the application that may act (T5); same
persona devices dial by device grant and other people's Turnstone as place
members by delegation, and `ambient/mere/<domain>` is exposed remotely as
well, with "its own admission and grant shape" set out first (T4).

## 1. What is already true

- Notochord admits a session for one requested action
  (`domain`, `path`, `action`) when the hello's delegation chain covers it and
  the owner's policy has a service rule that structurally owns the path
  (`LocalNetworkPolicy::service_rule`: the most specific rule whose path covers
  the requested one, at slash boundaries). A child path under a base service
  is admitted by the base's rule, and a delegation for a child path cannot be
  widened: `CapabilityScope::attenuates` requires the child's `path_prefix`
  to sit under the parent's and its actions to be a subset.
- On a carrier that authenticates the peer (p2panda), the hello's subject
  must be the carrier-proven peer.
- On the local door, `mere/<domain>` and `ambient/mere/<domain>` are opened by
  `AccessEndpoint`, which requires an admitted application and rechecks the
  owner's per-application decisions (`AppRouteGrants`: the default grant for
  admitted applications, recorded denials, the ambient opt-in) on every read
  and write.

So an application claim and a route choice fit in the requested path with no
change to Personae, Insigne or Notochord.

## 2. The paths

Under Graphshell's domain (`mere.graphshell`) and projection service
(`/services/projection`):

| Route | Requested path | Action |
|---|---|---|
| `mere/<domain>` | `/services/projection/mere/<domain>/app/<app>` | `connect` |
| `ambient/mere/<domain>` | `/services/projection/ambient/<domain>/app/<app>` | `ambient` |

`<domain>` and `<app>` are single path segments (no `/`, not empty). The
grammar is `mere_overmap::wire`'s (`RemoteMereRoute`), so a dialler and the
host parse one spelling.

## 3. The grant shapes

A grant is an ordinary delegation certificate. Its scope:

- **Explicit:** `domain = mere.graphshell`, `resource` = the dialable host's
  network id, `path_prefix` at or under
  `/services/projection/mere/<domain>/app/<app>`, `actions = {connect}`.
- **Ambient:** the same, with `path_prefix` at or under
  `/services/projection/ambient/<domain>/app/<app>` and
  `actions = {ambient}`.

The two never confer each other: their paths are disjoint and their actions
differ, and attenuation can narrow neither into the other. A broader grant
(`/services/projection/mere`, all meres) may be issued, and narrowed by its
holder, but see §4.

## 4. The application claim (T5)

The application is the `<app>` segment of the admitted path, and it is
accepted only when the delegation chain's leaf scope names it: the leaf's
`path_prefix` must sit at or under `…/<domain>/app/<app>`. A hello that names
an application its delegation does not name is refused before any endpoint
opens, so the persona's delegation, not the dialler, says which application
acts. The admitted endpoint context then carries that application
(`AdmittedEndpointContext::with_application`), exactly as the local door's
does, and `AccessEndpoint` applies the owner's per-application decisions
unchanged: a remote `turnstone` is denied a mere the owner denied to
`turnstone`, and reads ambient only where the owner opted `turnstone` in.

## 5. Who can dial (T4)

The dialable host's policy is closed and offers exactly two service rules:
`/services/projection/mere` (`connect`) and `/services/projection/ambient`
(`ambient`), both `MemberOnly`. Plain projection sessions
(`/services/projection` itself) are not served on this host. Trusted roots:

- **Same-persona devices:** the persona's own root. A device grant is a
  delegation from the persona's master key to the device's master key, with
  one of the scopes above.
- **Other people's Turnstone, as place members:** each place's root, added by
  the resident's configuration when the owner joins the place. A member's
  chain runs from the place root to the member's key; the member's own
  delegation to its Turnstone narrows to that application.

The host binds the hello's subject to the carrier-proven peer (p2panda), so a
grant is usable only from the key it names.

## 6. The host (T7)

`djinn::resident_projection::ResidentDialHost`, on a dedicated resident key:
derived from the persona with the salt `mere.djinn/resident-projection/v1`,
which neither personal sync's per-graph keys nor places' keys share. It
serves `ResidentProjectionHost` over a catalog of only the routes it exposes
(`mere/<domain>` and `ambient/mere/<domain>` for the meres the reservoir
serves), choosing the route per session from the admitted path
(`graphshell_host::projection_host::RouteSelector`). Its admission is the one
entry (`carrier::admit_stream` through `accept_projection_session`) and the
one action check (`admission::serves_action`, which now also serves the
`ambient` action under the ambient path).

## 7. Not decided here

- How an owner's Djinn learns a place root and how a place membership
  becomes a configured trusted root (Turnstone's place host already holds the
  chain; the resident's configuration surface for it is a product step).
- Whether a device grant from `pandect::wallet_grant` is issued with these
  scopes by the pairing flow, or by hand until the pairing UI names them.

## Progress

- **2026-10-15:** written before serving the routes remotely, then
  implemented with the lift plan's pass (b): `mere_overmap::wire::RemoteMereRequest`,
  `graphshell_host`'s `RouteSelector` and `AMBIENT_ACTION`, and
  `djinn::resident_projection`, with Djinn's two dialable-host receipts (lift
  plan Progress). Not yet: the binary starting the host, which waits on §7.
