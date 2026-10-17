"""Checkpoint executable design/receipt source review without changing production."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='design_receipt_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('tabard_ownership_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
aux=dict(old['auxiliary_read_sha256']);runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
inventory=load('source_inventory.json')['files'];coverage=load('read_coverage.json')
paths=[p for p in inventory if p.startswith('design_docs/') and p not in guards]
assert len(paths)==47 and sum(inventory[p]['chars'] for p in paths)==358536
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():
    assert h==inventory[p]['sha256']==coverage[p]['sha256']
    assert coverage[p]['through']==inventory[p]['chars']==len((ROOT/p).read_bytes().decode('utf-8-sig',errors='replace'))
assert load('pending_read.json')==[]
wrapper=load('receipt_wrapper_probe_receipt.json');seal=load('source_seal_probe_receipt.json')
for probe in [wrapper,seal]:
    assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
    assert all(sha(HERE/p)==h for p,h in probe['fixture_sha256'].items())
for p,h in wrapper['preserved_historical_receipt_sha256'].items():
    assert sha(ROOT/p)==h;aux[p]=h
for p in [
 'design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md',
 'design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/2026-09-08_custody_backend_receipt.md']:
    aux[p]=sha(ROOT/p)
assert wrapper['results'][1]['recorded_child_exit']==101 and wrapper['results'][1]['outer_process_exit']==0
assert wrapper['results'][3]['supplier_packages_per_root']==[0]*5 and wrapper['results'][3]['accepted_roots']==5
assert wrapper['results'][4]['outer_process_exit']==1
assert seal['cases']['css_changed_sealed'] and seal['cases']['fingerprint_unchanged'] and seal['cases']['rust_change_refused_exit']==1
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q393' and authority['entries'][-1]['id']=='A1177'
new=[
 ('Q394','Historical custody probe runner exits successfully when Cargo fails',
  'design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/run.py:12-17',
  'The unchanged runner copy writes injected child exit 101 into run.json, then exits zero itself. Success control records and returns zero. The historical actual run.json records zero and its log reports three passing tests.',
  'Propagate child returncode after preserving diagnostics. This is a research receipt runner, not a production custody service; the recorded historical successful run is not invalidated.',
  'receipt_wrapper_probe_receipt.json'),
 ('Q395','Supplier qualifier accepts roots containing zero recognized suppliers',
  'design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/qualify_final_graphs.py:15-32',
  'With registry-only netrender metadata, all five roots are accepted with empty supplier lists. Matched Git suppliers pass; an explicitly wrong Genet revision refuses. Empty names makes universal revision/duplicate checks vacuously true. Published historical supplier counts are 45,27,10,27,23, not empty.',
  'Declare required supplier families/packages per manifest root, then validate their sources/revisions and duplicates. Do not require all four families blindly where a root legitimately omits one. Controlled metadata injection is not a new Cargo resolution and does not overturn existing nonempty receipts.',
  'receipt_wrapper_probe_receipt.json'),
 ('Q396','Browser source seal omits compiled shared CSS from its fingerprint',
  'design_docs/mere_docs/testing/receipts/2026-10-11_classic_browser_gates/identity.py:25-30,51; crates/cambium/cambium/src/workspace_chrome.rs:20',
  'Copied actual workspace_chrome.rs includes instrument.css. After preflight, changing only the copied CSS leaves the snapshot fingerprint unchanged and whole unchanged seal CLI accepts sealed=true. Changing copied Rust refuses. Filter includes Rust/Cargo/scenarios and web-only JS/HTML/CSS, omitting this shared compilation input.',
  'Track all relevant compiler inputs, including include_str/include_bytes resources, shader/WIT/style inputs and build-time assets, or seal an explicit complete input manifest. Fixture package/log/metadata are synthetic; no real build/browser/GPU ran and no actual published stale package is asserted.',
  'source_seal_probe_receipt.json')]
for ident,title,source,evidence,repair,probe in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,
      'status':'reproduced unchanged copied verifier behavior; documented, not repaired',
      'scope':'Audit-owned fixtures and real outer Python processes; child Cargo/Git metadata boundaries explicitly mocked. Historical receipt bytes preserved.',
      'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':[probe]})
rows=[
 ('Static component catalogues are snapshots, not live host acceptance','Narrow and regular HTML share generated CSS and structure; states and custom leaf slots are rendered examples.',
  ['design_docs/cambium_docs/testing/receipts/component_catalog_narrow.html','design_docs/cambium_docs/testing/receipts/component_catalog_regular.html'],
  'Both full HTML/CSS sources read. No embedded native callback implementation or runtime acceptance is established by their presence. Duplicated snapshot markup is not itself a second production state authority.'),
 ('Research probes and current custody ownership must be separated','Arena, identity and wake models bound historical contract experiments rather than a current shared runtime.',
  ['design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/arena/probe/src/lib.rs','design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/identity/identity_custody_probe.py','design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/execution/r1_drive_wake_model.ps1'],
  'Arena pinned candidate uses observer on/off, foreign arena and stale-ID controls; identity Python explicitly substitutes SHA256; PowerShell wake model demonstrates revision/register/recheck. None is current Genet or physical wake proof. Preserve historical source identity rather than silently promoting model outputs.'),
 ('Historical custody adapter is superseded, and its wrapper owns exit status','Research serialized Muniment/reopen tests and current backend transaction boundary are distinct.',
  ['design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/src/lib.rs','design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/run.py'],
  'Canonical receipt corrected 2026-10-06 explicitly records Backend::transact landing 9ba9f790 on 2026-09-09. Do not repeat the historical missing-transaction conclusion as current. Q394 is wrapper failure propagation; historical child success remains real recorded evidence.'),
 ('Resource resolution probes exercise bounded adapters, not a complete media product','Ranged HTTP, cache policy, held Iroh readers and synthetic cost probes have separate authorities.',
  ['design_docs/mere_docs/testing/receipts/2026-09-20_resource_resolution_probes/ranged/src/lib.rs','design_docs/mere_docs/testing/receipts/2026-09-20_resource_resolution_probes/held/src/lib.rs','design_docs/mere_docs/testing/receipts/2026-09-20_resource_resolution_probes/cost/lock_delta.py'],
  'Pinned historical NoStore range remedy, IfRange/status/finalURL/persona controls and Iroh blocking-reader bridge are actual probe code. Same-server 3MB/300-read comparisons are not Internet/TLS performance. Lock delta keys omit source and resolve some ambiguous dependency names by first match; this is a snapshot cost aid, not canonical current source identity.'),
 ('Image and Vello preview probes state different acceptance boundaries','Atlas diagnostic captures and CPU fragment expansion are not identical to GPU scene acceptance.',
  ['design_docs/cambium_docs/testing/receipts/native_image_atlas_20261010/original-control/tabard-patchless-atlas-repro.rs','design_docs/mere_docs/testing/receipts/2026-10-11_vello_releases/probe/src/main.rs'],
  'Atlas emits captures and changed-color diagnostics without a complete pixel acceptance assertion. Vello uses production CPU preview and direct versus SparseFragmentRegistry-expanded equality; hybrid source admission and snapshot image CPU composition do not execute a GPU. No historical probe rerun here.'),
 ('Browser identity seal and browser behavior qualify different evidence','Source/package hashes, local serving assumptions and actual producer pixels are separate checks.',
  ['design_docs/mere_docs/testing/receipts/2026-10-11_classic_browser_gates/identity.py','design_docs/mere_docs/testing/receipts/2026-10-11_classic_browser_gates/qualify.cjs'],
  'Q396 proves an omitted compiled CSS input. Browser qualifier checks local package hashes and HTTP HEAD lengths, relying on isolated server mapping instead of hashing served bytes. Hardware fallback is explicitly limited. Producer geometry/idle accent checks avoid UI-only nonblank passes, protocol/mode/rebind scenarios are substantial, and catches return failure. None was freshly browser-run here.'),
 ('Supplier identity checks must require presence as well as agreement','Git supplier revisions and duplicate-name checks cannot prove a source family absent from their census.',
  ['design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/qualify_final_graphs.py','design_docs/mere_docs/testing/receipts/2026-10-15_netrender_adoption/web-smoke-resolved-lock.toml'],
  'Q395 proves empty-census acceptance. Historical actual lists are nonempty. Resolved lock records historical Genet7ffe/Netrenderf4f9/Vello276b and wgpu30, with unused sonic patch distinct from resolution. It is not a fresh current root compile or proof of required-family presence in every root.'),
 ('Negative controls must not acquire shared production edit ownership','Coupling qualifier commands propagate subprocess failure, but mutating controls temporarily replace shared source.',
  ['design_docs/mere_docs/testing/receipts/2026-10-12_coupling_effects/runtime/negative_controls.py','design_docs/mere_docs/testing/receipts/2026-10-12_coupling_effects/runtime/audit_docs.py','design_docs/mere_docs/testing/receipts/2026-10-12_coupling_effects/trace/audit_docs.py'],
  'Mutators restore exact hashes in finally, but concurrent observers and forced termination remain risks; none run in place here. Test FAILED/name assertions are meaningful bounds, not all-runtime acceptance. Historical docs bytes evaluated by a current dynamically imported auditor are a comparison, not a frozen historical auditor or full source review.'),
 ('Archived SSH generation patch is provenance, not current integration authority','Custody-generated public keys and preserved master connection controls are historical patch source.',
  ['design_docs/mere_docs/testing/receipts/djinn-ssh-peers/custody-generation.patch'],
  'Patch uses old ports/castellan paths while current source is ports/djinn/castellan. Read entire patch and controls; do not infer current integration or rerun operations against real custody/peers from its existence.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1178):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'source read; historical versus current execution explicitly bounded; full audit ongoing','verification':NAME})
for area,detail in [
 ('Receipt source identity migration','Q395 requires per-root required supplier identities; Q396 requires complete compiler-input source census. Source agreements over an incomplete set cannot establish a single authoritative implementation.'),
 ('Receipt rerun isolation','Historical qualification source may overwrite receipts, use obsolete target/worktree paths or temporarily edit production. Run copies with explicit fixtures; preserve cited evidence and concurrent work. No historical qualification rerun in place.'),
 ('Browser serving identity followup','HEAD length plus local package hashes assumes isolated server mapping. Review served-byte binding separately before claiming a same-length substituted served package is rejected; no actual wrong-served package observed.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2796 and stats['read_chars']==32594553 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
assert all(coverage[p].get('sha256')==v['sha256'] and coverage[p].get('through')==v['chars'] for p,v in inventory.items() if p.startswith('design_docs/'))
notes['completed_source_areas'].append({'prefix':'design_docs/ remaining executable receipt source','files':47,'chars':358536,'verification':NAME,'basis_commit':head,'qualification':'All inventoried design_docs source now read; Markdown only targeted context, not a claim of full document reading. Five wrapper cases and three source seal CLI cases use audit-only copies and injected boundaries.'})
next_read={'path':'ports/djinn/tests','start':0,'reason':'Continue remaining ports, scripts and supplier patch source without narrowing inventory.'}
notes['current_area']='All inventoried executable design/receipt source read; continue ports'
notes['next_area']=next_read['reason']
pending='No audit-owned process running. Production/shared WIP/index preserved. Local main 7c718105 and origin 00e44428 remain unchanged; eight reviewed upstream commits are not installed because shared report overlaps. Five wrapper cases and three seal cases passed their behavior assertions; injected metadata/build inputs are not real Cargo/browser acceptance. Full source review remains incomplete.'
notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending})
notes['checkpoint']['new_receipts'].extend([NAME,'receipt_wrapper_probe_receipt.json','source_seal_probe_receipt.json'])
addition=f'''

### Executable receipt source and verifier authority review, 2026-10-17

Read the remaining 47 source/manifest/style/patch files under design_docs (358,536 characters), including both complete HTML catalogues and the full historical resolved lock. All inventoried source under design_docs is now read. Markdown was consulted for specific historical/current boundaries, not credited as a complete document review. Overall coverage: {stats['complete_files']:,}/{stats['total_files']:,} files, {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale or pending credit. Remaining ports, scripts and vendor patches stay in the denominator. Main remains {head}; origin remains {upstream}. The reviewed eight-commit upstream advance remains uninstalled because of the shared report overlap.

Q394: the historical custody runner records Cargo exit 101 but exits zero itself. Q395: the supplier qualifier accepts all five roots with zero recognized suppliers from a registry-only fixture; its revision and duplicate-source checks only cover names already recognized. Matched supplier and wrong-revision controls pass/refuse respectively. Historical actual custody success and nonempty supplier lists remain byte-preserved and are not overturned. Q396: the browser identity filter excludes shared CSS actually compiled through include_str!. A copied real Cambium CSS input changed after preflight, but the complete unchanged seal CLI accepted sealed=true with an identical fingerprint; a Rust-source change refused. No actual stale published product package is asserted.

These are real outer Python process and file-write outcomes over exact copied verifier source. Child Cargo/Git outputs and fixture package/build-log inputs are explicitly synthetic. Five wrapper cases and three source seal CLI cases completed with their expected controls. No Cargo compilation, browser, GPU, network or production mutation was performed. Two initial harness errors were preserved separately: stdout type mismatch in the first wrapper injection, and Windows key separators in the first seal assertion. Neither is credited as a product failure.

A1178-A1186 retain ownership distinctions: generated component catalogues versus live callback behavior; pinned research arena/identity/wake models versus current runtime; historical custody adapter versus subsequently landed Backend::transact; bounded range/cache/Iroh probes versus media playback; CPU preview equality versus GPU execution; source and served package identity versus producer pixel behavior; and archived patch provenance versus current integration. Current custody documentation already corrects the historical missing-transaction conclusion. The browser qualifier has real producer geometry/idle controls and an honest failure path, but its HEAD-length serving check depends on isolated server mapping. Source sealing and supplier census need complete input membership before agreement can support an authority claim. Coupling mutators temporarily edit shared source and were only read, never invoked here.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [wrapper cases](../../../support/receipts/authority-success-audit/receipt_wrapper_probe_receipt.json), [source seal cases](../../../support/receipts/authority-success-audit/source_seal_probe_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. Evidence remains below the 100 MB cap. No new worktree, target or Cargo home. Full audit continues.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_design_receipt_review.py','receipt_wrapper_probe.py','receipt_wrapper_probe_initial.py','receipt_wrapper_probe_receipt.json','source_seal_probe.py','source_seal_probe_initial.py','source_seal_probe_receipt.json',*wrapper['fixture_sha256'],*seal['fixture_sha256'],*seal['initial_harness_failure']['fixture_sha256']]
evidence.extend(p.relative_to(HERE).as_posix() for p in (HERE/'receipt-wrapper-fixtures').rglob('*') if p.is_file())
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,
 'prior_receipt':{'path':'tabard_ownership_review_receipt.json','sha256':sha(HERE/'tabard_ownership_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,
 'new_findings':[r[0] for r in new],'authority_entries':ids,'full_repository_source_read':False,'all_design_docs_inventory_source_read':True,
 'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],
 'shared_generated_cache':old['shared_generated_cache'],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'qualification':'Progress, not full repository completion.47 source files read; eight copied CLI cases with explicit injected boundaries. Historical evidence preserved. No production edits, real Cargo/build/browser/network/GPU execution or new isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
