# Portable application experience: spaces, applets, browsing and sharing

**Date:** 2026-11-04
**Status:** design direction from the 2026-11-03–04 discussion. The product
requirements below are recorded owner choices; interface sketches, service
selection and qualification order are proposals. This brief ships no feature
and does not establish browser, mobile, performance or anonymity qualification.
**Home:** Mere owns this cross-application experience record. The
[family composition thesis](2026-08-12_family_composition_thesis_brief.md)
owns the underlying composition model; the
[release-log plan](mere_docs/implementation_strategy/2026-11-03_release_log_plan.md)
owns release identity, verification, retention and carriage. This brief adds
the user-facing contract across clients, rather than another transport plan.

## 1. The application direction

Mark's framing:

> making, sharing, and using data, ephemeral to persistent spaces with anonymous to reputational stakes, and applications/applets from the stack and beyond

Persistence, identity, stakes and tools are independent choices. A temporary
space can have accountable participants; a retained collection can use an
unassociated identity. The interface should not infer public distribution,
reputation or indefinite retention from the mere fact that a space exists.

A **mere is a data domain**, not one application and not a mandatory graph
interface. Domain owners retain their native data and mutation rules.
Applications present, project and act on those domains through admitted
interfaces. A common resource identity connects views without making a
projection the authority for its source. A host forwards a typed intent to the
owner for authorization and execution; rendering an editable control alone
confers no write permission.

The goal is practical cross-device applications and user-controlled storage:

> first and foremost, i'm making applications, not novel transports.

Anytype-like freedom to share and Syncthing-like control of one's storage are
experience references. They are not decisions to adopt either product, replace
the stack's storage, or introduce a new privacy network before applications
can ship. The fork naming direction is `p2mere-*`, as recorded in release-log
D7; naming direction does not prove crate publication or consumer migration.

## 2. Applets fit mods and contributions

An applet should be shareable and forkable like other content, with access to
the data explicitly granted to its instance in a space. Keep the existing
layers rather than introducing a competing plugin registry:

| Concern | Place in the model |
|---|---|
| Portable package and source lineage | Content/release or pack identity, declared requirements and provenance |
| Loading and compatibility | Existing mod/package admission and a supported execution dialect |
| What it contributes | Presentation, pane adapter, pane or servitor |
| What an instance may do | Host-issued grants scoped to subject, space, data and operations; domain owner checks |
| How it appears | A frame, an embedded presentation or a larger composing application surface |

The [GUI catalog](mere_docs/design/2026-10-26_gui_catalog_design.md#8-refinement-of-rulings-1-and-3-a-frame-hosts-one-presentation)
and `crates/system/registry/src/mod_loader/loader/contributes.rs` own the
contribution vocabulary. A package signature establishes a provenance claim;
it does not authorize execution, network access or reading every mere.
Forking content does not inherit its author's signature or the original
instance's grants. A changed package must be admitted on its own terms.

Three integrations remain distinct. An ordinary page or PWA can be displayed
where framing and browser policy permit. A cooperative applet can implement a
bounded, explicit host interface. A native application can expose a reusable
presentation or app-owned projection provider. Merely embedding a PWA supplies
neither the host API nor the ability to host other applications.

The existing **no injected web APIs** ruling remains: ordinary pages receive
no ambient access to the host. A scoped cooperative web-applet interface would
need its own design and review against that ruling. Its exact API is open.
The [capability door](mere_docs/implementation_strategy/2026-10-30_capability_door_plan.md)
and [app projection offers](mere_docs/implementation_strategy/2026-10-30_app_projection_offers.md)
provide existing authority and provider seams. The bounded capsule dialect in
`ports/graphshell/src/capsule_applet.rs` is not a general applet SDK.

## 3. Focused applications and composing hosts

**Focused mobile applications must work offline with Graphshell absent.** They
own their local domain behavior and persistence. Graphshell mobile can provide
broader reservoir access and compose selected presentations, without making
each focused application carry a general mod workbench.

This is a product-profile cut, not a universal ban on mobile hosting. Rendering
an applet, composing presentations and serving data in the background have
different requirements. Mobile serving remains constrained by platform
lifecycle and foreground/background policy. Sharing an identity does not make
two app sandboxes a shared store or guarantee that a suspended peer is awake.

On desktop, a focused application can stand alone or add the shared Graphshell
host capability. Graphshell can host the application's exposed presentation.
Domain storage, headless operation and graph UI remain separable. Embedding a
native application requires a presentation module, projection protocol or web
surface it actually exposes; it is not arbitrary process-window embedding.

Keep resource identity, access identity and appearance identity separate. One
resource can appear in several panes with different navigation, placement and
selection state. Closing a pane releases its instance and grants according to
their lifecycle, without deleting the domain resource or another appearance.

## 4. The browser experience to build toward

The first extension experience is Turnstone relating browsing as it happens,
with smolweb search and reading in the attached browser. It should:

- Fleece pages into link clouds. Hovering a node exposes the exact source
  context for the relationship, including several contexts for one resource.
  Observed hyperlinks, browsing traversals, annotations and generated
  suggestions retain their distinct evidence and provenance.
- Arrange the graph automatically, dynamically and manually, using fields,
  tags and other selected mechanics. Automatic updates respect manual intent.
- Open ordinary web nodes as pages in the attached browser; present supported
  smolweb documents within the extension's own reader surface.
- Expand from the browser-attached surface to a workspace/new-tab or home
  entry where supported. Reuse presentation code in a domain-hosted PWA, while
  declaring each host's actual permissions and available capabilities.

A PWA does not acquire extension tab/history access by sharing its UI. Browser
capture needs deliberate permissions and local provenance. The current
[extension portal](../ports/graphshell/web/extension/README.md) is a starting
seam, not proof of this complete experience.

**60 fps is the interaction target**, across declared supported hardware and
representative workloads. It is not a current result. Extraction, indexing,
persistence, remote reads, layout updates and inference need bounded work and
backpressure. Measure frame pacing and missed refreshes while ingesting,
hovering, dragging and opening pages, including older supported hardware; an
idle graph or average fps alone does not qualify the experience.

The native application adds direct protocol access, durable hosting,
publication and richer wgpu-backed content surfaces. Selected Woodshed,
Redshank, Isocosm, Signalman, Knot and Cleromancy presentations can compose as
they expose the required contracts. Distillery enrichment stays optional and
asynchronous. WPT engine coverage and Widevine integration are separate
qualification efforts; neither gates the useful initial browsing companion.

## 5. Extension-only smolweb and the mesh

**The extension should work without a native companion.** For protocols the
browser cannot open directly, an HTTPS retrieval gateway can fetch the native
resource and return bounded original bytes plus typed protocol metadata. The
extension parses, renders, Fleeces and stores locally. Preserve the canonical
source address separately from the gateway URL and route.

Define a narrow gateway profile before building it: supported schemes and
read operations, source/status metadata, redirects and prompts, byte/time
limits, failures and origin/CORS policy. Rich submissions require an explicitly
supported route and authorization; never downgrade an unsupported submission
to GET. Search needs an index or search provider separately selected from the
fetch route.

For Gemini, remote TLS and origin certificate observations are the gateway's
observations. They are not a browser-to-Gemini-origin authenticated channel.
Client-certificate identity needs a deliberately chosen credential route; a
shared gateway must not silently supply its own identity as the reader's.
The [Gemini protocol](https://geminiprotocol.net/docs/protocol-specification.gmi)
and [extension network rules](https://developer.chrome.com/docs/extensions/develop/concepts/network-requests)
bound this adapter; HTTP transport does not erase native protocol semantics.

Gateways can be services offered by one's own devices, selected contacts or
other holders. Keep these roles distinct:

| Role | What it does and learns |
|---|---|
| Holder | Retains and serves specified release bytes; learns requests it handles |
| Fetch gateway | Fetches a source on a reader's behalf; sees that source request |
| Privacy relay | Forwards under a defined route profile; has its own metadata exposure |
| Discovery/catalog service | Supplies provider hints, not content authority or grants |

One device can offer several roles. Giving it one role does not authorize the
others. Own paired devices are natural preferred providers; selected contacts
can hold role-specific agreements. A fresh extension still needs replaceable,
redundant bootstrap hints or catalogs. Bootstrap discovery need not receive
each requested URL, and an unknown mesh cannot discover itself without an
initial reachable source of hints.

The accepted boundary is: **no gateway receives the complete browsing history
or graph, and no single gateway is mandatory for every smolweb request.** Send
only the request needed for the selected service. Do not bulk upload ordinary
tab history for route selection. This boundary does not promise that a
provider cannot correlate the requests it handles.

Route choice should normally be automatic within the user's policy and be
inspectable: “via your desktop”, “via Alex's gateway”, or the actual relay and
gateway arrangement. Distinguish locally verified release evidence from
provider-reported transport facts. No silent downgrade of the selected route's
identity or privacy requirements.

An optional [OHTTP route profile](https://www.rfc-editor.org/rfc/rfc9458.html)
can be considered later. It brings separate relay/gateway trust and deployment
requirements; it is not the default prerequisite. Native fetching gives
control of the requests the native service owns, including credentials and
route policy. It does not make ordinary browser tabs anonymous. The browser
still handles displayed plaintext. The
[Tor Browser guidance](https://support.torproject.org/tor-browser/security/using-tor-with-other-browsers/)
is relevant prior art for why an extension plus proxy cannot claim a whole
browser's anonymity properties.

This retrieval gateway is distinct from release-log R9's **publication HTTP
gateway**, which makes held releases readable by mainstream browsers.

## 6. Contacts, discovery and voluntary seeding

Knowing a person, proving an endpoint belongs to them, authorizing a service,
and verifying a release are separate judgments. Preserve the distinctions
between own devices, known contacts, recent contacts, shared-space peers and
referrals. A referral or recent encounter supplies evidence, not inherited
permission to inspect private meres, retain data or fetch on one's behalf.
Serving verifiable public bytes asks less trust than observing unsigned-page
requests.

Proposed composition, following existing owner boundaries:

| Owner | Responsibility |
|---|---|
| Murm | Carrier operations, discovery observations and route facts |
| Gazette / Gaz | Endpoint resolution / retained contacts, bindings and purpose-specific evidence |
| Djinn and the composing application | Admission, provider selection under policy and bounded service lifetime / product wording and user choices |
| Signalman contribution | An inspectable network/discovery/route presentation that can be mounted as a pane |
| Domain/release owner | Resource authority, admissible access and verified content |

The Signalman discovery pane is a proposal. Its existing LXMF Comms adapter
does not already implement this broader service catalog. Owner context lives
in `dramatis/design_docs/2026-09-30_dramatis_tier_architecture.md`,
`murm/design_docs/2026-11-02_signalman_management_surface_implementation_plan.md`
and `murm/design_docs/2026-10-28_transport_consolidation_plan.md`.

Fetching, caching, retaining and serving are distinct states. A user can opt
into “keep and help serve”; possession alone does not advertise availability.
Seeding needs retained bytes, a reachable admitted serving path and availability
advertisement. Holder evidence establishes what was retained under its stated
terms, not perpetual future availability. A holder does not become the author.

## 7. Link first; authority belongs to what is being shared

Mark chose a portable link before a contact flow:

> create a link, first and foremost. my contacts exist in tons of places, at the os level, with various services, and being able to share data across 'em to the correct address is valuable. so a portable link beats a contact flow.

Make **Copy link** primary; OS share sheets and service-specific delivery are
optional destinations. Creating a link must not require importing contacts or
choosing a recipient. Creating an identity-restricted grant still requires
enough identity evidence to bind its audience; delivery to an address alone is
not that evidence.

| What is shared | Who controls access and subsequent lifecycle |
|---|---|
| Reference/address to another party's resource | That resource's authority; forwarding the reference grants no new control over it |
| Access to one's own mere, resource or annotated view | The owner can constrain audience, operations and permitted delegation; forwarding the link need not grant access |
| An exported annotated edition one serves | The publisher selects one reader, a private kin/kith subset or public distribution, subject to rights in included material |
| Synchronized replica in an existing domain/namespace | Its authorized synchronization and retention rules continue to apply |
| Independent retained export | The recipient controls their retained copy; the original publisher can stop serving or change future access, not erase that copy remotely |

The recipient default is **a copy/access they control**. Ongoing access to the
author's working state is a deliberate co-op session, governed Moot exposure
or published site, with that mode's authority and availability terms. A signed
edition retains its source evidence; editing/forking creates new lineage and
does not let the recipient speak as the original author.

Do not equate an unlisted link, bearer capability and named-recipient grant.
The link's intended semantics must be explicit, including audience, referenced
edition or live scope, verification anchor and whether retention is
independent. These are requirements for a link design, not a new wire schema.
Release-log D9–D11 and R6 carry the corresponding plan refinement.

### Distributed deletion and Willow

[Willow's data model](https://willowprotocol.org/specs/data-model/) uses
destructive replacement and prefix pruning: authorized newer entries remove
superseded entries from the participating synchronized store. That lifecycle
differs from keeping every release in an append-only history. It cannot erase
a separately exported copy kept outside that synchronization domain.

[Meadowcap](https://willowprotocol.org/specs/meadowcap/) controls scoped read
and write authority. It does not by itself cause remote erasure. The
[Willow URI proposal](https://willowprotocol.org/specs/uris/) also distinguishes
a digest-qualified reference from retention: a matching entry may have been
overwritten or pruned, in which case retrieval fails. A precise reference does
not pin bytes. These are semantic precedents, not an adoption decision.
Authorization, synchronization and retention would all need to implement any
promised destructive lifecycle; adding grants to the current log is
insufficient.

## 8. Interoperability constraints and next design work

Use existing standards where they describe a real boundary. Preserve their
scope and maturity rather than treating a standards name as qualification:

| Boundary | Candidate or constraint |
|---|---|
| Browser surface | WebExtensions, PWA manifests, origin/CORS and framing policy; extension permissions remain host-specific |
| Browser applet distribution | Extension-store restrictions on remote executable code; an isolated web frame does not inherit privileged extension APIs |
| Admitted application interfaces | Existing WIT/component and capability-door contracts; a generic third-party SDK still needs design |
| Resource and annotation exchange | RDF and domain selector profiles as supported by the owner; RDF 1.2 Concepts is a Candidate Recommendation Snapshot, not a Recommendation; preserve unfamiliar data and reject unsupported transformations |
| Optional HTTP data interoperability | Linked Web Storage is a Working Draft to evaluate, not an adopted replacement for the mere model |
| Embedded miniapps | W3C MiniApp manifest, packaging and lifecycle drafts are discontinued; useful prior art, not a future compatibility promise |
| Browser engine compatibility | WPT tests platform behavior; EME support and licensed Widevine/CDM integration require separate evidence |

Maturity checked 2026-11-04. Primary references: [Web App Manifest](https://www.w3.org/TR/appmanifest/),
[Chrome MV3 policy](https://developer.chrome.com/docs/webstore/program-policies/mv3-requirements),
[Component Model](https://component-model.bytecodealliance.org/),
[RDF 1.2](https://www.w3.org/TR/rdf12-concepts/),
[LWS Core](https://www.w3.org/TR/lws10-core/),
[MiniApp Manifest](https://www.w3.org/TR/miniapp-manifest/),
[MiniApp Packaging](https://www.w3.org/TR/miniapp-packaging/),
[MiniApp Lifecycle](https://www.w3.org/TR/miniapp-lifecycle/),
[WPT](https://web-platform-tests.org/),
[EME](https://www.w3.org/TR/encrypted-media/),
[Widevine](https://developers.google.com/widevine/drm/overview).
Native distribution also needs the chosen platform's current applet, sandbox
and background-execution rules; this brief grants no deployment exception.

Proposed first qualification path: browse and Fleece locally; open the graph's
nodes in the attached browser; read smolweb through a replaceable gateway;
retain a verified resource offline; create a portable link and open it in a
second client under the selected audience semantics. A focused mobile app must
reopen its local domain with Graphshell absent. Then add native-provider
selection and opt-in holding. Keep source, synthetic tests, headed browser
behavior, mobile lifecycle, physical frame pacing and cross-device retention
as separate evidence.

Open design work: the precise link grammar and bootstrap policy; gateway
response/credential profile; applet packaging, ABI and upgrade/grant lifecycle;
supported mobile host profiles; permission presentation; and measured graph
workload bounds. The existing owner plans remain the implementation homes;
this ordering does not declare their open phases complete.
