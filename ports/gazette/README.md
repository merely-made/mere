# gazette

**Gazette** is the Mere platform's directory port: who someone is, where they
can be reached, and what they publish.

Named for the official gazette, and the name carries the whole roadmap in its
three senses. A *gazetteer* is an index. To be *gazetted* is to be officially
announced, and thereby resolvable. A *gazette* is the paper you read. Those
were never three crates, which is why the resolver was promoted from the
dramatis tier to a port on 2026-08-23 rather than having one founded beside
it.

Like [castellan](https://crates.io/crates/castellan), the port splits in two:

- **the embeddable half** — contact cards, and the one recipient picker Knot,
  Moot, and Signalman all draw instead of three private lists;
- **the authority half** — resolution, feed fetching, contact intake, and
  announcing, as a service djinn composes beside castellan's (ruled
  2026-10-01). Djinn is the always-on party, so it owns lifetime and
  scheduling; gazette owns the policy.

Reading a friend's feed reveals your interest to their host, so which
persona's network face does the fetching is a first-class setting here, not an
afterthought.

## State

**Built:** the embeddable contact Ledger projection, WebFinger resolution, and
supplied WebFinger address intake into Gaz.
The Ledger reads contacts × selected facets, carries contributor provenance,
addresses repeated list/row/detail instances, composes recipient-picker and
Ledger clauses through `chirograph::CoordinatedSelection`, emits a semantic
table, and cites its contact and facet authorities separately through
`incipit::ShelfmarkV1`.

WebFinger resolution
([RFC 7033](https://www.rfc-editor.org/rfc/rfc7033)) — an `acct:user@host`
handle to its JRD document, with aliases and links classified into typed
peer-discovery endpoints (gemini capsules, gopher resources, misfin mailboxes,
ActivityPub actors, HTTP profile pages, and a typed catch-all). NIP-05,
atproto-did, and the moot web-of-trust directory land beside it behind the
same facade.

`intake::WebFingerIntake::from_import` accepts a queried account and an existing
`WebFingerImport`, retains that resolver result, and exposes unverified address
claims for `gaz::ContactBook::intake_addresses`. The query selects the contact;
aliases never select or refile it. Valid account and generic DID aliases become
names, including `did:key` and `did:plc` strings, without supplying keys or
authenticated identity. Unmapped or malformed targets remain in the source.
The host retains that source if it needs resolver provenance; Gaz's contact
record remains its own model, and `WebFingerImport` is a classified result
rather than a complete JRD archive.

This adapter deliberately requires the returned subject to match the queried
account under Gaz's normalization. RFC 7033 permits a different subject, but
account migration needs a separate checked path. Returned subject and account
aliases require an explicit `acct:` scheme. Account user case is retained,
host case folds, and percent encodings normalize through Gaz's shared parser.
The bounded input accepts ASCII DNS/A-label hosts and bracketed IPv6; Unicode
account construction belongs to the host. The adapter does not fetch or mutate
a book. Repeated intake adds only missing addresses and preserves existing
trust and use history. Actual authenticated resolution, live host/store wiring,
NIP-05 and checked PLC resolution remain open.

**Unbuilt:** attaching the Ledger and recipient picker to the live
[gaz](https://crates.io/crates/gaz) store in a host, feed polling (whose engine is
`mere-crawl`), and the reading room over extracted articles.

WebFinger's wire code moves to
[finger-protocol](https://crates.io/crates/finger-protocol)'s sans-io
`webfinger` feature (ruled 2026-10-01): it builds the request URL and models
the whole JRD in both directions, which announcing needs, and it compiles for
the browser, which the blocking `reqwest` fetch here cannot. Gazette keeps its
endpoint classification and intake and drops the fetch, so the caller
supplies HTTP. Its resource normalization (a bare `user@host`, an `acct:` URI,
or a URL with its origin and port) moves upstream into finger-protocol.
Unbuilt.

`article_from_html` is the supplied-HTML-to-`fleece::Article` seam a future
poller calls. It accepts declared HTML only and neither fetches nor stores a
document. Feed polling, stored feed/article state, and the reading-room surface
remain unbuilt Gazette work.

## Boundaries

The boundaries are the point: not
[castellan](https://crates.io/crates/castellan) (which guards and presents
*you* — gazette finds *the other players* and hands what it learns to gaz,
which keeps them; the dramatis tier's two ports, pointing opposite ways), not
[gaz](https://crates.io/crates/gaz)
(the contact store this port composes rather than replaces), not a delivery
layer (private grants, cross-service posting, and inboxes are moot and murm
territory — gazette reads what is already public, and announces what
castellan has issued), and not the highlights (what you keep is Knot's; what
memory makes of it is alembic's).

Announcing is the one place gazette faces outward (ruled 2026-09-30):
castellan signs a persona's presentation and picks its grade, and gazette
serves it at that persona's handle (its WebFinger document, `nostr.json`, its
card) so other people's gazettes can resolve it. Gazette holds no key and
signs nothing. Announcing exports static files first (ruled 2026-10-01): a
WebFinger document must be served over HTTPS at the handle's own domain, and
djinn's site service binds loopback Gemini only today, so gazette writes the
documents for any HTTPS host to serve, and djinn carries them once it can bind
publicly. Unbuilt; see the dramatis tier architecture in mere's
`design_docs`.

Lives in the [mere](https://github.com/merely-made/mere) workspace at
`ports/gazette`.

## License

MPL-2.0 (see LICENSE).
