"""Record all document-lanes source, fresh feature gate and scoped consumer proofs."""
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
old=load('inker_review_receipt.json');probe=load('document_lanes_probe_receipt.json')
assert probe['tests']==7 and probe['test_exit']==0 and probe['generated_output_removed']
raw=(HERE/'document_lanes_suite.log').read_bytes()
suite=raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig')
assert '54 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/system/document-lanes/'))
assert len(complete)==12 and sum(inventory[p]['chars'] for p in complete)==243970
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1037 and stats['read_chars']==14594790
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'reproduced through current feature-enabled Cargo-built document-lanes public APIs; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'document_lanes_review_receipt.json','diagnostic':'document_lanes_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal forbids production edits','size':size,'reason':repair}}
findings=[
finding(154,'A registered Smolweb session engine ID does not select the parser that actually runs',
 'crates/system/document-lanes/src/session.rs:SmolwebSessionEngine::spawn; smolweb.rs:lower',
 'SmolwebSessionEngine registered as nematic.text accepts an HTTPS held-body request containing literal heading syntax. Its actual document source_kind is nematic.gemtext, with a Heading and derived title. spawn never uses self.engine_id in lowering; lower independently selects by scheme, then feed prefix, then Gemtext. An explicit text pin/registration can therefore describe a different parser from the one producing content.',
 'Native SessionEngine spawn with a NoFetch fixture, downcast, inspect and portable source facts. No host route UI, web fetch or network session executed.',
 'registration/route/MIME/session lowerer consolidation may exceed one-hour diversion',
 'Pass the selected format/engine or shared registry decision into lowering and retain the actual response MIME/final identity. Avoid another independent scheme/body selector. Keep deliberate block-versus-session engine kinds while sharing parser selection.'),
finding(155,'Nested document images never enter the host subresource request set',
 'crates/system/document-lanes/src/smolweb.rs:subresources, provide_subresource, replace_document image retention',
 'A top-level Image produces its URL in subresources. The identical Image inside a Quote or List produces no request. provide_subresource only accepts pending URLs from that top-level scan, and replace_document similarly retains cached images only if a top-level Image still names them. The portable model, reader lowering and document-canvas support nested blocks.',
 'Native public source/resource discovery with top-level, quote and list fixtures. No decoder/network failure or actual reader page image download executed.',
 'contained recursive image visitor shared by discovery/admission/cache retention',
 'Traverse Presented, Quote and List source blocks consistently and deduplicate URLs. Reuse that visit for pending requests, response admission and cache retention so different shallow scans do not become separate resource authorities. Preserve host fetch/decode limits.'),
finding(156,'Smolweb load drops redirect-final response identity before resolving relative content',
 'crates/system/document-lanes/src/smolweb.rs:load and load_with_inline_media; remote.rs:ResourceFetcher implementation',
 'A rich mock fetch response names final.test/actual/index.gmi but load_with_inline_media asks for alias.test/old/index.gmi through the byte-only fetch method. Its retained address and promoted relative image remain on alias.test/old/image.png. Parsing the same body at the final URL gives final.test/actual/image.png. RemoteFetcher already exposes ResourceResponse with final_url/content_type, then its legacy fetch strips them.',
 'Native mock rich ResourceFetcher, public load and inline-media promotion; direct final-URL parse control. No real redirect or live network executed. Exact external ResourceFetcher contract was traced in the current pinned Cargo checkout only.',
 'contained response-record plumbing; engine-selection integration broadens the repair',
 'Consume fetch_response and preserve requested address versus final source/base identity explicitly. Pass response MIME into the same selected-parser owner instead of dropping it. Handle requested fragments deliberately while retaining final provenance.'),
finding(157,'Ragged table accessibility attaches later text to preceding padded cell geometry',
 'crates/system/document-lanes/src/smolweb/a11y.rs:Builder::table; crates/inker/document-canvas/src/layout.rs:render_table',
 'For source rows [A0] then [B0,B1], document-canvas emits two cells for each row. The semantic builder advances one flat iterator only for authored cells, so B0 receives row0 column1 padding bounds and B1 receives row1 column0 bounds. Native projection shows A0 and B0 share y while B0 has the right-column x. Source text is therefore attached to the wrong presented place.',
 'Native retained frame/semantic projection, CPU layout only. No OS screen reader, platform adapter or screenshot executed. Layout padding is intentional and separately tested; the mismatch is the consumer iteration.',
 'contained shared grid traversal/cell identity',
 'Index rendered cells by the actual padded column count or carry explicit row/column/source cell identity. Preserve ragged/empty/header/alignment-only columns and interaction ownership. Avoid inferring layout row boundaries from authored row lengths.'),
finding(158,'A pointer-clickable image loses activation in the accessibility projection',
 'crates/system/document-lanes/src/smolweb/a11y.rs:Builder::block Image branch; smolweb.rs retained focus/interaction paths',
 'A retained Image has one ordinary link rectangle and its pointer center returns Navigate(image URL). The semantic projection emits only an Image with ScrollIntoView, no node advertising Click, and accessibility_click_target(image ID) returns None. The Image branch ignores the corresponding layout focus stop instead of exposing its activation.',
 'Native public retained session, pointer click, semantic nodes and click-target query. No decoder required; placeholder image retains the same navigation. No physical accessibility device or host input bridge executed.',
 'contained semantic image/link composition using retained interaction identity',
 'Represent the image and its navigation through the shared stop identity, keeping the alt label and image role without duplicating announcements. Cover promoted gemtext images and nested images while preserving ordinary pointer/keyboard semantics.'),
finding(159,'The synchronous remote HTTP bridge panics when called inside a Tokio task',
 'crates/system/document-lanes/src/remote.rs:HttpResourceHost::fetch_response and runtime',
 'Calling RemoteFetcher.fetch for HTTP inside a current-thread Tokio runtime produces Cannot start a runtime from within a runtime. catch_unwind confirms the panic occurs at the nested multithread runtime block_on before polling the HTTP future. Unsupported-scheme control returns None. The synchronous trait has no documented runtime-context refusal and this path panics instead of returning a failed fetch.',
 'Native controlled nested runtime and caught panic; loopback address future is never polled, so no HTTP request/response is claimed. No current product host crash executed.',
 'blocking/async fetch boundary ownership can exceed one-hour diversion',
 'Keep one async fetch owner and place blocking adaptation on an appropriate worker boundary, or expose explicit context refusal rather than panic. Do not add more private runtimes or call block_on recursively. Preserve shared cache, concurrency, timeout and cleanup semantics.'),
]
upsert(notes['findings'],findings,'id')
q144=next(f for f in notes['findings'] if f['id']=='Q144')
q144['consumer_followup']={'verification':'document_lanes_review_receipt.json','diagnostic':'document_lanes_probe_receipt.json',
 'evidence':'Native retained SmolwebDocument focuses Second with an earlier empty-label link. After link adornment None and a new frame, focused_interaction resolves Third. Nonempty first-label control retains Second. Current accessibility revision guards still reject old queued requests; the reproduced failure is live retained keyboard focus retargeting across presentation, not a stale-request bypass.'}
def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All12 document-lanes files read;54 feature-enabled library tests and7 native observations',
  'verification':'document_lanes_review_receipt.json'}
entries=[
entry(270,'Shared Reader source and independent appearances','One durable source packet can have independent visible appearances.',
 ['ReaderDocumentSource holds Arc<EngineDocument>, lineage and baseline theme', 'Each ReaderDocumentSession owns SmolwebDocument layout, scroll, viewport and virtual focus', 'new_appearance shares source by Arc and reconstructs local presentation'],
 'Actual source reuse rather than duplicate durable source authority. Original tests use pointer equality and independent geometry/scroll. Held HTML is required and static/scripted-empty/nonreadable failures remain explicit; reader does not refetch its primary source.'),
entry(271,'Engine routing and actual parser authority','A session registered under a format engine ID corresponds to that parser.',
 ['Inker route rules select explicit engine IDs', 'SmolwebSessionEngine retains ID only for registration', 'Smolweb lower has its own scheme/body selector and silent Gemtext error fallback'],
 'Q154 proves the public registration can lie about the parser. Duplicate selection tables defeat a claimed single controller. MIME, file extension, Knot, Markdown, Micron and JSON Feed handling need one shared decision path, not more special cases in each host.'),
entry(272,'Response identity and fetch ownership','Host retains final identity/type and engines consume response records.',
 ['RemoteFetcher.fetch_response preserves HTTP final_url and content_type', 'Legacy fetch intentionally discards metadata', 'Smolweb load chooses legacy fetch and parses at requested URL', 'Errand resource branch wraps bytes using the original URL'],
 'Q156 proves a supported richer response is discarded before base-dependent image resolution. Original HTTP cache/redirect/body-limit tests pass but do not validate the downstream consumer. Protocol metadata and transport policy are separate owners from parser/source identity.'),
entry(273,'Inline resource discovery and admission','The retained document exposes host-resolved image resources.',
 ['Top-level image discovery', 'Response admission reuses pending list', 'Replacement cache retention repeats a top-level image scan', 'Decode applies encoded/decoded/dimension limits'],
 'Q155 shows shared admission still inherits a shallow traversal defect. Gemtext promotion max_images only limits promoted links; already-lowered source images are a different admission route. Do not infer an unbounded decoder from this: dimensions/byte budgets are explicit and current decode tests pass.'),
entry(274,'Logical focus versus painted annotation identity','Presentation changes preserve retained focus and source semantics.',
 ['Focus::Link stores document-canvas SemanticInteractionId', 'Style replacement preserves focus then invalidates layout', 'Document-canvas allocation depends on visible annotation counts', 'Accessible revision hashes layout generation, scroll, size and focus'],
 'Q144 now has a native consumer reproduction: current keyboard focus retargets Second to Third. The revision guard protects old queued accessibility actions but cannot make the current retained ordinal mean the original source link. Keep this distinct from an unproven stale-action security failure.'),
entry(275,'Semantic and visual table ownership','Semantic cells recover source labels from measured visual cells.',
 ['Canvas pads rows to maximum column count including alignment-only columns', 'A11y table walks one flat cell iterator by authored source row lengths', 'Table/list/quote semantic source paths repeat canvas synthetic conventions'],
 'Q157 proves separate traversal rules drift. Q141 source-path migration remains relevant. Shared explicit grid/source identity would remove implicit padding and decimal-packed ABI assumptions rather than duplicate layout policy in semantics.'),
entry(276,'Image navigation and semantics','Every presented interaction can be focused and activated.',
 ['Canvas makes image/placeholder a navigable link', 'Retained keyboard focus includes its link stop', 'Semantic Image branch emits only ScrollIntoView'],
 'Q158 shows visual/keyboard/pointer information not carried into the semantic projection. Partial capability limitation text names coarse text geometry, not missing image activation. Fix stop composition rather than creating a platform-only activation route.'),
entry(277,'Presentation completion and action currentness','Semantics publish only after a completed frame.',
 ['presented flag is revoked by retained geometry changes', 'Reader revision and ID map are local and monotonic', 'Smolweb revision is a hash of layout generation/scroll/size/focus', 'Click targets recompute visible topmost points and return current revision'],
 'Original tests validate waiting for frame, reflow revocation, stale actions, current scroll and appearance focus. frame builds a Scene synchronously; it does not prove GPU presentation completed. Reader exposes only visible links with virtual Focus and explicitly names unavailable activation. Smolweb projects offscreen blocks intentionally so ScrollIntoView can reach them.'),
entry(278,'Remote fetch runtime and resource limits','One shared runtime/cache/permit pool bounds host remote work.',
 ['HTTP host shares cache and RAII permits across clones', 'HTTP timeout/body/redirect caps are explicit', 'Process runtime has two worker threads and synchronous block_on', 'Smolweb branch directly uses Errand and process-global TOFU install'],
 'Q159 demonstrates runtime-context composition failure. HTTP permit wait occurs before request timeout; Smolweb does not retain ResourceFetchPolicy and installs its own InMemoryTofu once, potentially replacing prior host policy. Those policy/timebase interactions remain source-qualified candidates, without a remote security/timeout failure claim.'),
entry(279,'Retained clip custody and presentation','Stored clip bytes remain durable authority; playback is a rebuildable cache.',
 ['Fleece portable_clip exports selected body under ClipLimits', 'ClipView projects text/voice controls and original anchor evidence', 'Playback path is content-hash-derived and checks existing bytes; new writes use tempfile sync/noclobber', 'ClipSessionEngine checks host-supplied document address and source_kind'],
 'Four current clip library tests pass, including selected-body privacy, seal refusal and cache corruption. This is presentation over a host-admitted record, not authentication from a self-declared source_kind. Playback controls only request caller-owned actions; no decoder/audio output or real cryptographic receipt is claimed.'),
entry(280,'Typography and presentation settings authority','Application typography projects from a consumer baseline.',
 ['Tabard Typography validation and role preferences', 'document_style_with_typography clones supplied baseline and overrides touched roles', 'Smolweb theme conversion supplies initial colors/font/padding', 'Generic fonts use document-canvas admission ledger'],
 'Actual shared preference validation and reset semantics; ordinary tests pass. Per-role explicit family and global set_font_family are different precedence layers that need accurate host use. Invalid/custom baseline heading sizes are not validated here; no malformed-layout failure promoted without a control.'),
entry(281,'Fresh proof and backend boundaries','Feature-enabled library tests establish current lane behavior.',
 ['54 library tests with smolweb/netfetch/retained-clips', 'Seven generated public CPU observations', 'One separate keyed GPU streaming integration test read only', 'HTTP tests use local mock servers; Smolweb negative test queries a .invalid host'],
 'Current Cargo feature build and library suite pass. Source model, scene construction and semantic projection are exercised; no separate GPU integration, browser, OS a11y, playback or live successful capsule executed. The probe caught a nested-runtime panic before polling HTTP. Direct Rust linker was repaired with Cargo build-script native paths; this was audit harness setup, not a production link defect.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Smolweb resource fetch ignores configured ResourceFetchPolicy and installs process-global TOFU itself',
  'source':'crates/system/document-lanes/src/remote.rs:RemoteFetcher::new, smolweb_get_bytes, install_tofu',
  'note':'Policy is retained only by HTTP host. Errand fetch gets no caller response/concurrency/timeout policy. First Smolweb request unconditionally calls global set_trust_store(InMemoryTofu), potentially replacing a host-chosen durable/custom store. No current pin-loss, MITM, response-limit or transport-timeout reproduction; inspect Errand/Gemini authority and actual host configuration before a coordinated repair. Can exceed one-hour diversion.'},
 {'title':'Remote HTTP permit wait is outside the per-request timeout',
  'source':'crates/system/document-lanes/src/remote.rs:HttpResourceHost::acquire and fetch_response',
  'note':'Condvar queue wait precedes tokio timeout, so the configured timeout bounds admitted request work rather than total caller latency. No contention/deadline probe run. Decide whether policy promises total latency before changing fairness/deadline semantics; document larger repair.'},
 {'title':'Smolweb parser errors become Gemtext without an explicit fallback diagnostic',
  'source':'crates/system/document-lanes/src/smolweb.rs:lower and looks_like_feed',
  'note':'Any XML declaration triggers FeedEngine in the fallback arm; render error is discarded and Gemtext result returned. Registered format ID and fetch MIME are already ignored by Q154/Q156. Invalid feed fallback diagnostics, JSON Feed, file-format selection and case-normalized schemes need shared parser authority review rather than another host table. Source-only, no malformed-feed probe.'},
 {'title':'In-page activation queue retains events across document replacement',
  'source':'crates/system/document-lanes/src/smolweb.rs:activate, replace_document, take_in_page_navigations',
  'note':'replace_document resets pending_reveal but does not clear already queued InPageNavigation. Events document that indices belonged to content current when queued, yet carry no address/revision. Actual host drain order may prevent stale address application; no misapplied history/action reproduced. Review caller ownership before choosing drop versus tagged event semantics.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/nematic/nematic/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete document-lanes12/12 source review, all current feature-gated source and GPU integration test source'
notes['next_area']='crates/nematic/nematic'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['inker_review_receipt.json','document_lanes_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'Inker default123 tests+9 observations; document-lanes feature54 tests+7 observations; current builds pass.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'This source pass35 complete files579090 chars across Inker and document-lanes. GPU integration source read but not run; external Genet dependency excerpt not local coverage.'}
upsert(notes['source_qualifications'],[{'area':'Complete document-lanes source review',
 'note':'All12 inventoried source/manifest files243970 chars read. Cargo build with smolweb/netfetch/retained-clips and54 library tests pass;7 public CPU observations pass. Local HTTP mock tests and .invalid negative Smolweb path ran in the original suite. No GPU streaming integration, native adapter/AT, browser, playback, live successful capsule or actual redirect executed. Nested Tokio panic was caught before HTTP future polling. Q144 live retained keyboard focus retarget is reproduced, separate from old-request guard. Genet-host-api response trait excerpt traced at current e37762d Cargo checkout, outside Mere denominator. Probe executable/PDB/owned failed-link objects absent; stable target reused, no isolation.'}],'area')
save('review_notes.json',notes);save('authority_map.json',authority)
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only document-lanes complete source review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll12 inventoried document-lanes files are read,243,970 more characters. Repository coverage is1037/3352 files and14,594,790/36,854,820 characters. Across this pass, Inker and document-lanes added35 complete files and579,090 characters. The full-source goal remains active. Production and concurrent source are unchanged. Resume at '+next_read['path']+'. Authority entries A270-A281 accompany the findings.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='Q144 consumer follow-up: '+q144['consumer_followup']['evidence']+'\n\n'
addition+='Reader shares an immutable Arc source and lineage while retaining local layout, scroll, viewport and virtual focus per appearance. Its no-held-bytes, unreadable and scripted-empty cases explicitly refuse. Reader semantics disclose visible-links-only and unavailable activation; revision checks reject stale queued requests. Smolweb retains scene geometry, typed interactions, fold keys and current pointer targets, with presented-boundary revocation on relayout. The local frame boundary means Scene construction, not a GPU/OS presentation receipt. Native tests validate these distinctions. Currentness protection does not repair Q144 source identity derived from presentation.\n\n'
addition+='The semantic projection repeats decimal-packed nested indices and table traversal conventions from the layout producer. Q157 is concrete drift at that hidden ABI. Image promotion and decode are explicitly host-configurable and bounded, but discovery/admission/cache retention share shallow top-level scans. Complete recursive source identity and shared parser/response selection are broader owner repairs; document them rather than extending this audit by over an hour.\n\n'
addition+='RemoteFetcher shares HTTP cache, redirect cap, concurrency permits and one runtime across clones; current local mock cache/revalidation/limit tests pass. Permit queue time is outside request timeout. Smolweb does not retain ResourceFetchPolicy and installs process-global InMemoryTofu on first use. A host-selected durable/custom trust store may be replaced; current caller/pin behavior needs Errand/Gemini owner review, not a speculative security claim. Parser errors silently fall back to Gemtext and in-page event queues survive source replacement; those remain source-qualified candidates pending actual caller/probe evidence.\n\n'
addition+='Retained clips use Fleece selected-body export/import and caller-owned custody limits. Playback cache identity is content-derived, existing bytes are checked, and new files use synced tempfile/noclobber publication. Current selected-body, sealed-refusal and corruption tests pass, without decoder/audio/cryptographic proof. Host-supplied clip presentation checks address and source_kind but is not independent source authentication. Typography reuses Tabard validation and clones the consumer baseline for reset/role overrides.\n\n'
addition+='Fresh feature-enabled Cargo build and54 original library tests pass with smolweb, netfetch and retained-clips. Seven generated CPU observations pass. The original suite includes local HTTP mock servers and a negative .invalid Smolweb request. A separate GPU streaming integration test was read but not run. No GPU rendering, browser/WebView, native accessibility adapter/physical AT, playback, successful live capsule or real redirect ran. Probe HTTP nesting panic was expected and caught before polling the HTTP future. Direct Rust linking initially lacked Cargo native-library search paths; the audit runner now imports them. This harness setup failure was corrected without product edits. Owned executable/PDB/failed-link objects are absent. Stable shared Cargo target retained; no isolated target/home/worktree. Exact scope and hashes are in the [document-lanes review receipt](../../../support/receipts/authority-success-audit/document_lanes_review_receipt.json).\n'
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'document_lanes_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['document_lanes_probe.rs','run_document_lanes_probe.py','document_lanes_probe_receipt.json',
 'document_lanes_probe_build.log','document_lanes_probe_output.log','document_lanes_library_build.jsonl',
 'document_lanes_library_build_stderr.log','document_lanes_suite.log','document_lanes_diff_check.txt']
external=Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/e37762d/components/genet-host-api/lib.rs')
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete document-lanes source reading; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'updated_findings':['Q144'],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,'library_features':['smolweb','netfetch','retained-clips'],
 'library_tests':54,'test_exit':0,'diagnostic_tests':7,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'external_contract_trace':{'path':str(external),'sha256':sha(external),'scope':'ResourceResponse/ResourceFetchPolicy/ResourceFetcher excerpt only; no local source coverage'},
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production changes.',
  'Current feature-enabled CPU library and scene/semantic data only; separate GPU streaming integration source read but not run.',
  'Original library suite uses local HTTP mock servers and a .invalid negative query, not a successful live capsule.',
  'No browser/WebView, GPU rendering, native adapter/AT, playback or actual redirect execution.',
  'Generated fetch response uses a mock; nested-runtime panic caught before HTTP future polling.',
  'Q144 current retained keyboard focus retarget reproduced; no stale accessibility request bypass claimed.',
  'External Genet fetch-contract excerpt is dependency tracing only.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('document_lanes_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
