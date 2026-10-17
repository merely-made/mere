"""One-shot full Fetch checkpoint; append evidence, preserve production/WIP."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
 for value in values:
    at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
    if at is None:items.append(value)
    else:items[at]=value
old=load('errand_review_receipt.json');probe=load('fetch_probe_receipt.json')
assert probe['tests']==11 and probe['test_exit']==0 and probe['generated_output_removed']
assert '36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in (HERE/'fetch_all_features_suite.log').read_text(encoding='utf-8-sig')
assert '16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in (HERE/'fetch_no_default_suite.log').read_text(encoding='utf-8-sig')
tree=(HERE/'fetch_no_default_tree.log').read_text(encoding='utf-8-sig')
assert not any(line.split()[0] in {'armillary','eidetic','pandect','inker','errand'} for line in tree.splitlines() if line.split())
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/system/fetch/'))
assert len(complete)==11 and sum(inventory[p]['chars'] for p in complete)==137838
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1145 and stats['read_chars']==15318751
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'observed through current all-feature Cargo-built Fetch APIs; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'fetch_review_receipt.json','diagnostic':'fetch_probe_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test source','size':size,'reason':repair}}
findings=[
finding(179,'Loading an empty persona retains the preceding persona cookie session',
 'crates/system/fetch/src/cookies.rs:session_jar; cookies_persist.rs:load_cookies',
 'After a synthetic cookie is persisted for persona A, load_cookies for empty persona B leaves the same cookie readable from session_jar. Updating that inherited cookie and persisting for B writes it under B\'s durable prefix. load_cookies merges records into a process-wide jar rather than replacing or selecting a persona-scoped jar. Its docs say a persona with no stored cookies leaves the jar empty and keys provide a hard partition.',
 'Current public loader/persister and real MemoryBackend; only synthetic cookies and in-process storage. No current application persona switch or real login exposed. Current Mere caller search found declarations/tests rather than a production caller of these persistence helpers.',
 'persona/session owner migration can exceed one-hour diversion',
 'Bind a jar and its load/persist state to the host-selected scope, or provide an explicit safe single-persona lifecycle that refuses cross-persona reuse. Keep the real Stores scope seam rather than adding another global selected-persona variable. Define failed/partial load behavior and exclude old-persona cookies before exposing the new scope.'),
finding(180,'Incremental cookie persistence uses one shadow across all personas and stores',
 'crates/system/fetch/src/cookies_persist.rs:last_persisted and persist_cookies',
 'Persisting the same synthetic cookie for persona A writes one blob. After marking dirty, persisting for persona B writes none: shadow.get matches A\'s cookie although B has no durable blob. Changing the cookie then writes into B. last_persisted is documented per persona but is a single HashMap keyed only by domain/path/name; store identity is absent too.',
 'Executed two-persona calls on one in-memory store, independent dirty marks and actual Backend list/get observations. Cross-store aliasing follows the same source structure but was not separately executed. This is distinct from the jar retention in Q179.',
 'scoped persistence shadow/ownership change can exceed one-hour diversion',
 'Associate the incremental shadow with the actual jar/store/persona owner and seed/reset it consistently on successful loads. Durable prefix partitioning alone cannot make an unrelated global equality cache authoritative for whether a particular destination contains a cookie.'),
finding(181,'Expiry alone never re-arms cookie persistence, so expired durable blobs remain',
 'crates/system/fetch/src/cookies.rs:COOKIES_DIRTY; cookies_persist.rs:persist_cookies',
 'A Max-Age=1 synthetic cookie is persisted, expires and disappears from usable jar records. A subsequent persist_cookies call leaves its durable blob because the dirty gate returns before comparing expirations. Explicitly marking dirty then persisting deletes it. The persister says gone/expired cookies are deleted, but elapsed time does not mark a write.',
 'One bounded1.5-second synthetic expiry, MemoryBackend only, usable-record control and explicit-dirty deletion control. No expired authentication was accepted: the jar correctly refuses the expired cookie. The defect is durable cleanup/currentness.',
 'contained expiry-aware dirty/currentness correction',
 'Account for the next expiry or perform a bounded cleanup comparison at persist boundaries. Preserve the optimization for unchanged live cookies while distinguishing no new Set-Cookie from no elapsed state change.'),
finding(182,'Untrusted Content-Range controls vector capacity and can panic before body admission',
 'crates/system/fetch/src/handle.rs:NetFetch::read_range',
 'An open-ended range with Content-Range bytes0-18446744073709551614/18446744073709551615 passes ordering/request checks, then Vec::with_capacity(expected as usize) panics with capacity overflow before reading the empty body. A one-byte range control succeeds. catch_unwind records the failure without attempting a huge allocation.',
 'Real NetFetch and current Netfetcher pipeline with a supplied one-hop wire response. This is a native public API panic from header admission, not an observed remote process crash or peak-memory/OOM test.',
 'contained checked capacity/bounded range admission; open-ended budget policy may be broader',
 'Do not allocate the entire peer-declared range up front. Validate requested ranges and checked platform-size conversions, grow within an explicit caller budget and return FetchError on capacity/representation failure. Unbounded open-ended reads need a named budget policy rather than treating a syntactically ordered header as allocation authority.'),
finding(183,'The HTTPS-only blob adapter accepts plaintext HTTP as a successful HTTPS source',
 'crates/system/fetch/src/https_blob.rs:HttpsFetcher::fetch_https and BlobFetcher::fetch',
 'BlobSource::Https containing an http:// loopback URL returns the synthetic plaintext bytes successfully through HttpsFetcher. The implementation dispatches by enum variant and sends the embedded string to ureq without scheme admission. The original test named fetches_https_response_body also uses mockito\'s plaintext HTTP URL.',
 'Actual local HTTP exchange and public BlobFetcher API. No TLS server, downgrade redirect, remote origin or real content credentials. Hash verification remains delegated to resolve_blob; this finding concerns transport/scheme identity rather than digest validation.',
 'contained initial scheme validation; redirect transport policy needs explicit owner review',
 'Validate that the declared HTTPS source uses HTTPS, and apply a clear policy to subsequent redirect hops in the chosen agent. A source variant name and a passing test whose fixture is plaintext do not establish TLS. Keep custom-agent/proxy configuration under the caller owner.'),
finding(184,'Fetchable scheme selection rejects canonicalizable uppercase addresses that the fetch API serves',
 'crates/system/fetch/src/lib.rs:scheme_of, is_fetchable, is_smolweb and do_fetch_ua_with_context',
 'is_fetchable returns false for a valid HTTP:// loopback address while fetch_page_capped parses it through url::Url and successfully returns its bytes. The lowercase control is admitted. Prefix selection splits the raw :// string and compares case-sensitive names, while the transport parses canonical URL components.',
 'Actual selector and local HTTP acquisition for the same address. No external navigation or host registration decision was executed. The analogous uppercase Smolweb routing disagreement is source-qualified, not separately exercised here.',
 'contained shared URL-component classification correction',
 'Classify the parsed/canonical scheme using one URL implementation and pass the resolved identity across routing boundaries. Keep an is-supported predicate separate from syntactic URL validation instead of adding more raw-prefix tables.'),
finding(185,'The legacy crawler entry still uses browser cookies and automatic redirect traversal',
 'crates/system/fetch/src/lib.rs:fetch_page_crawler, do_fetch_ua and session_stores; crawler.rs public single-hop API',
 'fetch_page_crawler sends the synthetic browsing-session cookie on both hops of a local302 and returns the followed target body under the bot UA. The maintained fetch_crawler_capped control sends no cookie and returns302 without following. Anonymous-page control also sends no cookie. Current Fleece crawl code uses the maintained crawler API; the old public helper remains with prose claiming the crawl actor uses it for every page/robots request.',
 'Actual loopback requests and headers, synthetic cookie only. This proves divergent public crawler implementations and stale ownership prose, not that current Fleece acquisition leaked a login or crossed a robots-policy gate.',
 'crawler entry consolidation/compatibility decision can exceed one-hour diversion',
 'Name/deprecate the legacy contract or adapt callers to the admitted single-hop API, preserving status, response budgeting and caller redirect policy. Merely changing the User-Agent on a session fetch does not supply the maintained crawler\'s isolation semantics.'),
finding(186,'Smolweb page caps wait for full acquisition and rely on a nonexistent Gopher read bound',
 'crates/system/fetch/src/lib.rs:fetch_page_capped, fetch_page_interactive_capped and fetch_bytes; actual gopher-protocol0.2.0 client send_and_read',
 'A Gopher fixture flushes eight bytes under max_bytes4 and keeps the socket open. fetch_page_capped remains pending until EOF, then returns the cap error. The dependency reads into Vec with read_to_end and no byte bound. The function discloses post-hoc Smolweb checking but justifies it by saying errand bounds its own read; that assurance is false for this actual route. The global PAGE_BODY_CAP prose also claims a hard streaming/OOM ceiling.',
 'Eight synthetic bytes, one owned loopback connection, observed pre-EOF pending and terminal cap refusal. No large transfer, allocation exhaustion or remote denial-of-service executed. Gopher dependency excerpt is source traced, not whole-file Mere coverage.',
 'bounded transport API and progress/cancellation migration can exceed one-hour diversion',
 'Carry a byte budget to the actual protocol acquisition or stream into a bounded sink that cancels on overflow. Gate progress and terminal buffering under the same request budget. A final length check and suppression of excess progress do not bound upstream allocation or stop the transfer.'),
finding(187,'Identity download silently decodes an unexpected encoded body and reports the encoded length',
 'crates/system/fetch/src/handle.rs:NetFetch::read_into and facts; actual Netfetcher fetch/finalize.rs',
 'The handle sends Accept-Encoding identity. A supplied gzip response contains38 encoded bytes; read_into succeeds with18 decoded bytes in its sink and Facts.content_length=38. The maintained crawler rejects the same response as UnsupportedEncoding. Netfetcher decodes before the handle sees the body, and the no-store response retains encoded Content-Length. The handle comment claims identity makes the declared length equal the length written.',
 'Current real decode pipeline with a supplied one-hop transport and bounded gzip fixture, no live HTTP/TLS wire or user file. The observed mismatch is in successful custody metadata; no arbitrary server protocol compliance is assumed.',
 'response-representation admission must be shared before decode; can exceed one-hour diversion',
 'Choose whether downloads require identity/raw representation or expose explicit encoding and decoded-length facts. Refuse unexpected encodings before transformation for a raw-custody contract, or revise Facts and caller integrity/range assumptions. An outbound request header alone is not a response guarantee.'),
finding(188,'The five-redirect Smolweb limit actually allows only four redirects before a final response',
 'crates/system/fetch/src/lib.rs:MAX_REDIRECTS and smolweb_fetch_response',
 'A local Spartan chain with four redirects and a final success uses five requests and succeeds. With five redirects, the function performs five redirect requests and reports too many redirects before asking for the final target. The constant is documented as the most redirects it follows, but the loop counts total attempts.',
 'Two actual loopback Spartan chains, no external origin or mutation. This is a bound-definition/off-by-one mismatch; no unbounded traversal claim.',
 'contained redirect-versus-attempt counting correction',
 'Define redirects and initial request separately and test zero, boundary and overflow chains. Preserve per-hop timeout and identity scope checks when adding the terminal allowed attempt.'),
finding(189,'A matching Titan page identity is admitted by the host type but refused by its Gemini-only fetch helper',
 'crates/system/fetch/src/lib.rs:GeminiClientIdentity::new/applies_to and smolweb_fetch_response; crates/system/errand/src/lib.rs:fetch_url_with_identity',
 'GeminiClientIdentity accepts a Titan origin and its scope predicate treats Titan as matching. A Page command for that origin with the identity routes to errand::fetch_url_timeout_with_identity, which only permits Gemini and returns unsupported scheme titan before connecting. An owned listener confirms no incoming connection; dummy key material is never used in TLS.',
 'Actual spawned Fetch actor command and typed terminal result, matching Titan origin, supplied NoTransport for HTTP and an unaccepted loopback listener. No Titan upload, real certificate exchange or identity leak executed.',
 'read/write capability and identity-route decision can exceed one-hour diversion',
 'Reconcile accepted identity families with the operations that use them. Keep explicit Titan upload-with-identity and the Q311/A311 zero-byte generic-fetch write boundary in view; do not broaden a read helper to perform more implicit writes merely to make this path return success.'),
finding(190,'Lossless flip-cookie export erases host-only versus domain scope',
 'crates/system/fetch/src/cookies_flip.rs:session_cookies_for; crates/inker/inker/src/flip/api.rs:Cookie; surface_engine.rs:Cookie',
 'Two real CookieRecord states differ only in host_only. The host-only state yields no child-host cookie; the domain state does. session_cookies_for exports identical portable Cookie vectors for both. The source jar retains the distinction, but both portable cookie DTOs omit it while their comments claim session/metadata losslessness. This strengthens A262 from an absent-field concern into an executed export collision.',
 'Synthetic source jar, actual child-host matching and public portable export comparison. No WebView import, subdomain request, real cookie or confirmed browser cookie-scope broadening.',
 'portable cookie schema/backend capability migration can exceed one-hour diversion',
 'Carry host-only scope through the portable cookie vocabulary and adapter/capability boundary, or explicitly report/refuse the unrepresentable loss. Do not infer preservation from two DTOs mirroring the same incomplete field list. Keep partition-key/CHIPS decisions separately explicit.'),
]
# Correct the cross-reference before serialization: A311 is the saved read/write authority entry.
findings[-2]['repair_triage']['reason']=findings[-2]['repair_triage']['reason'].replace('Q311/A311','A311')
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All 11 Fetch inventoried files read;36 all-feature and16 no-default original tests pass;11 generated API observations pass',
  'verification':'fetch_review_receipt.json'}
entries=[
entry(315,'Host stores versus process cookie ownership','One host-selected scope shares jar/cache/HSTS/Alt-Svc/transport across handles and actor.',
 ['Stores has Arc-shared host-provided stores and optional Transport', 'Stores::context bridges boxes through shared wrappers', 'spawn_fetcher_with and NetFetch use the same Stores seam', 'session_stores is explicitly a legacy process-global default'],
 'The host scope seam is real and original tests observe handle/actor/resource cookie and transport sharing. Q179/Q180 concern the separate process-global persistence helpers, not proof that separate Stores instances leak into each other. The governing plan still names consumer T2 migration as open.'),
entry(316,'Persona persistence owner','Persona-prefixed cookie keys establish a hard partition.',
 ['One global session jar', 'One global dirty bit', 'One shadow keyed only by domain/path/name', 'load_records merges into the same live jar'],
 'Q179/Q180 prove persistent-key namespacing does not partition runtime identity or incremental destination currentness. The shadow comment says per persona but its type carries neither persona nor store. Durable/session and performance-cache ownership must move together.'),
entry(317,'Cookie expiry versus dirty-gated currentness','Persist removes cookies gone through expiry or clearing.',
 ['SharedJar and mark_cookies_dirty flag explicit writes', 'Jar matching prunes expired records independently', 'persist returns before taking current snapshot when dirty is false'],
 'Q181 isolates time-based change from explicit writes. Jar admission correctly expires authentication, while the durable shadow/blob cleanup does not run. Failure re-arming for explicit store writes is present and useful; it does not solve elapsed expiry.'),
entry(318,'Portable cookie metadata authority','Structured flip export and two portable DTOs preserve complete cookie metadata.',
 ['Netfetcher CookieRecord has host_only', 'Fetch maps into flip::api::Cookie without it', 'surface_engine::Cookie mirrors the same fields', 'Partitioned is explicitly forced false with a documented limitation'],
 'Q190 proves the undeclared host-only loss with different real source matching and identical exports. The declared Partitioned limitation is not conflated with this omission. A262 previously qualified the absence alone; this pass adds direct evidence without claiming an executed WebView security defect.'),
entry(319,'Range and allocation authority','A validated206 returns exact requested bytes or typed FetchError.',
 ['Content-Range syntax/order/request checks', 'No-store bypasses whole-response cache correctly', 'Netfetcher owns Range identity header assembly', 'Peer-declared expected length feeds Vec::with_capacity'],
 'Original range/redirect/validator/cache controls pass and actual dependency source confirms Range automatically requests identity, so the absent local encoding header is not itself a bug. Q182 is the pre-body capacity panic. Syntactic header/request equality does not supply a platform allocation or open-ended byte budget.'),
entry(320,'Download representation and custody facts','Identity request makes declared body length equal sink bytes.',
 ['read_into sends identity and no-store', 'Netfetcher still decodes response Content-Encoding', 'Noncacheable headers retain encoded length', 'Crawler independently refuses nonidentity encoding'],
 'Q187 shows the two acquisition paths enforce representation differently. Successful download metadata is misleading for an unexpected encoded response. Shared admission belongs before transformation, while web-page decompression remains a legitimate Fetch behavior.'),
entry(321,'New crawler acquisition versus legacy bot helper','Crawler work uses explicit admitted one-hop acquisition.',
 ['crawler.rs uses Transport directly, identity encoding, zero credential/session state and bounded chunks', 'Native adapter reuses Netfetcher WireTransport over explicit rooted HttpTransport', 'lib fetch_page_crawler wraps session page fetch with a bot UA', 'Current Fleece caller uses crawler::fetch_crawler_capped'],
 'Q185 proves legacy divergence while the maintained path behaves correctly. Source/plan confirm the former duplicate hyper exchange was removed from crawler.rs. Its separate pool is an intentional policy owner, not a second HTTP wire implementation. Old helper prose about the active crawl actor is stale.'),
entry(322,'HTTP versus Smolweb resource budget authority','Page and subresource caps stop runaway acquisition.',
 ['HTTP read_capped stops extending at a chunk boundary', 'Smolweb Errand response is fully buffered', 'Gemini progress suppression does not cancel the transport', 'Actual Gopher read_to_end has no byte cap'],
 'Q186 disproves the supposed dependency read bound for Gopher. Post-hoc checking is partly disclosed, but the global hard ceiling/OOM assurance does not apply uniformly. Protocol streaming and acquisition cancellation need budget propagation, not a second terminal length helper.'),
entry(323,'URL scheme authority across routing','A supported network address is recognized consistently.',
 ['scheme_of performs raw :// split', 'is_fetchable and is_smolweb use case-sensitive comparisons', 'HTTP/handle/identity constructors use url::Url', 'Errand sees canonical schemes only after parsed dispatch'],
 'Q184 is an actual supported HTTP address rejected by the selector. This continues Q172/Q178 and earlier routing findings: sharing transport libraries does not eliminate competing local URL admission implementations.'),
entry(324,'Smolweb redirect and identity authority','One page/subresource path owns timeout, redirects, protocol statuses and certificate scoping.',
 ['smolweb_fetch_response is actually shared by page and binary resource paths', 'Per-hop timeout and host/effective-port identity checks', 'MAX_REDIRECTS loop counts attempts', 'Identity family predicate includes Titan but Errand identity helper excludes it'],
 'The common response path is real reuse; Q188/Q189 show its limits. Typed input/certificate-change/certificate-required responses are retained here, an improvement over historical memory. Consumer display/conversation adoption remains separate from this producer evidence.'),
entry(325,'Explicit mutation versus generic fetch','Submitted writes are never deduplicated, retried or followed as reads.',
 ['Submit command routes Titan/Spartan through explicit write helpers', 'SubmissionAnswer returns redirects to host', 'Spartan response redirect is origin checked', 'Generic Scheme::Titan still routes a zero-byte upload in the supplier'],
 'Explicit submission boundaries are useful and original tests cover redirect/status shaping. A311 remains a broader conflict with generic Titan navigation. Q189 must not be repaired by erasing that write authority distinction. Submission response buffering has no analogous caller response cap in this layer; no giant write-response probe was run.'),
entry(326,'Optional HTTPS blob acquisition','HTTPS-only adapter supplies bytes; manifest resolver verifies declared hash.',
 ['HttpsFetcher owns a separate configurable ureq Agent', 'Header check plus max+1 read enforces a real configurable response cap', 'BlobSource variant dispatch does no URL scheme check', 'Async trait method synchronously calls the blocking agent'],
 'Q183 proves plaintext admission despite the named transport identity. A separate optional agent is explicit and caller configurable, so it is not automatically evidence every scoped NetFetch policy applies to blob acquisition. Blocking work and timeout/cancellation policy are source-qualified follow-ups. Hash verification intentionally belongs to resolve_blob.'),
entry(327,'Optional Iroh acquisition and hash/custody layers','Reuse existing host stack and content-addressed transfer.',
 ['IrohFetcher holds shared existing BlobStore/P2pandaTransport', 'Narrow bounded hex ticket parser and builder', 'Transport fetch persists under its requested blob hash, then get_bytes reads it', 'Manifest resolver separately checks declared content_hash when invoked'],
 'No second Iroh node is created by the adapter constructor. Original suite constructs two in-process peers and transfers a synthetic blob successfully. This is current bounded transport evidence, not physical-device acceptance or proof every direct BlobFetcher call also invokes the manifest-level hash check. The convenience ticket is a narrow local format, not a complete opaque upstream BlobTicket parser.'),
entry(328,'Feature boundary and dependency ownership','No-default handle avoids actor, persona, flip and Smolweb crates.',
 ['Four default-on feature declarations', 'Optional HTTPS/Iroh adapters', 'Fresh no-default normal dependency tree', 'Fresh all-feature36 and no-default16 library tests'],
 'The normal no-default tree contains none of armillary/eidetic/pandect/inker/errand. This verifies the named dependency cut rather than assuming it from cfg. Test-only transport/identity dependencies are separate and much larger; no wasm compile or every target/feature combination is claimed.'),
entry(329,'Public result facts and producer projections','Response identity survives where the vocabulary carries it.',
 ['NetFetch Facts retains final URL, content type, length and validators', 'Resources forwards final URL/type into ResourceResponse', 'Fetched page vocabulary has bytes/text/type but no final URL', 'SubresourceOutcome returns raw bytes only'],
 'The resource handle seam retains useful facts and original redirect controls pass. Legacy page/actor byte results discard final identity/MIME needed by some consumers; this reinforces Q156 across the producer boundary rather than inventing another format table. Read the shared page-load vocabulary next before choosing a migration.'),
entry(330,'Fresh gates versus semantic success claims','Passing current tests qualify their configuration and exercised paths.',
 ['36 all-feature original tests including HTTP fixtures and in-process Iroh peer transfer', '16 no-default original tests', '11 generated native observations expose12 qualified findings', 'Owned binaries/PDB removed and source/WIP hashes preserved'],
 'Green suites coexist with persona leakage, header panic, representation loss and divergent crawler APIs. HTTPS-named original fixture is plaintext and export collision is not a WebView import. No remote user data, real credential, browser/GPU or durable user profile was used in generated observations. Full Mere source objective is incomplete.'),
]
upsert(authority['entries'],entries,'id')
previous=next(e for e in authority['entries'] if e['id']=='A262')
previous['fetch_followup']={'verification':'fetch_review_receipt.json','diagnostic':'fetch_probe_receipt.json',
 'evidence':'Q190: distinct host-only/domain CookieRecord states have different child-host matching but identical actual session_cookies_for output. This upgrades the prior absent-field concern to a public export collision; no WebView import/security bypass executed.'}
upsert(notes['candidates'],[
 {'title':'Typed shared fetch result needs response identity without another consumer sniff table',
  'source':'crates/system/fetch/src/lib.rs:Fetched shaping and SubresourceOutcome; crates/system/page-load',
  'note':'Facts/Resources retain final URL/type, while Fetched and byte-only subresource results cannot carry all of it. Current source reinforces Q156 producer loss. Read page-load/current callers before introducing a new outcome type. Coordinated MIME/final-URL/parser authority migration may exceed one-hour diversion.'},
 {'title':'Blob adapters need a distinct blocking/runtime and acquisition budget owner',
  'source':'crates/system/fetch/src/https_blob.rs and iroh_blob.rs',
  'note':'HttpsFetcher async method calls blocking ureq synchronously and default agent declares no local timeout; with_agent permits host configuration. Iroh fetch persists then reads whole blob into bytes with no per-call size parameter. No stuck-agent, runtime starvation or large-blob/OOM experiment executed. Keep manifest identity/hash policy separate from transport acquisition budget and avoid imposing unrelated page defaults without an owner decision.'},
 {'title':'Actor HTTP deadlines, concurrency and completion after panics need a caller policy review',
  'source':'crates/system/fetch/src/lib.rs:spawn_fetcher_with, do_fetch_ua_with_context and fetch_bytes',
  'note':'HTTP actor tasks are unbounded runtime.spawn calls; NetFetch has header/per-chunk patience but actor HTTP path has no local matching timeout or permit. Page tasks track cancellation by request; favicon/subresource/write tasks are not in that map. A protocol panic could bypass a terminal update. No saturation/timeout/completion-loss repro executed. Do not call every-request completion guaranteed from success/failure fixtures alone; broader scheduling/cancellation policy can exceed one-hour diversion.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/system/page-load/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete Fetch 11/11 source review including all optional blob adapters and tests'
notes['next_area']='crates/system/page-load'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['fetch_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'All-feature36 and no-default16 original library tests pass;11 generated observations pass and expose12 findings. No-default normal graph excludes the five named optional dependencies.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'11 complete files137838 chars added. Dependency and governing design excerpts traced without whole-file local reading credit.'}
upsert(notes['source_qualifications'],[{'area':'Complete Fetch source review',
 'note':'All11 inventoried source/manifest/test files137838 chars read, including optional HTTPS/Iroh source. All-feature36 and no-default16 original tests pass;11 generated public API observations pass. Generated observations use synthetic cookies/MemoryBackend, owned loopback HTTP/Gopher/Spartan and an unaccepted Titan listener; no real credentials, remote user data, browser/GPU, WebView import, TLS session or durable user files. Original suite includes two Iroh peers created in-process and a successful synthetic transfer. Capacity-overflow panic caught without huge allocation. Normal no-default tree verifies named optional dependency cut. Exact hashed artifacts retained for evidence; native probe executable/PDB removed; stable target reused, no isolation.'}],'area')

doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only Fetch complete source review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll 11 inventoried Fetch files are read, adding 137,838 characters, including optional HTTPS/Iroh adapters and their tests. Repository coverage is 1,145/3,352 files and 15,318,751/36,854,820 characters. Full-source goal remains active. Production and original tests/concurrent work are unchanged. Resume at '+next_path+'. Authority entries A315-A330 track scoped stores versus global cookie helpers, duplicate cookie metadata, representation/budget policy, crawler paths, real protocol reuse and feature evidence.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='The host-selected Stores seam works: shared Arc-backed cookie/cache/HSTS/Alt-Svc/Transport owners feed both handles and the actor, and original controls observe scoped cookies, redirects, no-store ranged reads and supplied wire use. The process-global cookie jar/shadow helpers are a separate legacy ownership model. Durable persona prefixes do not partition the jar or make one global equality cache current for each store/persona. Q179/Q180 expose that actual conflict; Q181 isolates expiration cleanup from authentication admission. The latter is correct in the source jar even when the durable blob remains. Current Mere production caller search did not establish a running persona-switch consumer of these helpers.\n\n'
addition+='A262 now has a concrete follow-up: host-only and domain cookies have different native child-host matching yet the actual flip export is identical. The flip and surface Cookie DTOs both omit this distinction while claiming metadata/session losslessness. This is an executed export-loss proof, not a WebView import or a confirmed subdomain security bypass. The declared Partitioned limitation remains a different issue. Schema/backend capability migration can exceed the audit diversion limit.\n\n'
addition+='The maintained crawler reuses the extracted HTTP wire implementation. Its separate rooted, retry-disabled pool is deliberate policy, not a duplicate hyper transaction. Current Fleece uses that admitted single-hop path. A public legacy helper still changes only the browser-session User-Agent and follows redirects with cookies, while claiming active crawler ownership in its prose. Q185 proves divergence without attributing the old path behavior to the current Fleece caller. Existing anonymous page fetch also passes the no-cookie control.\n\n'
addition+='The common Smolweb response path genuinely shares timeouts, redirect/status handling and certificate scoping between pages and binary subresources. Typed input and certificate conversations are retained by this producer, superseding historical memory about string-only results at this layer. Q188/Q189 show bound/identity mismatches within the shared path. Generic Titan still delegates to a zero-byte upload in its supplier (A311); broadening identity dispatch without settling read/write capability admission would be a poor local repair. The final Smolweb cap and suppressed progress do not cancel upstream reads. Actual registry Gopher uses unbounded read_to_end; Q186 gives a bounded eight-byte/pending-before-EOF observation rather than attempting exhaustion.\n\n'
addition+='NetFetch Range correctly obtains identity encoding through Netfetcher header assembly, so a missing local header is not a second encoding bug. Q182 is the separate vector capacity trust error. Downloads differ: an unexpected gzip response is decoded but retains its encoded length in no-store facts; the crawler explicitly refuses the same encoding. Requesting identity is not proof that the response obeyed it. ResourceResponse retains final URL/type, while legacy page/byte result projections lose some response identity needed by downstream consumers; Q156 is reinforced at its producer rather than repaired with another host sniff table. Read page-load next.\n\n'
addition+='The optional HTTPS adapter has a real max+1 byte cap and a caller-configurable separate ureq agent, but its named HTTPS-only scheme admission is absent. Its async implementation blocks synchronously, and no local default timeout is set in this source. IrohFetcher reuses host BlobStore/Transport and does not create another node; it parses a narrow local hex ticket and reads the fetched content-addressed blob. Manifest verification belongs to resolve_blob when that path is used; a direct BlobFetcher call is not automatically a second manifest-level check. Blocking/timeout/whole-blob budget policy and actor concurrency/deadline completion remain source-qualified larger follow-ups, not reproduced hangs or OOM events.\n\n'
addition+='Fresh all-feature library build and 36 original library tests pass. A separate no-default library gate passes16 tests. The no-default normal dependency tree names none of armillary/eidetic/pandect/inker/errand, verifying the governing D3 feature cut; this does not describe the much larger test-only dependency graph or wasm. Eleven generated public API observations pass and expose12 recorded findings. Cookies/backend contents are synthetic/in-memory, all generated HTTP/Gopher/Spartan connections are owned loopback fixtures, and the Titan dummy identity is refused before a connection or TLS use. The header capacity panic is caught without huge allocation. The original all-feature suite separately includes two Iroh peers constructed in-process and a successful synthetic blob transfer. No browser/GPU/WebView import, real credential, remote user content or durable user file was used by generated probes. Probe executable/PDB removed; stable target reused, no isolated target, Cargo home or worktree. Evidence/source hashes are in the [Fetch review receipt](../../../support/receipts/authority-success-audit/fetch_review_receipt.json).\n'
save('review_notes.json',notes);save('authority_map.json',authority)
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'fetch_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['fetch_probe.rs','run_fetch_probe.py','fetch_probe_receipt.json','fetch_probe_build.log','fetch_probe_output.log',
 'fetch_library_build.jsonl','fetch_library_build_stderr.log','fetch_all_features_suite.log','fetch_no_default_suite.log',
 'fetch_no_default_tree.log','fetch_no_default_tree_stderr.log','fetch_diff_check.txt']
genet=Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/e37762d/components/netfetcher/src')
traces=[genet/'cookie_jar.rs',genet/'context.rs',genet/'fetch/request_headers.rs',genet/'fetch/finalize.rs',
 Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gopher-protocol-0.2.0/src/client.rs'),
 ROOT/'design_docs/mere_docs/implementation_strategy/2026-09-20_ranged_fetch_plan.md']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete Fetch source reading; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'updated_authority_entries':['A262'],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'all_features_library_tests':36,'all_features_test_exit':0,'no_default_library_tests':16,'no_default_test_exit':0,
 'no_default_normal_graph_excludes':['armillary','eidetic','pandect','inker','errand'],
 'diagnostic_tests':11,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'excerpt_only_traces':{str(p):sha(p) for p in traces},
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production/original test changes.',
  'All11 Fetch files read; governing design/dependency excerpts are not whole-file denominator credit.',
  '36 all-feature and16 no-default original tests pass;11 generated current-library API observations expose12 findings.',
  'Original Iroh suite creates two peers in-process; no physical-device/real remote-user transfer claim.',
  'Generated cookies/Backend synthetic and in-memory; loopback fixtures only, no durable user files.',
  'Host-only cookie evidence is export collision and matching behavior, not executed WebView import or confirmed scope broadening.',
  'Capacity panic caught without large allocation; eight-byte cap proof is not an exhaustion test.',
  'HTTPS proof is actual plaintext loopback success; encoding proof is supplied transport plus real decode pipeline, not live TLS.',
  'No browser, script, GPU, real credentials, public server or user persistence in generated probes.',
  'No-default dependency cut applies to normal graph; test dev dependencies and wasm/all targets are not qualified.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
save('fetch_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
