# Resident delegation by petition

**Status (2026-10-25):** design note with questions P1–P8 for Mark. Nothing in
it is implemented. Read at mere `6c0a52c04`, turnstone `d9efd17`, eidetic
`aa94fe2`.

**Origin.** Doctrine Stack 2 ("a product never manufactures keys, grants or
trust roots") and the 2026-10-24 harness ruling that welcomed the Stack 2 fix
in Turnstone's `denizen.rs`.

## 1. What is wrong

- **Turnstone writes the grant.** `install` (`turnstone/src/denizen.rs`
  ~1045) calls `issue_install_certificates_for_generation`. That function builds
  each root certificate with `servitor::delegation::root_certificate`, choosing
  the capability (`install_caps`), the resource (`denizen:<subject>`), depth 0,
  no expiry and a nonce. Djinn only signs it. `remote_projection.rs:172` does
  the same for the projection endpoint's layout delegation.
- **Custody signs without looking.** `CustodyIdentity::issue_certificate`
  checks only that the certificate is well formed and that the issuer is the
  master. It then sends a plain `CustodyCall::Sign` under the scope's salt.
  Djinn's `sign()` (`ports/djinn/src/custody.rs`) checks `policy.allows_act`
  against `with_act(Persona, Prefix(delegation_signing_prefix("mere.denizen")))`.
  That rule covers the domain only, so custody never sees the subject, the
  capability, the resource or the expiry, keeps no record, and shows the
  owner nothing.
- **The owner reviews in the wrong place.** The owner reviews the install in
  Turnstone's own `PendingInstall` pane. Castellan's README says custody
  "answers participant-gate petitions", and the OTP release gate does work that
  way, but delegations do not.
- **Revocation is local.** Uninstall calls `DelegationTable::revoke_root_grants`
  in Turnstone's memory. `CustodyIdentity::issue_revocation` exists, but no
  Turnstone code calls it, and Servitor's `DelegationTable` cannot adopt a
  `SignedDelegationRevocation`; it takes only an id. So another holder of the
  certificate cannot verify that a resident was revoked.
- **Graphshell's guest invitations have the same shape.**
  `ports/graphshell/src/coop` builds each `DelegationCertificate` and has
  custody sign it under the `mere.graphshell` act rule. That path does sign its
  revocations.
- **The local door trusts whatever app id a client declares.** `graphshell-mcp`
  (and every app on the owner-only socket) names its own app id.
  `AllowedAppRoutes` admits it by that name. The remote route already does
  better: per T5 (`2026-10-15_remote_mere_route_admission.md` §4), it accepts an
  app only when a delegation leaf names it.

What already holds and should stay: the master key never reaches Turnstone. A
locked vault refuses install. Install checks the new chain under the current
root before it records the admission. `rebuild` adopts stored chains only if
they verify, and otherwise refuses (`a_rerooted_profile_refuses_automatic_certificate_reissue`).
The lifecycle record (`resident_admission`) is what tells revoked and paused
apart from "needs migration heal".

## 2. What would improve it

### 2.1 A typed issuance call, after `IssueStationGrant`

Custody already has one call that issues from a typed request rather than
signing opaque bytes. `CustodyCall::IssueStationGrant` takes a
`StationGrantRequest`, and djinn builds and signs the grant itself
(`issue_station_grant`). A resident delegation fits the same shape:

```text
CustodyCall::PetitionResident { petition: ResidentPetition }
  ResidentPetition {
    resident:   ResidentId (16 bytes, host-assigned, as Servitor says)
    subject:    Subject   (32 bytes)
    body:       BodyRevision (32 bytes) + kind + label, for presentation
    generation: u64       (must exceed any generation custody has recorded for this resident)
    grants:     [(cap wire string, Mode)]
    expires_at_ms: Option<u64>
  }
CustodyAnswer::ResidentChain { petition: Uuid, chain: Vec<SignedDelegationCertificate> }
  | Refused / Pending
```

Custody owns everything a product should not choose: the issuer, `parent =
Root`, depth 0, the resource (derived from the resident, not supplied), the
nonce (custody's, keyed on resident, generation and cap), `issued_at` from its
own clock, and an expiry ceiling. A product may ask only for `mere.denizen`
capabilities. Turnstone still proposes the capability set, since a petition is
supposed to say what it wants. The difference is that custody presents the
request and decides.

### 2.2 The owner decides on custody's surface

The approval machinery already exists. Castellan's `ApprovalBroker` keeps
pending requests, visible decisions, a history and a short-TTL cache. Djinn's
identity projection turns `pending_signing` into cards with Approve once and
Deny. A petition could become a second kind of pending entry there (a
`pending_petitions` list beside `pending_signing` in `IdentitySurfaceSnapshot`,
so the SSH-shaped `SigningRequest` stays as it is), rendered as a card that
names the resident, its body revision, each capability and the expiry.

There is a gap here. `native_only` on the signing cards changes the
explanation text, but `IdentityEndpoint::invoke` does not check which
application invokes the action. I did not verify whether a local-door app can
reach that endpoint. A petition decision is worth exactly as much as the rule
that keeps the petitioning app from pressing Approve.

One honest limit: on a single OS user, a same-owner process can drive any
GUI. What this design adds is that custody composes, records and can revoke
every resident grant. The owner sees custody's account of the grant, not the
product's. A hostile local process is not kept out.

### 2.3 Custody keeps a register

For each petition, custody records the requesting app, resident, generation,
issued delegation ids, expiry and later revocations, sealed per persona like
Castellan's other records. With a register, djinn's identity surface can list
every resident grant across products and revoke one without the product
running. That is new: today only Turnstone knows its residents' grants.

### 2.4 Signed revocation

`CustodyCall::RevokeResident { resident, generation }` would look up the
register and sign one `DelegationRevocation` per issued id (the
`issue_revocation` path, now with content custody chose), then return them.
Revocation only narrows authority, so it needs no approval. Custody records
it, and the owner can trigger it from the surface. On the holder's side,
`DelegationTable` would need to adopt a signed revocation and check its issuer
against the root (an Eidetic change). Then a revoked resident fails
verification for every holder, as device-grant revocation already does since
the 2026-08-12 migration.

### 2.5 Turnstone receives

Turnstone keeps `load_certs`/`save_certs_strict`, the probe-verify of a
received chain, `rebuild`'s verify-or-refuse, and the lifecycle record. It
drops `issue_install_certificates*`, `cert_nonce`, `residency_resource` as an
issuer input, and the `root_certificate` import. `PendingInstall` remains the
place to compose the petition (rings, watch, deadband), and its Install button
becomes "ask custody". `ring.rs:581` and `component.rs:175` already use a
`cfg(test)` `Local` root and can keep an in-memory issuer.

### 2.6 Closing the blind path

The petition call fixes Stack 2 only once `with_act(Persona,
Prefix(delegation_signing_prefix("mere.denizen")))` is gone. Until then,
anything admitted can still have custody sign an arbitrary `mere.denizen`
certificate. Existing chains stay valid after removal, because the scope key
does not change. Only new blind signing stops.

### 2.7 MCP admission (optional, same call family)

Local-door admission could reuse T5's rule: a hello naming an app is accepted
only with a delegation whose leaf names `…/app/<app>`. That delegation would be
issued by an owner-approved petition with an expiry, so each harness can be
revoked on its own and expires. Two limits apply. First, the harness has to
hold a key and the delegation somewhere a same-owner process can read.
Second, binding to the binary needs the peer's executable identity (the
`/proc` check that Castellan's Secret Service adapter does on Linux, or code
signing and audit tokens on macOS). So this buys expiry, revocation and
attribution, not unforgeability.

### 2.8 Stale plan text

The rebuild warning in graph behaviors plan §8.1 is stale (corrected beside
this note): `rebuild` no longer reissues, and the lifecycle record now covers
revoked and paused versus migration heal.

## 3. Questions for Mark

- **P1. The word.** TERMINOLOGY's *petition* is a participant's proposed change
  checked against its grant. Castellan already uses it for OTP release
  approval. Is "petition" right for "an app asks custody for a delegation", or
  should it have another name?
- **P2. Resident identity.** Option A: `Subject` stays the pack's blake3, and
  the petition adds `ResidentId` and `BodyRevision` for the record and the
  card. An edited pack is still a new subject and a fresh petition. Option B:
  custody derives `Subject` from `(persona, ResidentId)`, so the subject is
  stable across body edits. A new body then becomes a generation+1 adoption
  petition that custody presents as "same resident, new body, keep grants?",
  which is the §8.2 adoption event. B moves identity off pack bytes. A changes
  less. Both keep `ResidentId` host-assigned, as Servitor's docs say.
- **P3. One decision or two.** Should Turnstone's review pane become only the
  composition step, with the decision on djinn's card? Or should both stay,
  with Turnstone's review first and custody's approval second?
- **P4. The projection endpoint.** Its layout delegation has no owner review
  today. Should custody issue it under a standing policy (composed and
  recorded by custody, no prompt, like `SigningPolicy::Session`), or should
  the owner approve each one?
- **P5. Expiry.** Today every resident certificate has `expires_at_ms: None`.
  Should residents get a custody ceiling with renewal, or stay unbounded and
  rely on revocation?
- **P6. Scope of the cut.** Should Graphshell's guest invitations (the
  `mere.graphshell` act) move to a typed call in the same pass, or stay as
  they are?
- **P7. Wire types.** Servitor's `ResidentBinding`, `Cap` and `Mode` have no
  serde. Should the custody wire mirror them as bytes and wire strings
  (`StationGrantRequest` does this), or should Servitor gain serde (Eidetic
  joins the repin)?
- **P8. MCP admission.** Should §2.7 go in with this work or stay a cue?

## 4. Cost and what it batches with

The cut touches Mere (custody wire, djinn's handler and register, broker
kind, identity cards, removal of the act rule) and Turnstone. Under P7 or
§2.4 it touches Eidetic as well. Any Mere change brings the
Knot/Redshank/Turnstone repin and a 1–2 h validation run. Other Mere-side
changes known on 2026-10-25 could share that run: the MCP door's admission
(§2.7), Graphshell's guest invitations (P6), and the stack modularity map's
Mere cuts (mere-transport's fused iroh implementation, `mere-fetch`,
`mere-mesh`, `graphshell-host`'s default webrtc-join). On the Eidetic side,
the signed-revocation adopt could ride with stickleback's `net` feature from
the same map.
