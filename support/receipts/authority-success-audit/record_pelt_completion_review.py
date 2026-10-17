"""Checkpoint the complete inventoried Pelt source review and CLI owner findings."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='pelt_completion_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('pelt_receipt_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
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
pelt=[p for p in i if p.startswith('ports/pelt/')]
assert len(pelt)==127 and sum(i[p]['chars'] for p in pelt)==1646337
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in pelt)
paths=[p for p in pelt if p not in guards]
assert len(paths)==72 and sum(i[p]['chars'] for p in paths)==214666
sources={p:sha(ROOT/p) for p in paths};assert load('pending_read.json')==[]
probe=load('pelt_cli_dispatch_probe_receipt.json');assert probe['exit']==0 and probe['passed']==9
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==2 and all(not Path(p['path']).exists() for p in probe['removed_outputs'])
assert not (HERE/'pelt_cli_dispatch_must_not_exist.png').exists()
for p in [
 Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/7f13b85/components/genet-scripted/document.rs'),
 Path('C:/Users/mark_/.cargo/git/checkouts/genet-0bbbe1b4c8511eb3/7f13b85/components/shared/document-session-api/src/engine_ids.rs')]:
    external[str(p.resolve())]=sha(p)
viewer=ROOT/'ports/pelt/viewer.rs'
assert subprocess.check_output(['git','show',f'{upstream}:ports/pelt/viewer.rs'],cwd=ROOT,text=True)==viewer.read_text(encoding='utf-8-sig')
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q414' and authority['entries'][-1]['id']=='A1245'
new=[
 ('Q415','Early Pelt smoke dispatch silently bypasses another requested receipt and its validation',
  'ports/pelt/viewer.rs:269-308,341-389,400-415',
  'All arguments are collected before smoke dispatch, but successful smoke handlers return before workspace required-artifact/feature validation or named receipt execution. With an available successful smoke runner, article+artifact+netrender or appearance+artifact+netrender completes only the smoke; mixed+netrender can bypass even the missing artifact/Smolweb requirements. Two requested smoke modes run only the first. Nine child-process probes execute the exact parser/early-dispatch prefix and helpers with deterministic capability/smoke doubles: these cases exit0 without the PNG or owner dispatch, while the earlier product-artifact validation correctly exits2 and the no-smoke control reaches later dispatch. Source is unchanged at origin/main.',
  'Reject incompatible requested run modes before invoking any handler, or explicitly aggregate each requested outcome. Keep a successful selected smoke distinct from completing a named artifact receipt. This finding is conditional on the selected smoke being available and succeeding; a missing-feature/error smoke still exits nonzero.',
  'Source-extracted CLI prefix executed in real child processes with host/engine-capability/smoke doubles. No actual desktop binary, smoke/GPU, named fixture execution or real application artifact was produced.'),
 ('Q416','Workspace JS selection silently maps invalid backend values to Boa',
  'ports/pelt/viewer.rs:584-609,942-955; ports/pelt/desktop/workspace_viewer.rs:803-829; current Genet scripted/document.rs:1949-1957',
  'The CLI retains --js as a string. The single-document scripted path calls ScriptedEngine::parse and rejects invalid values; that actual supplier parser is case-insensitive. The workspace branch compares only js_engine == nova and selects the Boa registry ID for everything else. The exact branch probe returns genet.scripted for misspelled-backend, matching boa, while nova control returns genet.scripted.nova. Source also shows valid case variants such as Nova take the Boa branch despite being accepted as Vano/Nova by the single-document parser; this case variant was source-traced, not separately executed. Registered engine construction binds those IDs to Boa and the Vano compatibility backend. The CLI file is unchanged upstream.',
  'Parse the backend once through the existing declared parser and use that typed choice plus the same feature-availability check in both host paths. Reject an unknown requested backend rather than selecting an apparently successful fallback.',
  'Three source-extracted branch-selection child processes, with verified engine-ID constants and declared feature cfgs; no Boa/Vano runtime or complete scripted workspace was built or executed. The remaining six probes cover Q415 dispatch controls.')]
for ident,title,source,evidence,repair,scope in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'reproduced exact source-extracted CLI behavior; documented, not repaired',
      'scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['pelt_cli_dispatch_probe_receipt.json']})
rows=[
 ('Standalone CLI still chooses owners before the shared routing boundary','Pelt root entrypoint selects single-document, workspace, protocol or smoke paths before host construction.',
  ['ports/pelt/viewer.rs','ports/pelt/lib.rs','ports/pelt/main.rs'],
  'This is a concrete reference-host policy layer above shared Pelt core and registries. Q415 shows it lacks one authoritative run-mode admission rule. Library validations exist but cannot act if an earlier CLI return bypasses their owner. Windows attaches the parent console under a GUI subsystem; logging and process status remain separate from artifact completion.'),
 ('Backend selection has two independent parser authorities','Single-document scripted selection uses the shared ScriptedEngine parser; workspace uses a raw string branch.',
  ['ports/pelt/viewer.rs','ports/pelt/desktop/scripted_viewer.rs','ports/pelt/desktop/workspace_viewer.rs'],
  'Q416 proves silent unknown-value selection and source-traces case inconsistency. Boa and Vano engine construction already reuse shared engines; the duplication is selection/validation policy, not two JS implementations. Existing parser/size-matrix tests do not compose both backend-selection paths.'),
 ('Protocol dispatch retains a local compatibility selector beside shared routing','The bare Livery CLI checks a lowercase starts_with scheme list to enter direct Smolweb viewer; workspace uses Inker policy.',
  ['ports/pelt/viewer.rs','ports/pelt/desktop/smolweb_glue.rs'],
  'The CLI comment still says P4 will move this choice into routing although workspace routing already exists. Scheme spelling/case/availability and Reader held-source paths need one consistent owner. No uppercase-protocol navigation or real transport was executed, so this remains a migration followup rather than an added protocol-failure finding.'),
 ('Receipt fixture identities and forced responses are explicit test inputs','Fixtures encode known loading/error, native-ready, resource graph, retained editor and Reader limitations.',
  [p for p in paths if p.startswith('ports/pelt/examples/')],
  'Source fully read: redirected fixture is explicitly supplied by a receipt fetcher, WebView2 ready message is gated after script/animation frames, Scripted uses parser mutation/timer/cancelled click controls, and Reader destination text says host-owned. Fixture prose alone is not runtime success. Resource paths/scene values were read, not rendered or refetched.'),
 ('Retained scene baselines are structural expectations, not new visual receipts','Tracked reftests pair HTML/CSS and scroll sidecars with Debug-form Scene expectations.',
  [p for p in paths if p.startswith('ports/pelt/reftests/')],
  'All scene glyphs/colors/clips/transforms/font placeholders were read, including fixed counter-translation, scroll-overhang clamp, root overflow, percent-height, checked and legacy clips. Font blob IDs are placeholders in these snapshots. No current comparison/raster test was run, and no snapshot was regenerated. Pelt CLI exposes no reftest command here; repository-wide consumer/liveness tracing remains a followup before calling assets obsolete.'),
 ('Profile output and successful optional fields remain weaker than requested completion','Single-document wrappers print an Ok outcome and only print product receipt data when present.',
  ['ports/pelt/viewer.rs','ports/pelt/desktop/static_viewer.rs'],
  'This is the actual CLI consumer of Q408: it does not independently reject an absent requested receipt after an Ok library outcome. Workspace library has stronger named finalization, though Q411 and Q414 retain distinct completion-state faults. CLI smoke priority is Q415, not evidence that every ordinary mode falsely succeeds.'),
 ('Appearance storage stays application-owned at the root caller','Ordinary production workspace supplies durable application choice and shared definitions; named receipts use injected/session state.',
  ['ports/pelt/viewer.rs','ports/pelt/desktop/appearance.rs'],
  'Root preserves the application store path even when loading fails, collects visible storage notices, and separately selects the theme library. Explicit caller paths take priority. This is real owner mapping, not merely a claim of one storage object; the prior shared Workshop reopen protection issue still applies to the editor path.'),
 ('Pelt reading coverage is complete while native acceptance and migrations remain open','All127 inventoried current-local Pelt source/manifests/examples/scenarios/scene expectations are fully read.',
  ['ports/pelt/'],
  '1,646,337 characters total. Prior core CPU tests and current extracted-body probes qualify their exact scopes only; no full desktop/native/GPU/physical input/AT run is claimed. Upstream Pelt request/committed-text changes were previously reviewed as Git objects and remain uninstalled/unbuilt. Compatibility feature names, stale reference-host prose and siloed host policy remain tracked; full repository audit continues.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1246):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'complete current-local Pelt source read; native migration/acceptance boundaries explicit','verification':NAME})
notes['review_followups'].extend([
 {'area':'Protocol dispatch and old scene consumer ownership','detail':'Finish local starts_with protocol selector migration against shared routing, including case/availability and held Reader source. Trace repository/external consumers of retained Debug Scene baselines before labelling them obsolete. Source read is complete for Pelt; no protocol/raster test ran.','verification':NAME},
 {'area':'CLI run-mode completion','detail':'Q415 early smoke dispatch and Q416 separate backend parsers are bounded admission fixes, but remain documented in this read-only goal. Q408/Q411/Q414 are separate startup/multi-driver/capture completion gaps. Keep run-mode validation before side effects and final success tied to the requested owner outcome.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2953 and stats['read_chars']==34652945 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Remaining Pelt examples, structural Scene reftests/scroll sidecars and root lib/main/viewer','files':72,'chars':214666,'verification':NAME,'basis_commit':head,'qualification':'Completes all127 inventoried Pelt source1646337 chars.9 source-extracted CLI/branch child-process cases pass with deterministic capability/host doubles; no real JS backend/desktop/GPU/native/AT run. Failed probe-wrapper compile was corrected without production changes and retained separately.'})
next_read={'path':'ports/signalman/Cargo.toml','start':0,'reason':'Continue all Signalman source, then Tabard desktop, scripts and supplier patches/source; preserve all remaining inventory and unfinished ownership migrations.'}
pending='No audit-owned process running.9 source-extracted child-process checks pass;2 exact run-owned exe/PDB removed. All127 current-local Pelt files read; full repository remains incomplete. No JS/native/GPU/input/AT acceptance. Main/upstream unchanged, eight reviewed commits still uninstalled because shared report overlaps.'
notes['current_area']='All current-local Pelt inventory source read; Signalman next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'pelt_cli_dispatch_probe_receipt.json'])
addition=f'''

### Pelt source completion and command-line owner review, 2026-10-17

All127 inventoried current-local Pelt files are now completely read (1,646,337 characters). This pass adds the remaining72 example/HTML/CSS/JavaScript fixtures, structural Scene/scroll expectations and root lib/main/viewer sources:214,666 characters. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale/pending credit. The repository audit remains incomplete. Local/upstream remain {head}/{upstream}. The entire CLI file is unchanged upstream; previously reviewed Pelt request/committed-text changes in other files remain uninstalled/unbuilt because the shared report overlaps.

Q415: smoke handlers return before another requested receipt runs and before workspace artifact/feature admission. With an available successful smoke, named product/workspace modes can be skipped; two requested smokes run only the first. Q416: workspace JS selection uses js_engine==nova and silently maps every other value to Boa, unlike the shared single-document parser, which rejects unknown values and accepts case variants. Nine actual child processes execute exact source-extracted argument/early-dispatch/parser or workspace-choice bodies with deterministic engine-capability/host/smoke doubles. They demonstrate skipped owner dispatch and absent expected PNG, the correct earlier product-artifact refusal, later-dispatch reachability without smoke, and unknown/Boa/Vano-compatibility selection controls. Neither actual desktop nor JS engine ran. The initial probe-wrapper compile used an incorrect Option<String> return annotation, was corrected to its actual borrowed Option type, and its failed log is retained. No executable production body was changed. Both exact run-owned exe/PDB outputs were hashed and removed.

A1246-A1253 record the remaining host-policy authorities. Root run-mode and backend parsing are not yet unified; bare protocol dispatch still uses a local lowercase prefix list beside workspace routing, with stale P4 migration prose. That is a followup rather than an unexecuted uppercase-protocol failure claim. Ordinary application appearance paths remain caller-owned with explicit errors and separate choice/library storage. Actual CLI wrappers consume Ok without requiring an optional requested product receipt, making the Q408 library weakness relevant there. Fixtures explicitly distinguish forced redirected source, native-ready signalling, script timer/cancel controls, held Reader source and engine limits. All tracked Scene expectations were read, but not regenerated, compared or rasterized; placeholder font blobs and structural transforms are not current physical font/rendering evidence. Their external consumer/liveness remains to be traced before calling them obsolete.

The complete Pelt read does not close its native/ownership migrations. Prior actual core tests and this pass's extracted-body probes retain their exact scopes. No full desktop/original desktop suite, real native/GPU/physical input/AT or Boa/Vano execution was performed. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [CLI dispatch probes](../../../support/receipts/authority-success-audit/pelt_cli_dispatch_probe_receipt.json). Prior report bytes, source/shared WIP and index are preserved; no production edits, isolated target/home/worktree or live audit process. Continue Signalman source.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_pelt_completion_review.py','pelt_cli_dispatch_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'pelt_receipt_review_receipt.json','sha256':sha(HERE/'pelt_receipt_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[v[0] for v in new],'authority_entries':ids,'all_pelt_inventory_source_read':True,'pelt_files':127,'pelt_chars':1646337,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.72 new files214666 chars complete all127 current-local Pelt source1646337 chars.9 source-extracted CLI/branch child processes with deterministic host doubles; no actual desktop/JS/GPU/native/AT acceptance.2 binaries removed; no isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
