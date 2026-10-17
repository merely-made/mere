"""One-shot current-source reread checkpoint; no replacement of cited evidence."""
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
assert not (HERE/'pictograph_refresh_review_receipt.json').exists()
old=load('mesh_execution_review_receipt.json');probe=load('pictograph_refresh_probe_receipt.json')
assert probe['tests']==5 and probe['test_exit']==0 and probe['generated_output_removed']
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
assert '2 passed; 0 failed' in (HERE/'pictograph_refresh_library_tests.log').read_text(encoding='utf-8-sig')
assert '16 passed; 0 failed' in (HERE/'pandect_facet_refresh_tests.log').read_text(encoding='utf-8-sig')
assert '1 passed; 0 failed' in (HERE/'fleece_page_links_refresh_tests.log').read_text(encoding='utf-8-sig')
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
inv,cov=load('source_inventory.json')['files'],load('read_coverage.json')
reread=[p for p in load('current_main_review_receipt.json')['source_invalidations'] if p!='Cargo.toml']
added=['crates/canvas/pictograph/src/canvas/link_suggestions.rs','crates/canvas/pictograph/src/canvas/tests/link_suggestions.rs','crates/fleece/src/page_links.rs','crates/fleece/src/crawl.rs']
new=sorted(reread+added)
assert len(reread)==12 and len(new)==16 and sum(inv[p]['chars'] for p in new)==353818
assert all(cov[p]['sha256']==inv[p]['sha256'] and cov[p]['through']==inv[p]['chars'] for p in new)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1210 and stats['read_chars']==16060205 and not stats['changed_since_inventory'] and not stats['pending_chunks']
areas={}
for area,prefix in [('pictograph','crates/canvas/pictograph/'),('pandect','crates/system/pandect/'),('fleece','crates/fleece/')]:
 ps=[p for p in inv if p.startswith(prefix)]
 done=[p for p in ps if cov.get(p,{}).get('sha256')==inv[p]['sha256'] and cov[p]['through']==inv[p]['chars']]
 areas[area]={'complete_files':len(done),'total_files':len(ps),'complete_chars':sum(inv[p]['chars'] for p in done),'total_chars':sum(inv[p]['chars'] for p in ps)}
assert areas['pictograph']['complete_files']==areas['pictograph']['total_files']==165
assert areas['pandect']['complete_files']==areas['pandect']['total_files']==76
assert areas['fleece']['complete_files']==2 and areas['fleece']['total_files']==17
notes,authority=load('review_notes.json'),load('authority_map.json')
findings=[
 {'id':'Q209','title':'Derived suggestion creation and attention admission use inconsistent candidate sets and budgets',
  'source':'crates/canvas/pictograph/src/canvas/link_suggestions.rs:refresh_link_suggestions; suggestions.rs:offer_suggestions/admit_suggestions',
  'status':'three public native Canvas counterexamples reproduced; documented, not repaired',
  'evidence':'Two existing offers from an excluded source subtract the entire two-link minting budget, although shared SuggestionSettings::admit admits the two allowed links. Removing those excluded offers restores the links. Conversely, replacing existing offers after two links are minted removes both links from admitted Suggestion attention but leaves both derived graph nodes resident. A60x60 plate mints one link under the pre-mint fill, then immediately gives it no Suggestion attention under the post-mint fill. Minting and attention admission both call budget but use different sets/counts and different plate snapshots.',
  'scope':'Current public Canvas API, synthetic graphs and actual shared SuggestionSettings. No rasterized screenshot, live browser or persisted owner write. The observed contract failure is a split admission decision; counts are not evidence of arbitrary unbounded allocation.',
  'verification':'pictograph_refresh_review_receipt.json','diagnostic':'pictograph_refresh_probe_receipt.json',
  'repair_triage':{'disposition':'documented during read-only audit','size':'bounded admission reconciliation; viewport-driven pacing may expand scope',
   'reason':'Compute one admitted set after source filtering and use it for both derived membership and attention. Reconcile on every relevant offer/settings/fill change, and define which content counts toward plate occupancy so creation does not invalidate its own admission. Avoid a second private pacing implementation.'}},
 {'id':'Q210','title':'A shared suggested URL retains a page that withdrew it as its origin',
  'source':'crates/canvas/pictograph/src/canvas/link_suggestions.rs:refresh_link_suggestions existing derived URL handling',
  'status':'public native Canvas edge and kept-link provenance mismatch reproduced; documented, not repaired',
  'evidence':'Pages A and B both offer the same URL, deduplicated into one derived member. When A replaces its offer with empty, B is the only remaining source. Refresh keeps the existing member by URL, skips its origin tuple and edge update, and leaves its hyperlink from A with none from B. Selecting that member emits KeptLink.origin=A even though only B still offers it.',
  'scope':'Actual graph projected relations and public take_kept_links after CPU Canvas frame. No browser commit or wrong durable relation was executed. Stable URL-derived identity is not itself the defect; origin/edge state is stale.',
  'verification':'pictograph_refresh_review_receipt.json','diagnostic':'pictograph_refresh_probe_receipt.json',
  'repair_triage':{'disposition':'documented during read-only audit','size':'bounded derived origin/edge reconciliation',
   'reason':'Refresh source provenance and the owned ephemeral relation when the selected offer origin changes, preserving the stable member id. Retain enough relation identity to remove only the derived assertion; do not blanket-delete independent user relations.'}},
]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,impl,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':impl,'assessment':assessment,
  'state':'All upstream-invalidated files reread; current targeted gates and five native observations', 'verification':'pictograph_refresh_review_receipt.json'}
entries=[
 entry(402,'Current reading versus historical coverage','Source read credit is bound to current bytes.',
  ['Twelve previously invalidated files reread in full','Four newly reviewed files include two new link modules and Fleece extraction/crawler','No pending chunks or stale coverage hashes'],
  'Source inventory remains3355. Current Pictograph165/165 and Pandect76/76 are fully read; Fleece2/17 remains partial. The whole Mere objective is still incomplete.'),
 entry(403,'One settings type versus two admission owners','Only links admitted by settings are minted as derived members.',
  ['refresh_link_suggestions subtracts raw other-offer count','admit_suggestions later applies allowed-source filtering','offer_suggestions does not reconcile derived membership','Plate fill changes after minting'],
  'Q209 reproduces starvation and unadmitted resident nodes. Reusing budget arithmetic does not make two decisions with different inputs one authority.'),
 entry(404,'Stable URL identity versus current source provenance','A derived link is linked from the page that offers it.',
  ['URL determines stable derived UUID','Offers select first eligible page per URL','Existing derived id skips origin and edge update','KeptLink uses stored origin'],
  'Q210 demonstrates stale source edge and emitted provenance after an offer changes. Stable identity and changing provenance require separate reconciliation.'),
 entry(405,'Keep intent versus graph ownership','Selecting a derived link hands an owning host an intent to persist it.',
  ['tend_link_suggestions queues KeptLink','set_graph preserves entire LinkSuggestions including kept queue','Graphshell drains into its own queue and create_address path'],
  'Fifth probe observes an old-origin intent after graph replacement. Whether it targets the wrong durable owner depends on host graph/session switching. Do not present that unexecuted write as a proven cross-session corruption bug.'),
 entry(406,'Ephemeral members versus durable graph truth','Derived links are session-only until host-owned intake succeeds.',
  ['Canvas mints into its disposable graph','cartography_geometry filters derived ids from every member-keyed field','Graphshell own intake persists kept URL then reinstalls returned graph'],
  'The persistence separation is real. A selected/queued member is not an acknowledged durable write. Existing keep/reinstall tests pass, but failure/retry ownership remains a candidate.'),
 entry(407,'Saved geometry versus facet storage','Live positions and cold arrangement state have different owners.',
  ['Canvas reads live Seiche positions','CartographyGeometry carries saved seed/regions/mosaic','Pandect defines arrangement.seed/regions/mosaic helpers','Per-node versus container scope is explicit'],
  'The new helpers exist now; the previous historical statement that these shapes are absent is stale. Mere caller search found only their definitions/reexports, so end-to-end host adoption is not established.'),
 entry(408,'Mirrored types versus independent authority','Storage remains graphics-independent while carrying the same arrangement shape.',
  ['SavedRegion and ArrangementRegion mirror contour/label/members','SavedMosaic and ArrangementMosaic mirror four fields','Pandect does not import Canvas/Livery to encode those values'],
  'This is duplicate schema shape with a defensible dependency boundary, not proof of two durable arrangement stores. Track drift/adoption; a future neutral schema seam must preserve graphics-free storage.'),
 entry(409,'Family rewrite scope versus container key','Regions and mosaic are written on a named container.',
  ['Public writers accept one container','Shared rewrite_family clears that facet across all store entries','Readers accept one container','Current tests use one container'],
  'A multi-container caller would lose earlier same-family values. Existing rewrite semantics explicitly clear stale family state and current callers are unadopted; document this scope question instead of unilaterally redefining the save contract.'),
 entry(410,'Current rendering source versus older diagnostics','One footprint is meant to feed node body and face presentation.',
  ['Body applies grammar_face_scale','Derived-face side uses node_size directly','Face placement remains separate from raised body','Shared node_color strings remain separate from canonical palette'],
  'Changed modules are reread and the old Q098/Q104/Q105 source seams remain visible. Their historical native probes were not rerun wholesale and retain original evidence basis; new probes do not count as their runtime requalification.'),
 entry(411,'Theme declarations versus text engine ownership','Hosts lend typography; BoardText owns shared label shaping.',
  ['set_font_roles stores root declarations','BoardScene and BackdropSurface reuse BoardText labels','Actual Livery TextSystem shapes fonts','BackdropSurface fit is reused by paint and picking'],
  'This is concrete shared text machinery. Nonuniform template-height presentation is applied consistently to picking/paint in source, with instrument aspect preserved. This pass does not rerun the full rendering/template test suites or claim headed font acceptance.'),
 entry(412,'Shared raw extraction versus repeated adapter policy','Link extraction and acquisition remain separate.',
  ['page_links and crawl each select HTML/XHTML and resolve href against page URL','Both invoke shared Genet Fleece extract_links','Page helper deduplicates and omits same-page links','Crawler delegates final admission to Frontier'],
  'Repeated syntax/resolution code is a consolidation opportunity, not a private second HTML engine. Their final filtering serves different callers. Sharing the page-links feature alone does not mean crawler calls the new helper.'),
 entry(413,'Crawler transport facts versus decoded content','Acquisition reports actual received-byte evidence.',
  ['Supplied single-attempt CrawlFetch contract','Remaining budget caps each response','Failure bytes counted','Dropped timed-out/cancelled future marks completeness false','Explicit cancellation checked after ready response before processing'],
  'These are concrete ownership checks in the now fully read crawl.rs. No live crawl or current crawl-feature test ran; frontier/robots/sitemap/test modules remain unread, so full Fleece safety is not inferred.'),
 entry(414,'API availability versus consumer persistence','An exported helper is not a completed host save path.',
  ['Pandect facade exports new arrangement helpers','Repository call search returns definitions/reexports only','Helpers have16 passing native unit tests','Graphshell uses separate scene save geometry path'],
  'Describe implemented storage vocabulary separately from product adoption. Unit roundtrips alone do not establish browser/native session reopening for the new fields.'),
 entry(415,'Targeted controls versus full-suite success','Current tests qualify named paths only.',
  ['Pictograph page-link tests2 pass and531 filtered','Pandect facet tests16 pass and310 filtered','Fleece page-links test1 pass','Five added native observations characterize two defects and one ownership question'],
  'No whole Pictograph/Pandect/Fleece suite or browser/GPU/network run is claimed. Probe mutation uses the real public graph delta owner; a preliminary fixture compile using private add_node was corrected without production changes.'),
 entry(416,'Read-only review versus repairs','Source review advances without absorbing parallel WIP.',
  ['Current main04474731 unchanged','Prior source hashes checked','Fresh build artifacts/hash closure recorded','Probe executable/PDB removed','Stable reusable target and bounded compact receipts'],
  'Source-reading coverage is not a bug-fix claim. Larger admission/ownership/persistence questions are documented for repair without turning the audit into an implementation lane.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Pending kept-link intent outlives a Canvas graph replacement without owning graph identity',
  'source':'crates/canvas/pictograph/src/canvas/lifecycle.rs:set_graph; link_suggestions.rs:KeptLink; ports/graphshell/src/web_tree/product.rs:keep_links',
  'note':'Native probe selects a derived link, replaces graph with a different graph lacking its origin, then receives the old KeptLink. Keeping offers across owner reinstalls is intentional; carrying an unbound pending request across a different durable owner needs a host rule. Graphshell queue creation uses current app and ignores origin during create_address. No wrong durable host write was executed, so this is an observed ownership question rather than proven cross-session corruption. Binding/cancel/retry policy may exceed one-hour diversion.'},
 {'title':'Kept-link failure acknowledgement is absent from the Canvas request vocabulary',
  'source':'crates/canvas/pictograph/src/canvas/link_suggestions.rs:tend_link_suggestions/take_kept_links; ports/graphshell/src/web_tree/product.rs:keep_links',
  'note':'Selection marks kept and removes offers before durable acknowledgement. Canvas queue drains once; host pops before checking app and reports create_address error without visible requeue in this helper. It also has save/selection guards and a separate intake pipeline. Inspect complete product lifecycle before claiming lost user data; no native/browser failure injection executed. Define retry/restoration rather than treating local kept=true as durable success.'},
 {'title':'Container-scoped arrangement writers currently clear a whole same-facet family',
  'source':'crates/system/pandect/src/arrangement_facets.rs:write_arrangement_regions/write_arrangement_mosaic/rewrite_family',
  'note':'Writing container B removes container A same-facet state from the supplied NodeFacetStore. The API reads per container but family rewrite contract is intentionally global. Current repository has definitions/reexports only for the new helpers, and tests use a single container. Establish allowed store/container scope before changing semantics; no multi-container native probe or host data-loss claim made.'},
 {'title':'HTML page-link resolution does not inspect an authored HTML base element',
  'source':'crates/fleece/src/page_links.rs:html_links; crawl.rs:enqueue_fetched_html_links',
  'note':'Both adapters resolve extracted hrefs against supplied page_url and do not read document base metadata here. That matches their stated provided-address contract but may differ from a browser target for pages using base href. Keep raw extraction versus URL policy ownership explicit. No dedicated base-element test or browser mismatch executed.'},
],'title')
next_path=next(p for p,v in sorted(inv.items()) if p.startswith('crates/fleece/') and (cov.get(p,{}).get('sha256')!=v['sha256'] or cov[p]['through']<v['chars']))
next_read={'path':next_path,'start':0,'reason':'Continue Fleece family after its new page-link seam and whole crawler adapter; remaining15 files unread'}
notes['current_area']='Upstream refresh complete: current Pictograph165/165 and Pandect76/76; Fleece2/17'
notes['next_area']='Complete remaining Fleece source, then continue remaining Mere families'
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['pictograph_refresh_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,
 'diagnostic_outcomes':'2 Pictograph tests,16 Pandect tests,1 Fleece test pass;5 native observations characterize2 new defects plus pending-intent scope question.',
 'coverage_note':'16 whole/current files353818 chars added. Every upstream invalidation cleared by full rereading. Pictograph/Pandect current source complete; Fleece2/17 and full Mere remain incomplete.'}
upsert(notes['source_qualifications'],[{'area':'Current Pictograph/Pandect refresh and partial Fleece adapter reading',
 'note':'All12 upstream-invalidated files reread, plus4 newly read files353818 chars total. Current Pictograph165/165 (2244039 chars), Pandect76/76 (1221768), Fleece2/17 (21209/250496). Fresh current canvas build and named targeted tests pass;5 native public API probes confirm split admission and stale origin, with pending keep intent graph scope left qualified. Old rendering findings retain original runtime receipts despite current source reinspection. No production/original test/source changes; shared WIP preserved. CPU Scene composition only in probes, no rasterizer/browser/host writes/fetch or user files. Stable target, removed executable/PDB, compact receipts under100MB.'}],'area')
doc=old['document']['path'];dest=ROOT/doc;before=dest.read_bytes();assert sha(dest)==old['document']['sha256']
heading='### Current Pictograph and Pandect refresh, partial Fleece review, 2026-10-17'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nThe previous goal turn was progress: complete Mesh reading and native evidence changed the next action. All twelve source files invalidated by the main merge have now been reread in full; four additional files cover new page-link suggestions/tests and Fleece page-link/crawler adapters. This pass adds353,818 read characters across16 files. Current coverage is1,210/3,355 files and16,060,205/36,897,906 characters, with no stale coverage hashes or pending chunks. Pictograph is165/165 files (2,244,039 characters), Pandect76/76 (1,221,768), Fleece2/17 (21,209/250,496). Mere objective remains active. Production and original tests/shared WIP are unchanged at04474731. Resume at '+next_path+'.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='Authority entries A402-A416 record a concrete split: one shared SuggestionSettings type still feeds two decisions with different candidates, counts and occupancy. Excluded other-source offers consume minting capacity; a later offer change can leave derived graph members without admitted Suggestion attention; creation itself can alter plate fill enough to revoke its own admission. The correction seam is one admitted set, not a product-private settings fork. The stale-origin case needs source tuple and owned ephemeral edge reconciliation while preserving stable URL identity.\n\n'
addition+='The fifth probe records an ownership question, not a third proven durable bug: a selected link request survives replacement with a different graph that lacks its origin. Keeping link offers across ordinary owner reinstalls is explicitly intended. KeptLink has member, URL and origin but no durable graph identity; the host drains requests into a queue and uses its current create_address intake. A wrong durable host write or lost retry was not executed. Preserve a named ownership/acknowledgement decision before changing graph-switch behaviour; that can exceed a one-hour diversion.\n\n'
addition+='Actual sharing remains visible. Canvas excludes derived IDs from the save geometry across all member-keyed fields; Graphshell intake owns persistence. BoardScene and BackdropSurface share BoardText and Livery rather than a private text engine; host font-role declarations reach roots. BackdropSurface uses the same presentation fit in painting and picking, including stretched template rows and ordinary instrument aspect ratio. This is source review, not a newly executed headed/template/font acceptance campaign. Older palette/body/face geometry findings remain on their original native receipts; the relevant changed source is now reread without pretending the old whole diagnostic suite was rerun.\n\n'
addition+='Pandect now implements arrangement.seed/regions/mosaic alongside the existing family and exports those helpers. Sixteen current helper tests pass. The earlier snapshot describing absent helper shapes is stale; current repository call search still finds definitions/reexports rather than host adoption. Mirrored SavedRegion/ArrangementRegion and SavedMosaic/ArrangementMosaic shapes preserve a graphics-free persistence dependency but create a schema-drift seam to track. Container-specific writer names call the global rewrite_family clear, so a multi-container store would need a scope rule; current tests use one container and no production adoption/data-loss claim is established.\n\n'
addition+='Fleece is partial. The entire crawl.rs and new page_links.rs are read; both choose HTML/XHTML and resolve hrefs locally while sharing the Genet Fleece raw extractor. The new feature dependency is shared, but the crawler does not call the new helper. Candidate filtering is intentionally different: page suggestions deduplicate/drop self links, acquisition delegates admission to Frontier. Duplicate MIME/parse/URL mechanics are a consolidation opportunity, not a second HTML engine. Base-element resolution remains a source-qualified question. The crawler explicitly counts raw success/failure bytes, marks cancelled/timed-out in-flight byte evidence incomplete, and checks cancellation after ready responses; its frontier/robots/sitemap/tests remain unread and no crawl ran.\n\n'
addition+='Fresh checks: Pictograph canvas library build succeeds; two existing page-link tests pass (531 others filtered); sixteen Pandect arrangement tests pass (310 filtered); one Fleece page-link test passes. Five new public Canvas observations pass, meaning the two defects and the qualified pending-intent observation were reproduced. The fixtures use the public graph delta owner; a preliminary audit-fixture compile using private add_node was corrected without product changes. Probe frames compose CPU Scenes, with no GPU rasterization, browser, durable host save, application network fetch or user-file operation. Only compact build/test/probe/source evidence is retained under the default100MB cap; previous cited receipts remain unchanged. Probe executable/PDB removed, stable shared target reused, no isolated home/target/worktree. The [refresh receipt](../../../support/receipts/authority-success-audit/pictograph_refresh_review_receipt.json) binds all inputs and scopes.\n'
for a,b in [('adds353','adds 353'),('across16','across 16'),('is1,210','is 1,210'),('and16,060','and 16,060'),('is165','is 165'),('Pandect76','Pandect 76'),('Fleece2','Fleece 2'),('at044','at 044'),('default100MB','default 100 MB')]:addition=addition.replace(a,b)
dest.write_bytes(before+addition.encode());save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'pictograph_refresh_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
files=['pictograph_refresh_build.jsonl','pictograph_refresh_build_stderr.log','pictograph_refresh_library_tests.log','pandect_facet_refresh_tests.log','fleece_page_links_refresh_tests.log','pictograph_refresh_probe.rs','run_pictograph_refresh_probe.py','pictograph_refresh_probe_build.log','pictograph_refresh_probe_output.log','pictograph_refresh_probe_receipt.json','pictograph_refresh_diff_check.txt']
trace=['crates/cambium/scenes/scenograph/src/suggestion.rs','ports/graphshell/src/web_tree/product.rs','ports/graphshell/src/web_tree/forme.rs']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Current source refresh complete; Fleece and full Mere source goal incomplete','production_edits_this_pass':False,
 'coverage':stats,'area_coverage':areas,'reread_files':reread,'added_complete_files':new,'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'source_sha256':{p:sha(ROOT/p) for p in new},'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'library_build_exit':0,'pictograph_targeted_tests':2,'pictograph_test_exit':0,'pandect_targeted_tests':16,'pandect_test_exit':0,'fleece_targeted_tests':1,'fleece_test_exit':0,'native_observations':5,'native_probe_exit':0,
 'document':{'path':doc,'sha256':sha(dest),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'trace_sha256':{p:sha(ROOT/p) for p in trace},'evidence_sha256':{p:sha(HERE/p) for p in files},'diff_check':0,'retained_isolation':[],'generated_probe_removed':True,
 'receipt_budget':{'cap_bytes':100_000_000,'audit_retained_bytes':sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()),'required':'Compact read/build/test/probe/merge-preservation evidence; no new captures'},
 'limits':['Full Mere source goal incomplete; current Pictograph/Pandect complete reading is not a whole-product correctness claim.',
 'All merge-invalidated source fully reread; source inventory3355 includes concurrent WIP.',
 'Five native observations: three split-admission examples, one stale-origin defect, one pending-intent ownership limit. No wrong durable owner write proven.',
 'Only named targeted tests executed; other Pictograph/Pandect/Fleece tests not inferred green.',
 'Fleece2/17 read; crawler source read but no current crawl-feature/runtime/robots/frontier qualification.',
 'New arrangement helpers exist but host save/reopen adoption not established. Multi-container clearing and schema mirrors remain qualified.',
 'Old rendering findings retain historical native evidence, not falsely promoted as current reruns.',
 'No production/original test changes, rasterizer/browser/GPU, durable owner write, application network fetch or user files.',
 'Stable shared target reused; owned probe executable/PDB removed; no isolated target/home/worktree.']}
assert receipt['receipt_budget']['audit_retained_bytes']<100_000_000
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('pictograph_refresh_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'areas':areas,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
