"""One-shot trainer, aggregation, identity and retirement audit checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='trainer_identity_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert not (HERE/NAME).exists()
old=load('supervisor_custody_review_receipt.json')
pre=load('trainer_review_premerge_state.json');post=load('trainer_review_postmerge_state.json')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
assert head==post['head'] and pre['head']==old['basis_commit']
assert all(sha(ROOT/n)==s for n,s in pre['dirty_sha256'].items())
prior={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
changed={n:{'before':s,'after':sha(ROOT/n)} for n,s in prior.items() if sha(ROOT/n)!=s}
assert set(changed).issubset(pre['upstream_files'])
for n in changed:
    # Git's tracked LF object and this checkout's CRLF bytes are distinct.
    # Confirm semantic Git cleanliness against the merged commit and retain
    # the exact before/after working-file hashes separately.
    assert not subprocess.check_output(['git','diff','--no-ext-diff',head,'--',n],cwd=ROOT)
preserved={n:sha(ROOT/n) for n in prior}
assert load('pending_read.json')==[]
whole=[
 'ports/distillery/src/trainer.rs','ports/distillery/src/flora.rs','ports/distillery/tests/trainer.rs',
 'ports/distillery/tests/flora_autodiff_round.rs','ports/distillery/tests/flora_social_receipt.rs',
 'ports/distillery/Cargo.toml','crates/intel/esp/src/infer/decoder/lora.rs',
 'ports/distillery/tests/mesh_host_supervised_reclaim.rs','ports/distillery/tests/record_ingress.rs',
 'ports/distillery/tests/resident.rs','ports/distillery/tests/transport_residue.rs',
 'ports/distillery/tests/mesh_host_blob_delivery.rs','ports/distillery/tests/walk_fixtures.rs',
 'ports/distillery/athanor/Cargo.toml','ports/distillery/athanor/src/lib.rs',
 'crates/eidetic/eidetic-core/src/typed.rs','crates/eidetic/eidetic-core/src/deleted.rs',
 'crates/eidetic/eidetic-core/src/manifest.rs','crates/eidetic/eidetic-core/src/schema.rs',
 'crates/domain/personal-facts/Cargo.toml','crates/domain/personal-facts/src/lib.rs',
 'crates/domain/personal-facts/src/tests.rs',
 'ports/distillery/probe/native-fixture/Cargo.toml','ports/distillery/probe/native-fixture/src/main.rs',
 'ports/distillery/probe/native-fixture/src/bin/decoder.rs',
 'ports/distillery/probe/reference-decoder.py','ports/distillery/probe/reference-fixtures.py']
inv,cov=load('source_inventory.json')['files'],load('read_coverage.json')
assert len(whole)==27
assert all(cov[n]['sha256']==inv[n]['sha256']==sha(ROOT/n) and cov[n]['through']==inv[n]['chars'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['pending_chunks']==0
probes={n:load(n) for n in ['trainer_shutdown_probe_receipt.json','typed_identity_probe_receipt.json','retirement_probe_receipt.json']}
for n,p in probes.items():
    assert p['test_exit']==0 and p['generated_output_removed']
    # The trainer's successful premerge build deliberately predates the new
    # Cargo lock/manifest. Its runtime source files must remain identical.
    assert all(sha(ROOT/k)==s for k,s in p['source_sha256'].items() if k not in ['Cargo.toml','Cargo.lock'])
assert probes['trainer_shutdown_probe_receipt.json']['tests']==2
assert probes['typed_identity_probe_receipt.json']['tests']==3
assert probes['retirement_probe_receipt.json']['checks']==2
for stem in ['trainer-shutdown','typed-identity','retirement']:
    assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-'+stem+'-probe.*'))
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q219' and authority['entries'][-1]['id']=='A491'
findings=[
 {'id':'Q220','title':'Successful mesh host shutdown can leave a real CPU trainer blocking worker alive',
  'source':'ports/distillery/src/mesh_host/host.rs:shutdown; ports/distillery/src/trainer.rs:TrainerResource::execute/run_train_job',
  'status':'native real-resource shutdown counterexample reproduced; documented, not repaired',
  'evidence':'A real CPU TrainerResource runs through a local MeshHost. The probe holds the shared artifact-store mutex, observing the actual spawn_blocking closure acquire a third store Arc. Host shutdown returns Ok while that worker still owns the Arc and remains gated before model loading. Releasing only the audit gate makes the absent synthetic model end the worker. The control drains the worker before shutdown and retains only the two expected probe/trainer owners.',
  'scope':'Two native checks; one local endpoint and synthetic memory storage. No model training, GPU work, installed authority, user store or physical remote peer. All audit-owned worker/network work drains before probe exit. The successful build was at the premerge basis recorded in the prior checkpoint; runtime source hashes remain unchanged through the authorized merge.',
  'diagnostic':'trainer_shutdown_probe_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only full-source audit','size':'shared blocking-resource drain ownership',
   'reason':'Host shutdown must own a truthful drain contract for CPU work as well as the existing GPU occupancy guard. Retain/join actual blocking work or introduce a shared resource-lifetime drain seam. Aborting the awaiting async task does not abort its started spawn_blocking closure. Do not add a second scheduler or a false idle flag.'}},
 {'id':'Q221','title':'Payload-only manifest identity aliases schemas and allows replacement metadata beneath one stable id',
  'source':'crates/eidetic/eidetic-core/src/typed.rs:save_typed_sealed; schema.rs:ManifestId; manifest.rs:save_manifest; ports/distillery/athanor/src/lib.rs immutability/idempotency claims',
  'status':'native public typed-store alias and metadata replacement reproduced; documented, not repaired',
  'evidence':'Two distinct TypedPayload schemas serialize the same valid JSON. Both saves succeed and return one id. After B saves, loading A by its previously successful id fails schema mismatch, while B loads. A separate same-schema second save replaces LocalOnly with explicitly supplied PublicPortable plus replacement timestamps/provenance under that same id. A different-byte control preserves both schema reads. save_typed hashes payload bytes only, then save_manifest unconditionally replaces manifest:<payload hash>.',
  'scope':'Three native checks against exact retained Cargo-built Eidetic artifacts and synthetic in-memory records. The second save explicitly requests the changed privacy: no unauthorized publication, cryptographic bypass, remote disclosure or real data loss is claimed. Metadata replacement itself contradicts blanket immutable-manifest/no-op claims. Current reviewed Eidetic source is unchanged through merge; unrelated postmerge dependencies have a separately pending gate.',
  'diagnostic':'typed_identity_probe_receipt.json',
  'repair_triage':{'disposition':'document architectural repair; do not divert full audit beyond one hour','size':'storage identity and compatibility decision',
   'reason':'Distinguish shared blob identity from schema-bearing artifact/manifest identity and decide how metadata history or conflicting writes are admitted. Preserve blob deduplication. Hashing or rejecting metadata differently affects existing ids, readers, provenance, privacy/seal handling and migrations; a local patch cannot establish the full contract.'}},
 {'id':'Q222','title':'Age-based retirement deletes unexpired tombstones sharing an expired tombstone node id',
  'source':'ports/distillery/athanor/src/lib.rs:propose_retirement/apply_retirement; crates/eidetic/eidetic-core/src/deleted.rs:purge_deleted',
  'status':'native exact production-module retirement counterexample reproduced; documented, not repaired',
  'evidence':'An expired tombstone and a fresh tombstone have the same node_id. propose_retirement selects the expired record but reduces its selection to that node id. apply_retirement calls purge_deleted, whose correct per-item permanent-delete contract removes all records for the id. It reports two removed and leaves the bin empty. The different-node control removes one expired record and retains the fresh record.',
  'scope':'Exact Athanor production module compiled into a native probe against retained Cargo-built dependencies, with actual public proposal/apply and Eidetic record/list APIs. This is not a new Cargo package test or resident scheduler/UI/disk/network gate. Synthetic memory-backed records only.',
  'diagnostic':'retirement_probe_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only full-source audit','size':'bounded retention selection contract',
   'reason':'Carry exact tombstone/manifest identities or an explicit accepted cutoff through retirement and delete only that accepted aged subset. Keep purge_deleted for its deliberate all-history per-node command. Include same-node old/fresh and post-proposal re-deletion controls before claiming selective retention.'}},
]
for f in findings:f['verification']=NAME
notes['findings'].extend(findings)
for c in notes['candidates']:
    if c.get('title')=='Generic host shutdown only explicitly drains detached blocking work through its composed GPU gate':
        c['qualification']='Promoted to Q220 by real CPU TrainerResource native observation; the earlier source-qualified note remains historical.'
notes['candidates'].append({'title':'URL-based content forgetting may need a shared-reference/current-policy recheck',
 'source':'ports/distillery/athanor/src/lib.rs:propose_forgetting/apply_forgetting',
 'note':'The proposal maps evictable node ids into URL keys; apply drops those URL-keyed cache entries without a graph/current-content stamp. Image-reference forgetting separately protects role/digest identity and image GC checks supplied current references. Trace whether canonical graph URL uniqueness, owner admission and proposal acceptance protect promoted/fresh users of the same cached URL before asserting a live deletion. No native content-forgetting race or duplicate-URL probe ran.'})
rows=[
 ('Shared trainer execution','Finite-difference and autodiff arms share one request enum and call the existing ESP trainers.', ['TrainerSettings','esp::train controlled functions','Eidetic model/eval artifacts'], 'Distinct explicit algorithms are not duplicate runtime authorities. Feature-disabled autodiff refuses rather than falling back. This pass performed no actual tensor training.'),
 ('CPU shutdown ownership','Resource completion must describe the lifetime of the actual blocking worker.', ['TrainerResource::execute spawn_blocking','MeshHost::shutdown','GpuAccess guard'], 'Q220 proves a real CPU worker outlives successful shutdown; the GPU guard already tracks detached hardware occupancy. Generalize the drain contract without inventing a second scheduler.'),
 ('PEFT schema duplication','FLoRA privately re-declares a subset of the ESP PEFT loader config.', ['flora::PeftLoraConfig','decoder::lora::PeftLoraConfig','PEFT_LORA_LOAD_FORMAT'], 'The loader identity is shared but config decoding/variant checks are copied. FLoRA denies unknown fields and requires exact target equality; the runtime loader permits extra config fields and validates actual model dimensions. Share a deliberate common subset without erasing stricter transform constraints.'),
 ('Aggregation versus round authority','The transformer composes artifacts while Gemot owns governed participants and weights.', ['aggregate_exact','Gemot Flora/Tulpa receipts','Mien standing events'], 'Factors/order/scaling are checked in the pure transform. The social fixture signs and accepts operations into two local stores; this is not a multi-process network or training proof.'),
 ('Training fixture duplication','Synthetic tokenizer/config/base-weight builders are repeated in training fixtures.', ['tests/trainer.rs','tests/flora_autodiff_round.rs'], 'Fixture maintenance cruft is distinct from copied production training algorithms. Both tests call the existing ESP trainer implementations.'),
 ('Native and independent controls','Rust native fixtures reuse ESP; Python references use Transformers/PyTorch.', ['native-fixture/src/main.rs','native-fixture/src/bin/decoder.rs','reference-fixtures.py','reference-decoder.py'], 'CPU-versus-browser comparison is a backend control, while Python supplies an independent numerical implementation. Reading these tools did not execute any model, install Torch or prove current native/browser parity. The native decoder label is a fixed SmolLM fixture identity rather than a derived model-content hash.'),
 ('Blob identity versus typed identity','All typed payloads share one Eidetic store but payload-only identity does not distinguish schemas.', ['save_typed_sealed','ManifestId::of_blob','save_manifest'], 'Q221 demonstrates a schema overwrite under the same blob-derived id. Preserve physical deduplication while defining artifact identity/admission; a second siloed store is not the repair.'),
 ('Manifest mutability claims','Content hash stability does not make manifest metadata immutable.', ['schema::ManifestId','manifest::save_manifest','Athanor consolidation no-op comments'], 'Repeated payload saves can replace metadata under one id. Athanor really reuses the shared compose path and suppresses already-linked proposals, but a saved id alone is not proof of immutable provenance or a write-free application.'),
 ('Retirement selection versus permanent purge','Scheduled age policy and explicit per-node permanent deletion have different selections.', ['propose_retirement','apply_retirement','purge_deleted'], 'Q222 crosses this boundary by losing per-tombstone identity. Existing tests use different node ids and therefore miss the same-node fresh-record loss.'),
 ('Image lifetime sharing','Image GC marks shared content references rather than deleting on each dropped node reference.', ['propose_image_gc','apply_image_gc','apply_image_reference_forgetting'], 'GC rechecks the supplied current snapshot; reference forgetting compares role and digest. This is source review, not proof that every asynchronous caller supplies an atomic current snapshot.'),
 ('Receipt fixture evidence levels','Original fixtures distinguish real endpoint work from authored/headless stand-ins.', ['mesh_host_blob_delivery.rs','mesh_host_supervised_reclaim.rs','walk_fixtures.rs','transport_residue.rs'], 'Network host fixtures use two local peers. Walk histories/maintenance receipts are authored, and its surface uses ScriptedDom. The residue executable has no libtest harness and gates heap canaries during released-key binding. None were rerun in this pass.'),
 ('New profile-subject admission','The personal-facts library shares one structural/transition validator; the resident supplies availability authority.', ['validate_new_profile_subjects','profile_subjects','validate_for_persona'], 'New profile ids need one positive caller lookup; historical references may survive unavailability. Full current domain lib/tests/manifest reread after merge. Remaining resident/Pandect owner changes are explicitly stale coverage awaiting reread, not qualified by domain source alone.'),
]
entries=[{'id':f'A{492+i:03d}','domain':d,'claim':c,'implementations':impl,'assessment':a,
 'state':'current source review; executed scope named in assessment','verification':NAME} for i,(d,c,impl,a) in enumerate(rows)]
authority['entries'].extend(entries)
notes['basis_commit']=head
notes['basis_transitions'].append({'from':pre['head'],'to':head,'fetched_origin_main':post['origin_main'],
 'receipt':NAME,'invalidated_read_paths_after_domain_reread':stats['changed_since_inventory'],
 'historical_findings_qualification':'Authorized merge preserved all34 dirty paths byte-for-byte. Premerge native artifacts retain their own basis. Domain personal-facts reread fully; seven earlier covered source files await reread at the new hashes.'})
notes['current_area']='Trainer/FLoRA and PEFT loader, remaining Distillery integration fixtures, Athanor, Eidetic typed identity and tombstones; refreshed personal-facts domain source'
notes['next_area']='Reread upstream-invalidated Pandect/Djinn owners and Cargo manifest; continue Distillery browser/remote/session probes'
pending={'session_id':32507,'command':'cargo --config build.build-dir="C:/t/cargo-targets/mere/build" test -p distillery --lib --features trainer,flora --locked -j 1 --target-dir C:/t/cargo-targets/mere flora:: -- --test-threads=1',
 'log':'trainer_flora_fetched_unit_tests.log','state':'network-enabled retry is still fetching newly pinned Murm/Genet sources; no terminal result yet. Poll same handle before starting another gate.',
 'earlier_attempt':'trainer_flora_current_unit_tests.log: exit101 before compilation/tests because the new Murm revision was unavailable offline.'}
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full-source objective incomplete',
 'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME],'production_edits_this_turn':False,'authorized_upstream_merge':post,
 'next_read':{'path':'crates/system/pandect/src/graph_session.rs','start':0,'reason':'Refresh owner admission after upstream main changed it, then other invalidated files and remaining Distillery probes.'},
 'diagnostic_outcomes':'Two real trainer shutdown checks, three public typed identity checks and two exact production-module retirement checks pass. All probe native outputs removed. Postmerge original FLoRA Cargo gate remains pending dependency fetch.',
 'pending_verification':pending,'coverage_note':'27 whole files read or reread; seven previously covered paths invalidated by upstream are excluded from current complete coverage.'}
doc=ROOT/old['document']['path'];original=doc.read_bytes();assert hashlib.sha256(original).hexdigest()==old['document']['sha256']
addition=f'''

### Trainer, typed identity and retirement audit, 2026-10-17

This pass reads or rereads 27 whole files ({sum(inv[n]['chars'] for n in whole):,} characters), including the complete remaining Distillery trainer/FLoRA source, its remaining integration fixtures, the shared ESP LoRA loader, Athanor, four Eidetic identity/tombstone modules, the current personal-facts domain, and native/Python model controls. Current coverage is {stats['complete_files']:,} of {stats['total_files']:,} files and {stats['read_chars']:,} of {stats['total_chars']:,} characters. Seven previously read source files changed with upstream main and are excluded until reread; the lower current count of characters reflects honest invalidation, not lost historical receipts. The full Mere source review remains active.

Upstream main f8bbd092 was fetched and integrated at {head}. All 34 prior dirty files were preserved byte for byte, with no overlap against the 26 upstream paths. Earlier source/verification receipts retain their own commit and hashes. The successful trainer library build and its native probes use the exact premerge artifact set; reviewed trainer/Eidetic runtime sources remain unchanged through the merge. The new personal-facts domain manifest, library and original tests were reread in full. New profile ids are checked through a caller-supplied resident lookup, once per new id; historical references remain independent of current availability. The remaining changed Pandect/Djinn owners still need a current full reread.

Q220: successful host shutdown does not necessarily mean its real CPU trainer worker has stopped. A native public MeshHost runs the actual TrainerResource while the probe gates its shared artifact-store mutex. The blocking closure holds a third store Arc after shutdown returns Ok. Releasing the audit gate makes the absent synthetic model end that worker before tensor training. The drained-worker control retains only the two expected owners. Both checks pass and all audit-owned work drains. This is CPU lifetime evidence, not a model-training, GPU, installed-resident or physical peer receipt. A shared drain contract must cover the real blocking work; cancelling its awaiting async task is insufficient. The previous source-qualified CPU candidate is now reproduced.

Q221: one payload hash currently identifies both blob bytes and schema-bearing mutable manifest metadata. Distinct TypedPayload schemas with identical serialized JSON receive the same successful id; the second save overwrites the schema and makes the first typed read fail. A separate explicitly requested save changes privacy and provenance under that same id. The different-byte control preserves independent schema reads. All three native checks pass. No unauthorized privacy transition, publication or real-data exposure is claimed. The problem is the blanket immutable-manifest/no-op claim and missing schema-sensitive identity/admission. Keep physical blob deduplication while deciding artifact identity and metadata history; this is a broad compatibility/migration issue to document rather than a quick architecture rewrite. Athanor really calls the shared codicil compose path, but stable payload identity does not prove immutable provenance or a write-free reapplication.

Q222: age-based retirement reduces eligible tombstones to node ids and then calls the deliberate all-history per-node purge. An expired and an unexpired record for the same node both disappear, while a different-node control retains the fresh record. Two native checks compile the exact Athanor production module against retained Cargo-built dependencies and use actual Eidetic record/list APIs. This is not a Cargo package/resident/UI/disk/network gate. Preserve exact accepted tombstone identity or cutoff through selective retirement; leave the explicit permanent per-node deletion command's broader contract intact.

Authority entries A492-A503 also record real sharing and narrower duplication. Both trainer arms call ESP and share one explicit request enum, without feature-disabled fallback. FLoRA duplicates a private subset of the PEFT loader's config decoding and checks, while the loader identity is shared; its strict transform validation and actual resolved-model dimension validation have different jobs. Consolidate a deliberate shared subset without treating every stricter check as a bug. Training fixtures repeat tiny model/tokenizer builders but do not contain independent training engines. Gemot governs round membership/weights, while Distillery transforms supplied artifacts. Signed social fixture operations accepted into two local stores do not demonstrate a multi-process network or actual training. Rust native numerical controls reuse ESP; Python references supply independent Transformers/PyTorch calculations. Reading either is not executing parity. The walk fixture uses authored boards/maintenance receipts and a headless DOM, while host delivery/reclaim fixtures really use two local endpoints.

Image reference forgetting checks role/digest and shared image GC rechecks a supplied snapshot. URL-based cache forgetting may need an equivalent current-reference or accepted-content guard, but graph uniqueness and host acceptance contracts must be traced first; no such deletion race was injected. This remains a candidate. No production or original tests were edited by the audit. No isolated target, Cargo home or worktree was created. All three native probes' executables/PDB/import artifacts were removed after their bounded observations.

The postmerge original FLoRA unit gate first exited101 before compilation/tests because the new Murm dependency revision was unavailable offline. Its network-enabled retry is session32507, still fetching newly pinned dependencies; poll that same handle and record its terminal result before starting another retry. Do not cite the live log as a completed test receipt. The [trainer/identity checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds coverage, the merge guard, seven native observations/controls, and the remaining verification boundary.
'''
doc.write_bytes(original+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'trainer_identity_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['record_trainer_identity_review.py','trainer_review_premerge_state.json','trainer_review_postmerge_state.json',
 'trainer_shutdown_probe.rs','run_trainer_shutdown_probe.py','trainer_shutdown_probe_compile.log','trainer_shutdown_probe_output.log','trainer_shutdown_probe_receipt.json',
 'typed_identity_probe.rs','run_typed_identity_probe.py','typed_identity_probe_compile.log','typed_identity_probe_output.log','typed_identity_probe_receipt.json',
 'retirement_probe.rs','run_retirement_probe.py','retirement_probe_compile.log','retirement_probe_output.log','retirement_probe_receipt.json',
 'trainer_flora_probe_build.jsonl','trainer_flora_probe_build_stderr.log','trainer_flora_current_unit_tests.log','trainer_identity_diff_check.txt']
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Whole trainer/FLoRA/Athanor/Eidetic review plus authorized upstream merge; full objective incomplete',
 'production_edits_this_pass':False,'authorized_upstream_merge':post,'prior_receipt':{'path':'supervisor_custody_review_receipt.json','sha256':sha(HERE/'supervisor_custody_review_receipt.json')},
 'coverage':stats,'whole_current_files':whole,'whole_current_chars':sum(inv[n]['chars'] for n in whole),
 'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':preserved,'prior_guard_paths_changed_only_by_committed_upstream':changed,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'native_observations_and_controls':7,'native_probe_exit':0,'native_probe_sources':'trainer_shutdown and typed_identity reuse exact premerge Cargo-built libraries; retirement additionally compiles the exact production Athanor module directly',
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(original),'preserved_prefix_sha256':hashlib.sha256(original).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'pending_verification':pending,'next_read':notes['checkpoint']['next_read'],
 'probe_outputs_removed':True,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Full source review incomplete. Seven upstream-invalidated source hashes await reread. No training/GPU/headed/installed/physical trial; no real privacy disclosure. Original postmerge feature gate pending dependency fetch.'}
save(NAME,receipt)
print(json.dumps({'receipt':NAME,'coverage':stats,'whole_files':len(whole),'whole_chars':receipt['whole_current_chars'],
 'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'guard_paths':len(preserved),'retained_bytes':budget},indent=2))
