"""One-shot checkpoint: complete Errand reading, observed defects and owner map."""
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
old=load('nematic_review_receipt.json');probe=load('errand_probe_receipt.json')
assert probe['tests']==10 and probe['test_exit']==0 and probe['generated_output_removed']
assert '64 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out' in (HERE/'errand_suite.log').read_text(encoding='utf-8-sig')
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/system/errand/'))
assert len(complete)==22 and sum(inventory[p]['chars'] for p in complete)==133441
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1134 and stats['read_chars']==15180913
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'reproduced through current Cargo-built Errand/Nematic APIs; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'errand_review_receipt.json','diagnostic':'errand_probe_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test source','size':size,'reason':repair}}
findings=[
finding(169,'Directory root containment is lexical while reads follow junctions outside the published tree',
 'crates/system/errand/src/serve/dir.rs:Directory::resolve and Source::get',
 'An owned Windows junction inside the published fixture points at an owned sibling directory. Directory::get returns that outside sentinel as a Document. The /../outside/sentinel.txt control refuses. resolve rejects lexical parent escape, but tokio metadata/read follow filesystem links; the comment claims a request can never name anything above root however spelled or encoded.',
 'Actual Windows junction and file read, entirely under the marker-identified stable Cargo target. No user document or external capsule, no public listener. Junction removed non-recursively and exact fixture files removed after verification.',
 'filesystem capability and reparse-point policy can exceed one-hour diversion',
 'Define an explicit publication policy for links/reparse points and enforce containment at open/read time. A canonicalized precheck alone does not resolve check/use races. Preserve a trusted root handle or equivalent platform-safe capability rather than treating component arithmetic as filesystem authority.'),
finding(170,'Directory dotfiles are hidden from listings but directly served despite the not-published claim',
 'crates/system/errand/src/serve/dir.rs:listing and Source::get',
 'The owned .audit-hidden.txt fixture is absent from the root Listing but a direct request returns its bytes as a Document. listing explicitly says dotfiles are not published and suggests renaming to publish one; get applies no corresponding admission rule.',
 'Owned synthetic bytes only, direct Source API and root-listing control. No claim a user secret was exposed or an external server currently publishes this source.',
 'contained publication-policy consistency correction',
 'Make hidden-file publication configurable and apply the chosen policy consistently to listing, direct files, intermediate components and configured index lookup. If hiding only affects discovery, describe it accurately rather than claiming publication denial.'),
finding(171,'One structural listing entry can inject additional native protocol links or rows',
 'crates/system/errand/src/serve/project.rs:to_gemtext and to_gophermap; serve/mod.rs:Entry',
 'A newline plus gemtext link syntax in Entry.display makes one Entry become two parsed links. Tabs/CRLF in another display make one Entry become three valid Gopher rows, including an injected external destination. Actual shared protocol parsers confirm both. A benign Gopher entry remains one row. Public fields and constructors declare no validated-delimiter precondition.',
 'Pure formatter then actual parser observations, no served network bytes or navigation. This proves structural injection, not code execution or current hostile source access.',
 'shared field-admission/serialization contract may exceed one-hour diversion',
 'Centralize wire delimiter admission for all user-controlled display, target, title, preamble and host fields. Choose a visible rejection or representable normalization policy per format, because these protocols do not offer a universal escaping convention. Do not call parser-accepts-output tests proof that projection preserves source entry identity.'),
finding(172,'Absolute URL detection treats valid mailto destinations as local Gopher selectors',
 'crates/system/errand/src/serve/mod.rs:Entry::is_absolute; serve/project.rs:to_gophermap',
 'url::Url accepts mailto:reader@example.test as an absolute URL, while Entry::is_absolute returns false because it searches for ://. Gopher projection and reparse yield a gopher URL on local.test rather than the mailto destination. An HTTPS control remains external.',
 'Pure public Entry/project/parser composition. No email, process launch or navigation occurred. The finding concerns the documented absolute-URL versus host-path distinction.',
 'contained existing URL-parser classification correction',
 'Use the existing URL authority for absolute-reference classification with an explicit policy for local/network paths. Avoid another table of scheme punctuation or removal of intentionally supported opaque schemes.'),
finding(173,'Timeout helpers return failure before the common fetch tracing event can run',
 'crates/system/errand/src/lib.rs:fetch_url_traced, trace_fetch_result and fetch_url_timeout family',
 'A loopback Gopher connection waiting on an unaccepted local listener times out with Error::Timeout and zero errand events. The same capture receives one event for an immediate unsupported-scheme refusal. timeout wraps and drops the inner future before trace_fetch_result after await. The common-entry prose promises one structured event for every public fetch helper.',
 'Current Cargo-built native async API, current-thread subscriber, one owned loopback listening socket with no acceptor and no remote network. Identity/streaming variants share the source structure but were not separately executed.',
 'contained timeout-result tracing correction across helper variants',
 'Place final outcome telemetry where timeout and transport results converge, preserving one event per call and caller cancellation semantics. Do not simply add an outer event while retaining inner completion events, which would double successful requests.'),
finding(174,'Fetch tracing strips query and fragment but retains URL passwords',
 'crates/system/errand/src/lib.rs:trace_url and trace_fetch_result',
 'An unsupported HTTPS URL carrying synthetic userinfo reaches the Errand warning event with the synthetic password retained. Query and fragment are absent as controls. Only boolean observations are persisted, not the captured URL. trace_url only clears query and fragment.',
 'In-process tracing capture and unsupported-scheme refusal, no network or actual credential. The existing documentation promises query/fragment removal rather than explicitly promising password redaction; this is unnecessary credential-bearing logging, not a misquoted guarantee.',
 'contained userinfo redaction in the existing trace URL helper',
 'Remove username/password from logged URLs at the shared helper and keep the current query/fragment controls. Audit structured error text separately because a redacted URL field does not automatically sanitize third-party errors.'),
finding(175,'Feed loss diagnostics disappear at the shared AST to portable-document boundary',
 'crates/system/errand/src/parse/feed.rs:Feed::diagnostics; crates/nematic/nematic/src/feed.rs:build_document_blocks',
 'The current XML parser records two diagnostics for an unknown extension element and invalid enclosure length. FeedEngine renders the same input into nonempty blocks with no diagnostics. It only transfers the html_stripped counter into a rendering warning and drops Feed.diagnostics.',
 'Actual Errand XML parse and Nematic FeedEngine projection using one exact Cargo feature closure. No live feed, persistence or host degraded badge executed.',
 'contained diagnostic propagation at one projection boundary',
 'Carry parser diagnostics into the existing EngineDocument diagnostic model with source/engine attribution. Keep an unknown extension warning distinct from refusal and from HTML summary stripping; successful parsing is not evidence of lossless representation.'),
finding(176,'Feed namespace identity depends on prefix spelling while foreign local names can override core facts',
 'crates/system/errand/src/parse/feed.rs:parse, inspect_element and State::end_element',
 'The same iTunes namespace URI under prefix i loses artwork/duration and gains unsupported-extension diagnostics. Prefix itunes bound to an unrelated URI is accepted and supplies both fields. A foreign x:title before the core title wins the entry title. The walker uses Reader, raw qualified strings for selected extensions and local names for core facts without namespace resolution.',
 'Native parsing of bounded synthetic XML with equal-URI, alternate-prefix, unrelated-URI and foreign-title controls. No feed database mutation or remote spoofing claim.',
 'namespace-aware and context-aware parser migration can exceed one-hour diversion',
 'Resolve bound namespace identities and validate parent/context before admitting core or extension facts. Keep known extension support separate from unknown-extension diagnostics and preserve raw source where required. Do not patch this by adding more accepted prefix strings.'),
finding(177,'Arbitrary well-formed XML returns feed success and can become an empty successful document',
 'crates/system/errand/src/parse/feed.rs:parse and State::start_element; crates/nematic/nematic/src/feed.rs:FeedEngine::render',
 'A generic XML document with a title returns Ok Feed, then an empty EngineDocument with no diagnostic. A generic document containing entry/title fabricates a FeedEntry outside any RSS/Atom root. The walker checks XML structure but never admits a recognized feed root/context. document-lanes also selects FeedEngine for any XML declaration, so its fallback-on-error branch is not reached for this successful empty result.',
 'Errand and Nematic paths executed. The document-lanes XML sniff/fallback composition is source traced from its previous whole-file review, not a new hosted-session or screen reproduction.',
 'feed grammar/root admission plus downstream selector review can exceed one-hour diversion',
 'Require a recognized feed root and valid entry context, distinguish non-feed XML from an empty valid feed, then make the caller retain a visible fallback reason. Reuse the parser verdict rather than another caller-side XML name table.'),
finding(178,'Nex directory base resolution appends child names inside a query',
 'crates/system/errand/src/parse/nex.rs:base_url; crates/nematic/nematic/src/nex.rs:render_directory',
 'For nex://example.test/path/page?q=/wrong/ and child, the emitted URL keeps path /path/page and changes query to q=/wrong/child. The query-free control emits path /path/child. base_url uses raw ends_with and rfind slash over the full address, then Nematic concatenates the entry name.',
 'Actual parser helper through current NexEngine and url::Url inspection, no network or directory navigation. Query semantics for a remote Nex server are not asserted; the emitted child is demonstrably not a path resolution.',
 'contained URL-component-aware directory resolution correction',
 'Resolve against parsed URL path components and handle query/fragment deliberately, using the existing URL implementation. Keep entry-name admission and directory/content classification separate from address resolution.'),
]
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All 22 Errand inventoried files read; 64 library tests pass, 1 ignored; 10 generated native observations pass',
  'verification':'errand_review_receipt.json'}
entries=[
entry(300,'Protocol transport and grammar ownership','Spec-level transactions and grammars live in their protocol crates; Errand composes them.',
 ['Gemini/Gopher/Scroll grammar types are real re-exports', 'Finger/Gopher/Nex/Spartan/Guppy/Gemini/Scroll adapters call protocol clients', 'Misfin delegates to its client and keeps raw status', 'Nematic consumes the shared parser types'],
 'Real reuse is present. Errand owns normalized response/error mapping and shared routing, rather than another TLS/UDP/parser implementation for each consumer. The published dependency closure is outside the Mere source denominator; inspected dependency excerpts are separately qualified.'),
entry(301,'Single source and native protocol projections','One structural Listing projects faithfully into each protocol.',
 ['Public Entry/Listing source model', 'Gemtext/Gopher/plain formatters', 'Gemini/Gopher/Finger server adapters', 'Original tests reparse benign formatter output'],
 'The source seam and native output are useful shared implementation. Q171 shows its fields lack delimiter admission, so parseable output alone does not preserve entry identity. Item::Document is explicitly served as-is and carries a different responsibility from Listing projection.'),
entry(302,'Directory containment authority','Component walking proves requests cannot escape root.',
 ['resolve admits only lexical in-root components', 'metadata follows junctions', 'read follows the resolved filesystem target', 'Host can configure index names'],
 'Q169 disproves the universal containment claim with an actual owned Windows junction. Configured index-name joins can also cross lexical boundaries, but configuration is host supplied; no attacker-controlled configuration bypass is asserted. Filesystem authority must cover actual opened objects and reparse policy.'),
entry(303,'Publication versus discovery authority','Dotfiles are not published.',
 ['listing skips dot-prefixed names', 'get serves directly resolved files', 'Index lookup applies its own name loop'],
 'Q170 demonstrates discovery filtering and request admission are different implementations with different rules. A configurable publication policy should cover both. Hiding from a menu is not proof a resource is inaccessible.'),
entry(304,'Absolute URL authority in serving','An absolute URL must not be joined to the serving host.',
 ['Entry::is_absolute searches ://', 'to_gophermap uses that boolean to choose h URL versus local selector', 'url::Url already exists in the same dependency closure'],
 'Q172 shows a valid opaque absolute URL routed locally. This is a second address recognizer rather than a reason to add more per-scheme punctuation heuristics.'),
entry(305,'Nex address base versus URI components','One shared helper resolves a directory base for Nematic.',
 ['Errand base_url scans the whole string for slash', 'Nematic appends the entry directly', 'Other owners use parsed URL joins'],
 'Sharing the helper is real, but shared incorrect behavior remains incorrect. Q178 proves query text becomes the append destination. A shared URI implementation should own components; parser line classification is a separate concern.'),
entry(306,'Feed core and extension identity','One XML walker supports RSS/Atom and selected podcast extensions.',
 ['Local-name path/context state', 'Raw qualified prefix matching for extensions', 'Raw-prefix unsupported diagnostics', 'No namespace resolver or root-flavor admission'],
 'Q176 and Q177 qualify the semantic limits. This is one walker with conflicting identity rules, not separate RSS/Atom parsers. content:encoded is also marked unsupported by prefix inspection even though the end handler recognizes it, so diagnostics and fact recognition already disagree internally.'),
entry(307,'Feed degradation authority across projection','Feed diagnostics describe facts not losslessly projected.',
 ['Errand Feed stores unsupported-extension and invalid-length diagnostics', 'Nematic uses shared Feed shape', 'Nematic propagates html_stripped only', 'EngineDocument has a ready diagnostic channel'],
 'Q175 proves two existing warnings vanish at a shared boundary. A successful document loses evidence already available from its source parser; this is not an absence of diagnostic infrastructure.'),
entry(308,'Single load-bearing tracing entry','Every public fetch helper emits one event for its outcome.',
 ['fetch_url_traced emits after inner await', 'Streaming helper has its own after-await event', 'Timeout helpers wrap and drop the traced future', 'URL parse and some early checks precede that event path'],
 'Q173 proves timeout failure bypasses the promised event while an immediate refusal emits it. Public helpers do not converge at one final result boundary. Early parse failures and direct upload/send/exchange calls need explicitly scoped telemetry claims rather than assuming all protocol activity is covered.'),
entry(309,'Trace URL redaction','Query and fragment are removed before logging.',
 ['One trace_url helper clears query/fragment', 'The remaining URL still carries userinfo', 'Errors are independently formatted'],
 'The narrow query/fragment claim is accurate. Q174 demonstrates avoidable password retention, with only synthetic values used. A redacted URL field does not imply independently formatted protocol error text is safe.'),
entry(310,'Host-selected trust-store ownership','Hosts choose a durable or in-memory TOFU store.',
 ['Errand re-exports exact Gemini-protocol TofuStore/set_trust_store types', 'fetch::install_smolweb_tofu forwards the caller store', 'document-lanes::remote first-use OnceLock installs its own InMemoryTofu', 'Actual registry Gemini setter takes RwLock and replaces the store; latest call wins'],
 'The shared store type is real, but competing initialization owners can replace host policy. Source tracing now confirms the previous document-lanes candidate reaches a replacement setter, despite dependency overview prose saying installed once. No TLS handshake, current pin loss or MITM was reproduced; coordinated session/host ownership repair may exceed one hour.'),
entry(311,'Read fetch versus write capability','Write companions are direct calls outside fetch routing.',
 ['Errand introductory prose excludes write companions from fetch', 'Scheme::Titan is included and fetch_url_inner calls titan::fetch', 'Actual registry Gemini Titan fetch invokes zero-byte upload_inner', 'Explicit titan_upload and misfin_send remain direct APIs'],
 'The prose is false for Titan. A zero-byte upload is still a write-shaped protocol request; absence of payload does not establish read-only behavior. No endpoint mutation was executed. Caller resource/crawl admission must be reviewed before broadening read success claims; this is a larger capability boundary candidate.'),
entry(312,'Protocol metadata versus normalized response','Normalized Status and raw_status preserve different levels of detail.',
 ['Protocol-specific status mapping retains raw integer where meaningful', 'Scroll rich response available through scroll_fetch', 'Generic Response holds MIME/body/status', 'Trust-store implementation belongs to Gemini protocol'],
 'Dropping rich metadata in generic Response is an explicit narrow transport shape, with a real richer escape hatch for Scroll. Normalized success is not signature, trust, completeness, byte-budget or published-content proof. No additional unshared transport implementation was found in these adapters.'),
entry(313,'Live dependency identity versus inherited ownership prose','Protocol siblings are workspace members and feed parser is feature gated.',
 ['Errand manifest still says protocol crates are workspace members under protocols/', 'Root workspace dependencies resolve exact registry versions', 'Errand module prose still mentions parse-feed gating', 'Manifest includes unconditional quick-xml and has only serve/default features'],
 'Current Cargo artifacts establish the registry owners. The feed grammar is unconditional, consistent with the manifest explanation but inconsistent with module prose. The manifest license comment also claims MIT/Apache while the actual license and source headers say MPL-2.0. These are stale inherited claims; this audit makes no independent legal relicensing ruling.'),
entry(314,'Optional serving evidence and raw index documents','Serving projects listings, while documents are as-is.',
 ['serve feature adds three protocol server halves', 'Directory tries shared ordered index names', 'Adapters forward Document bytes rather than parse/translate them', 'Fresh suite includes actual loopback Gopher serve/fetch'],
 'All optional serving source is read and the current serve library suite passes. A gemtext index served through Gopher would remain raw document bytes by this contract, not a native structural Listing. MIME/index publication policy needs explicit caller intent; no cross-protocol index corruption or headed server UX is claimed from the passing benign listing test.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Smolweb resource fetch ignores configured ResourceFetchPolicy and installs process-global TOFU itself',
  'source':'crates/system/document-lanes/src/remote.rs:RemoteFetcher::new, smolweb_get_bytes, install_tofu; crates/system/fetch/src/lib.rs:install_smolweb_tofu',
  'note':'Errand review traces actual registry gemini-protocol0.1.7 setter: latest call replaces an RwLock Arc, not install-once admission. The document-lanes local OnceLock only guards its own unconditional InMemoryTofu installation, which can replace a host custom store supplied by fetch::install_smolweb_tofu. Source-proven competing policy authority, no TLS/pin-loss/MITM reproduction. Resource policy is still retained only by HTTP host. Coordinated ownership migration can exceed one-hour diversion.'},
 {'title':'Titan is included in generic fetch even though its navigation implementation sends an upload request',
  'source':'crates/system/errand/src/lib.rs and titan.rs; actual gemini-protocol0.1.7/src/titan.rs',
  'note':'fetch routing delegates Titan to zero-byte upload_inner over TLS. Introductory Errand prose says write companions are not fetchable schemes. Need caller capability/resource/crawler admission review before claiming fetch is read-only. No request sent or endpoint changed; document the boundary rather than run an external write proof. Can exceed one-hour diversion.'},
 {'title':'Directory index-name and raw-document policy needs explicit per-protocol ownership',
  'source':'crates/system/errand/src/serve/dir.rs:with_index_names and Source::get; serve/adapt.rs',
  'note':'Host-configured index names are joined without independent containment admission and shared across protocols, while Document bytes are served as-is. Host-selected config is not shown attacker-controlled. Separate trusted configuration, link policy, source publication and format conversion before adding more adapter guesses. Q169/Q170 qualify actual request-read/publication mismatches.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/system/fetch/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete Errand 22/22 source review, including optional serving'
notes['next_area']='crates/system/fetch'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['errand_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'Fresh serve-feature library suite:64 passed,1 ignored;10 generated native observations pass, including owned junction/files and one loopback timeout.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'22 complete inventoried files133441 chars added. External registry excerpts and fetch API excerpts traced separately without adding whole-file coverage.'}
upsert(notes['source_qualifications'],[{'area':'Complete Errand source review',
 'note':'All22 inventoried source/manifest files133441 chars read, including optional serve. Fresh serve library build, composition build and64 tests pass;1 live Nex smoke test ignored. Ten generated Cargo-built API observations pass. Actual owned Windows junction and direct file reads, formatter/parser roundtrips and one loopback timeout executed; no remote network, TLS handshake, browser, GPU, user persistence or public server. Registry Gemini TOFU/Titan and current fetch setter excerpts are source-traced dependencies/callers, not whole-file local coverage. Marker-owned fixtures, junction, executable and PDB removed; stable target reused, no isolation.'}],'area')

doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only Errand complete source review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll 22 inventoried Errand files are read, adding 133,441 characters, including optional serving. Repository coverage is 1,134/3,352 files and 15,180,913/36,854,820 characters. The full-source goal remains active. Production and original tests/concurrent work are unchanged. Resume at '+next_path+'. Authority entries A300-A314 track actual protocol reuse, filesystem/publication policy, wire projection admission, URI helpers, feed identity/loss, telemetry and competing trust-store initialization.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='The transport wrappers mostly delegate to their real protocol owners. Gemtext/Gopher/Scroll AST re-exports are actual shared types, and Nematic consumes them. Errand adds normalized status/error/MIME and optional native serving projections. These are useful layers rather than independent network engines. However, a shared helper can still be wrong: the Nex full-string base scan and Entry absolute-URL punctuation check duplicate semantics already present in url::Url. A benign formatter reparse confirms syntax acceptance but not structural identity when raw delimiter-bearing strings can introduce additional links/rows.\n\n'
addition+='Trust initialization has competing owners. fetch::install_smolweb_tofu forwards a host-chosen store; document-lanes::remote first-use initialization installs a new InMemoryTofu. The actual registry Gemini-protocol0.1.7 setter replaces an RwLock-held Arc and explicitly says the latest call wins. This strengthens the prior candidate into a source-confirmed replacement path; it is not a TLS handshake, observed pin loss or MITM proof. Choosing session/host ownership and compatible initialization can exceed a one-hour audit diversion. Keep it documented.\n\n'
addition+='Titan also contradicts the read/write overview. Scheme routing includes Titan; actual registry titan::fetch calls upload_inner with zero bytes. Introductory Errand prose says write companions are not fetchable schemes. A zero-byte payload does not make a write-shaped request read-only. No Titan request was sent and no endpoint mutation was attempted. Resource/crawler capability admission needs a broader caller review rather than another transport shortcut.\n\n'
addition+='Other inherited prose has drifted: protocol dependencies are currently exact registry crates, not workspace members under protocols/; parse-feed is not an available feature and quick-xml is unconditional; the manifest license comment says MIT/Apache while the actual license and source headers say MPL-2.0. These are documentation inconsistencies, not an independent legal relicensing conclusion. Directory configured index names are host inputs; as-is Document bytes and structural Listing projections are different contracts. No hostile configuration or cross-protocol index defect was reproduced.\n\n'
addition+='Fresh Cargo serve-feature library build and combined Errand/Nematic composition build succeed. Original Errand library suite: 64 passed, 1 ignored live Nex smoke test. Ten generated native observations pass. They include an actual Windows junction and owned files under a marker-identified stable Cargo target directory, actual format/reparse and feed projections, and one owned loopback listener left unaccepted to observe timeout. All fixture files, the non-recursively removed junction, executable and PDB are absent after cleanup. No remote network, TLS handshake, browser/script, GPU, user persistence or public serving ran in the generated probes. The original suite includes a separate real loopback Gopher serve/fetch control. Stable shared target reused; no isolated target, Cargo home or worktree. Evidence, exact hashed artifacts and source hashes are in the [Errand review receipt](../../../support/receipts/authority-success-audit/errand_review_receipt.json).\n'
save('review_notes.json',notes);save('authority_map.json',authority)
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'errand_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['errand_probe.rs','run_errand_probe.py','errand_probe_receipt.json','errand_probe_build.log','errand_probe_output.log',
 'errand_library_build.jsonl','errand_library_build_stderr.log','errand_composition_build.jsonl','errand_composition_build_stderr.log','errand_suite.log','errand_diff_check.txt']
registry=Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gemini-protocol-0.1.7/src')
traces=[registry/'tofu.rs',registry/'titan.rs',ROOT/'crates/system/fetch/src/lib.rs',ROOT/'Cargo.toml']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete Errand source reading; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'serve_library_tests_passed':64,'serve_library_tests_ignored':1,'serve_test_exit':0,
 'diagnostic_tests':10,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'excerpt_only_traces':{str(p):sha(p) for p in traces},
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production/original test changes.',
  'All22 Errand files read; dependency/caller excerpts are not whole-file denominator credit.',
  'Serve library64 passed/1 ignored and10 current Cargo-built native observations pass.',
  'Generated probes use owned Windows junction/files and one loopback timeout listener; fixtures and binary/PDB removed.',
  'No remote network, TLS handshake, browser/script, GPU, user persistence or public server in generated probes.',
  'TOFU replacement and zero-byte Titan write-shaped routing are source-confirmed, not executed endpoint/pin-loss/MITM proofs.',
  'Hidden-file policy, handle-safe filesystem containment, namespaces, root admission and read/write capability repairs remain documented.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
save('errand_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
