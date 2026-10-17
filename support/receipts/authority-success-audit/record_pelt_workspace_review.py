"""Checkpoint complete workspace owner/a11y source and bounded CPU witnesses."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='pelt_workspace_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('pelt_desktop_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
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
paths=['ports/pelt/desktop/workspace_viewer.rs','ports/pelt/desktop/workspace_viewer/accessibility.rs','ports/pelt/desktop/workspace_viewer/appearance_native.rs']
i=load('source_inventory.json')['files'];c=load('read_coverage.json');sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():assert h==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars']
assert sum(i[p]['chars'] for p in paths)==305145
assert load('pending_read.json')==[]
probe=load('pelt_workspace_owner_probe_receipt.json');assert probe['exit']==0 and probe['passed']==7
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==2 and all(not Path(p['path']).exists() for p in probe['removed_outputs'])
contract=Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/7f13b85/components/shared/document-session-api/src/a11y.rs')
external[str(contract.resolve())]=sha(contract)
# Targeted old implementation/test/supplier rereads, not new whole-file credits.
for path in ['ports/pelt/desktop/workspace_viewer/tests.rs','ports/pelt/desktop/workspace_viewer/receipts/routing.rs','crates/inker/engines/scrying-engine/src/producer.rs','crates/inker/engines/scrying-engine/src/translation.rs']:
    aux[path]=sha(ROOT/path)
rootlock=tomllib.loads((ROOT/'Cargo.lock').read_text());testlock=tomllib.loads((HERE/'cpu-pelt-workspace-runtime/Cargo.lock').read_text())
suppliers={}
for name in ['accesskit','genet-host-api','document-session-api']:
    rows=lambda lock:[{'version':p['version'],'source':p.get('source')} for p in lock['package'] if p['name']==name]
    assert rows(rootlock)==rows(testlock);suppliers[name]=rows(testlock)
remote= subprocess.check_output(['git','show',f'{upstream}:ports/pelt/desktop/workspace_viewer.rs'],cwd=ROOT,text=True)
cuts={
 'receipt_validation':('    if config.workspace_receipt.is_some()','    let tree = tree_from_urls(&config.urls);'),
 'receipt_driver_guards':('        if self.config.workspace_receipt.is_some() && self.redraws > 0 && !self.receipt_complete {','\n        self.refresh_chrome();\n        let (logical_width, logical_height)')}
upstream_bodies={}
for key,(start,end) in cuts.items():
    a=remote.index(start);b=remote.index(end,a);h=hashlib.sha256(remote[a:b].encode()).hexdigest()
    assert h==probe['extracted_body_sha256'][key];upstream_bodies[key]=h
assert subprocess.check_output(['git','show',f'{upstream}:ports/pelt/desktop/workspace_viewer/accessibility.rs'],cwd=ROOT,text=True)==(ROOT/paths[1]).read_text(encoding='utf-8-sig')
focus='''            WindowEvent::Focused(focused) => {
                self.accessibility.update_window_focus(focused);
                let effect = self.workspace.input(SessionInput::Focus(focused));
                self.apply_effect(effect);
            },'''
assert focus in remote and focus in (ROOT/paths[0]).read_text(encoding='utf-8-sig')
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q410' and authority['entries'][-1]['id']=='A1225'
new=[
 ('Q411','Concurrent Pelt receipt drivers share success and can skip one requested proof',
  'ports/pelt/desktop/workspace_viewer.rs:524-555,1748-1753,2450-2506,666-680',
  'Configuration rejects named-plus-legacy and acceptance-plus-cancellation, but admits interaction+capability and interaction+tearout. In the actual render guard prefix, a ready capability proof sets receipt_complete before the interaction branch, so interaction never runs. An interaction completion likewise skips the later requested tearout. The output flags all enabled legacy drivers using that same bool, and tearout finalization trusts it. Source-extracted tests execute the unchanged admission/guard bodies with deterministic driver doubles: both mixed shapes admit and skip the second proof; pending-capability and refused-combination controls behave distinctly. Guard and validation bodies remain unchanged at origin/main.',
  'Reject every unsupported multi-driver combination before windowing, or give each admitted driver its own completion and finalization state. Do not infer one proof from a shared aggregate bool.',
  'Four source-extracted CPU cases with deterministic receipt-driver/event-loop doubles, not actual native capability/interaction/tearout execution.'),
 ('Q412','Both Pelt child-tree lowerings discard the supplied accessibility description',
  'ports/pelt/desktop/workspace_viewer/accessibility.rs:532-685,953-1141; document-session-api/src/a11y.rs:236-249',
  'The current neutral DocumentA11yNode carries description separately from name/value. Both append_document_child and the copied primary-window loop copy name and value but never description. Exact complete helper/primary-lowering bodies over real current Inker neutral nodes and AccessKit0.24.1 drop a supplied supplemental description while preserving label, role, value and SetValue support. A direct AccessKit description control retains the text. The helper claims sharing across primary and tearouts, but primary has an independent copy. This file is unchanged at origin/main.',
  'Carry neutral description into the AccessKit node through one lowering implementation used by primary and secondary trees, with both consumers covered. This is lost supplemental text, not a missing control name or a claim of physical screen-reader acceptance.',
  'Three source-extracted CPU cases with actual neutral and AccessKit types and deterministic shell/namespace fixtures. No full desktop compilation, original desktop suite, real DOM or physical AT run.'),
 ('Q413','Primary host chrome drags survive window focus loss outside the pool cancellation boundary',
  'ports/pelt/desktop/workspace_viewer.rs:3607-3636,3681-3704,3756-3789,5564-5568',
  'Primary Focused(false) updates accessibility and forwards Focus(false) to the workspace, but leaves WorkspaceApp.gesture unchanged. Divider and tab drags live only in that host field; pointer_move processes them before content input. If the terminal mouse release is not delivered while focus is away, a later cursor move continues divider mutation or marks the old tab drag moved, and pointer_up can commit its old drop. The only gesture take is pointer_up_inner; there is no focus-loss retirement. Core pool focus cancellation already clears admitted document captures, which does not retire this separate chrome gesture. The focus handler is unchanged at origin/main.',
  'Retire host-owned Divider/Tab/Content gesture state on focus loss alongside the existing core focus notification; keep receiver cancellation in the core. Cover focus-loss/missing-release/return-move rather than assuming a document capture test exercises chrome lifetime.',
  'Source-confirmed state-machine gap under a missing terminal-release condition. No OS focus/mouse sequence, physical drag or observed layout/drop symptom was reproduced.')]
for ident,title,source,evidence,repair,scope in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'documented, not repaired; '+('source-confirmed' if ident=='Q413' else 'reproduced exact source-extracted executable bodies'),
      'scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,
      'diagnostics':[] if ident=='Q413' else ['pelt_workspace_owner_probe_receipt.json']})
rows=[
 ('Workspace controller selection uses shared registries with local host policy','Pelt constructs shared Livery/Reader/scripted/Smolweb engines and host-supplied surface producers.',
  paths[:1],
  'Routing policy maps default Genet Web and fallback to Livery, while explicit overrides remain visible. Reader initial overrides acquire held response identity/body. Per-engine application colors and reference profile paths are host policy, not duplicate engine implementations. Scry host/importer is installed only when an initial route is a surface; later route/producer changes need the same ownership initialization rather than only a tile key.'),
 ('Named workspace finalization is real but legacy proof ownership is conflated','Workspace startup retains errors and named capture/tearout completion is checked after the event loop.',
  paths[:1],
  'This actually mitigates Q408 for the workspace startup path. Q411 isolates unsupported concurrent legacy drivers sharing one receipt_complete bool, not failure of the exclusive named finalization. Interaction receipt alone has no matching final incompletion error; early close remains distinguishable only in its returned false field. No native receipt ran.'),
 ('Tearout preflight retains source custody before one explicit accept boundary','Hidden destination surface presents a source-owned frame through the same RenderCore before workspace.accept_tearout.',
  paths[:1],
  'SurfaceTearoutImportReceipt claims the source cache only after swap acquisition and returns it on rejection. Decline restores source geometry and identity; rehost ordinary failure restores, indeterminate migration transfers to an explicit destination error state. These are concrete control paths, not just ownership prose. Window/producer/GPU execution remains unverified this pass.'),
 ('Native import cache identity is narrower than session identity','Primary cache retains native surfaces by current Surface route TileId; imported epochs/metadata identify texture reuse.',
  ['ports/pelt/desktop/dx12_surface.rs',paths[0],'crates/inker/engines/scrying-engine/src/translation.rs'],
  'Host replacement/route actions do not explicitly invalidate the primary cache by producer instance. Scry translation really emits OwnedPayload and copies producer generation into resource_epoch. Whether a concrete replacement can reuse the same generation/geometry for a different resource needs producer-generation and replacement tracing; no stale native texture is yet reproduced. This is an unfinished lifetime/identity seam, not a GPU finding.'),
 ('Synchronization failures have distinct terminal and rollback owners','Primary stage_wait errors terminate the host; accepted secondary errors enter a retained shell-only alert.',
  ['ports/pelt/desktop/dx12_surface.rs',paths[0]],
  'This narrows the earlier suspicion that taking pending_wait before fallible validation silently retries in the normal loop. It does not: the primary exits, and accepted secondary stops polling/composing its producer. Hidden preflight failures can return the cache to a still-live source; pending-wait restoration after that rollback remains a separate failure-injection followup.'),
 ('Host chrome gesture custody remains outside document capture custody','WorkspaceApp owns one Divider/Tab/Content gesture while PeltWorkspace/pool own admitted receiver captures.',
  paths[:1],
  'Q413 shows the missing host retirement on focus loss. Correct core multi-pointer cancellation does not prove this host field is retired. Physical/layout coordinate conversion and hit authority use shared Frisket geometry; the primary and secondary still have separate event-routing implementations and feature coverage.'),
 ('Child semantic namespaces correctly bind session identity and never recycle issued IDs','Each native window has its own bridge/action map, per-tile namespace and process-unique controller instance/generation.',
  [paths[0],paths[1]],
  'Session replacement removes old namespace mappings; new child IDs avoid current shell IDs and all historical child IDs. Foreign TreeId requests refuse. Virtual Focus differs from actual input focus, and tile geometry/placement generation are revalidated. The historical assigned-ID set grows with churn; bounded-state policy is still a followup. No blanket stale shell-ID alias is established by DOM rebuild alone.'),
 ('Claimed shared child lowering still has two executable implementations','append_document_child says it serves primary and tearouts, but only secondary calls it.',
  [paths[1]],
  'Primary workspace_accessibility_projection repeats the entire semantic-to-AccessKit loop. Q412 reproduces missing description in both. Namespace allocation is shared; semantic lowering is not. Reconcile the source claim and remove the divergent copy through its owning migration, rather than treating same source file or same types as one implementation.'),
 ('Desktop semantic admission has not migrated to the complete pool snapshot boundary','Both primary and secondary duplicate session/geometry/placement/revision/action checks and dispatch controllers directly.',
  [paths[0],paths[1],'ports/pelt/core/src/pool_a11y.rs'],
  'The core pool already has retained full-projection publication/owned dispatch. Desktop checks revision and node action flags after prepare, retaining only an action-bit snapshot. This is real unshared validation, but the neutral projection contract requires revision to change with observable semantics; no same-revision compliant-engine stale action or new bypass is claimed without a consumer proof. Primary click checks press_handled; secondary copy omits that guard.'),
 ('Appearance authoring and activation remain explicitly separate authorities','Shared Workshop saves definitions; Pelt activates application choice through its ordinary controls.',
  [paths[0],paths[2]],
  'The native proof checks held controller instance/generation, address/history/focus/tile count/content aperture around activation and then hands capture to the existing workspace pipeline. Requested typography and fallback notices are checked as computed CSS, not proof of actual chosen physical font. Scenario failure/interruption becomes receipt_error. Native workflow was read, not rerun.'),
 ('Host chrome state has positive explicit owner bindings','Engine menu stores opening TileId and dismisses on focus change; appearance indices resolve against the current catalogue.',
  [paths[0],paths[1]],
  'Pointer and accessibility tab/close actions both clear address/menu/appearance, ruling out the initially suspected simple address-edit retarget on those paths. Saved theme activation uses current index and validates through ThemeCatalog; applied theme stays pinned across library refresh until explicit choice. Broader catalogue publication changes still need review with their owner, not a fabricated stale-index bug.'),
 ('Secondary presentation errors preserve custody while exposing a stopped renderer','Accepted tearout retains its workspace and native import receipt and paints a host-owned alert after error.',
  paths[:1],
  'This prevents a silent frozen surface from being treated as continued render progress. Composition expects a fixed document-versus-native import shape after acceptance; route/lane replacement and needed native host backend/importer reinitialization remain caller followups. One shared wgpu device does not by itself unify controller, native resource, accessibility and window lifetimes.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1226):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'workspace/accessibility/native-proof source complete; receipts/tests/native acceptance pending','verification':NAME})
for area,detail in [
 ('Native producer instance, lane and backend transition','Primary native cache uses tile/producer epoch and Scry emits OwnedPayload generation. Trace concrete generation reset/replacement, late initial-surface initialization, accepted tearout lane changes and preflight fence rollback. These broader effects remain documented; no stale texture, live wrong-device fault or physical failure claim.'),
 ('Complete semantic publication migration','Both desktop hosts copy weaker revision/action validation rather than use the existing core owned publication boundary. Preserve neutral revision contract and gather a compliant engine counterexample before claiming a bypass. Share child lowering first: Q412 is already proved over actual supplied description.'),
 ('Host chrome drag retirement','Q413 identifies primary-only gesture state surviving Focused(false). Read original desktop tests and qualify missing-release/refocus through the actual host before physical claims.'),
 ('Native appearance acceptance boundary','NativeAppearanceProof checks held-session invariants and ordinary activation, but all its source/tests were read only. Its lifecycle completion is separate from physical font realization, native input and physical AT. Existing shared Workshop protection finding remains in the prior ledger.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
notes['ruled_out'].append({'title':'Ordinary pointer and accessibility tab activation clear an active chrome address edit',
 'source':'workspace_viewer.rs:pointer_down; accessibility.rs:apply_accessibility_request','note':'Both explicit tab/close paths clear address/menu/appearance before changing focus. The unbound address edit field alone is insufficient evidence of a wrong-tile navigation on those paths.','verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2875 and stats['read_chars']==34143692 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Pelt workspace parent, accessibility and native appearance proof','files':3,'chars':305145,'newly_read_chars':304798,'verification':NAME,'basis_commit':head,'qualification':'All three files read; first347 parent characters were already credited.7 source-extracted CPU cases over actual neutral/AccessKit types with shell/driver doubles. Full original desktop tests/receipts and native execution remain pending.'})
next_read={'path':'ports/pelt/desktop/workspace_viewer/receipts.rs','start':0,'reason':'Read all remaining Pelt workspace receipt implementations and original tests, then remaining Pelt source/examples, Signalman, Tabard desktop, scripts and supplier source.'}
pending='No audit-owned process running.7 extracted-body CPU cases pass with actual neutral/AccessKit types and deterministic host fixtures.2 run-owned exe/PDB outputs removed; stable shared target reused. No full desktop/original desktop suite or native GPU/input/AT run. Full repository review remains incomplete; eight reviewed upstream commits still uninstalled.'
notes['current_area']='Workspace parent, accessibility and native appearance proof source complete; receipt drivers/tests next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'pelt_workspace_owner_probe_receipt.json'])
addition=f'''

### Pelt workspace ownership and composite semantics review, 2026-10-17

The entire239,351-character workspace parent,50,614-character accessibility owner and15,180-character native appearance proof are now read. This adds304,798 characters after the previously credited347-character parent prefix. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale or pending credit. Receipt implementations and original desktop tests remain unread except targeted contextual excerpts; the repository audit is incomplete. Shared source/report/index guards match. Local/upstream remain {head}/{upstream}; affected upstream workspace changes were reread and only concern newer committed keyboard-text forwarding. Q411/Q412 bodies and Q413 focus handler remain unchanged upstream; those commits are still uninstalled/unbuilt.

Q411: accepted concurrent legacy receipt configurations share one completion bool. A ready capability driver skips the requested interaction driver; interaction completion can skip tearout. Output flags and tearout finalization trust the same bool. Exact admission/driver guards executed with deterministic driver/event-loop doubles reproduce both shapes; pending/refusal controls distinguish them. Q412: both the secondary child helper and copied primary lowering drop supplied neutral accessibility descriptions while preserving role/name/value/actions. Exact complete lowering bodies over current real Inker/Pelt neutral types and AccessKit0.24.1 reproduce this; direct AccessKit description control retains the text. Seven source-extracted CPU checks pass in total. These are not whole desktop/original-suite/real-DOM/native/physical-AT receipts. Both exact run-owned test exe/PDB outputs were hashed and removed from the approved shared target.

Q413 is source-confirmed only: primary host Divider/Tab/Content gesture state survives Focused(false), even though core pool captures correctly retire. With an undelivered terminal release, later movement/release can continue the old chrome drag. No native focus/mouse/visual symptom was reproduced. The user-requested migration ledger remains alongside bugs: A1226-A1237 record shared engine registries with host policy, explicit hidden preflight/typed cache custody, source rollback/indeterminate rehost, and visible stopped-renderer alerts. They also record native tile/epoch cache identity versus producer/session lifetime, late backend/importer initialization and accepted secondary lane transitions as unresolved broader seams. These are documented rather than turned into speculative native findings.

The accessibility helper comment says primary and secondary share child lowering; primary actually contains a second complete loop. Both omit description. The hosts also duplicate semantic admission instead of consuming the core's complete owned publication boundary. Session/placement/revision/action checks and per-window namespaces are real controls, and the neutral revision contract must be respected before claiming a new stale-action bypass. The prior taken-fence-wait concern is narrower after caller reading: primary errors exit and accepted secondary errors stop producer composition; hidden-preflight rollback still needs a failure witness. Ordinary pointer and accessibility tab/close handlers both clear address editing, ruling out that initial retarget suspicion. Native appearance separates saved definition from explicit application activation and verifies held-session/history/aperture invariants in source; no scenario or physical font was rerun.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [CPU owner probes](../../../support/receipts/authority-success-audit/pelt_workspace_owner_probe_receipt.json). Prior report bytes and concurrent WIP are preserved. No production edits, isolated target/home/worktree or running audit process. Continue remaining receipt implementations and original desktop tests.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_pelt_workspace_review.py','pelt_workspace_owner_probe_receipt.json',*probe['evidence_sha256']]
cache={'path':'C:/t/cargo-targets/mere/build','owner':'Shared Mere build cache; audit refreshed Pelt core and neutral/AccessKit consumers','reason':'Approved stable reusable target/build-dir;2 exact workspace probe exe/PDB files removed. No isolated output or live audit process.'}
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'pelt_desktop_review_receipt.json','sha256':sha(HERE/'pelt_desktop_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,'supplier_sources':suppliers,
 'upstream_unchanged_failure_body_sha256':upstream_bodies,'new_findings':[v[0] for v in new],'authority_entries':ids,'all_pelt_inventory_source_read':False,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':cache,
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full source completion.3 complete source files305145 chars with347 previously read.7 source-extracted CPU cases with actual neutral/AccessKit contracts and deterministic shell/driver doubles. No whole desktop, original desktop suite or native/GPU/AT acceptance.2 binaries removed; no isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
