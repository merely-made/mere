"""One-shot read-only checkpoint, before the separately recorded main sync."""
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
old=load('mesh_examples_review_receipt.json');probe=load('mesh_board_probe_receipt.json')
assert probe['tests']==9 and probe['test_exit']==0 and probe['generated_output_removed']
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
assert '123 passed; 0 failed' in (HERE/'mesh_board_lib_tests.log').read_text(encoding='utf-8-sig')
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
new=sorted(p for p in inventory if p.startswith('crates/mesh/mesh/') and p not in preserved
           and coverage.get(p,{}).get('through')==inventory[p]['chars'])
assert len(new)==10 and sum(inventory[p]['chars'] for p in new)==197076
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1186 and stats['read_chars']==15775885
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
mesh_paths=[p for p in inventory if p.startswith('crates/mesh/mesh/')]
mesh_complete=[p for p in mesh_paths if coverage.get(p,{}).get('through',0)==inventory[p]['chars']]
assert len(mesh_paths)==43 and len(mesh_complete)==23
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'reproduced against unchanged public APIs and current Cargo artifacts; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'mesh_board_review_receipt.json','diagnostic':'mesh_board_probe_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test source', 'size':size,'reason':repair}}
findings=[
 finding(198,'Unleased completion overwrite violates the order-independent board contract',
  'crates/mesh/mesh/src/board.rs:fold_from_snapshot result gathering; fold.rs:Posted::resolve',
  'Two correctly signed sequential JobDone operations by the same valid claim winner carry A and B. Folding the identical set as post/claim/A/B returns Done(B); post/claim/B/A returns Done(A). Results are gathered by author and overwritten in input order, unlike claims keyed by operation hash. The board/module overview promises order independence.',
  'Native direct public fold reproduced. Both operations form an ordinary signed author chain. Actual MeshStore iterates indexed author logs in sequence order, so this does not establish live store peer divergence or an accepted author-log fork.',
  'semantic result arbitration decision; broader than a mechanical overwrite fix',
  'Name a deterministic rule for multiple winner outputs and mixed generations, retain the selected operation identity, and qualify permutations and checkpoint replay. Do not silently choose first/last by timestamp or hash as an unruled protocol change.'),
 finding(199,'Parsed addressing and cached signed extensions disagree across verification and storage',
  'crates/mesh/mesh/src/wire.rs:verify/from_operation; store.rs:MeshPolicy::admit; crates/stickleback/src/processor.rs:preflight',
  'An operation signed for foreign mesh F is initially refused by accept(M). Mutating public header.extensions.mesh_id to M leaves encoded signed bytes and cached operation hash unchanged, yet mesh::verify succeeds, direct fold(M) admits it, and real MeshStore::accept(M) reports insertion. Reading the retained header decodes F; ops(M) contains one record while board(M) is empty. The application policy indexes parsed extensions while durable bytes use the supplier cached CBOR extensions.',
  'Native public API and real MemoryBackend store reproduction with synthetic valid signatures. No forged signature, corrupted wire acceptance, remote network ingress exploit or durable user store is claimed. Exact compiled p2panda supplier is 0.7.1 from 1f69c9b, not historical upstream API recollection.',
  'shared supplier/admission consistency repair, potentially over one hour',
  'Establish one immutable or canonical decoded header boundary across p2panda typed extensions, Mere preflight, policy selection and persistence. Re-check all consumers and test features; a one-off mesh-id check would leave the shared representation problem in place.'),
 finding(200,'Accepted checkpoint reconstruction loses completed leased output',
  'crates/mesh/mesh/src/board.rs:fold_from_snapshot; fold.rs:GatheredJobLeases::completed_output; store.rs:board_from',
  'A valid leased job reaches Committed. A non-live completed snapshot passes retention policy. Empty-tail reconstruction yields Claimed while lease_at still says Done. The actual store build_checkpoint/accept/board sequence reproduces the loss before any pruning, while the original completion operation remains retained. The snapshot output is seeded into the unleased results map; leased completion lookup only searches newly gathered completion facts.',
  'Direct snapshot API and actual MemoryBackend store checkpoint path. This is a completed lease, outside the explicitly rejected live-Held checkpoint case. No network, redb reopen or physical persistence claim.',
  'contained restoration symptom with checkpoint contract implications',
  'Preserve the accepted completion/output identity through snapshot reconstruction and later tails. Verify checkpoint migrations, already terminal jobs and missing completion bodies before promoting a local state fallback as a complete retention repair.'),
 finding(201,'Leased result lookup publishes an output rejected by grant admission',
  'crates/mesh/mesh/src/fold.rs:GatheredJobLeases::resolve/completed_output',
  'The lease resolver validates output against the signed spec before forming a Completed end. A subsequent completed_output lookup independently chooses the first operation-hash-ordered completion matching holder/lease, without validation or matching the selected closing fact. A valid 5-byte completion closes at time400; an otherwise well-formed 17-byte output against a16-byte grant is ignored alone but published as Committed when paired with that valid completion and a lower hash. The real store accepts both structurally valid records and its board exposes the invalid output. Bounded search found the lower hash in7 attempts.',
  'Native direct fold and actual MemoryBackend admission/board. Invalid output remains self-valid; job-specific admission is intentionally delegated to the fold. Existing single-invalid-result tests pass and do not cover coexistence with an accepted completion. No worker, blob transfer or peer execution was needed.',
  'bounded accepted-fact/output binding repair; retain exact resolver identity',
  'Return or retain the selected accepted completion identity/output from lease resolution instead of performing a second weaker selection. Validate competing valid/invalid outputs, outside-window facts, equal-time tiebreaks and snapshot restoration under that one selection owner.'),
 finding(202,'Checkpoint frontier removes device attestations from replay without snapshotting the directory',
  'crates/mesh/mesh/src/retention.rs:JobBoardSnapshot; board.rs:fold_from_snapshot; store.rs:build_checkpoint/board_from',
  'JobBoardSnapshot clones only jobs; fold_from_snapshot initializes an empty DeviceDirectory. A real store with one valid master-signed self-attestation reports one device. build_checkpoint includes that event in its frontier; accepting the checkpoint changes board.devices to empty even though the attestation operation remains retained. All events below the frontier are filtered out before the fold, and no snapshot directory is available to restore them.',
  'Both snapshot-only API and actual MemoryBackend checkpoint acceptance reproduced. Device cryptographic admission remains correct; the observed loss is replay state coverage, before pruning. Consumer scheduling/UX effects and remote catch-up were not executed.',
  'snapshot version and replay-coverage repair, potentially over one hour',
  'Choose whether the governed snapshot includes authenticated device facts or retention separately preserves/replays attestations outside the job frontier. Coordinate encoded checkpoint version/commitment compatibility and catch-up export; avoid adding a second private directory owner.'),
]
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'Ten newly read Mesh source files; native probes against unchanged source',
  'verification':'mesh_board_review_receipt.json'}
entries=[
 entry(373,'Deterministic claims versus input-ordered results','The same operation set always yields the same board.',
  ['Claims use operation-hash keys','Unleased results use author keys and last input wins','Board and crate docs promise order independence'],
  'Q198 falsifies the full direct-fold promise. Ordinary indexed log order masks this at the current store, so remote peer divergence is not established.'),
 entry(374,'Signed cached extensions versus mutable typed addressing','The decoded header is already trusted and cannot replay into another mesh.',
  ['Supplier cached CBOR preserves original extension bytes','Public MeshExt can be mutated','verify compares cached hash and shared validation','Policy reads typed mesh_id','Storage encodes cached extensions'],
  'Q199 reproduces direct-fold acceptance and successful storage under an inconsistent topic. Historical signature decoding and fresh hash equality do not bind current typed extensions to persisted bytes.'),
 entry(375,'Shared structural gate versus application policy','OperationProcessor owns structural checks; Mesh owns its body/address rules.',
  ['MeshPolicy configures common MunimentStore processor','Shared preflight validates operation and cached hash','Both Mesh fold and shared processor rely on supplier provenance guarantee'],
  'This is actual shared storage ownership, not a private alternate store. Its boundary still admits the representation mismatch in Q199. Supplier test_utils may change hash/signature behavior; the recorded production-feature artifacts are essential.'),
 entry(376,'Accepted lease end versus returned output','Only a grant-honoring completion closes a leased job.',
  ['resolve calls output.validate_against','Lease resolution admits holder/time/window/end ordering','completed_output independently finds first holder/lease completion'],
  'Q201 is duplicated authority within one module: a strong acceptance decision followed by a weaker lookup. The terminal state can carry a rejected output even while the accepted end remains correct.'),
 entry(377,'Job terminal state versus retained lease phase','Snapshot replay preserves settled board state.',
  ['Snapshot serializes terminal output and LeaseRecord','Seeded output enters unleased results map','Leased output lookup searches tail only','Projection sees retained Completed end'],
  'Q200 yields Claimed and lease Done simultaneously after actual checkpoint acceptance. Live-Held checkpoint refusal does not cover this completed case.'),
 entry(378,'Job-only snapshot versus board directory','The checkpoint is replayable board state at its frontier.',
  ['Board owns DeviceDirectory','JobBoardSnapshot stores jobs only','Frontier covers every event including DeviceAttested','Tail removes all represented events'],
  'Q202 proves omitted state, not a signature-validation defect. One checkpoint is enough to lose device mappings while original records remain available.'),
 entry(379,'Self-attestation ownership','A device may only attest itself under its mesh-author derivation salt.',
  ['attests checks master signature and salt','Derived key must equal operation author','Store and directory call the same attests function'],
  'This is deliberate reuse of one rule at two doors. Original directory tests pass. Transport uses derived author directly; stale master-address wording in one test does not override ruling92.'),
 entry(380,'Clock-free facts versus explicit liveness','Only the observation projection asks what is live now.',
  ['Grant and fact resolution compare signed times','phase_at accepts explicit at_ms and skew','Expired differs from heartbeat silence','Trusted own-device clock limitation is disclosed'],
  'This boundary is concrete and honestly scoped. No independent wall clock was introduced during the fold. Future heartbeat/progress interpretation is a separate unexecuted candidate.'),
 entry(381,'Manifest bounds versus resource authority','A signed JobSpec is a manifest rather than authorization.',
  ['Spec bounds names/counts/grants/lease terms','Output self-check differs from grant check','Namespace view enforces assigned input/output capabilities'],
  'The separation is intentional. Digest width-only validation is already tracked with namespace/BlobRef qualification; do not claim every syntactically valid reference is a locally verified blob.'),
 entry(382,'Retention authority versus semantic replay coverage','An authorized snapshot commitment makes checkpoint use safe.',
  ['Policy checks authority/policy/mesh/version/commitment/frontier','Live Held lease refused','Store snapshots current board then filters frontier','Directory/output reconstruction incomplete'],
  'Cryptographic self-consistency does not establish that the snapshot contains all state needed to replay. Q200/Q202 affect checkpoints minted by the owner itself, without malicious authority.'),
 entry(383,'Checkpoint claim freeze versus full history equivalence','An already resolved winner is preserved across a checkpoint.',
  ['Synthetic zero operation key seeds old winner','Original claim operation hash/time absent','next_claimants is serialized but not reseeded as claims'],
  'The source explicitly intends winner freezing and excludes live-lease compaction. Treat late lower-hash claimant semantics and expired/released epoch continuation as policy/replay questions, not a newly ruled fix.'),
 entry(384,'Existing green tests versus exercised negative composition','123 passing tests establish the job/retention path.',
  ['Fresh locked offline default library build','All123 original library tests pass','Nine additional public API/store observations reproduce five defects','No example/network campaign run'],
  'Passing observations characterize defects and are not correctness gates. Mesh23/43 files are read; broader source objective remains incomplete. Source hashes bind all43 Mesh inputs during execution.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Checkpoint continuation after expired or released leases needs original claim-identity qualification',
  'source':'crates/mesh/mesh/src/board.rs:snapshot seeding; retention.rs:live_leases; store.rs:a_live_lease_blocks_a_checkpoint_until_it_ends',
  'note':'Policy refuses only Held snapshots and tests expired lease acceptance. Snapshot loses original claim hash/time and does not seed next_claimants; synthetic at_ms0 winner may affect later epoch proof. Source says live checkpoints are unsafe, but expired/released continuation needs a separate end-to-end test and a rule for retaining claim identity. Do not conflate this with the reproduced completed-output loss or silently choose a new winner policy. Broader checkpoint semantics may exceed one hour.'},
 {'title':'Time-indexed lease phase uses latest retained heartbeat progress even before that heartbeat time',
  'source':'crates/mesh/mesh/src/lease.rs:apply_facts; projection.rs:phase_at',
  'note':'Retained epoch keeps only latest last_seen/progress. phase_at selects grant/end by observer time but has no heartbeat history to filter, and end clamps last_seen without rewinding progress. Earlier-time queries may reflect a future heartbeat or post-end progress. No extra probe executed. Define whether at_ms is only current liveness or historical reconstruction before changing storage shape.'},
 {'title':'Mesh author log namespace may alias distinct mesh ids in one backend',
  'source':'crates/mesh/mesh/src/store.rs:latest/latest_in, MeshPolicy; wire.rs:MeshLogId',
  'note':'latest explicitly ignores mesh_id and indexes author plus Events/Checkpoints; topic resolution associates those same logs with mesh topics. Distinct mesh ids sharing one author/backend may not have independent sequence/frontier or pruning. No multi-mesh store experiment executed; verify intended one-store-per-mesh lifetime and sync author key scope before claiming cross-mesh corruption. Shared log-id migration is potentially larger than one hour.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/mesh/mesh/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Mesh board, fold, wire, lease/projection, spec, retention, directory, store and crate facade'
notes['next_area']=next_path+' and remaining Mesh production/tests'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['mesh_board_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress',
 'diagnostic_outcomes':'123 unchanged library tests pass;9 native observations reproduce5 direct/store defects.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'10 complete files197076 chars added; Mesh23/43. Remote fetch subsequently found14 incoming commits; synchronization and coverage transition are recorded separately.'}
upsert(notes['source_qualifications'],[{'area':'Mesh board/lease/store full-file review',
 'note':'Ten whole source files197076 chars read; Mesh23/43 complete. Fresh locked/offline build and123 existing library tests pass. Nine synthetic signed-operation and actual MemoryBackend observations reproduce five defects. Directory and leased-output loss occur immediately after owner-built checkpoint acceptance, before pruning. Parsed extension mismatch reaches shared insertion but no wire-signature forgery/network exploit is claimed. No durable user files, redb reopen, worker/native/GPU or peer divergence campaign. Default artifacts and exact supplier hashes preserved. Source unchanged, generated executable/PDB removed; stable target only.'}],'area')
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only Mesh board, lease and checkpoint review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nTen whole Mesh source files add197,076 read characters: board, directory, fold, lease, crate facade, projection, retention, spec, store and wire. Mesh is23/43 files complete; repository coverage is1,186/3,352 files and15,775,885/36,854,820 characters. Full-source objective remains active. Production and original tests/concurrent WIP are unchanged. Authority entries A373-A384 map the split claims and accepted-fact/replay owners. Resume at '+next_path+'.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='The strongest duplicated authority is within leased completion handling: one resolver validates the signed grant and chooses the accepted end, then another lookup selects an output using weaker conditions. Q201 executes both the direct fold and real store path. The synthetic over-grant record is correctly ignored on its own; the defect requires a valid closing completion alongside it. Selecting the accepted completion once is the repair seam, with competing facts and snapshots covered together.\n\n'
addition+='Q199 is a representation split across the actual shared supplier/processor/policy/storage boundary. The supplier retains original CBOR extensions while exposing mutable typed extensions. In this production-feature build, the cached hash and already-decoded provenance assumption do not bind the current typed address. The existing wire negative changes encoded bytes, which is a different test from mutating the typed field. Exact decoded-wire signature controls are not alleged broken. Keep a supplier-level consistency repair separate from claims of a network exploit.\n\n'
addition+='The shared MunimentStore is real reuse, and directory attests is one rule called by both admission and fold. Merely repeating a call is not duplicated policy. The contradictions are the output selector and incomplete checkpoint state: cryptographic checkpoint self-consistency and authorized frontier advancement do not prove that all represented state can be reconstructed. Q200/Q202 occur on the normal owner-minted checkpoint path, before physical erasure. A JobBoardSnapshot includes only jobs; the board also owns its device directory.\n\n'
addition+='Declared boundaries remain distinct: live Held checkpoints are refused, signed lease facts are clock-free, liveness takes explicit observer time/skew, heartbeat silence is not worker failure, and own-device clocks are trusted rather than adversarial. Snapshot winner freezing is explicitly intended, so lower-hash late claimant behavior needs a semantic ruling rather than an arbitrary change during audit. Expired/released epoch continuation, historical heartbeat projection and shared backend mesh-id/log aliasing remain source-qualified candidates with no fresh reproduction. Their potential snapshot/log/schema repairs can exceed the one-hour diversion threshold and are documented.\n\n'
addition+='Fresh gates at4523cb6f: locked/offline default mere-mesh library build succeeds; all123 original library tests pass; nine additional native CPU/API observations reproduce the five findings. Passing diagnostic assertions mean the defects were observed, not repaired. Synthetic signed operations and real MemoryBackend only; no network, live peer divergence, redb user store, browser/GPU, worker/blob delivery or physical radio. Probe executable/PDB removed; shared stable target reused with no isolated target, Cargo home or worktree. Required retained receipts are the compact build/test/probe logs, source/coverage hashes and [review receipt](../../../support/receipts/authority-success-audit/mesh_board_review_receipt.json), under the default100 MB total retained-byte cap (entire audit evidence currently about12 MB). No cited receipt is replaced by the subsequent main synchronization.\n'
for a,b in [('add197','add 197'),('is23/43','is 23/43'),('is1,186','is 1,186'),('and15,775','and 15,775'),('at4523','at 4523'),('all123','all 123'),('default100','default 100'),('about12','about 12')]:addition=addition.replace(a,b)
save('review_notes.json',notes);save('authority_map.json',authority);destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'mesh_board_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
trace=[Path(probe['libraries']['p2panda_core']['target_source']).parent/'operation'/x for x in ['header.rs','validation.rs','operation.rs']]
trace+=[ROOT/'crates/stickleback/src/processor.rs']
evidence=['mesh_board_probe.rs','run_mesh_board_probe.py','mesh_board_probe_receipt.json','mesh_board_probe_build.log','mesh_board_probe_output.log','mesh_board_build.jsonl','mesh_board_build_stderr.log','mesh_board_lib_tests.log','mesh_board_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Mesh board/lease/checkpoint complete-file review; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'coverage':stats,
 'mesh_coverage':{'complete_files':23,'total_files':43,'complete_chars':sum(inventory[p]['chars'] for p in mesh_complete),'total_chars':sum(inventory[p]['chars'] for p in mesh_paths)},
 'added_complete_files':new,'source_sha256':{p:sha(ROOT/p) for p in new},'preserved_production_and_test_hashes':preserved,
 'next_read':next_read,'library_build_exit':0,'library_tests':123,'library_test_exit':0,'native_observations':9,'native_probe_exit':0,
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},'trace_sha256':{str(p):sha(p) for p in trace},
 'retained_isolation':[],'generated_probe_removed':True,
 'receipt_budget':{'cap_bytes':100_000_000,'audit_retained_bytes':sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()),'scope':'required compact build/test/probe/coverage evidence; no captures; pre-existing cited evidence preserved'},
 'limits':['Full Mere objective incomplete; no production/original test changes.', 'Input-order counterexample is direct fold; ordinary indexed store order is deterministic, no live peer divergence claimed.',
 'Extension mismatch reaches actual store, but no forged wire signature or network ingress exploit claimed.', 'Completed lease and directory losses reproduced through owner-built/accepted checkpoint before pruning.',
 'MemoryBackend/synthetic operations only; no redb reopen, user files, network, browser/GPU or worker execution.', 'Remote14 incoming commits found after gates; separate sync transition required.']}
assert receipt['receipt_budget']['audit_retained_bytes']<receipt['receipt_budget']['cap_bytes']
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('mesh_board_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'mesh_coverage':receipt['mesh_coverage'],'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
