"""One-shot full Fleece audit checkpoint after the authorized upstream merge."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

name = 'fleece_boundary_review_receipt.json'
assert not (HERE / name).exists()
old = load('pictograph_refresh_review_receipt.json')
before = load('oct17_upstream_sync_before.json')
after = load('oct17_upstream_sync_after.json')
probe = load('fleece_boundary_probe_receipt.json')
assert probe['tests'] == 3 and probe['test_exit'] == 0 and probe['generated_output_removed']
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['libraries'].values())
assert all(sha(ROOT / p) == h for p, h in before['preserved_dirty_hashes'].items())
assert not after['changed_dirty_files']
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip() == after['head']
assert load('pending_read.json') == []
tests = (HERE / 'fleece_full_feature_tests.log').read_text(encoding='utf-8-sig')
for count in (56, 1, 6):
    assert f'test result: ok. {count} passed; 0 failed' in tests
mesh_test = (HERE / 'mesh_availability_refresh_test.log').read_text(encoding='utf-8-sig')
assert 'test result: ok. 1 passed; 0 failed' in mesh_test
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
stats = json.loads(subprocess.check_output(['python', str(HERE/'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1211 and stats['total_files'] == 3722
assert stats['read_chars'] == 15608211 and stats['total_chars'] == 39342110
assert len(stats['changed_since_inventory']) == 14 and stats['pending_chunks'] == 0
areas = {}
for area, prefix in [('fleece','crates/fleece/'), ('mesh','crates/mesh/mesh/'), ('pictograph','crates/canvas/pictograph/'), ('pandect','crates/system/pandect/')]:
    ps = [p for p in inv if p.startswith(prefix)]
    done = [p for p in ps if cov.get(p,{}).get('sha256') == inv[p]['sha256'] and cov[p]['through'] == inv[p]['chars']]
    areas[area] = {'complete_files':len(done), 'total_files':len(ps), 'complete_chars':sum(inv[p]['chars'] for p in done), 'total_chars':sum(inv[p]['chars'] for p in ps)}
assert areas['fleece']['complete_files'] == areas['fleece']['total_files'] == 17
assert areas['fleece']['complete_chars'] == 250496
assert areas['mesh']['complete_files'] == areas['mesh']['total_files'] == 43
assert areas['pictograph']['complete_files'] == areas['pictograph']['total_files'] == 165
assert areas['pandect']['complete_files'] == areas['pandect']['total_files'] == 76
inherited = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
preserved = {p:h for p,h in inherited.items() if p not in before['incoming_paths']}
assert all(sha(ROOT / p) == h for p,h in preserved.items())

findings = [
    {'id':'Q211', 'title':'Sitemap scanning sends XML entity spelling as a page URL and omits CDATA locations',
     'source':'crates/fleece/src/crawl/sitemap.rs:parse_sitemap; crawl.rs:run_crawl sitemap seeding',
     'evidence':'A supplied XML sitemap loc ending search?a=1&amp;b=2 reaches the public run_crawl fetch callback literally with amp;b=2. The intended a=1&b=2 URL is never requested. Equivalent CDATA text produces no search request. Both runs finish FrontierExhausted; the entity case reports two successful page fetches through the synthetic adapter. Frontier URL validation cannot decode XML syntax.',
     'scope':'Native public acquisition driver with a supplied in-memory fetch callback, no real HTTP request or server behavior. The scanner explicitly disclaims a full XML parser, but ordinary XML text representation still changes or drops the declared acquisition target.',
     'repair_triage':{'disposition':'documented in read-only continuation', 'size':'bounded XML location decoding', 'reason':'Use a bounded XML text reader or explicit supported location decoder before URL admission. Preserve request/byte/fanout/scope budgets and refusal evidence; do not expand sitemap-index recursion as part of this fix.'}},
    {'id':'Q212', 'title':'Visual clipping reports success for an empty intersection and includes pixels outside a partial selection',
     'source':'crates/fleece/src/clip.rs:attach_cropped_visual/scaled_start/scaled_len',
     'evidence':'On a 100x100 PNG, an inside rectangle yields the correct 20x20 control. Rect(-50,10,100,20) attaches 100x20 although its visible intersection is 50x20. Rect(200,10,30,20) attaches a 1x20 border strip and returns true despite having no intersection. Clamping origin and length independently substitutes visible pixels for an off-image rectangle.',
     'scope':'Native public helper, synthetic image and caller-supplied ClipRect. Repository call search found the helper definition and its original unit test, not an active product call; no browser capture, user screenshot or end-to-end selection leak executed.',
     'repair_triage':{'disposition':'documented in read-only continuation', 'size':'bounded rectangle-intersection correction', 'reason':'Transform both rectangle edges, intersect with image bounds, and return false for an empty/non-finite result before encoding. Keep existing resize limits and test inside, partial, outside and scaled controls.'}},
    {'id':'Q213', 'title':'Distinct valid canonical annotation targets alias to one exported annotation identity',
     'source':'crates/fleece/src/eidetic_bridge.rs:WebAnnotationEnvelope::from_target',
     'evidence':'Two valid rendered DOM records for the same caller-owned acquired representation, selecting characters 0..5 in Alpha and Bravo, have different canonical-text resource IRIs but the same annotation.id. A third record with a different source scope also has that id. All pass validate_integrity and annotation_json_ld; changing capture hash changes the id as a control. Saving/loading the first two through Eidetic yields distinct content-addressed records, so the alias is specifically the exported annotation identity.',
     'scope':'Native caller-supplied DOM and in-memory storage. Actual JSON-LD envelopes share id while their targets differ; no downstream RDF merge, peer overwrite, durable-store corruption or product annotation adoption executed.',
     'repair_triage':{'disposition':'document larger compatibility decision', 'size':'potentially over one hour with exported-ID compatibility', 'reason':'Choose whether identity names an immutable annotation or a logical family and state update semantics. If immutable, bind canonical-text resource, source/capture evidence and selector identity. Preserve or migrate existing exported IDs explicitly; a string-format patch alone leaves historical aliases unresolved.'}},
]
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q210'
assert authority['entries'][-1]['id'] == 'A416'
for f in findings:
    f.update(status='public native counterexample reproduced; documented, not repaired', verification=name, diagnostic='fleece_boundary_probe_receipt.json')
notes['findings'].extend(findings)

rows = [
 ('Raw extraction ownership','Mere integrates the Genet Fleece extractor rather than providing another HTML engine.', ['crates/fleece/src/lib.rs reexports fleece_extract','optional DOM/URL/crawl/clip/structured-data/Eidetic features'], 'Two Cargo artifacts share lib name fleece. Probe selects the local package by target source path, preserving the external extractor distinction.'),
 ('Traversal versus acquisition','Supplied graph/content traversal and explicit network acquisition have separate bounds.', ['traversal.rs run_traversal','crawl/frontier.rs Frontier','crawl.rs run_crawl'], 'They are intentionally distinct algorithms: provider-owned observations and ranking versus HTTP robots, politeness and transport budgets. A single universal frontier is not claimed.'),
 ('Crawl work accounting','max_pages bounds frontier handouts, while max_requests and byte budgets include supporting and failed requests.', ['Frontier::next/fetched','CrawlSummary examined versus fetched','bounded_request'], 'Current tests exercise zero budgets, partial failure bytes, broken oversized adapters and in-flight cancellation/timeout incomplete evidence. This is stronger than a success-shaped page count alone.'),
 ('Robots ownership','Origin-specific robots rules are acquisition policy.', ['crawl/robots.rs bounded subset','origin_of includes effective port','polite_wait and supplied transport callback'], 'Runtime fails closed on unreachable robots and permits explicit absence. Fresh tests cover encoded paths and distinct ports; full RFC compliance is not claimed.'),
 ('Sitemap target identity','XML text needs interpretation before HTTP URL admission.', ['crawl/sitemap.rs literal loc scan','Frontier::enqueue_seeds'], 'Q211 proves incorrect URL spelling and dropped CDATA. Existing limits and URL parser do not supply XML text decoding.'),
 ('Crawl graph destination','The owning session supplies a graph ID and ignores superseded run events.', ['crawl/graph.rs CrawlSession','fold_event active run guard','fold_update contribution collection'], 'Gathering returns contributions. The local variable named applied is a returned list, not evidence of a durable graph commit; destination application remains host-owned.'),
 ('Clip engine routing','Host-supplied registry/policy handles supported document engines.', ['clip.rs engine_document','fragment_from_body HTML fallback','fragment_blocks'], 'No second parser registry is introduced. The helper constructs a meerkat workspace route and has no retained node/view pin input; context limitations remain explicit. Workspace ID affects route target naming, not policy selection in the inspected route seam.'),
 ('Visual selection bounds','A crop receipt must correspond to the selected image intersection.', ['attach_cropped_visual','scaled_start','scaled_len'], 'Q212 proves false crop success and extra pixels from independent clamping. PNG encoding success cannot establish selection correctness.'),
 ('Semantic clip adaptation','Clip text/link traversal adapts shared Inker blocks.', ['append_block_text delegates inline_text','collect_block_links/collect_span_links','unsupported block text markers'], 'Presented blocks, tables, menus and nested spans are handled. Unknown block text retains a named marker; submission targets and in-page links are not imported as navigation links.'),
 ('Codec implementation sharing','Fleece has a local handwritten base64 encoder while existing consumers use the base64 crate.', ['clip.rs encode_base64','ports/graphshell/src/retained_content.rs base64::Engine','crates/system/luggage/src/signing.rs base64::Engine'], 'Mechanical consolidation opportunity, no new encoding error or false single-authority claim reproduced. Do not conflate a codec helper with document capture policy.'),
 ('Portable clip custody','Portable clips use Eidetic typed storage, sealing and receiver-owned manifests.', ['portable_clip.rs deposit','retain_clip/import_clip_package','sealed_selected_transfer_preserves_existing_receiver_custody'], 'No private storage engine is manufactured. Unavailable existing objects and schema collisions refuse reuse; storage errors may leave immutable payloads, explicitly documented rather than claimed atomic.'),
 ('Portable disclosure closure','Caller-approved selected parts bound export traversal.', ['export_selected_clip approved_roots','check_record selected dependencies','NoFetcher'], 'Private backing is excluded; no source graph or URL acquisition occurs. Fresh tests cover cycles, missing/altered/extra payloads, budgets, source tags, UTF-8 anchors and receiver-only disk reopen.'),
 ('Annotation selector authority','Canonical text and selector generation belong to external Fleece.', ['CanonicalTextRecordV1','CanonicalTextSelectorProjection','local Web Annotation serialization structs'], 'Local mirrored position/quote/fragment structs serialize interchange. Construction and validation reuse canonical projection and resolution, rather than independently deriving offsets. Mirroring still creates schema-drift surface.'),
 ('Capture evidence versus authentication','Host-supplied capture facts are integrity-bound evidence.', ['CaptureIdentity/CaptureEvidenceV1','from_resource_response','has_authoritative_acquisition_context'], 'Hash consistency and timestamp/DOM-mode presence do not authenticate a caller. Legacy records remain explicitly non-authoritative for absent acquisition facts; raw bytes and canonical text retain distinct identities.'),
 ('Annotation interchange versus custody IDs','Content-addressed storage and exported annotation identity currently disagree about distinct records.', ['WebAnnotationEnvelope::from_target','save_fleece_annotation','annotation_json_ld'], 'Q213 aliases different valid targets while native Eidetic saves remain distinct. Compatibility and annotation-family semantics need a named decision before ID migration.'),
 ('Structured-data projection','The shared linked-data processor expands exact retained DOM text with caller-owned base/context cache.', ['project_json_ld_blocks_with_base_iri','project_json_ld_import_blocks_with_base_iri','json_ld_contributions/json_ld_imports'], 'Envelope and contribution adapters duplicate iteration/outcome mapping but reuse one processor. Detailed APIs retain errors; best-effort wrappers explicitly filter them. No hidden context fetch or second JSON-LD engine is claimed.'),
 ('Mesh availability versus authority','Resource availability is an owner-sampled scheduling hint.', ['MeshResource::available_for','next_action_with_availability','next_action legacy wrapper'], 'New snapshot suppresses claim/grant/execute and preserves reclaim/heartbeat priority. It does not replace execution/admission checks. Three changed Mesh files reread; targeted availability test rerun. Older seven Mesh runtime observations retain their historical receipts.'),
 ('Backdrop camera ownership','Painting, picking and owner-controlled editing share one presentation fit.', ['BackdropSurface::viewport_point','scene_point','occurrence_rect','pick and paint'], 'Whole changed file reread. New public interaction geometry reuses the same independent template Y fit; handles are opt-in and source mutation remains with the embedding owner. No new headed rendering acceptance executed.'),
]
entries = [dict(id=f'A{417+i}',domain=domain,claim=claim,implementations=impls,assessment=assessment,state='current whole-source review; runtime scope named in assessment',verification=name) for i,(domain,claim,impls,assessment) in enumerate(rows)]
authority['entries'].extend(entries)
authority['current_basis'] = after['head']
notes['basis_commit'] = after['head']
notes['basis_transitions'].append({'from':before['head'],'to':after['head'],'fetched_origin_main':before['upstream'],'receipt':name,'invalidated_read_paths':after['stale_coverage_paths'],'historical_findings_qualification':'18 files invalidated by authorized merge; four reread in full, fourteen pending. Earlier runtime receipts retain original basis.'})
next_read = {'path':'Cargo.toml','start':0,'reason':'Reread the changed root manifest, then remaining thirteen stale Djinn files; continue unread Mere families including newly landed inference and kernel patch source.'}
notes['current_area'] = 'Fleece17/17 current; Mesh43/43 and Pictograph165/165 current source after four rereads; 14 stale source files remain elsewhere'
notes['next_area'] = 'Clear remaining upstream-invalidated reads and continue the full Mere source inventory'
notes['checkpoint'] = {'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':[name],'next_read':next_read,'production_edits_this_turn':False,'authorized_upstream_merge':after['head'],'coverage_note':'15 newly completed Fleece files229287 chars, four reread files78578 chars. Upstream adds367 inventory files and invalidates18 reads;14 remain stale. Net current count1211/3722 does not represent lost historical work.','diagnostic_outcomes':'56 Fleece library +1 authenticated peer-transfer +6 portable-clip tests pass;3 native defect observations pass;1 current Mesh availability test passes.'}
notes['source_qualifications'].append({'area':'Full Fleece current source and Oct17 upstream merge','note':'All17 files250496 chars read. Fresh all-feature native build,63 original tests and3 audit observations pass. Authenticated local peer transfer and disk reopen actually ran in original tests; synthetic observations use no network or user files. Canonical annotation ID alias is distinct from storage identity.14 changed root/Djinn files remain stale,367 added inventory files remain subject to review. Shared local WIP hashes preserved; audit production/original test edits absent.'})

doc = old['document']['path']
dest = ROOT/doc
original = dest.read_bytes()
assert sha(dest) == old['document']['sha256']
heading = '### Full Fleece review and current upstream transition, 2026-10-17'
assert heading.encode() not in original
addition = '\n\n'+heading+'\n\n'
addition += 'The previous goal turn made progress and the full-source goal remains active. Fleece is now read in full:17/17 files,250,496 characters. This checkpoint completes15 additional Fleece files (229,287 characters) and rereads four merge-changed files (78,578 characters): Mesh lib/resource/worker and Pictograph BackdropSurface. Reading coverage is hash-bound; searches, compiler reach and historical tests do not substitute for rereading.\n\n'
addition += 'Fresh origin/main was36 incoming commits beyond the previous merged basis. Authorized merge70a15376 incorporates origin/main8498608c. All38 local modified/untracked file hashes survived unchanged. The inventory grew from3,355 to3,722 source files, including the resident inference lane and cubek-matmul patch. Eighteen read hashes became stale; four are now reread, leaving14 root/Djinn files pending. Current coverage is1,211/3,722 files and15,608,211/39,342,110 characters. The lower current percentage reflects newly added source and honest invalidation. Earlier receipts remain historical evidence with their original hashes.\n\n'
for f in findings:
    addition += f['id']+': '+f['title']+'. '+f['evidence']+'\n\nScope: '+f['scope']+' Repair: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition += 'Authority entries A417-A434 distinguish reuse from duplication. Genet Fleece owns raw extraction and canonical-text selectors; Mere adds host-provided URL/capture facts and optional adapters. Generic supplied traversal and bounded HTTP acquisition deliberately use different frontier policies. The two page-link adapters still duplicate MIME/DOM/URL mechanics. The clip module handwrites base64 despite existing crate-based encoding in other consumers. Annotation serialization mirrors selector structs but reuses the canonical projection/resolution owner. Structured-data contribution/import-envelope adapters duplicate iteration while sharing linked-data expansion; detailed outcomes retain errors and best-effort filtering is disclosed. These are concrete sharing seams, not invented claims that every similarly named implementation must merge.\n\n'
addition += 'Portable clip export walks only caller-approved selected-part closure, excludes private backing and uses NoFetcher. Retention/import reuse typed sealed Eidetic storage and preserve existing receiver custody. Inputs are checked before writes, while storage failure may leave immutable payloads; atomic graph admission is expressly outside the helper. Capture evidence integrity is not caller authentication. The acquired representation hash, canonical-text resource hash and content-addressed annotation manifest are separate identities; Q213 is the exported annotation ID collapsing valid distinctions between them.\n\n'
addition += 'The changed Mesh resource/worker files now expose availability sampling and next_action_with_availability. The snapshot suppresses claim/grant/execute while preserving higher-priority reclaim/heartbeat obligations. Availability is not execution authority. Existing historical Mesh defects and native receipts are not silently promoted to fresh whole-suite reruns. BackdropSurface now exposes forward/inverse interaction geometry and occurrence rectangles using the same fit as paint/pick, with opt-in handles and owner-controlled edits. Its whole file is reread; no new headed rendering gate ran.\n\n'
addition += 'Fresh all-feature Fleece native library build passes;56 library tests, one authenticated peer-transfer test and six portable-clip integration tests pass. The peer test actually connects authenticated local peers and reopens the receiver store; portable tests also reopen disk and exercise sealed-custody preservation with a wiring double, not a new cryptographic proof. Three synthetic public API observations pass, meaning Q211-Q213 were reproduced, not repaired. One current Mesh availability test passes. No browser/user screenshot or product graph write was executed by the audit probe. A preliminary runner ambiguity between the external and local fleece library artifacts was corrected by selecting the local target source path; product code was unchanged.\n\n'
addition += 'No production/original tests were edited by this review; upstream changes arrived through the authorized merge. No isolated target, Cargo home or worktree was created. The stable shared target is reused, probe executable/PDB removed, compact evidence remains under100MB and cited historical receipts are preserved. Resume at the changed root Cargo.toml, then the remaining thirteen Djinn invalidations, before continuing all unread Mere source. The [Fleece checkpoint receipt](../../../support/receipts/authority-success-audit/'+name+') binds source, build, tests, findings and the transition.\n'
for a,b in [('full:17','full: 17'),('completes15','completes 15'),('was36','was 36'),('merge70','merge 70'),('main849','main 849'),('All38','All 38'),('from3,355','from 3,355'),('leaving14','leaving 14'),('is1,211','is 1,211'),('and15,608','and 15,608'),('passes;56','passes; 56'),('under100MB','under 100 MB')]:
    addition = addition.replace(a,b)
dest.write_bytes(original + addition.encode('utf-8'))
save('review_notes.json',notes)
save('authority_map.json',authority)
diff = subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'fleece_boundary_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode == 0
evidence = ['oct17_upstream_sync_before.json','oct17_upstream_sync_after.json','fleece_full_feature_build.jsonl','fleece_full_feature_build_stderr.log','fleece_full_feature_tests.log','fleece_boundary_probe.rs','run_fleece_boundary_probe.py','fleece_boundary_probe_build.log','fleece_boundary_probe_output.log','fleece_boundary_probe_receipt.json','mesh_availability_refresh_test.log','fleece_boundary_diff_check.txt','record_fleece_boundary_review.py']
source = sorted([p for p in inv if p.startswith('crates/fleece/')] + ['crates/mesh/mesh/src/lib.rs','crates/mesh/mesh/src/resource.rs','crates/mesh/mesh/src/worker.rs','crates/canvas/pictograph/src/canvas/backdrop_surface.rs'])
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':after['head'],'fetched_origin_main':before['upstream'],'kind':'Full Fleece reading; ongoing full Mere source audit','production_edits_this_pass':False,'authorized_upstream_merge':True,'coverage':stats,'area_coverage':areas,'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'source_sha256':{p:sha(ROOT/p) for p in source},'preserved_production_and_test_hashes':preserved,'prior_receipt':{'path':'pictograph_refresh_review_receipt.json','sha256':sha(HERE/'pictograph_refresh_review_receipt.json')},'authorized_source_invalidations':after['stale_coverage_paths'],'remaining_stale_reads':stats['changed_since_inventory'],'next_read':next_read,'library_build_exit':0,'fleece_library_tests':56,'fleece_peer_tests':1,'fleece_portable_tests':6,'fleece_test_exit':0,'mesh_availability_tests':1,'mesh_test_exit':0,'native_observations':3,'native_probe_exit':0,'document':{'path':doc,'sha256':sha(dest),'preserved_prefix_bytes':len(original),'preserved_prefix_sha256':hashlib.sha256(original).hexdigest()},'evidence_sha256':{p:sha(HERE/p) for p in evidence},'retained_isolation':[],'generated_probe_removed':probe['generated_output_removed'],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'limits':'No current whole-workspace test or full-source completion. Fourteen changed files stale and substantial unread inventory. Three observations characterize defects; no repairs. No downstream RDF merge, browser capture, remote HTTP request or product mutation by audit fixture.'}
save(name,receipt)
print(json.dumps({'receipt':name,'findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'coverage':stats,'areas':areas,'evidence_bytes':budget},indent=2))
