"""Append complete Inker source review with exact preservation and execution scope."""
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
old=load('document_canvas_review_receipt.json');probe=load('inker_probe_receipt.json')
assert probe['tests']==9 and probe['test_exit']==0 and probe['generated_output_removed']
raw=(HERE/'inker_default_suite.log').read_bytes()
suite=raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig')
assert '123 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/inker/inker/'))
assert len(complete)==23 and sum(inventory[p]['chars'] for p in complete)==335120
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1025 and stats['read_chars']==14350820
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(n,title,source,evidence,scope,size,repair):
    return {'id':f'Q{n:03}','title':title,'source':source,
        'status':'reproduced through current Cargo-built default Inker public APIs; documented, not repaired',
        'evidence':evidence,'scope':scope,'verification':'inker_review_receipt.json','diagnostic':'inker_probe_receipt.json',
        'repair_triage':{'disposition':'documented; active goal forbids production edits','size':size,'reason':repair}}
findings=[
finding(145,'Navigation currentness accepts unrelated same-length replacement blocks',
 'crates/inker/inker/src/document/navigation.rs:21-75 DocumentNavigation and InPageTarget contracts',
 'A real two-block heading/body table with an anchor and fold reports is_current=true for unrelated Rule/body blocks. resolve still returns the old anchor. A one-block replacement is rejected. is_current compares only block_count, despite claiming indices still describe the supplied blocks and copied targets stay inert.',
 'Native public navigation table and replacement/control. No hosted copied-target action, fold UI or stale session bypass executed.',
 'source identity/revision and transform contract migration may exceed one-hour diversion',
 'Name the guard as a length check or bind navigation/targets to an explicit document identity/revision. A complete fix must cover same-count replacements, copies into another document and consumers, while preserving explicit clear/remap transforms.'),
finding(146,'Markdown/Knot export conflates literal syntax with authored structure and raw frontmatter values with fields',
 'crates/inker/inker/src/document/render.rs:to_knot, write_knot_frontmatter, write_inline_markdown',
 'Literal Text("*x*") and Emphasis(Text("x")) are distinct source blocks but export exactly the same Markdown and Knot strings. HTML distinguishes them. Title "Alpha\\nsource: Beta" without canonical URI exports exactly the same Knot as title "Alpha" with canonical URI "Beta". Two distinct inputs with identical bytes cannot both round-trip as the claimed equivalent document.',
 'Native serializer non-injectivity proof with HTML control. No Nematic parser reparse, trust escalation, executable HTML/JS, or saved user note executed.',
 'literal escaping is contained; complete structural/frontmatter round-trip contract can exceed one-hour diversion',
 'Escape syntax according to each output context, choose safe code fences, and encode multiline frontmatter values unambiguously. Qualify the round-trip claim and test supported field equivalence through the actual Knot parser. Do not infer authentication from a serialized trust label.'),
finding(147,'Knot semantic round-trip silently drops stable feed-entry identity',
 'crates/inker/inker/src/document/render.rs:Block::write_knot FeedEntry branch; document.rs:FeedEntry',
 'Changing only FeedEntry.guid from first to second produces exactly equal Knot output; a changed block-kind control differs. The serializer also omits published, updated, enclosures and content_address by source inspection. Public to_knot documentation promises equivalent document-level metadata and every structural/semantic block.',
 'Native output collision for guid; other omitted fields source-qualified. No feed database identity collision or parser reparse executed.',
 'semantic fence schema and parser/consumer evolution may exceed one-hour diversion',
 'Define which semantic fields Knot preserves and extend producer/parser together, or explicitly describe a lossy projection. Keep stable feed identity and offline content address distinct from display date/article navigation. Presented semantic blocks also take ordinary Markdown rather than semantic-fence serialization and need a control in that repair.'),
finding(148,'Independent authority splitters both misparse IPv6 hosts',
 'crates/inker/inker/src/document/render/export.rs:push_link; routing.rs:host_from_address',
 'gopher://[::1]:7070/1/notes exports host "[" and port70; a DNS host with7070 is correct. host_from_address independently returns "[" for https://[::1]:7070/, and a configured [::1] host override is missed while a DNS override works. Both split authority on the first colon.',
 'Native string export and default policy with host overrides. No Gopher server/client, DNS, socket or live navigation executed.',
 'contained bracket-aware authority handling or shared URL parsing',
 'Use an existing typed URL/authority owner or a shared validated helper rather than fixing two ad hoc splitters independently. Specify bracket conventions at override/export boundaries and retain userinfo/port controls.'),
finding(149,'Availability-filtered routing still returns an unavailable fallback engine',
 'crates/inker/inker/src/routing.rs:route_filtered fallback selection',
 'A filter returning false for every ID still produces external.protocol; an empty EngineRegistry does not contain that decision. An available Genet control routes correctly. The fallback bypasses is_available, contrary to the API explanation that filtering avoids decisions pointing to unregistered engines. Existing test also expects an unavailable fallback without asserting availability.',
 'Native policy and registry membership. No actual external protocol launch, application dispatch or host registration graph executed.',
 'explicit no-route result and host adoption may exceed one-hour diversion',
 'Validate fallback availability and expose explicit absence/refusal, or document a separate host-handled sentinel contract. Do not silently select another unavailable engine. EngineRegistry already refuses missing IDs, so this is a route-contract gap rather than proof of successful dispatch.'),
finding(150,'The claimed type-enforced one-hop flip invariant is an unenforced implementor convention',
 'crates/inker/inker/src/flip/api.rs:FlipBack; flip/mod.rs; flip/orchestrator.rs module invariants',
 'An ordinary downstream Secondary type implements FlipBack, FlipDonor and FlipReceiver together, compiles, then forwards its state to B and B to C through flip_forward. Public traits contain no mutual exclusion, role token or constrained primary identity, although docs claim there is no type path for secondary chaining.',
 'Native generated implementor and two forward calls. Built-in ScryForward itself does not implement FlipDonor. No real black-box state extraction or lossy browser chain executed.',
 'documentation correction is contained; enforced role/provenance API may exceed one-hour diversion',
 'State the current convention accurately. If enforcement is required, make primary/source custody an explicit API input or constrained role and review downstream implementors. Merely naming separate open traits does not enforce negative trait membership.'),
finding(151,'Scry loses host-observable cookie and restore failures while finishing the flip',
 'crates/inker/inker/src/flip/scry.rs:begin, on_nav, tracing_warn',
 'A mock set_cookie refusal still yields begin Ok and navigates. A run_script refusal then yields terminal is_done. Debug warnings appear in stderr, but neither error reaches a result/event/diagnostic accessor. In non-debug source configuration tracing_warn discards its argument. Docs say cookie failures are logged by/decided by the host, yet this seam never reports them to it.',
 'Native mock surface with default/debug Cargo library. Release suppression is source-qualified; no release build, real WebView, authentication flow or user cookie store executed. is_done explicitly includes failure and is not itself misdocumented as success.',
 'typed degraded outcome and asynchronous host adoption may exceed one-hour diversion',
 'Preserve degrade-without-blocking while returning/retaining structured per-layer failures. Keep admission, load completion and restoration outcome separate. The host can then decide how to surface missing state without scraping debug stderr.'),
finding(152,'An XML declaration prevents feed sniffing and defeats the default feed route',
 'crates/inker/inker/src/sniff.rs:sniff_structured and atom_feed_is_detected_inside_xml; routing.rs default MIME rules',
 'The same Atom root without an XML declaration sniffs application/atom+xml and routes to the feed engine; adding the ordinary XML declaration sniffs application/xml and routes to Genet for HTTPS. The default policy has no application/xml feed rule. The test permits either result and claims generic XML can still dispatch without checking the actual combined policy.',
 'Native sniff plus default policy composition on two byte fixtures. No fetch host second-pass logic, parser, RSS endpoint or actual rendering executed.',
 'contained sniff precedence with generic XML controls',
 'Recognize a feed root before returning the generic XML label within the same bounded window. Test both generic XML and feed declarations through sniff plus routing; do not route all application/xml to feeds.'),
finding(153,'SVG sniffing escapes the advertised small prefix window',
 'crates/inker/inker/src/sniff.rs:sniff_image, starts_with_after_ws, head_window',
 'Two inputs with exactly equal first1024 bytes and65536 leading spaces produce different MIME values when the later marker is SVG versus HTML. SVG is detected beyond the advertised window because sniff_image trims the entire input; HTML remains limited to head_window. Existing bound test exercises HTML only.',
 'Native65536-byte whitespace fixtures and public sniff result; no denial-of-service, huge allocation or benchmark claim.',
 'contained prefix-bound consistency',
 'Apply the documented window before whitespace-dependent image sniffing, keeping fixed magic-byte checks and BOM policy explicit. Test every signature family at/after the bound, not only the structured-text branch.'),
]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'All23 Inker files read;123 default library tests and9 native observations',
        'verification':'inker_review_receipt.json'}
entries=[
entry(254,'Portable source facts and retained layout','Inker owns portable semantic source; layout adds geometry.',
 ['EngineDocument/Block/InlineSpan carry facts, provenance, trust and navigation', 'Document-canvas consumes these facts; presentation wrappers preserve source', 'BlockProvenanceMap is a sparse top-level sidecar with document fallback'],
 'A shared source model exists. Doc prose still credits later layout to Platen; live measured packet owner was reviewed in document-canvas. Sidecars are not independent durable stores but need transform/remap discipline. No current provenance misattribution was executed.'),
entry(255,'Navigation lifetime authority','is_current means indices describe the current document and copied targets remain inert.',
 ['DocumentNavigation stores block_count only', 'FoldState stores source-spelling/occurrence keys', 'InPageTarget has block index but no owning document identity'],
 'Q145 separates count consistency from document/revision identity. Key-based fold rebase and count-only admission solve different problems. Host session revision defenses remain to read; do not infer a current stale-action bypass.'),
entry(256,'Structural and semantic export fidelity','Knot promises equivalent parse/export/reparse for all blocks and metadata.',
 ['Raw Markdown Text and fixed fences', 'Raw line-oriented frontmatter values', 'Semantic fence writer omits newer FeedEntry fields', 'Presented blocks call ordinary Markdown serialization'],
 'Q146/Q147 disprove complete round-trip by exact output collisions. HTML escaping is a useful independently correct projection, not an interchangeable serializer. Semantic/source preservation needs parser-paired field coverage and explicit loss declarations.'),
entry(257,'URI authority parsing reuse','Host overrides and protocol export recover the destination host and port.',
 ['routing::host_from_address strips userinfo then splits colon', 'Gopher export independently splits colon and defaults invalid port70', 'address_scheme separately validates a scheme token'],
 'Q148 demonstrates duplicated parsing defects at two owners. Shared URL vocabulary already exists elsewhere in the workspace; locate its suitable admission boundary before introducing another authority parser. Scheme-token validation does not repair authority handling.'),
entry(258,'Engine selection and dispatch authority','Availability-filtered routing avoids unregistered engine decisions.',
 ['Policy prioritizes explicit pin, MIME, host override, scheme, fallback', 'Document and surface registries each replace duplicate IDs in their own HashMap', 'Each dispatch refuses its own missing ID', 'Policy fallback bypasses filter'],
 'Q149 is an admission mismatch, not a registry success lie. Separate stateless-document and long-lived-surface lifetimes justify separate registries, but no cross-registry uniqueness/conflict check lives in Inker. Host must choose dispatch ownership; inspect the actual consumer rather than declare two registries inherently redundant.'),
entry(259,'Default routing policy versus user intent','Pins are most authoritative; MIME outranks per-host preferences.',
 ['Pinned engine branch uses CompositedTexture', 'Host override branch also uses CompositedTexture', 'Scheme/MIME rules carry explicit modes'],
 'The precedence is explicit and tested, not a hidden second policy. Pin/host mode defaults are disclosed but can diverge from a headless or native-overlay engine own contract; no concrete consumer mismatch executed. A registration capability-derived mode may need host adoption rather than another rule table.'),
entry(260,'Flip role authority','The one-hop/no-chain rule is expressed in the type system.',
 ['Open FlipDonor, FlipBack and FlipReceiver traits', 'Generic forward/back free functions', 'ScryForward implements receiver only', 'GenetDonor is feature-gated and implements donor'],
 'Q150 shows the public API enforces no mutually exclusive role. Built-in adapter roles follow convention. Documentation should distinguish that convention from compile-time proof; preserve source provenance if introducing real role custody.'),
entry(261,'Flip success and degradation','Degrade never block; host decides what missing layers mean.',
 ['LayerSet intersection masks forward state', 'flip_back returns false if primary does not accept NAV', 'Scry cookie/script errors go only to debug stderr', 'is_done explicitly covers success/failure/no target'],
 'Q151 identifies missing structured degradation telemetry. Module claim that flips always present already has an explicit back-without-NAV exception. Neither forward void nor back bool proves a load or restored session. Do not treat terminal state as misnamed success where its docs disclose failure.'),
entry(262,'Portable cookie representation','Surface and flip cookie types mirror fields without metadata loss.',
 ['flip::api::Cookie and SameSite', 'surface_engine::Cookie and SameSite duplicate same field shapes', 'Live CookieCapabilities describe supported attributes independently'],
 'The module says these are deliberately not unified, but the payloads are currently duplicate DTOs requiring adapter maintenance. No host-only/partition-key preservation or cookie admission proof follows from this field list. Record the duplication without asserting a security bug from absent fields alone.'),
entry(263,'Content sniff authority and bounded work','One cheap bounded guess feeds MIME-driven engine routing.',
 ['XML declaration wins before feed-root detection', 'Tests accept generic XML as feed detection success', 'SVG whitespace trimming scans full buffer', 'Other structured/text checks share1024-byte head'],
 'Q152 proves the test assumption fails in the public default policy composition. Q153 proves the bound differs by branch. Sniffing is explicitly heuristic; neither finding establishes a fetched engine result or performance attack.'),
entry(264,'Evaluation and transclusion ownership','Hosts opt in and provide execution/fetch; Inker transforms blocks.',
 ['Default-deny language and scheme policies', 'Top-level-only processing, no eval-output rescan', 'Transclusion URL visited set and depth passes', 'Successful transforms clear navigation'],
 'Real shared pure transforms, with host-owned I/O and execution budgets. BlockEvaluator accepts max_ops while evaluate_blocks uses an injected closure; host supplies backend policy. Depth0 normalizes to1 when enabled and merits explicit semantics. No transport bypass or execution timeout proof was produced.'),
entry(265,'Inline traversal reuse and projection scope','Links and statements are extracted from shared facts.',
 ['document outgoing-links traversal includes typed semantic navigation fields', 'statements traverses predicate-bearing inline links only', 'Markdown/Gemini/Gopher/HTML each have format-specific recursive walks'],
 'Predicate collection correctly leaves relation resolution/assertion to linked-data. Some duplication is format-specific lowering rather than competing durable authority, but recursive variant updates need shared schema coverage. Images are subresources and Submit/InPage are not ordinary fetch links; excluding their target from outgoing URLs is deliberate.'),
entry(266,'Hosted document capabilities sole authority','WebSurfaceCapabilities.document is the sole find/zoom/capture/navigation status.',
 ['DocumentCapabilities type re-exported from external document-session-api', 'WebSurface::document_capabilities defaults to capabilities().document but can be overridden', 'SurfaceEngine type-level a11y declaration differs from instance WebSurface status'],
 'Default delegation is actual reuse. Open trait overrides and separate accessibility levels leave consistency as implementor contract, not enforced equality. No contradictory live producer declaration was executed. The external contract lives in Genet, not another Mere implementation.'),
entry(267,'Native frame custody and import authority','Opaque frame transfer preserves owned native resources without binding Inker to wgpu.',
 ['Explicit Borrowed/Transferred raw D3D12 ownership', 'OwnedSurfaceFrame boxed payload with consuming downcast and retry custody', 'resource_epoch/format/sync fields name importer obligations'],
 'Original downcast/mismatch controls pass. Mismatch keeps payload ownership but loses payload_kind to unknown wrapper. Native handles still rely on documented host duties; no GPU import, fence or cross-platform leak claim is qualified by the CPU gate.'),
entry(268,'Surface acceptance and completion','Result-bearing commands settle once on one ordered event stream.',
 ['Caller-minted WebRequestId and capture request IDs', 'Unsupported defaults return explicit errors', 'No filtered convenience pollers in the surface contract', 'Tests use a deterministic event-queue mock'],
 'Shared event design explicitly separates synchronous refusal from accepted completion. Tests prove the fixture and contract vocabulary, not every backend callback. Permission/auth request origins and surface-scoped IDs are explicit; HttpCredentials Debug redacts fields and no serde derive exposes them.'),
entry(269,'Fresh execution versus optional engine claims','The current default Inker library is the tested unit.',
 ['123 original library tests', 'Nine generated public observations linked to exact Cargo rlib', 'All optional GenetDonor source/tests read', 'document-session-api is a Genet git dependency, re-exported rather than locally copied'],
 'Default Cargo build/suite and native observations pass. Optional genet-donor was not built/run; no browser, WebView, engine render session, GPU, native a11y or network execution. Dependency source is not Mere inventory coverage. Source evidence is not upgraded to an integration receipt.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Enabled transclusion depth zero normalizes to one pass',
  'source':'crates/inker/inker/src/document/transclude.rs:resolve_transclusions',
  'note':'max_depth.max(1) permits one authored include pass when enabled with max_depth0. Default deny remains disabled. Clarify zero semantics before choosing a policy; no fetch probe or bypass claim made. Duplicate include URLs are deliberately never revisited, including after a failure.'},
 {'title':'Surface routing mode and capability declarations need actual host-consumer review',
  'source':'crates/inker/inker/src/routing.rs; surface_engine.rs',
  'note':'Pins/host overrides hardcode CompositedTexture rather than engine rule mode. document_capabilities can override capabilities().document; type a11y and instance status can differ intentionally. No actual conflicting host path qualified. Full host contract migration can exceed one-hour diversion.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/system/document-lanes/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete Inker23/23 source review, including optional Genet donor source'
notes['next_area']='crates/system/document-lanes'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
 'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':['inker_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
 'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'Current default Cargo build and123 original library tests pass;9 native public observations pass.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'23 complete files335120 chars added. Optional genet-donor source/tests read, not executed. No external Genet dependency coverage credit.'}
upsert(notes['source_qualifications'],[{'area':'Complete Inker source review',
 'note':'All23 inventoried source/manifest files335120 chars read. Default Cargo library build and123 library tests pass;9 generated native public observations pass. Serializer collisions prove non-injectivity without parser reparse. Scry uses mock failures with debug library; release warning suppression source-qualified. No optional genet-donor, network, live engine, hosted browser/WebView, GPU or OS a11y execution. External Genet document-session-api is re-exported and outside Mere source denominator. Probe executable/PDB removed, stable shared target reused, no isolation.'}],'area')
save('review_notes.json',notes);save('authority_map.json',authority)
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only Inker complete source review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\n'
addition+='All23 inventoried Inker files are read,335,120 more characters. Repository coverage is1025/3352 files and14,350,820/36,854,820 characters. Full-source goal remains active. Production/concurrent source remains unchanged. Resume at crates/system/document-lanes/Cargo.toml. The session contract itself is external Genet document-session-api, re-exported rather than copied into Mere; its dependency source earns no local coverage credit. Authority entries A254-A269 accompany the individual bugs.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='The portable EngineDocument facts, sparse provenance sidecar, source presentation wrappers and host-supplied evaluation/transclusion seams are implemented shared boundaries. Default policies deny execution/fetch expansion. Successful transforms clear navigation; top-level-only evaluation does not rescan generated output. Broader host budgets and enabled max_depth0 semantics need owner review. Predicate extraction remains pure and leaves graph relation resolution/assertion to linked-data. Distinct format traversal code is sometimes appropriate lowering, but new variants can drift across those repeated walks.\n\n'
addition+='Document and surface registries intentionally separate request/response from long-lived frame lifetimes, but do not enforce cross-registry ID uniqueness. Registration replaces duplicate IDs locally. Actual host dispatch ownership still needs complete reading. Pin and host override contracts default to CompositedTexture; concrete headless/native-overlay consumer consistency is not established here. Cookie/SameSite DTOs are duplicated between flip and surface seams, explicitly described as deliberate even though their payload fields currently match. Adapter maintenance and complete cookie metadata require consumer review; absence of fields alone was not promoted to a security finding.\n\n'
addition+='The web-surface document capability default delegates to the shared external DocumentCapabilities, while an override can produce a separate answer. Type-level accessibility and instance status are different declarations; no contradictory backend was executed. Surface commands use one ordered event stream with request IDs, explicit unsupported defaults and clear acceptance/completion semantics. Mock tests prove the contract fixture, not all backend completions. Native frame types name format, resource epoch and custody; original owned downcast/mismatch tests pass. No GPU importer/fence receipt follows from that. HttpCredentials Debug redacts secrets, and no serde derive exports them.\n\n'
addition+='Fresh default Cargo library build and123 original library tests pass;9 generated native observations pass. The serializer proof is byte equality between distinct inputs, without parser reparse. Scry debug errors were observed; release suppression is source-qualified. Optional genet-donor source/tests were read but not built/executed. No actual browser/WebView, hosted session, network, GPU or native accessibility execution. Owned probe executable/PDB removed. No isolated target, Cargo home or worktree created. Exact hashes and execution scope appear in the [Inker review receipt](../../../support/receipts/authority-success-audit/inker_review_receipt.json).\n'
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'inker_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['inker_probe.rs','run_inker_probe.py','inker_probe_receipt.json','inker_probe_build.log','inker_probe_output.log',
 'inker_library_build.jsonl','inker_library_build_stderr.log','inker_default_suite.log','inker_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
 'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete Inker source reading; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,'default_library_tests':123,'default_test_exit':0,
 'diagnostic_tests':9,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production changes.',
  'Current native default library only; optional genet-donor source/tests read but unexecuted.',
  'No host/browser/WebView, GPU, network, parser reparse, OS accessibility or real engine session execution.',
  'Serialization collision proves non-injectivity; no trust bypass or database identity collision claimed.',
  'Release warning suppression source-qualified, not release-built; default debug stderr observed.',
  'External Genet document-session-api is actual shared dependency; not local source coverage.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('inker_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
