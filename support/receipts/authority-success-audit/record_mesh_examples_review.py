"""One-shot Mesh manifest/example reading checkpoint with fresh failed check gates."""
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
old=load('witness_proofs_review_receipt.json');evidence=load('mesh_examples_evidence_receipt.json')
assert len(evidence['missing_includes'])==6 and all(not Path(x['resolved']).exists() for x in evidence['missing_includes'])
assert evidence['inventory_example_check']['shell_exit']==1 and evidence['inventory_example_check']['rust_errors']==2
assert evidence['retained_example_check']['shell_exit']==1 and evidence['retained_example_check']['rust_errors']==4
assert evidence['positive_check']['shell_exit']==0
assert evidence['hardcoded_p2panda_revision']!=evidence['actual_p2panda_revision']
assert all(sha(Path(p))==h for p,h in evidence['trace_sha256'].items())
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p=='crates/mesh/mesh/Cargo.toml' or p.startswith('crates/mesh/mesh/examples/'))
assert len(complete)==12 and sum(inventory[p]['chars'] for p in complete)==114742
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1176 and stats['read_chars']==15578809
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
mesh_paths=[p for p in inventory if p.startswith('crates/mesh/mesh/')]
mesh_complete=[p for p in mesh_paths if coverage.get(p,{}).get('through',0)==inventory[p]['chars']]
assert len(mesh_paths)==43 and len(mesh_complete)==13
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,status,detail,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,'status':status,'evidence':detail,'scope':scope,
  'verification':'mesh_examples_review_receipt.json','diagnostic':'mesh_examples_evidence_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test/example source','size':size,'reason':repair}}
findings=[
finding(196,'Both Mesh retained-ingress receipt examples fail to compile after the Murm supplier extraction',
 'crates/mesh/mesh/examples/inventory_ingress/main.rs:source_hash; examples/retained_receive/identity.rs:SOURCES',
 'fresh current Cargo check fails for both example targets; documented, not repaired',
 'Fresh locked/offline inventory_ingress check with admitted-ingress fails on2 deleted crates/murm/transport source includes. Fresh retained_receive check fails on4 equivalent deleted includes. All6 compile-time provenance paths resolve inside the former Mere tree and do not exist; the actual transport supplier is an external locked Murm Git package. Positive control checks the admitted-ingress library and mesh-peer example successfully under the same workspace/toolchain.',
 'Actual native Cargo check gates:2 failures and1 positive control. No Retinue executable or network campaign ran. These are unbuildable receipt paths, not a failed codec/owner admission experiment or a claim Mesh itself cannot compile.',
 'contained provenance path/supplier binding correction',
 'Bind provenance to the resolved supplier actually compiled and keep selected local owner sources distinct. Do not recreate a private crates/murm copy merely to satisfy include paths. Recheck both targets and their metadata before presenting historical composition receipts as current reproducibility.'),
finding(197,'Retained-receive provenance hardcodes a p2panda revision that differs from the locked compiled supplier',
 'crates/mesh/mesh/examples/retained_receive/carrier.rs:Carrier::finish p2panda_net_revision',
 'source literal mismatch against current locked/resolved Cargo supplier; documented, not repaired',
 'The report field is the fixed revision '+evidence['hardcoded_p2panda_revision']+', while both Cargo.lock and fresh locked/offline Cargo metadata select mere-p2panda-net0.7.5 at '+evidence['actual_p2panda_revision']+'. No assertion or lookup connects that literal to the library used by this build. Hashing Cargo.lock/source elsewhere in the report does not make this named revision field correct.',
 'Literal source versus current lock and resolved metadata. Q196 currently prevents this example from compiling, so no newly emitted report carrying the false value is claimed. The mismatch remains after merely repairing the missing includes.',
 'contained supplier-derived provenance correction',
 'Derive the reported supplier identity from the exact build closure, or explicitly label historical fixture revisions. Keep any runtime checkout observation separate from compile-bound package identity; avoid freezing dependency facts in success JSON literals.'),
]
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All12 Mesh manifest/example files read; Mesh13/43 including prior namespace; two example check failures and library/mesh-peer positive check',
  'verification':'mesh_examples_review_receipt.json'}
entries=[
entry(361,'Transport supplier versus stale in-repository provenance','Mesh imports the shared transport supplier rather than owning another endpoint.',
 ['Manifest uses optional imported transport for admitted ingress', 'p2panda endpoint/gossip supplied to shared sync lane', 'Two example source catalogs still include old crates/murm paths', 'Inventory Python wrapper separately resolves actual supplier metadata'],
 'Q196 shows the source ownership move was not propagated into both compile-time receipt catalogs. The library and simpler rehearsal still compile. Keeping stale source catalogs beside a correct dynamic metadata lookup creates two provenance implementations with different authority.'),
entry(362,'Report field versus build identity','Retained-receive reports the p2panda revision it exercised.',
 ['Hardcoded p2panda_net_revision JSON literal', 'Cargo.lock current selected source', 'Fresh resolved package metadata', 'Separate source/executable hashes in the report'],
 'Q197 is a concrete false supplier field, independent of Q196. Checksums of other evidence do not repair the field meaning. Since the example currently fails check, this is source-proven stale output logic rather than an executed new false receipt.'),
entry(363,'M1 rehearsal versus current typed/leased APIs','mesh-peer is a transport rehearsal, not a complete supervisor.',
 ['M1 post uses inline payload because blob delivery is absent', 'M2 ResourceRegistry run paths exist', 'DevicePolicy::unsupervised excludes leased actions', 'Grant/Heartbeat/Reclaim branch is a hard error if reached'],
 'This limitation is explicitly disclosed and supported by the current source, not a hidden fake host. Historical June M1-only memory and the manifest description are narrower than the present typed/lease API surface. Current source does not claim this example can heartbeat or stop in-flight work.'),
entry(364,'Peer registration versus observed synchronization','The no-placebo rule prints real board and sync status.',
 ['connect_ticket calls add_peer_ticket and set_topics then prints connected peer', 'Later status prints sync rounds/received operations', 'Post waits for a real JobState::Done', 'No standalone ready/handshake assertion on that console line'],
 'The later completion/board observations are actual state-based success checks. Whether the earlier connected wording is only registration requires supplier behavior review; no unreachable-ticket run was executed here. Record the possible readiness overstatement without claiming proven connectivity failure.'),
entry(365,'Persistent source IPC and old one-shot IPC','Both bounded peers return opaque recovered bytes to the same owner seam.',
 ['Inventory Source uses Tokio Child kill_on_drop and typed request identity/deadline/budgets', 'Retained peer uses std Child plus writer/stdout/stderr threads', 'Both compare recovered bytes/digests', 'Both record external implementation/executable identity'],
 'These are two experiment interfaces/scenarios, not two new production codec engines. The persistent lane has stronger lifecycle and monotonic request checks. The older one-shot helper has post-wait joins outside its parent deadline; descendant-held pipes/early error cleanup are source-only concerns, not a reproduced process hang.'),
entry(366,'Opaque reconstruction versus operation admission','Transport identity never substitutes for the original signed operation author.',
 ['Public throwaway transport identity is checked distinct from author', 'Recovered original bytes are compared to supplied record', 'Strict CBOR/re-encoding and shared operation decoder run before MeshStore acceptance', 'Wrong mesh/body/signature negatives check unchanged owner state'],
 'The code has a real separation between carrier reconstruction and application authority. Outer success does not skip inner checks. No campaign was executed in this audit, and the external peer decoder implementation was not independently reviewed; actual RaptorQ claims remain existing harness claims tied to a supplied executable, not fresh audit execution.'),
entry(367,'Exact inventory versus reconstruction or storage authority','Inventory offers identify absent retained original records.',
 ['Candidate set compares local retention to observed remote inventory', 'Repair-only recover gets exact bytes then submit_original calls admitted owner', 'Duplicate outcomes are distinguished', 'Retained original/header/body equality is checked separately'],
 'The projection/selection seam is not used as a second durable record owner. Passing current library check does not prove the example gate; Q196 blocks it. No fresh inventory synchronization or repair reconstruction happened in this pass.'),
entry(368,'Typed frame refusal versus generic absence of ACK','The two experiments qualify invalid framing differently.',
 ['Retained carrier matches TooLarge/UnexpectedEof/Timeout exactly before zero-callback claim', 'Inventory helper checks no owner receipt and unchanged owner call count after accepted hello', 'Both include healthy/owner controls', 'Carrier active path observation rejects non-loopback routes immediately'],
 'These are distinct evidence strengths and finite experiment bounds, not identical safety guarantees. Absence of ACK alone does not name a refusal cause. The retained helper explicitly refuses infrastructure errors as framing evidence; no current runtime boundary cases were rerun.'),
entry(369,'Finite experiment budgets versus production queues','Local one-at-a-time carrier does not prove bounded global accepts.',
 ['Retained Carrier reserve bounds sessions/frame/ACK bytes and elapsed exchanges', 'Carrier JSON explicitly says global production queue not bounded', 'Inventory campaign uses production admitted service with active/pending budget observations', 'Worker execution/blob delivery/native UI/physical radio explicitly false in scope'],
 'Honest scope limits are present. A single serialized experiment is not proof of production scheduling. Fixed descriptive flags/revisions still require supplier qualification; the revision is concretely wrong in Q197. No current queue saturation or carrier observation was executed.'),
entry(370,'Durable reopen evidence scopes','Closed-handle reopen and independent-process reopen are separately identified.',
 ['Retained campaign labels its redb reopen in_process_handle_close_and_reopen', 'Inventory campaign additionally spawns its own --reopen child', 'Both compare original signed header/body identities after reopening', 'No worker computation or remote blob result is inferred'],
 'These are materially different, accurately named checks in source. They do not establish power-cut durability, physical-device replication or actual compute correctness. This pass reads/checks the harness but supplies no fresh redb execution receipt.'),
entry(371,'Paired-process repeatability versus all-observation equality','Python wrapper compares selected semantic fields from two fresh runs.',
 ['Explicit semantic_fields list', 'Timing/actor observations explicitly not compared', 'Peer executable before/after SHA guard', 'Runtime source/Cargo metadata fields recorded after execution'],
 'The repeatability assertion has a visible subset, not whole report equality. Current compiled/run provenance versus runtime source metadata could drift under shared edits; no such drift was reproduced here. Q196 stops the current wrapper before a composition run can occur.'),
entry(372,'Shared source/owner progress versus full Mesh acceptance','Manifest and all examples are now read; most Mesh production source remains.',
 ['12 new files114742 chars', 'Prior namespace13654 chars already read', 'Mesh13/43 files now complete', 'Fresh locked/offline checks qualify only named targets'],
 'Do not promote checked library/type reach into source reading or tested job execution. The next whole-source section is board.rs. Known source build failures and stale provenance are recorded; full Mere objective remains active.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'One-shot retained peer parent timeout does not bound post-exit pipe joins',
  'source':'crates/mesh/mesh/examples/retained_receive/peer.rs:invoke',
  'note':'The loop bounds child.try_wait/kill, then performs blocking joins of writer/stdout/stderr reader threads with no remaining deadline. Descendant-held inherited pipes can outlive the parent, and early errors after spawn have no kill-on-drop guard. The helper also blocks the current-thread Tokio runtime. No descendant/hung-child experiment executed. Prefer a single bounded shared IPC lifecycle if this harness is revived; cleanup/process-tree policy may exceed one-hour diversion. The newer persistent Source is a different, stronger interface.'},
 {'title':'Example receipt runtime source metadata needs explicit compile-bound matching under concurrent edits',
  'source':'crates/mesh/mesh/examples/inventory_ingress/run.py; retained_receive/identity.rs',
  'note':'Compiled selected source/lock hashes and executable identity coexist with runtime observed git status and source/Cargo metadata. No general before/after source stability or automatic equivalence assertion binds every runtime field to the build closure. Current audit observed stable sources and no such mismatch; Q197 is the separate actual hardcoded revision error. Coordinated provenance schema repair can exceed one-hour diversion.'},
 {'title':'mesh-peer connected line may label ticket/topic registration as readiness',
  'source':'crates/mesh/mesh/examples/mesh-peer.rs:connect_ticket',
  'note':'Print occurs after add_peer_ticket/set_topics, before this helper observes a handshake or sync round. Supplier methods may schedule asynchronous connection, so inspect their actual guarantee before changing the wording or alleging a failure. Later board/sync counters and Done wait are actual observations. No unreachable-ticket run performed.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/mesh/mesh/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete Mesh manifest and all11 example files; Mesh13/43 including prior namespace'
notes['next_area']='crates/mesh/mesh/src/board.rs and remaining production/test source'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['mesh_examples_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'Fresh inventory_ingress Cargo check fails2 missing includes; retained_receive fails4. Admitted-ingress library plus mesh-peer positive check succeeds. Source literal revision disagrees with locked/resolved supplier.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'12 complete files114742 chars added; Mesh13/43. No examples, external Retinue process, peers or durable stores executed.'}
upsert(notes['source_qualifications'],[{'area':'Complete Mesh manifest and examples source review',
 'note':'All12 manifest/example files114742 chars read. Inventory and retained examples fail current locked/offline check with2 and4 stale Murm includes; admitted-ingress library and mesh-peer check pass. Source revision metadata mismatch is checked against current locked/resolved Cargo supplier, not executed output. All examples remain unrun in this audit: no external Retinue IPC, RaptorQ, QUIC, redb/child reopen, native resident host, worker/blob delivery or physical radio receipt. Historical M1-only memory is stale for current typed namespace/lease APIs, while unsupervised rehearsal limitations remain explicit. Stable target reused, no isolated target/home/worktree or owned probe executable.'}],'area')

doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only Mesh manifest and examples review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nThe Mesh manifest and all11 inventoried example files are read,12 files and114,742 characters added. Mesh coverage is13/43 files including the previously read namespace module. Repository coverage is1,176/3,352 files and15,578,809/36,854,820 characters. Full-source objective remains active; production and original example/tests/concurrent work are unchanged. Resume at '+next_path+'. Authority entries A361-A372 separate imported suppliers, stale provenance, scoped rehearsals, inner record authority, queue/durability/repeatability observations and actual audit gates.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='The two old and new handoff experiments share the actual MeshStore/operation admission owner rather than maintaining an alternate graph store. Carrier/transport subject is checked distinct from signed operation author. Strict CBOR plus re-encoding, original record/header/body equality, wrong-mesh/body/signature refusals, duplicate owner behavior and redb reopen checks are explicit source controls. The retained carrier names in-process close/reopen; inventory adds an independent-process --reopen child. Native UI, physical radio, worker execution and blob delivery are explicitly outside the experiments. No such campaign was run here, and the supplied external Retinue decoder source was not independently audited. Recovered bytes/flags/executable identity are harness evidence requirements, not fresh audit execution.\n\n'
addition+='The newer inventory lane uses shared admitted ingress and an exact retained-object inventory projection, with original opaque bytes submitted to the same owner. It tracks action permissions, owner-call counts, capacity refusal, duplicate identity and active/pending limits. The older QUIC carrier is finite and serialized locally and explicitly denies a global production queue bound. It validates exact framing error categories and rejects infrastructure failure as evidence; the inventory helper only observes lack of receipt/owner calls after an admitted handshake for its invalid frames. Its later healthy controls matter. These are different evidence strengths rather than interchangeable receipt labels. Active route checks reject out-of-scope routes and only retry absence of active path state.\n\n'
addition+='Current code has typed namespace, resource and leased-work contracts, so the historical June M1-only description is stale as a description of this checkout. The manifest/CLI still use the M1 rehearsal label. mesh-peer deliberately posts inline legacy work because it supplies no blob delivery, and declares DevicePolicy::unsupervised because it cannot heartbeat or cancel inline execution. Unexpected Grant/Heartbeat/Reclaim is a hard failure. This is an honest transport rehearsal boundary, not a complete leased host. Its connected console line still needs supplier guarantee review: registration/topic success is not independently observed readiness in this helper, although later sync counters and Done wait use actual state. No unreachable ticket was tested.\n\n'
addition+='Two provenance implementations have drifted: compile-time source catalogs still refer to deleted in-repository transport files, while the inventory Python wrapper knows how to resolve the imported supplier. The retained carrier also freezes an obsolete p2panda revision in success JSON. Source/executable/lock hashes elsewhere do not make that named field true. Runtime git observations and source metadata should remain separate from exact build-bound supplier identity. Paired runs compare a named semantic subset and explicitly exclude timing/actor observations, which is an honest repeatability scope. No current paired run exists because the example check gate fails.\n\n'
addition+='The persistent Source helper uses bounded lines/lifetime bytes, monotonic request/time identity, deadlines and Tokio kill-on-drop. The older one-shot peer helper uses std child plus three pipe threads; parent timeout precedes unbounded joins and early errors have no equivalent kill-on-drop guard. Descendant-held pipes or blocking-runtime consequences were not reproduced. A shared bounded IPC lifecycle, provenance schema binding and real host scheduling are larger follow-ups to document rather than prolong this audit with an implementation rewrite.\n\n'
addition+='Actual fresh locked/offline Cargo checks: inventory_ingress with admitted-ingress fails on2 missing includes; retained_receive fails on4 missing includes; the admitted-ingress library and mesh-peer positive control pass. Fresh Cargo metadata and the unchanged lock select p2panda1f69c9bc, while the report literal saysd8acabc9. The static mismatch is not a newly emitted false report; failed targets were not executed. No external peer, network, codec recovery, redb, child process campaign, user file or physical device ran. No owned runtime probe/output directory was created. Stable shared target reused; no isolated target, Cargo home or worktree. Evidence and source hashes are in the [Mesh examples review receipt](../../../support/receipts/authority-success-audit/mesh_examples_review_receipt.json).\n'
for a,b in [('all11','all 11'),('and114,742','and 114,742'),('is13/43','is 13/43'),('is1,176','is 1,176'),('and15,578','and 15,578'),('on2 missing','on 2 missing'),('on4 missing','on 4 missing'),('p2panda1f69','p2panda 1f69'),('saysd8ac','says d8ac')]:
 addition=addition.replace(a,b)
save('review_notes.json',notes);save('authority_map.json',authority)
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'mesh_examples_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
files=['inspect_mesh_example_evidence.py','mesh_examples_evidence_receipt.json','mesh_examples_metadata_stderr.log',
 'mesh_inventory_example_check.log','mesh_retained_example_check.log','mesh_example_positive_check.log','mesh_examples_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete Mesh manifest/examples reading; broader Mesh and full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'mesh_coverage':{'complete_files':13,'total_files':43,'complete_chars':sum(inventory[p]['chars'] for p in mesh_complete),'total_chars':sum(inventory[p]['chars'] for p in mesh_paths)},
 'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'inventory_example_check_exit':1,'inventory_missing_include_errors':2,'retained_example_check_exit':1,'retained_missing_include_errors':4,
 'admitted_library_and_mesh_peer_check_exit':0,'generated_runtime_output_created':False,'retained_isolation':[],
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in files},
 'limits':['Full Mere objective incomplete; no production/original example/test changes.',
  '12 new files read; Mesh13/43 complete including prior namespace. Compiler reach is not whole Mesh source coverage.',
  'Two actual example check failures, not passing gates; positive library/mesh-peer control passes.',
  'Frozen revision discrepancy source-proven against locked/resolved supplier; no newly emitted wrong report claimed.',
  'No example campaign, Retinue IPC, network/peer, RaptorQ recovery, redb reopen, native UI, worker/blob result or physical radio executed.',
  'IPC join/readiness/provenance concurrency concerns remain source-qualified candidates, not reproduced hangs or connectivity failures.',
  'Historical M1-only memory was checked against current source, not reported as current.',
  'Stable target reused; no isolated target/home/worktree or generated runtime probe/evidence directory.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(Path(p))==h for p,h in evidence['trace_sha256'].items())
save('mesh_examples_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'mesh_coverage':receipt['mesh_coverage'],'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
