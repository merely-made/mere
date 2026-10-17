"""Append the bounded desktop source audit without changing production files."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='pelt_desktop_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('pelt_core_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
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
i=load('source_inventory.json')['files'];c=load('read_coverage.json')
paths=[p for p,v in i.items() if p.startswith('ports/pelt/desktop/') and p not in guards and c.get(p,{}).get('through')==v['chars']]
assert len(paths)==20 and sum(i[p]['chars'] for p in paths)==334254
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():assert h==i[p]['sha256']==c[p]['sha256']
assert load('pending_read.json')==[]
partial={'path':'ports/pelt/desktop/workspace_viewer.rs','through':347,'sha256':sha(ROOT/'ports/pelt/desktop/workspace_viewer.rs')}
assert c[partial['path']]=={'through':partial['through'],'sha256':partial['sha256']}
probe=load('pelt_desktop_success_probe_receipt.json');assert probe['exit']==0 and probe['passed']==6
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==2 and all(not Path(p['path']).exists() for p in probe['removed_outputs'])
# These were targeted caller/supplier reads only, not full-file coverage.
aux['ports/pelt/viewer.rs']=sha(ROOT/'ports/pelt/viewer.rs')
for name in ['livery.rs','scripted.rs']:
    p=Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/7f13b85/components/genet-documents/src/engines')/name
    external[str(p.resolve())]=sha(p)
# Refresh the affected upstream diff and prove that the tested failure bodies
# are still exact upstream bodies. Do not claim upstream is installed or built.
remote=subprocess.check_output(['git','show',f'{upstream}:ports/pelt/desktop/static_viewer.rs'],cwd=ROOT,text=True)
cuts={
 'run_headed_with':('pub(crate) fn run_headed_with<','\n#[cfg(feature = "present")]\npub(crate) mod windowed'),
 'resumed':('        fn resumed(&mut self, event_loop: &ActiveEventLoop)','\n        fn window_event('),
 'outcome':('        pub(crate) fn outcome(&self)','\n        fn window_title(')}
upstream_bodies={}
for key,(start,end) in cuts.items():
    a=remote.index(start);b=remote.index(end,a);h=hashlib.sha256(remote[a:b].encode()).hexdigest()
    assert h==probe['extracted_body_sha256'][key];upstream_bodies[key]=h
for path in ['ports/pelt/desktop/smoke_netrender.rs','ports/pelt/desktop/smoke_windows.rs']:
    assert subprocess.check_output(['git','show',f'{upstream}:{path}'],cwd=ROOT,text=True)==(ROOT/path).read_text(encoding='utf-8-sig')

notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q407' and authority['entries'][-1]['id']=='A1215'
new=[
 ('Q408','Single-document viewer reports success after native window or renderer startup failure',
  'ports/pelt/desktop/static_viewer.rs:2053-2068,2299-2319,2488-2521; ports/pelt/viewer.rs:917-938',
  'resumed logs create_window and SurfaceHost::boot failures and exits without retaining an error. run_headed_with checks only receipt_failure and then returns Ok(outcome). Exact extracted bodies with deterministic platform doubles return Ok, created_window=false, redraws=0 and no product receipt for both failures even when a named article receipt was requested. Injected semantic-failure control returns Err; successful-startup control has a window. The caller prints successful viewer status on Ok. These startup/caller/outcome bodies are byte-equivalent to origin/main.',
  'Retain a startup/application error and return it before the outcome, matching the existing platform-smoke error precedence. A requested product receipt also needs a final completion check. Treat intentional headless no-window behavior separately.',
  'Six source-extracted CPU cases with deterministic OS/event-loop/renderer doubles, not a complete desktop build or observed native startup failure.'),
 ('Q409','Basic NetRender smoke returns Ok for blank and wrong-color readback',
  'ports/pelt/desktop/smoke_netrender.rs:12-65; ports/pelt/viewer.rs:1557-1568',
  'The smoke submits a full red scene but only counts any nonzero RGBA pixel and always returns Ok. The actual complete function body with a deterministic readback double returns Ok painted_pixels=0 for all-zero pixels and Ok painted_pixels=4096 for all-white pixels. The CLI logs either outcome and does not reject zero or assert red. The nearby WebGL smoke already rejects wrong expected sample colors; receipt_capture also refuses blank capture. The smoke source is unchanged at origin/main.',
  'Require nonblank output and meaningful expected red-pixel/region assertions before returning success; keep readback/presentation/backend qualification separate.',
  'CPU function-body/readback-double proof. No claim that a real current renderer produced blank or white, and no GPU or display run.'),
 ('Q410','Windows presentation smoke redraws the configured extent after a native resize',
  'ports/pelt/desktop/smoke_windows.rs:275-279,296-300,362-372',
  'The Resized handler ignores the new size and requests redraw; every subsequent Scene is constructed from config.width/config.height. The adjacent comment claims current backing-pixel resolution. macOS and Wayland smoke paths actually read window.inner_size on every redraw. The Windows outcome likewise reports configured dimensions. Source is unchanged at origin/main.',
  'Read current physical client size for rendering and report achieved dimensions, with an explicit zero-size/minimized policy. Retain fixed-size smoke semantics only if the API and comments name them honestly.',
  'Source-confirmed extent selection disagreement. No native window resize, GPU swapchain consequence or visual symptom was exercised.')]
for ident,title,source,evidence,repair,scope in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'documented, not repaired; '+('source-confirmed' if ident=='Q410' else 'reproduced exact extracted executable bodies'),
      'scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,
      'diagnostics':[] if ident=='Q410' else ['pelt_desktop_success_probe_receipt.json']})
rows=[
 ('Pelt DX12 import and cache custody','Pelt owns per-TileId native-surface imports, waits and same-device tearout transfer.',
  ['ports/pelt/desktop/dx12_surface.rs'],
  'Raw transferred handles have RAII cleanup, metadata checks and explicit COMMON handoff. Nonzero raw handles reimport even at the same epoch. Owned Scry payload reuse keys generation and geometry, without producer-instance identity here. Trace replacement invalidation before claiming aliasing. stage_wait takes pending wait before backend/fence validation; caller retirement/error handling remains unread. No GPU import or fence execution.'),
 ('Frisket geometry is actual shared layout authority','Cambium views serialize into ScriptedDom; Livery supplies paint, hit tests and content holes.',
  ['ports/pelt/desktop/frisket_surface.rs'],
  'Full implementation and original geometry/theme/a11y tests read. Pelt host projects Workbench topology and chrome; it does not duplicate a layout algorithm. Generic Frisket CSS, Pelt skin variables and host-authored stylesheet have separate scopes. Preset colors and host typography remain concrete policy; tests were read, not executed.'),
 ('Frisket rebuilds require semantic focus and publication lifetime ownership','Tree, chrome, stylesheet and viewport changes replace the retained document.',
  ['ports/pelt/desktop/frisket_surface.rs','ports/pelt/desktop/workspace_viewer/accessibility.rs'],
  'Semantic virtual focus explicitly survives DOM-ID replacement. Raw DOM-derived AccessKit IDs and current target maps still need host publication/action revalidation, so duplicate or stale-action claims wait for the accessibility caller read. Saved theme/mode actions use indices and likewise need current-catalogue validation. Only frisket_surface is fully read in this pass; accessibility caller is a named next read.'),
 ('Declared content accessibility remains separate from actual child projection','Shared Genet projects shell DOM; named Region apertures describe engine capabilities.',
  ['ports/pelt/desktop/frisket_surface.rs'],
  'The source explicitly says child-engine projections need a namespaced merging protocol. Visible click support uses the live retained target helper. Loading status and error alert overlays keep content input routes live. Capability declaration, inspectability and merged native accessibility are distinct acceptance claims; no physical AT or headed test ran.'),
 ('Single-document compatibility shell is still a separate host adapter','ControllerViewerContent uses the public PeltController, while private ViewerContent supports older direct documents.',
  ['ports/pelt/desktop/static_viewer.rs','ports/pelt/desktop/scripted_viewer.rs','ports/pelt/desktop/smolweb_glue.rs','ports/pelt/desktop/scripted.rs'],
  'Scripted and Livery/Reader entrypoints share one single-window shell; ScriptedDocument itself is a re-export of the shared engine owner. The ordinary Smolweb entrypoint still loads a direct SmolwebDocument with no-op click/default title metadata, while its forcing receipt uses a registered controller with retained navigation. This is a real unfinished consumer migration, not proof that the ordinary path has receipt coverage.'),
 ('Input decoding is not yet wholly shared across desktop hosts','Single-document host translates winit input privately; workspace retains another native-key adapter.',
  ['ports/pelt/desktop/static_viewer.rs','ports/pelt/desktop/workspace_viewer.rs'],
  'Targeted current supplier read rules out the proposed ordinary-character omission: Livery key_input inserts SessionKey::Character. Workspace decoder is local too, not a shared decoder as one progress message implied. Latest reviewed upstream adds KeyboardText forwarding and explicit request fetch to this host; those changes remain uninstalled/unbuilt. Track layout/physical committed text and IME composition against the matching supplier contract.'),
 ('Capture receipt and native display completion are distinct gates','capture_composition reads an owned same-device target and presents that same target.',
  ['ports/pelt/desktop/receipt_capture.rs','ports/pelt/desktop/static_viewer.rs'],
  'Capture refuses a blank frame and writes PNG before display acquisition. The receipt outcome is assigned only after queue.present, which is a stronger gate than capturing an unrelated target. Window startup still has Q408; early close and failed acquire have no final required-receipt check. Do not promote readback or queue submission to physical display/AT acceptance.'),
 ('Platform smoke copies have stronger startup errors but drift in resize policy','macOS/Wayland use shared compositor factory and explicit display handoff; Windows directly constructs the DXGI backend.',
  ['ports/pelt/desktop/smoke_macos.rs','ports/pelt/desktop/smoke_wayland.rs','ports/pelt/desktop/smoke_windows.rs'],
  'All retain startup errors and return them before outcomes, mitigating the same false-success mechanism Q408 still has. They force the platform GPU backend and share the rendering contract, but their host loops and device construction remain copied. Q410 isolates actual fixed Windows extent versus current-size peers. Early user close can return fewer frames; callers must inspect completion. No platform build/display run.'),
 ('Two GPU smoke recipes enforce different success predicates','The basic smoke counts nonzero pixels; the WebGL smoke validates canonical green and blue samples.',
  ['ports/pelt/desktop/smoke_netrender.rs','ports/pelt/desktop/smoke_webgl.rs'],
  'Q409 is not a reason to discard WebGL validation: the latter uses real same-device producer composition interleaved at a scene-op boundary and rejects mismatched sample colors in source. No actual renderer ran in this pass. A frame count, imported-surface count or nonzero-alpha count does not establish a named image or present success.'),
 ('Reference scenarios exercise shared appearance authoring under explicit stores','Pelt scenarios target ordinary Appearance/Edit controls, saved definitions and application choice.',
  ['ports/pelt/desktop/scenarios/appearance.scn','ports/pelt/desktop/scenarios/appearance_reopen.scn','ports/pelt/desktop/scenarios/typography.scn','ports/pelt/desktop/scenarios/typography_fallback.scn','ports/pelt/desktop/scenarios/typography_reopen.scn','ports/pelt/desktop/scrying_receipt.rs','ports/pelt/desktop/profile.rs','ports/pelt/desktop/lib.rs'],
  'Scenarios assert dirty/save/reopen, invalid-size refusal, undo/redo, generic/installed font and unavailable-family diagnostics. They were read, not rerun. Windows Scrying receipt lazily installs HWND/shared fence and delegates controls; production native lifetime and imported texture identity still need workspace tracing. Headed/headless profile is a declared host policy, not implicit rendering acceptance.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1216):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'bounded desktop source read; full desktop and native execution incomplete','verification':NAME})
notes['ruled_out'].append({'title':'Ignoring winit event.text does not alone prove ordinary Livery typing is dropped at the current local supplier',
 'source':'static_viewer.rs key adapter; current Genet livery.rs:1990-2013','note':'SessionKey::Character explicitly inserts text without ctrl/meta/alt. No Latin typing bug is asserted. Latest upstream separately forwards KeyboardText against a changed supplier contract; no newer supplier build was run. Workspace decoder is local, not shared.', 'verification':NAME})
for area,detail in [
 ('Native surface resource identity and retirement','Trace cache invalidation across producer/session replacement and tearout. Owned payload reuse currently compares TileId, generation and geometry; pending_wait is taken before fallible backend/fence validation. Source suspicions only until host error/retirement paths are read; no GPU repro.'),
 ('Single-window and Smolweb host migration','Ordinary direct Smolweb document differs from retained-controller receipt in navigation/title ownership, and run_smolweb_viewer bypasses headless handling. CLI always supplies Headed. Trace public embedder contracts before broader consequence; do not count forcing-receipt semantics as ordinary viewer acceptance.'),
 ('Retained shell rebuild and action identity','Finish workspace/accessibility caller read for raw DOM-ID publication, semantic focus, theme-index revalidation, capture and required-receipt finalization. Do not assert a stale action solely from document rebuilding.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2872 and stats['read_chars']==33838894 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Pelt desktop native import, Frisket, single-document and smoke hosts, adapter/re-export/profile and five scenarios','files':20,'chars':334254,'verification':NAME,'basis_commit':head,'qualification':'20 full source files; first347 characters of workspace_viewer are separately credited. Six extracted-body CPU cases, no actual desktop/renderer build or headed execution.'})
next_read={'path':partial['path'],'start':347,'reason':'Continue full workspace host, accessibility and native appearance, receipts/tests, then remaining Pelt/Signalman/Tabard/scripts/supplier source.'}
pending='No audit process running. Six extracted-body CPU cases pass; exact run-owned exe/PDB removed. No full desktop build, real GPU, platform window/input/AT, font or scenario acceptance. Upstream refreshed unchanged, eight reviewed commits remain uninstalled. Full source review incomplete.'
notes['current_area']='Pelt single-document, Frisket and platform-smoke source read; workspace host next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'pelt_desktop_success_probe_receipt.json'])
addition=f'''

### Pelt desktop source and success-path review, 2026-10-17

Twenty more files are completely read (334,254 characters): DX12 surface import, Frisket retained chrome and original tests, single-document viewer and original route tests, scripted/Smolweb/Scrying glue, capture, platform smokes, profile/re-exports and five appearance scenarios. The first347 characters of workspace_viewer.rs are separately credited. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale or pending credits. The workspace host, accessibility caller, receipts and remaining repository still require reading. Fresh fetch found unchanged origin/main {upstream}; local main remains {head}. The eight reviewed commits remain uninstalled because of the shared report overlap. The current upstream static/Smolweb diffs were reread; upstream adds request-aware fetch and KeyboardText forwarding, but all Q408-Q410 failure bodies remain unchanged there. No upstream build receipt is implied.

Q408: window creation and SurfaceHost startup failures log and exit without setting the error checked by run_headed_with, which returns Ok with no window, frames or requested product receipt. Q409: the basic NetRender smoke submits red but accepts both blank and wrong-color readback; its CLI only logs the returned count. Six focused CPU checks passed using exact extracted executable bodies with deterministic event-loop/window/SurfaceHost/readback doubles, including successful-startup and semantic-error controls. These qualify the failure propagation, not an actual native or GPU failure. Both exact run-owned exe/PDB outputs were hashed and removed. Q410 is source-confirmed only: Windows smoke ignores resize dimensions and keeps rendering/reporting the configured extent, unlike its current-size macOS/Wayland peers. No visual resize effect was measured.

A1216-A1225 distinguish actual shared owners from remaining adapters. Frisket uses shared Livery geometry/hits and Genet semantic projection; shell controls do not establish a merged engine accessibility tree. DOM rebuilds keep semantic focus but require tracing host publication and action identity. Pelt's controller is shared beneath the single-document shell, while the ordinary direct Smolweb viewer still differs from its controller-based forcing receipt in navigation/title ownership. Input decoders remain local in both hosts. Current local Livery explicitly inserts SessionKey::Character, ruling out the proposed ordinary typing omission; the progress update's suggestion that the workspace decoder was shared was incorrect. Updated upstream committed-text behavior needs its matching supplier, not inference from older source.

Native DX12 ownership, explicit handle closure, metadata checks and tearout transfer are real. Cache reuse still needs producer-instance/invalidation tracing, and taken pending fence waits need caller-error tracing; these remain source followups, not promoted bugs. Capture reads and presents the same owned target and refuses blank frames, but required-receipt finalization after early close/acquisition failure remains a host followup. Platform smokes already retain startup errors and validate display handoff; copied loops still drift. Appearance scenarios and real WebGL sample assertions were read, not rerun. No production edits, full desktop compilation, renderer/native input, font/scenario run, physical display or AT acceptance.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [source-body success probes](../../../support/receipts/authority-success-audit/pelt_desktop_success_probe_receipt.json). Prior report bytes, shared WIP, source and index are preserved. Continue workspace_viewer.rs at character347.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_pelt_desktop_review.py','pelt_desktop_success_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'pelt_core_review_receipt.json','sha256':sha(HERE/'pelt_core_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'partially_read_source':partial,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'upstream_unchanged_failure_body_sha256':upstream_bodies,'new_findings':[v[0] for v in new],'authority_entries':ids,'all_pelt_inventory_source_read':False,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full source completion.20 complete source files plus347 characters of workspace.6 exact extracted-body CPU cases with platform doubles; no whole desktop build or native/GPU/AT acceptance.2 owned binaries removed; no isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
