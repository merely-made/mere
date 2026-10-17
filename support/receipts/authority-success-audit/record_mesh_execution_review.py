"""One-shot complete-Mesh reading checkpoint; preserve production and prior evidence."""
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
assert not (HERE/'mesh_execution_review_receipt.json').exists()
old=load('current_main_review_receipt.json');probe=load('mesh_execution_probe_receipt.json')
assert probe['tests']==7 and probe['test_exit']==0 and probe['generated_output_removed']
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
assert '3 passed; 0 failed' in (HERE/'mesh_execution_lease_tests.log').read_text(encoding='utf-8-sig')
assert '124 passed; 0 failed' in (HERE/'mesh_execution_admitted_lib_tests.log').read_text(encoding='utf-8-sig')
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
inv,cov=load('source_inventory.json')['files'],load('read_coverage.json')
mesh=[p for p in inv if p.startswith('crates/mesh/mesh/')]
assert len(mesh)==43 and all(cov.get(p,{}).get('sha256')==inv[p]['sha256'] and cov[p]['through']==inv[p]['chars'] for p in mesh)
assert sum(inv[p]['chars'] for p in mesh)==556938
new=sorted(p for p in mesh if p not in preserved)+['Cargo.toml']
assert len(new)==21 and sum(inv[p]['chars'] for p in new)==285338
assert cov['Cargo.toml']['sha256']==inv['Cargo.toml']['sha256'] and cov['Cargo.toml']['through']==inv['Cargo.toml']['chars']
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1194 and stats['read_chars']==15706387 and len(stats['changed_since_inventory'])==12
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,'status':'native public-API observation against unchanged source; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'mesh_execution_review_receipt.json','diagnostic':'mesh_execution_probe_receipt.json',
  'repair_triage':{'disposition':'documented during read-only full-source continuation','size':size,'reason':repair}}
findings=[
 finding(203,'Legacy worker selection bypasses the configured resource allowlist',
  'crates/mesh/mesh/src/worker.rs:HostOffer::can_run legacy branch',
  'With allowed_resources containing only mesh.echo/v1, an M1 Blake3 post is still considered runnable, selected for Claim and then Execute. The equivalent V2 Blake3 spec correctly returns Idle. The legacy branch checks only payload presence, registration and adapter hardware requirements; V2 calls DevicePolicy::accepts.',
  'Pure worker/board public APIs, valid synthetic signed operations and built-in registry. No actual excluded workload, host daemon or remote peer executed. This is a concrete configuration bypass, not a claim of untrusted remote GPU execution.',
  'bounded common legacy/V2 policy admission correction',
  'Apply the configured resource rule after mapping the legacy kind to its resource. Keep any implied legacy checkpoint-class rule explicit. Verify both generations under the same policy, preserving the single actual run_legacy -> run_job executor.'),
 finding(204,'The worker starts held work before checking owner withholding or capacity',
  'crates/mesh/mesh/src/worker.rs:next_action stages1-3 and later offer.accepting gate',
  'A valid live lease held by this author but absent from HostOffer.running yields Execute while conservative policy reports ForegroundActivity and accepting=false. The same held-but-unstarted job yields Execute when max_concurrent_jobs=0. When the job is placed in running, the owner-withholding control correctly yields Reclaim. Reclaim only checks running jobs and the start gate occurs after the Execute loop.',
  'Pure public decision function with an actual folded lease. The probe establishes a start decision contrary to current settings; no native OS supervision, physical workload continuation or broader host race was executed.',
  'bounded scheduling gate correction with a held-versus-running distinction',
  'Separate keeping an already running job from starting a held reservation. Owner withholding must prevent a start; capacity should prevent an additional run without reclaiming work already active. Keep heartbeats and reservation release policy deliberate rather than suppressing all lease maintenance.'),
 finding(205,'Cancellation during the final real-adapter await still commits output',
  'crates/mesh/mesh/src/resources/delayed.rs:execute final sleep; registry.rs:run_job_inner post-execute commit',
  'The real DelayedResource(1,100) is manually polled to its first Pending after reporting done=1. The host cancels before the next poll, and JobControl confirms Cancel. Awaiting the run nevertheless succeeds and stores the expected output. The adapter has no cooperative point after its final sleep; the runner has no final control check before commit. Existing cancelled-run tests only cancel before execution.',
  'Deterministic cancellation ordering through the real built-in adapter and public runner, synthetic memory blobs. No malformed adapter or timing race is required: cancellation precedes execute returning. A stronger atomic cancellation-versus-sink-commit guarantee remains a separate design question.',
  'bounded final cooperative check; full commit linearization can exceed one-hour diversion',
  'Check control at the final cooperative boundary and before handing returned bytes to the sink. Define cancellation arriving during an in-flight sink write separately; do not promise rollback of a committed blob merely by adding one flag read.'),
 finding(206,'Exact verification verdicts ignore the rerunning adapter weaker class',
  'crates/mesh/mesh/src/registry.rs:verify_output',
  'An Observed spec legitimately runs a ProducerOnly local adapter. Its ordinary output is NotCheckable, but an ExactBytes producer output for that same resource causes this local adapter to emit Reproduced for matching bytes and Diverged for different bytes. verify_output inspects only the producing output class; run_job only requires the local adapter to satisfy the weaker posted spec. The identifier contract explicitly gives weaker implementations no cross-implementation exactness promise.',
  'Public registry APIs with a small legal ProducerOnly test adapter and memory blobs. Hash comparison itself is accurate; the unsupported part is treating its mismatch as the exact conformance verdict described by Diverged. No malicious producer, GPU or remote execution is claimed.',
  'bounded rerunner-class qualification or clearer verdict taxonomy',
  'Require an appropriately qualified local class before attributing exact divergence, or distinguish an observed byte comparison from a conformance check. Preserve Unavailable for absent adapters and NotCheckable for unsupported verification semantics; do not infer a stronger rerun class from the producer declaration.'),
 finding(207,'Delayed resource configuration changes exact output without changing either identity',
  'crates/mesh/mesh/src/resources/delayed.rs:DelayedResource::new and prepare',
  'Two legal registrations of DelayedResource with units1 and2 carry identical resource id mesh.delayed/v1, implementation id mesh.delayed.blake3-chain/v1 and ExactBytes class. The identical signed job spec and input produce different outputs. Verifying one with the other returns Diverged attributed to the same implementation id. Unit count is host constructor state and never bound into the job input/spec or either identity.',
  'Actual shipped adapter through public run_job/verify_output; no copied hash-chain implementation. Default registries both use16 units and are not alleged to diverge. Delay-only configuration does not affect bytes; the unit count does.',
  'resource/input identity decision, potentially over one hour across rehearsal consumers',
  'Bind semantic unit count into canonical input or a resource generation contract; scheduling delay can remain host policy. A different implementation id alone would not repair the promised ResourceId algorithm equivalence. Keep the rehearsal configurable without claiming one exact algorithm for distinct computations.'),
 finding(208,'Checkpoint-plus-tail catch-up selection omits predecessors still required by owner admission',
  'crates/mesh/mesh/src/drop_export.rs:catch_up/in_scope; store.rs:accept_outcome; stickleback processor continuity',
  'A real source store holds post seq0, an owner-built accepted checkpoint covering it, and a later post seq1 linked to seq0. The catch-up selector omits seq0 and selects checkpoint plus tail. An empty receiver accepts the checkpoint and reconstructs its snapshot, but rejects the selected tail as MissingPredecessor. Supplying the omitted seq0 lets that same tail enter. Checkpoint state/frontier does not initialize the shared per-author log continuity anchor.',
  'Actual source/receiver MemoryBackend MeshStores, real checkpoint policy and public selector. No file serialization/import transport or live catch-up campaign executed. The selected set is incompatible with this unchanged authoritative owner admission path; a future authorized prune anchor is a different scenario.',
  'retained-frontier/bootstrap contract repair, potentially over one hour',
  'Define an authenticated continuity anchor or require the needed signed boundary/prune operation in catch-up selection. Coordinate domain checkpoint authority with the shared processor instead of weakening MissingPredecessor for arbitrary non-root operations or bypassing MeshStore.'),
]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,impl,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':impl,'assessment':assessment,
  'state':'All43 Mesh files read; seven current native observations; actual library/integration gates', 'verification':'mesh_execution_review_receipt.json'}
entries=[
 entry(386,'One execution route versus two selection policies','Legacy and V2 work share one run path.',
  ['run_legacy constructs ephemeral spec then calls run_job','V2 selection invokes DevicePolicy::accepts','Legacy selection directly checks adapter availability'],
  'Actual executor reuse is present. Q203 is divergent admission before that executor, not two independent Echo/Blake3 engines.'),
 entry(387,'Held reservations versus running work','Owner reclaim always wins and a full device starts nothing else.',
  ['Reclaim requires live held lease plus running membership','Execute accepts held-but-not-running jobs','accepting gate occurs after Execute loop'],
  'Q204 proves the settings can be bypassed by an already held reservation. Keeping existing work is not the same as starting it.'),
 entry(388,'Runner cancellation versus adapter cooperation','A cancelled run commits nothing.',
  ['Runner checks after prepare','Delayed checks at start of each unit','Final unit sleep precedes return without another check','Runner commits without checking again'],
  'Q205 refutes the unqualified cancellation/commit claim with the shipped adapter. This does not establish an atomic rollback guarantee for a cancellation during sink write.'),
 entry(389,'Producer claim versus rerunner qualification','An exact result can be attributed as reproduced or divergent by another implementation.',
  ['Producer ExactBytes enables verification','Local adapter is matched only against posted spec','ProducerOnly local class is otherwise NotCheckable'],
  'Q206 identifies a missing class qualification. Literal hash equality remains true, but a mismatch by a weaker implementation is not the exact conformance failure the documented verdict alleges.'),
 entry(390,'Algorithm identity versus constructor configuration','One exact ResourceId denotes one interoperable algorithm.',
  ['Delayed resource and implementation ids are fixed','units comes from constructor rather than signed input','Same spec/input with units1 versus2 diverges'],
  'Q207 qualifies a false exactness/provenance boundary in the rehearsal resource. The default16-unit path remains deterministic. The mod.rs cost-bounded-by-posted-spec claim also lacks a signed unit-count/work budget; host configuration owns it.'),
 entry(391,'Checkpoint board frontier versus operation-log frontier','Current checkpoint plus replay tail is enough for catch-up.',
  ['Selector trims event prefix by checkpoint frontier','Checkpoint log is accepted independently','Shared processor still requires first retained non-pruned event predecessor'],
  'Q208 proves missing continuity material through the same owner. Do not work around it with a second permissive import store.'),
 entry(392,'Device always travels versus catch-up scope','Attestations always travel at checkpoint priority.',
  ['Privacy admits DeviceAttested','Priority ranks it with checkpoints','Catch-up frontier removes old attestations','Snapshot contains no device directory'],
  'The seventh probe extends Q202: archive selects the attestation, catch-up omits it. A visibility/priority claim does not override scope filtering. RemoteSessionClaim requires the lost attestations on both client/server admission.'),
 entry(393,'Remote authority versus its carrier','A claim binds the poster, exact active lease and authenticated devices.',
  ['Private signed fields plus domain-separated signing bytes','Bounded strict credential decode','Client must post job and match connected peer','Server must hold exact epoch/lease and have attestation'],
  'These are concrete checks rather than an alternate Burn/Iroh authority. Authorization is an opening-time check by stated contract; ongoing session revocation belongs to its owning supervisor. No fresh remote-protocol campaign was run.'),
 entry(394,'Inventory observation versus durable record authority','The service availability window is not another operation store.',
  ['Notochord settles scoped action/transport identity','MeshStore accepts original signed record','Records holds at most4 recent raw originals','Advisory results explicitly permit ambiguous lost reply and idempotent retry'],
  'Real shared authority and scope limits exist. The volatile bytes and inventory are a separate temporary availability projection, not future custody or proof of durability. Potential erasure coordination with that copy is documented separately.'),
 entry(395,'Ingress resource bounds versus owner commit','Active/queued calls and whole-call I/O have explicit finite bounds.',
  ['Two active semaphore slots','Four transport queue slots','24KiB read and8KiB write lifetime caps','16KiB original and bounded control/result frames','Timeout starts at carrier arrival and joins/cancels on shutdown'],
  'Source checks are concrete and the admitted-ingress library test passes. Existing example campaigns remain blocked by Q196 and were not rerun; this audit adds no fresh saturation/framing/physical-radio receipt.'),
 entry(396,'Shared rules versus copied control planes','Admission and protocol framing retain their original owners.',
  ['Atomic admitted count reservation under one mutex','Notochord action permission checked before body handling','Strict DropRecord CBOR plus re-encoding','Shared decode_operation_record then MeshStore accept'],
  'This avoids a separate private codec/store. A timeout or reply loss cannot prove no commit, and the client says so. Returning from policy admission is not represented as worker execution or verified output.'),
 entry(397,'Lexical semantics versus canonical wire adapter','ESP owns embedding behaviour; Mesh supplies the namespace/codec seam.',
  ['LexicalEmbedResource calls shared embed_batch','Native and wasm probe include same codec source','Pinned byte digest exists in current native test','No private second embedding engine'],
  'This is actual reuse. The historical wasm receipt was read but not freshly rerun; current native tests do not substitute for that target. Public LexicalVectors shape/encoder invariants remain a source-qualified candidate.'),
 entry(398,'Checkpoint progress flag versus preserved resumable state','A held checkpoint exists locally.',
  ['Delayed branch sets hold_checkpoint(true)','Returns only completed/total counts','Hash-chain state is local to consumed execute future','No resume-state value or persistence in adapter'],
  'The rehearsal proves stopping/reporting, not preserved resumable state. No actual resumed computation was tested. Local resume representation is a larger owner decision, not a reason to claim the flag proves a checkpoint exists.'),
 entry(399,'Shared joined runtime versus authoring outcome','Mesh is thin domain code over JoinedSpace.',
  ['Status/round types re-export shared definitions','join supplies MeshStore accept callback','Authoring commits before live publish','Concurrent same-key author calls explicitly prohibited'],
  'Shared lifecycle reuse is real. A publish error after accepted storage leaves an ambiguous authored result; no injected post-commit failure was run. resync explicitly observes settling rather than forcing a network refresh.'),
 entry(400,'Source-wide evidence versus test completeness','Mesh source review is complete.',
  ['All43 inventoried files556938 chars read','All20 previously unread files231466 chars added','Root manifest53872 chars reread after upstream change','Seven native observations plus3 integration and124 feature-library tests'],
  'This completes this crate source, not Mere. Default library includes in-process network sync and a temporary redb reopen test; probe scope remains CPU/MemoryBackend. Optional ingress test execution is not a live ingress deployment.'),
 entry(401,'Centralized Cargo identity versus aliases and retained archaeology','Workspace pins choose the actual shared suppliers.',
  ['identity/personae and tinct/tincture deliberately alias same package/source','Mere/Retinue source-key patches redirect immutable older consumers','P2panda compatibility tag pins actual supplier','Retired vendored sources remain but are not selected'],
  'Alias spelling and preserved donor source are not alone evidence of duplicate runtime authority. Actual lock/artifact source is decisive. Root comments retain some historical M1/stub wording and do not override currently executed code.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Raw admitted-ingress availability copies are not tied to later Mesh payload erasure',
  'source':'crates/mesh/mesh/src/ingress/session.rs:records.retain; records.rs:bytes; store.rs:checkpoint payload erasure',
  'note':'Accepted original DropRecord bytes, including inline input, stay in volatile Records until FIFO eviction or retained_for expiry. Later checkpoint body erasure in MeshStore has no callback removing this availability copy. Determine whether the governed privacy ceiling applies to all owned copies and coordinate retention accordingly. No admitted carrier reproduction or post-erasure retrieval was run; do not assert a remote privacy leak. Cross-owner erasure semantics can exceed one-hour audit diversion.'},
 {'title':'Delayed checkpoint flag reports a held checkpoint without retained chain state',
  'source':'crates/mesh/mesh/src/resources/delayed.rs:execute Checkpoint branch; resource.rs:Checkpoint',
  'note':'Branch sets checkpoint_held=true then returns counts only; current hash state and Prepared are consumed/dropped, and adapter has no state store/resume API. The existing test proves stop plus flag, not usable preserved state. Clarify rehearsal-only meaning or supply owner-held state before treating this as a resumability receipt. No resumed-work probe executed; broader resume contract can exceed one hour.'},
 {'title':'Canonical LexicalVectors encoder has no public shape validation',
  'source':'crates/mesh/mesh/src/resources/lexical_codec.rs:LexicalVectors::encode',
  'note':'Public dimensions and vector rows can disagree. encode flattens row lengths without verifying every row matches dimensions, while decode reparses by dimensions/count. Differently ragged rows can encode identically or reassign values to rows. Actual LexicalEmbeddingProvider returns conforming shapes, so current shipped adapter corruption is not established; no dedicated malformed-shape probe run.'},
 {'title':'Lease receipt prose claims absence of sleeping/racing despite a timed cancellation run',
  'source':'crates/mesh/mesh/tests/lease_receipts.rs:module docs and owner_reclaim_hands_the_job_to_a_second_worker',
  'note':'The integration test spawns a default16-unit delayed run, sleeps50ms, then expects cancel to arrive before completion. Each zero-delay adapter unit uses Tokio sleep(0). All3 tests passed on this Windows run. Runtime/timer/target variation can finish before cancel, so this receipt is not timing-independent as its prose claims. Prefer an explicit progress/barrier handoff; no current failure or cross-platform flake was reproduced.'},
 {'title':'Worker owner reclaim only targets currently live Held jobs',
  'source':'crates/mesh/mesh/src/worker.rs:next_action reclaim loop',
  'note':'A running job whose local lease projection has already lapsed is excluded from Reclaim, even with owner withholding. A real host may independently stop its in-flight job; inspect that supervisor before claiming workload continuation. Current Q204 only proves held-but-unstarted start-gate bypass and running-live control. A stop action distinct from authoring an in-window revoke may require a broader lifecycle design.'},
 {'title':'Mesh authoring can return publish failure after durable acceptance',
  'source':'crates/mesh/mesh/src/sync.rs:author_event',
  'note':'store.accept commits before joined.publish. A publish error returns MeshSyncError::Backend without exposing the committed operation in the result. Distinguish durable authoring from live dissemination for callers; do not roll back unrelated authority state merely on this broad Err. No fault injection executed here. Related shared JoinedSpace caller concern was already tracked.'},
],'title')
next_path=stats['changed_since_inventory'][0]
next_read={'path':next_path,'start':0,'reason':'upstream-changed previously reviewed file; full reread required'}
notes['current_area']='Complete Mesh source including optional ingress and lease integration; current root manifest reread'
notes['next_area']='Reread upstream-changed Pictograph/Pandect sources, then continue remaining Mere source'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['mesh_execution_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'diagnostic_outcomes':'7 native observations reproduce6 new defects plus Q202 export extension;3 lease integration and124 admitted-ingress library tests pass.',
 'coverage_note':'21 complete/current files285338 chars added; Mesh43/43 complete. Root manifest reread removes1 freshness invalidation;12 changed files remain.'}
upsert(notes['source_qualifications'],[{'area':'Complete Mesh source and current manifest',
 'note':'All43 Mesh files556938 chars now read, including all examples, optional ingress and lease integration.21 new/current whole files285338 chars this pass:20 Mesh plus root manifest53872 chars.7 native public-API observations reproduce6 new defects and catch-up attestation omission.3 original integration tests pass;124 original feature-library tests pass. Existing library tests include real in-process sync endpoints and temporary redb reopening; new probes use CPU/MemoryBackend only. No new example campaign, wasm rerun, admitted ingress deployment/saturation, GPU, physical device, remote compute or external peer executable. Source unchanged, probe executable/PDB removed, stable target reused; no isolated target/home/worktree.'}],'area')
doc=old['document']['path'];dest=ROOT/doc;before=dest.read_bytes();assert sha(dest)==old['document']['sha256']
heading='### Complete Mesh source and execution-authority review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll43 inventoried Mesh source/manifest/example/test files are now read,556,938 characters. This pass adds20 previously unread Mesh files231,466 characters and rereads the current root Cargo.toml53,872 characters after the upstream merge. Repository current coverage is1,194/3,355 complete files and15,706,387/36,897,906 characters. Twelve changed Pictograph/Pandect files still need full reread; next is '+next_path+'. Full Mere source objective remains active. Production and original tests/concurrent work are unchanged. Authority entries A386-A401 distinguish real sharing, divergent admission, cancellation/verification claims and catch-up anchors.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='The seventh native observation extends Q202 rather than inventing another directory bug: archive selection gives DeviceAttested checkpoint priority, but catch-up scope omits an attestation represented by the frontier. Its snapshot contains only jobs. The privacy comment saying devices always travel is therefore false for this path. The newly read RemoteSessionClaim owner checks both client and server attestations, so lost directory state affects that admission dependency. The source checks actual poster identity, exact resource, exact active epoch/lease, offered device and authenticated endpoints; it is a real shared transport-neutral authority rather than a private Burn or Iroh identity gate. No new remote session campaign ran.\n\n'
addition+='Real reuse remains visible: run_legacy binds an ephemeral namespace and uses run_job; Echo/Blake3 share preparation; the lexical adapter uses ESP plus one codec source also included by the wasm probe; SyncedMesh uses JoinedSpace and re-exports status; both admitted service actions use Notochord then unchanged MeshStore admission. The worker policy bypass occurs before the common executor, not in a second compute engine. The verifier, by contrast, has weaker local qualification than its exact verdict contract, and the delayed adapter hides semantic unit count outside that contract. Fixing either by inventing a product-private implementation would miss the owner seam.\n\n'
addition+='The optional ingress source has finite active and queue limits, whole-call I/O budgets, frame limits, deadline from carrier arrival, atomic admitted-count reservation, abort/join shutdown and exact canonical original-record decoding. Its inventory/bytes window is explicitly volatile and limited to four records, with no future custody promise. Results are advisory; lost response is explicitly ambiguous and retry uses original identity. Actual code does not equate transport admission with operation acceptance or compute success. The new unit gate reaches this code, but the older example campaign is still unbuildable for Q196, and no fresh saturation/framing/network ingress campaign is claimed.\n\n'
addition+='Larger follow-ups stay documented: raw availability bytes can outlive later MeshStore input-body erasure until TTL/eviction, so privacy/erasure ownership needs a rule and a dedicated carrier repro before any remote-leak claim; checkpoint_held is set by DelayedResource while its hash state is dropped and only counts escape, so current tests prove stopping/reporting rather than a resumable checkpoint; public LexicalVectors encodes unchecked row shapes, though the shipped ESP adapter supplies valid shapes. Live-lapsed owner-stop handling needs host-supervisor review. author_event can return publish failure after durable acceptance and needs a qualified outcome contract, not an inferred rollback.\n\n'
addition+='All three original lease integration tests passed. Their top-level claim that nothing sleeps/races is too broad: one spawns a delayed run, waits50 ms, then expects cancellation before completion. Different timer/runtime behaviour may change that order; no failure occurred here. Preserve this as a qualified receipt limitation and replace the handoff with a barrier if repaired. Native lexical pin passed inside the library suite; historical wasm exactness evidence was read but not freshly rerun. Default library sync tests actually bind in-process peers and its store test closes/reopens a temporary redb database. Those existing test effects are separate from the seven new CPU/MemoryBackend probes and are not a new full deployment or radio receipt.\n\n'
addition+='Fresh current-main gates: seven additional native observation assertions pass (six new findings and a Q202 extension), all3 original lease integration tests pass, and all124 admitted-ingress library tests pass. Green observation assertions mean defects reproduced, not repaired. The current root manifest is fully reread: aliases intentionally share sources; old repository source-key patches redirect immutable consumers; the exact lock/artifact supplier matters more than stale M1 comments or retained inactive donor folders. Required compact logs/probe/coverage evidence and preserved original inputs remain under the default100 MB retained-byte cap, with no captures or replacement of cited receipts. Probe executable/PDB removed; stable shared target only, no isolated home/target/worktree. See the [complete Mesh review receipt](../../../support/receipts/authority-success-audit/mesh_execution_review_receipt.json).\n'
for a,b in [('All43','All 43'),('read,556','read, 556'),('adds20','adds 20'),('files231','files 231'),('Cargo.toml53','Cargo.toml 53'),('is1,194','is 1,194'),('and15,706','and 15,706'),('waits50','waits 50'),('all3','all 3'),('all124','all 124'),('default100','default 100')]:addition=addition.replace(a,b)
dest.write_bytes(before+addition.encode());save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'mesh_execution_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
files=['mesh_execution_probe.rs','run_mesh_execution_probe.py','mesh_execution_probe_receipt.json','mesh_execution_probe_build.log','mesh_execution_probe_output.log','mesh_execution_lease_tests.log','mesh_execution_admitted_lib_tests.log','mesh_execution_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete Mesh source review; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'mesh_coverage':{'complete_files':43,'total_files':43,'complete_chars':556938,'total_chars':556938},
 'added_complete_files':new,'source_sha256':{p:sha(ROOT/p) for p in new},'preserved_production_and_test_hashes':preserved,
 'next_read':next_read,'integration_tests':3,'integration_test_exit':0,'admitted_ingress_library_tests':124,'admitted_ingress_library_test_exit':0,'native_observations':7,'native_probe_exit':0,
 'document':{'path':doc,'sha256':sha(dest),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in files},'retained_isolation':[],'generated_probe_removed':True,'diff_check':0,
 'receipt_budget':{'cap_bytes':100_000_000,'audit_retained_bytes':sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()),'required':'Compact reading/probe/build/test/merge-preservation evidence; no new captures'},
 'limits':['Mesh43/43 complete does not complete Mere;12 invalidated upstream paths remain unread at current hashes.',
 'Seven native probes characterize defects; production/original test source unchanged.',
 'Q206 hash comparisons are accurate; weaker-rerunner conformance classification is unsupported.',
 'Catch-up selected-set failure is actual owner admission, not a serialized file/transport import campaign.',
 'Existing original library tests include in-process peer sync and temporary redb reopen; new probes are CPU/MemoryBackend only.',
 'No fresh wasm, live admitted ingress deployment/saturation, physical radio/GPU/remote compute/external peer campaign.',
 'Resume-state, erasure-copy, lexical-shape, expired running-stop, publish outcome and timing-flake concerns remain source-qualified.',
 'Stable shared target reused; no isolated target/home/worktree; owned probe executable/PDB removed.']}
assert receipt['receipt_budget']['audit_retained_bytes']<100_000_000
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
save('mesh_execution_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'mesh_coverage':receipt['mesh_coverage'],'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
