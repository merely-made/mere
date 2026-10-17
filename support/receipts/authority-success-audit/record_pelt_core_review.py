"""Checkpoint Pelt core reading, actual default CPU suite and owner findings."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='pelt_core_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('opfs_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256'])
runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256'];external=dict(old.get('external_context_sha256',{}))
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(Path(p))==h for p,h in external.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
inventory=load('source_inventory.json')['files'];coverage=load('read_coverage.json')
paths=[p for p in inventory if p.startswith('ports/pelt/core/')]
assert len(paths)==21 and sum(inventory[p]['chars'] for p in paths)==441183
paths+=['ports/pelt/Cargo.toml','ports/pelt/build.rs','ports/pelt/desktop/Cargo.toml','ports/pelt/desktop/appearance.rs']
assert len(paths)==25 and sum(inventory[p]['chars'] for p in paths)==484406
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():
    assert h==inventory[p]['sha256']==coverage[p]['sha256'] and coverage[p]['through']==inventory[p]['chars']
assert load('pending_read.json')==[]
probe=load('pelt_core_owner_probe_receipt.json');assert probe['exit']==0 and probe['passed']==114 and probe['original_tests']==111
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==28 and all(not Path(item['path']).exists() for item in probe['removed_outputs'])
aux['ports/pelt/core/README.md']=sha(ROOT/'ports/pelt/core/README.md')
external_paths=[
 Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/7f13b85/components/shared/document-session-api/src/session_engine.rs'),
 ROOT.parent/'turnstone/src/shell/content_pool.rs']
for p in external_paths:external[str(p.resolve())]=sha(p)
root_lock=tomllib.loads((ROOT/'Cargo.lock').read_text());test_lock=tomllib.loads((HERE/'cpu-pelt-core-runtime/Cargo.lock').read_text())
supplier_sources={}
for name in ['genet-host-api','document-session-api']:
    actual=[p['source'] for p in test_lock['package'] if p['name']==name and 'source' in p]
    expected=[p['source'] for p in root_lock['package'] if p['name']==name and 'source' in p]
    assert len(actual)==1 and actual==expected;supplier_sources[name]=actual[0]
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q404' and authority['entries'][-1]['id']=='A1205'
new=[
 ('Q405','Adopted Pelt session can render but panic during declared capability inspection','ports/pelt/core/src/lib.rs:383-433,605-615; tests/content.rs:460-485',
  'Public from_session_shared accepts an already owned session and empty SessionRegistry without spawning or validation; frame works. a11y_capability then panics at the alleged invariant that a live controller keeps its registered engine. A registry-backed adoption control returns Partial. Existing adoption and placement-a11y fixtures themselves use empty registries but do not call this capability accessor.',
  'Make adopted-session metadata explicit or validate that construction supplies it, with a typed missing-metadata outcome instead of a hidden panic. Preserve no-refetch adoption. Targeted current Turnstone reader path supplies real shared factories, which mitigates this exact trigger there; its timer test adopts without factories. No live product inspection crash was observed.'),
 ('Q406','History restores the recorded document engine but leaves later route metadata active','ports/pelt/core/src/lib.rs:174-181,1271-1289; src/content.rs:623-633',
  'Actual routed content opens first engine, navigates to second, then Back restores first engine and original address while route.selected_engine and route.active_engine still report second. Reload preserves that mismatch; Forward restores agreement. HistoryEntry retains engine ID/request but no route and traverse_to does not restore route. Existing engine-history positive test checks engine/address but omits route equality.',
  'Keep history engine and its route/provenance together, or reconstruct an honest recorded-engine route on successful traversal. Do not reroute history through current policy blindly. Workspace metadata consumes this route for labels/diagnostics. Reproduction is linear-history engine loading with supplied factories; host-history graph navigation is a separate owner and no headed wrong label was measured.'),
 ('Q407','Placement Session Cancel is discarded while nonzero pointer gestures retain capture','ports/pelt/core/src/input.rs:70-89; src/pool.rs:641-668; src/lib.rs:752-774',
  'Actual pool admits pointer37 and49 in one focused placement. input(placement,SessionInput::Cancel) lowers to pointer0 terminal, is rejected for lack of pointer0 capture, delivers nothing and leaves both nonzero captures. Direct controller Cancel delivers, while input_at(window,Focus(false)) correctly cancels both through existing helpers. Controller explicitly treats Session Cancel as global within its session.',
  'Define and consistently implement placement-level Session Cancel scope across pool/content/controller, using retained receiver envelopes for admitted gestures. Keep explicit per-pointer and window-wide cancellation distinct; this audit does not choose an all-window policy. Current direct/window helpers are real mitigations. No physical host trigger or loss of a real user gesture was measured.')]
for ident,title,source,evidence,repair in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'reproduced current public CPU API behavior; documented, not repaired',
      'scope':'Actual unchanged engine-free Pelt core and contract suppliers. Test-supplied session factories, positive/negative controls and caught panic. No real engine, browser, desktop, hardware input or physical AT execution.',
      'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['pelt_core_owner_probe_receipt.json']})
rows=[
 ('Arrangement authority and retained receiver custody are actually separated','Workbench owns TileTree reduction; PeltWorkspace projects it into a retained PeltControllerPool.',
  ['ports/pelt/core/src/workspace.rs','ports/pelt/core/src/pool.rs','ports/pelt/core/tests/workspace.rs'],
  'Compatibility controller constructors and routed constructors remain, but both now use the same pool. Workspace pointer_capture is a legacy projection of pool pointer0, not the actual per-pointer routing authority. Original tests prove retained inactive tabs, close/reopen lifetime and host-accepted tearout transfer. Source/engine state is not duplicated merely because a host has its own arrangement.'),
 ('Content identity and placement identity govern different lifetimes','Content keys identify sessions; placement keys identify window geometry, visibility, focus and admission generation.',
  ['ports/pelt/core/src/pool.rs','ports/pelt/core/tests/pool.rs'],
  'Repeated appearances share scroll/history when content key is shared, while independent Reader appearances use separate content keys. Exact session identity combines process-unique instance and generation. Original tests prove no respawn on appearance reopen, geometry revalidation and current-receiver scheduling across windows; mere generation1 is not a safe externally visible identity.'),
 ('Shared routing decisions coexist with incomplete history metadata restoration','PeltRegistries chooses per load; HistoryEntry keeps only engine ID and spawn request.',
  ['ports/pelt/core/src/lib.rs','ports/pelt/core/src/content.rs','ports/pelt/core/src/workspace.rs','ports/pelt/core/tests/content.rs'],
  'Q406 proves route metadata can disagree with the actual restored engine. Unavailable user pins remain selected with explicit active fallback, automatic routing filters availability, and native overlay/embedded contracts refuse or fall back rather than claiming composition. Passed originals cover fallback and failed reconstruction preservation, not every route/history identity composition.'),
 ('Linear controller history and host graph history are explicit alternatives','Host history returns navigation/submission intentions while retaining one current entry; linear mode owns back/forward.',
  ['ports/pelt/core/src/lib.rs','ports/pelt/core/tests/host_history.rs','ports/pelt/core/README.md'],
  'The two modes have different owners by design. Real default CPU tests prove no-load host intentions, resolved actions, failed POST refusal in linear mode and held-body replacement. Surface engines keep their own history explicitly. Q406 does not establish a Turnstone graph-history failure or authorize merging those authorities.'),
 ('PageLoad is real shared loading state while the host owns acquisition','Controller queues exact Fetch/Cancel identities and accepts typed PageLoad progress/outcomes.',
  ['ports/pelt/core/src/lib.rs','ports/pelt/core/tests/host_loading.rs','crates/system/page-load/src/lib.rs'],
  'Original tests pass for supersede/Stop/reload freshness, streamed prefix versus body replacement, downloads and typed prompts/identity/certificate outcomes. Failed session replacement retains old history/identity. No external HTTP/protocol fetch ran. Address, source-response artifact and retained request are separate representations; redirected/streamed reconstruction remains a consumer-composition followup.'),
 ('Input retirement has real controls and an unresolved global-envelope seam','Pool owns pointer admission and capture routing; controller/surface retain exact receiver-side admitted gestures.',
  ['ports/pelt/core/src/input.rs','ports/pelt/core/src/content.rs','ports/pelt/core/src/pool.rs','ports/pelt/core/tests/capture_terminals.rs','ports/pelt/core/tests/capture_geometry.rs','ports/pelt/core/tests/workspace_retirement.rs'],
  'Original actual tests prove Cancel before receiver drop, failure/noop preserving old capture, failed geometry retirement with old envelope/scale, sensor/button masks and window-specific focus loss. Q407 isolates the separate Session Cancel lowering disagreement, not failure of those controls. Per-pointer, per-placement, per-session and window cancellation policies must remain explicit.'),
 ('Published accessibility routes are checked against complete current semantics','Pool snapshot keeps owner/content/placement/generation/session/publication and captured projection.',
  ['ports/pelt/core/src/pool_a11y.rs','ports/pelt/core/tests/pool_a11y.rs','ports/pelt/core/src/lib.rs'],
  'Six original placement semantic tests pass: exact restored full projection can translate revision only inside atomic owned dispatch; source changes/field changes/side effects and old geometry refuse. Q405 is separate pre-spawn capability metadata assumption at adopted construction. Projection support is not physical AT or desktop adapter acceptance.'),
 ('Native engine registry is shared reuse with narrow concrete presentation adaptation','Opt-in native.rs composes Genet Livery, Reader, normalized documents and retained clips; fonts are host-held.',
  ['ports/pelt/core/src/native.rs','ports/pelt/core/README.md'],
  'Entire source read but feature genet not compiled this pass. Normalized document identity checks address/source-kind, HeldResources resolves inventory/data only, and font injection downcasts known session types. FontEngine forwards engine_id/spawn but not newer request-aware/capability methods; actual SessionEngine default is Partial, so do not invent an Opaque result or claim wrong current capability without supplier tracing. Metadata/request forwarding remains migration followup.'),
 ('Appearance selection is host-owned over shared Tabard definitions and stores','Pelt basic presets and ThemeCatalog use shared ThemeChoiceStore/ThemeLibraryStore and shared appearance lowering.',
  ['ports/pelt/desktop/appearance.rs','ports/pelt/desktop/Cargo.toml'],
  'Full appearance source read. Preset actions keep application font requests; catalogue validates saved theme/mode before choice publication, preserves requested choices on fallback and uses strict storage errors. Current library collision guards mitigate Q392; application generic font resolver is the Q393 consumer. These desktop/catalogue tests were read, not rerun; remaining Pelt desktop source still unread.'),
 ('Feature aliases and descriptor prose still span old and new engine ownership','Pelt viewer-engine/livery-scripted/scripted-nova identifiers preserve compatibility while current Livery and shared document lanes are the implementation.',
  ['ports/pelt/Cargo.toml','ports/pelt/build.rs','ports/pelt/core/Cargo.toml','ports/pelt/desktop/Cargo.toml'],
  'Manifest graph explicitly defaults to Livery/Reader/netfetch/image-decode; nearby older prose claiming a default free of async networking needs reconciliation. Script-engine Vano keeps Nova compatibility identifiers. Windows stack reserve is bin-specific. No full default Pelt desktop, scripted engine, native window or presentation build was performed.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1206):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'complete core source read and default CPU suite; remaining Pelt desktop review ongoing','verification':NAME})
for area,detail in [
 ('Pelt adopted-session capability ownership','Q405 constructor accepts no-factory ownership but metadata accessor assumes a registered factory. Define adopted capability/request metadata explicitly; targeted current Turnstone reader adoption supplies factories and mitigates this trigger.'),
 ('Pelt route and capture migration','Q406 history route provenance and Q407 cancellation scopes must follow the same actual receiver boundary as existing correct replacement retirement. Do not treat independent counters/fields as sufficient identity or choose an all-window cancellation policy implicitly.'),
 ('Pelt native forwarding and held source followup','Check FontEngine forwarding of newer SessionEngine request/capability methods against real suppliers; default is Partial, not Opaque. Surface address/body reconstruction, direct content invalid scale versus pool guards and redirected streamed source binding remain source-only consumer followups.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2852 and stats['read_chars']==33504293 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'ports/pelt/core plus root manifest/build and desktop manifest/appearance','files':25,'chars':484406,'verification':NAME,'basis_commit':head,'qualification':'All21 inventoried core files read;111 original default CPU tests and3 audit cases pass. Opt-in native engines read, not compiled. Desktop appearance read only; full Pelt remains incomplete.'})
next_read={'path':'ports/pelt/desktop/dx12_surface.rs','start':0,'reason':'Continue remaining Pelt desktop/source/tests/examples, then Signalman, Tabard desktop, scripts and supplier patches without narrowing inventory.'}
pending='No audit-owned process running.114 actual core CPU tests pass,111 original3 audit.28 exact owned exe/PDB outputs removed. No opt-in Genet/native or desktop/render/input hardware acceptance.25 files added; full Pelt and repository source review remain incomplete. Main/upstream unchanged; eight reviewed upstream commits uninstalled due shared report overlap.'
notes['current_area']='Pelt core complete and initial desktop appearance read; continue remaining desktop'
notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending})
notes['checkpoint']['new_receipts'].extend([NAME,'pelt_core_owner_probe_receipt.json'])
addition=f'''

### Pelt core receiver and history authority review, 2026-10-17

All21 Pelt core source/manifest/test files are read (441,183 characters), plus root Cargo/build and desktop Cargo/appearance:25 files,484,406 characters this pass. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. No stale or pending credit. One oversized tool emission was truncated, discarded without acknowledgment and reread in full at smaller chunks; it contributes no duplicate credit. Full Pelt desktop and the repository remain incomplete. Main/upstream remain {head}/{upstream}, with the eight reviewed upstream commits uninstalled because the shared report overlaps.

Q405: public adopted-session construction accepts a live session and empty factory registry and renders correctly, but declared capability inspection panics on an unchecked registered-engine assumption. Registry-backed adoption control returns Partial. Existing adoption/semantic fixtures use the empty registry shape without calling that accessor. Targeted current Turnstone Reader adoption supplies shared real factories, mitigating this exact trigger; no live product crash is asserted. Q406: Back restores the first engine/address after navigating to a second engine, but selected/active route metadata still reports the second; Reload retains the mismatch, Forward agrees again. Existing history test checks engine/address but omits route equality. Q407: placement SessionInput::Cancel lowers to pointer0 and is discarded with only nonzero captures, leaving those captures intact; direct controller Cancel and window Focus(false) controls deliver. Per-pointer, placement, session and window scope need an explicit shared boundary, not an inferred all-window policy.

Actual unchanged engine-free core root passed114 CPU tests:111 originals across its unit suite and all12 integration files, plus3 audit cases. Existing controls exercise receiver cancellation before destruction, geometry/scale failure, chord/sensor identity, typed host load/Stop/supersede, retained close/reopen, host tearout acceptance, shared-registry fallback, repeated appearance geometry and exact semantic publication revalidation. Supplied engines are test factories; no native document engine, real surface/input hardware, browser, desktop GPU/presentation or physical AT ran. native.rs was fully read but feature genet was not compiled. All28 exact test exe/PDB outputs named by the completed run were marker-checked and removed; shared dependency cache remains in the approved stable target. No new target/home/worktree or production edit.

A1206-A1215 record actual consolidation and unfinished ownership. Workbench owns arrangement; the pool owns retained receiver/placement admission, while the workspace's pointer0 field is a compatibility projection. Shared content keys intentionally share scroll/history; independent Reader appearances use independent keys. Host graph history and reference linear history are explicit alternatives, not a reason to merge owners. PageLoad is actual shared state and the host owns acquisition; semantic publication keeps complete current projection equality, not just counters. Native factories reuse shared engines but concrete font adaptation and forwarding of newer request/capability metadata need tracing; SessionEngine's current default is Partial. Pelt appearance already composes shared Tabard choice/library/resolution with host policy, including guards previously mitigating Q392 and the Q393 font resolver consumer. Appearance tests and full native factory behavior were not rerun. Compatibility feature names and stale default-networking prose remain tracked alongside runtime migrations.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [actual core CPU suite](../../../support/receipts/authority-success-audit/pelt_core_owner_probe_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. Receipt bytes remain under100 MB. Continue from desktop/dx12_surface.rs.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_pelt_core_review.py','pelt_core_owner_probe_receipt.json',*probe['evidence_sha256']]
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
cache={'path':'C:/t/cargo-targets/mere/build','owner':'Shared Mere build cache; audit refreshed Pelt core contract suppliers','reason':'Approved reusable shared target/build-dir;28 exact run-owned test exe/PDB files removed. No isolated output or running audit test process.'}
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'opfs_review_receipt.json','sha256':sha(HERE/'opfs_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,'supplier_sources':supplier_sources,
 'new_findings':[r[0] for r in new],'authority_entries':ids,'all_pelt_core_inventory_source_read':True,'all_pelt_inventory_source_read':False,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':cache,
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.25 source files read;114 actual default core CPU tests pass,111 original3 audit.28 run-owned binaries removed. Native feature and desktop source-only boundaries explicit. No production edits or new isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
