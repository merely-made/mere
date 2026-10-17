"""Checkpoint selected Burn remote client/session source; transport review remains open."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='remote_session_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('retained_burn_references_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256']);runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256'];external=dict(old['external_context_sha256'])
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(Path(p))==h for p,h in external.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json')
paths=[p for p in i if p.startswith('support/patches/burn-remote/') and c.get(p,{}).get('through',0)>0]
complete=[p for p in paths if c[p]['through']==i[p]['chars']];partial=[p for p in paths if p not in complete]
assert len(complete)==39 and partial==['support/patches/burn-remote/src/transport/error.rs'] and c[partial[0]]['through']==2038
assert sum(c[p]['through'] for p in paths)==311447
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] for p in paths)
assert load('pending_read.json')==[];sources={p:sha(ROOT/p) for p in paths}
probe=load('remote_client_ownership_probe_receipt.json');assert probe['exit']==0 and probe['passed']==12
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert all(not Path(p['path']).exists() for p in probe['removed_outputs'])
external.update(probe['external_context_sha256']);assert all(sha(Path(p))==h for p,h in external.items())
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i)
assert all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q426' and authority['entries'][-1]['id']=='A1297'
new=[
 ('Q427','Selected Burn remote server-survival test accepts failed read/value assertions as success',
  'support/patches/burn-remote/src/lib.rs:finishes_within,test_client_disconnect_handled_cleanly_by_server',
  'The test opens a new remote client after abrupt disconnect, then expects a tensor read and asserts values[30,60] inside finishes_within. That helper catches and discards any panic and sends completion unconditionally; the outer assertion checks only completion. Exact unchanged helper returns true for a synthetic wrong-result assertion panic, just as for an empty successful body. A blocked-body control returns false and is then explicitly released. A connection/read/value failure can therefore leave this server-survival test green.',
  'Keep panic-tolerant completion only for tests whose exact contract is failure surfacing without a hang. For server-survival/correctness, return the worker Result or panic payload and propagate it after the timeout; retain timeout refusal separately. Lease-close no-hang tests intentionally accept error/panic and need different helpers or explicit outcomes.',
  'Three CPU helper cases execute the unchanged std-thread helper. No remote server, disconnection, real failed tensor result or full Cargo integration test executed. This proves test admission weakness, not that the current server actually fails after disconnect.'),
 ('Q428','Selected Burn remote shares process-local transfer counters across unrelated server sessions',
  'support/patches/burn-remote/src/client/runner.rs:get_next_transfer_id,change_backend_local; src/shared/task.rs:LocalTransferId; src/server/local_comm.rs:expose,take; src/server/worker.rs:ExposeTensorLocal,RegisterTensorLocal',
  'Each independently linked client process starts its static LocalTransferId counter at0. One SessionManager shares a LocalCommService across all its sessions, keyed only by LocalTransferId. expose unconditionally inserts and overwrites an existing id; take removes by that id without source/client/target session validation. Two module-local copies of the unchanged counter simulate independent process state and both issue0. Full unchanged LocalCommService on actual Tokio receives payload111 from source session100, then222 from source200 under0; take for the first id returns222 and empties the map. Distinct-id and source-purge controls preserve the correct payloads. Worker wire handlers pass the id directly with no namespace translation.',
  'Give same-host transfer identity a shared scope agreed by source and target sessions, or use a server-issued/random capability bound to the intended participants. Reject duplicate exposure rather than replacing a live payload. Validate the taker and make missing/cancelled source terminal so a target cannot indefinitely prevent owner reclaim. This changes a cross-session/wire ownership contract; document before choosing the protocol migration.',
  'Three CPU rendezvous cases compile full unchanged LocalCommService and exact counter functions against existing actual Tokio. Tensor payload is an i32 backend double, SessionId a u64 double, and two module-local statics model separate processes. No real multi-process/server/transport/credential/GPU tensor run. The demonstrated overwrite/wrong payload can cause a second target to wait with no entry; an actual close/reclaim hang and cross-credential deployment disclosure are source-level effects to qualify separately.')]
for ident,title,source,evidence_text,repair,scope in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence_text,'status':'reproduced unchanged selected-patch helper/registry behavior with explicit IR/process-state doubles; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['remote_client_ownership_probe_receipt.json']})
P='support/patches/burn-remote/src/'
rows=[
 ('Selected remote patch has an explicit client/service/task ownership split','RemoteClient delegates to DeviceHandle; RemoteService owns the session and batch; writer/demux own network tasks.',
  [P+'client/base.rs',P+'client/runner.rs',P+'client/service.rs',P+'client/runtime.rs'],
  'Complete client/server/common source read, with transport still pending. Native init defers connection off CubeCL global registry lock and connects on device runner using Burn-owned runtime; native async connect uses spawn_blocking. Browser connect captures a plan, awaits outside service and installs back. CustomOpClient joins the same router/fusion authority instead of allocating mismatched output ids. Wrapper submission is not remote execution acknowledgement.'),
 ('Endpoint identity is shared while dialing hints are mutable','Registry keys peer/application endpoint/credential/device index and refreshes hints without changing id.',
  [P+'client/service/conn.rs',P+'client/service/registry.rs',P+'client/host.rs',P+'credential.rs'],
  'Registry caps IDs atU16 before narrowing and drops mutex before panic, so the earlier unchecked-narrowing suspicion is not a finding. Settings/device count are OnceLocks from handshake; a disconnected session retains these cells and writer presence. Credential/endpoint Debug redact bytes. Native listing connects device0 then returns others lazily; browser listing connects all. Host device_index usize narrows toU32 before request; oversized caller index remains a followup, not current ordinary enumeration failure.'),
 ('Pending-response disconnect is one atomic ownership decision','Registration and disconnect share one mutex; disconnected callbacks are immediately closed.',
  [P+'client/service/pending.rs',P+'client/service.rs',P+'client/service/writer.rs'],
  'Six unchanged PendingResponses tests run against actual existing Tokio and pass, including disconnect draining current callbacks and refusing later registration. Response decode errors log/continue; writer serialization/send failure drops batches with warnings. Writer failure does not itself signal PendingResponses; response-demux termination supplies that state change. Requests can lack bounded deadlines and try_connect treats writer Some as connected even after failure. These are lifetime/error propagation followups, not evidence successful native sync ignored a returned error.'),
 ('Browser sync returns queue-flush success, not a device completion barrier','Wasm sync flushes and returns Ok immediately; native sync awaits a response.',
  [P+'client/service.rs',P+'client/runner.rs'],
  'The source explicitly documents the difference. Browser profile_start refuses because sync cannot measure a server-completion window, preserving unmeasured rather than fabricating timing. Profile tokens map to opening client stream; unknown/double close produces no measurement, server error resolves None with log. Callers must not promote wasm sync Ok into confirmed server execution or reclaim. No browser-hosted execution in this slice.'),
 ('One transport-neutral pump owns handshake admission and teardown','Both transports are intended to supply frame halves to drive_session and SessionService.',
  [P+'server/pump.rs',P+'server/service/base.rs',P+'server/service/mod.rs'],
  'Pump parses one Init, reserves before application authorization, authorizes before device disclosure, then binds. Duplicate reservation fails before another session can be torn down; unrelated Close and repeated Init are rejected. This production duplicate-id refusal limits the earlier suspected dual-installed browser demux scenario: both concurrent plans share one session id, so two successful concurrent handshakes are not established. Admit comment claiming no state exists before authorization is stale after reservation migration. Pump tests use fake service/frames; source-read only, not run here.'),
 ('Server session completion distinguishes release from requested closure','SessionManager retains a reserved/bound/finishing record, watch completion and worker acknowledgement.',
  [P+'server/session.rs',P+'server/worker.rs',P+'server/spawn.rs'],
  'close_session waits for Clean/Failed, not merely sending a close signal. Worker teardown syncs, drops interpreter, runs memory cleanup, waits after release and returns failure; failed sessions stay visible and later closes report failure. Fault-injection test source checks this but was not run. Pending local/remote transfer waits are not cancellation-aware inside process_task; closing queues alone may not release the worker, so reclaim under a missing transfer remains a high-effect followup. Async response tasks are aborted on native panic, while normal close/untracked remote expose tasks have separate lifetimes to qualify.'),
 ('Ordering ownership migrated to one FIFO while SessionManager prose still claims stream parallelism','Actual SessionHandler drives every task in global submission order on one worker.',
  [P+'server/session.rs',P+'server/worker.rs'],
  'SessionManager docs describe a dispatcher with per-stream workers and independent streams progressing while one blocks. Worker source explicitly rejects that model because cross-stream tensor producer/consumer order must hold; actual process_tasks awaits one task at a time. StreamId only selects backend context. This is a false current concurrency claim, not a hidden second dispatcher. Separate sessions can progress, but one session blocking at local take or collective blocks all its streams. Correct the canonical explanation when implementation changes are allowed.'),
 ('Same-host transfer migration stops at the client process boundary','Client counter is process-unique; shared server rendezvous needs a cross-session namespace.',
  [P+'client/runner.rs',P+'shared/task.rs',P+'server/local_comm.rs',P+'server/worker.rs'],
  'Q428 proves collision/overwrite with independent counter state and actual Tokio registry. Source-session tag is used only for cleanup, not as part of key or take admission. Notify interest is enabled before map check, which correctly avoids the check/await lost-wakeup gap. Source purge removes parked handles but supplies no terminal missing-transfer result to waiters. Random target-bound cross-server capability is a different ownership model; do not silently equate local ID with that capability.'),
 ('Completion-only test helper is duplicated across incompatible test claims','Two local finishes_within helpers catch panics; one is used for survival correctness and one for lease-close no-hang.',
  [P+'lib.rs'],
  'Q427 isolates the misuse in server-survival test. Lease targeted-close test deliberately accepts a disconnected read failure as a non-hang outcome, so that test is not falsely labelled successful tensor execution. Other fusion/source/clone tests directly assert reads and values, including source-only and surviving-clone graph cases. All those integration test sources were read; none were executed in this audit.'),
 ('Telemetry is a best-effort projection of submitted/completed events, not an execution receipt','Broadcast drops lag; logger folds graph payload estimates into reported traffic savings.',
  [P+'telemetry.rs',P+'metrics.rs',P+'client/service.rs',P+'server/worker.rs'],
  'Client graph events emit before submitting tasks; remote transfer Completed emits after source readback/exposure, before target download completion. Async subscription recv skips lag without telling the aggregator, while drain_into reports lag only for Open and discards its count on Closed. TrafficAggregator keys graph cost byGraphId and ignores SessionClosed, retains cost entries, and cannot reconstruct missed registrations; serialization length errors become0. Logged network savings cover serialized op/bindings estimates, not actual transport framing/retries/delivery. This is an intentionally best-effort view whose labels must not be used as a single execution/accounting authority.'),
 ('Application authorizer is shared across transport sessions while transport authenticates identity','ServerSettings stores one authorizer and backend-typed custom-op registry.',
  [P+'server/authorize.rs',P+'server/backend.rs',P+'server/settings.rs',P+'server/logging.rs',P+'runtime.rs'],
  'AllowAll/default and plaintext WebSocket token limitation are explicit. TokenAuthorizer rejects empty credential, stores BLAKE3 digest and uses hash equality; raw credential type Debug redacts. ServedSession derives Debug on credential bytes, leaving an inconsistent debug-redaction seam to trace at consumers. Async server leaves subscriber to application; blocking server installs once. sessions_of sets process-global RuntimeKind::Async, which is wider than one server lifetime and should remain visible as a shared runtime policy effect.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1298):
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'selected patch client/session/common source read; transport/tests remain pending; CPU evidence scoped','verification':NAME})
notes['review_followups'].extend([
 {'area':'Same-host remote transfer authority and reclaim','detail':'Q428 is a process/server namespace mismatch, with wrong-payload overwrite reproduced. Scope transfer capability/participant identity across source and target sessions; reject duplicate exposure and qualify cancellation/terminal failure during close. This is a wire/lifetime design issue, documented without production changes.','verification':NAME},
 {'area':'Remote success and projection semantics','detail':'Q427 survival test catches its own correctness panics. Wasm sync only flushes; telemetry events and traffic estimates are best effort and can predate actual execution/download. SessionManager per-stream prose contradicts actual global FIFO. Preserve these distinctions when examining remaining transports and end-to-end tests.','verification':NAME},
 {'area':'Remote session lifetime edges','detail':'Trace response decode-error request liveness, dead writer reconnect/OnceLock settings, owner close during stalled transfer, normal-close response drain, detached remote-expose task lifetime, browser concurrent connect loser and ServedSession credential Debug. Some suspicions are narrowed: duplicate-id reservation rejects a second handshake, and registry checks U16 bounds before DeviceId narrowing.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3174 and stats['read_chars']==36247055 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'support/patches/burn-remote client/server/shared/telemetry plus manifests','files':39,'chars':sum(i[p]['chars'] for p in complete),'partial_read':{p:c[p]['through'] for p in partial},'verification':NAME,'basis_commit':head,'qualification':'39 full files plus2038 chars of transport/error.rs;311447 chars this slice.12 CPU cases include6 unchanged pending-response tests against actual Tokio,3 unchanged completion helper cases and3 actual rendezvous cases with explicit IR/process-state doubles. Remaining transport/integration sources not claimed complete.'})
next_read={'path':partial[0],'start':2038,'reason':'Finish Burn remote transport and integration test source, then remaining supplier patches/support/workflows. Full repository audit remains active.'}
pending='No audit-owned process running.12 CPU cases pass with explicit extraction/IR/process-state scopes.3174/3738 source files read;564 remain, including19 incomplete Burn remote files. All3738 live source membership/hashes unchanged. No production edit, generated binary, isolation, live server or GPU/network test.'
notes['current_area']='Selected Burn remote client/session/common complete; transport and integration source pending';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'remote_client_ownership_probe_receipt.json'])
addition=f'''

### Selected Burn remote client and session ownership review, 2026-10-17

The39 complete files in this slice cover both manifests, client, service/batching/response correlation, server pump/session/worker, wire definitions and telemetry. An additional2,038 characters of transport/error.rs are read, with its remainder explicitly pending. This slice is311,447 characters. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters;564 files remain incomplete. All3,738 live source membership/hashes are unchanged. Burn remote remains root-selected; its transport and integration test source review is unfinished.

Q427: the server-survival test catches failed read/value assertion panics and still returns a passing completion outcome. Exact helper CPU probes distinguish panic, successful completion and bounded blocking; the actual server was not run. A separately duplicated helper in lease-close tests intentionally checks only absence of a hang, so its error/panic outcome is not called a correctness witness.

Q428: process-local transfer counters feed a shared server map keyed only by that id. Two independent counter states both issue0; the unchanged registry on actual Tokio lets a second session overwrite the first payload, then returns that second payload for the first id. Distinct-id and source-purge controls pass. Tensor payload/backend and session identity are explicit CPU doubles, and the two statics model separate processes. No real cross-process/server/GPU/credential disclosure trial occurred. The source handlers supply no additional namespace/participant check. A missing second payload can also leave transfer/reclaim waiting; that full-session effect remains to qualify. Repair needs a shared transfer/participant contract and failure lifetime, so it is documented as an ownership issue without changing production source.

A1298-A1308 track the real thin-client/service/task split, endpoint/credential identity, response disconnect protection, shared admission pump, teardown acknowledgement and projection boundaries. Native sync awaits its response; browser sync returns flush success immediately and profiling explicitly refuses. SessionManager prose still claims per-stream workers although the actual worker is one global FIFO. Telemetry may precede execution or target download, can lose events, and estimates payload savings rather than observed wire delivery. None of those views should substitute for a confirmed execution or reclaim receipt. Source review narrowed two suspicions: registry admission bounds compact IDs, and duplicate session reservation prevents two successful simultaneous bindings with one id.

Twelve CPU cases pass, including6 unchanged PendingResponses tests using actual existing Tokio. Two owned executable/PDB outputs were verified and removed. Prior report bytes, protected production/shared files and index remain preserved; no audit process, isolated target/home/worktree or live server remains. Retained evidence is below100MB. Local/upstream remain {head}/{upstream}. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [CPU ownership probes](../../../support/receipts/authority-success-audit/remote_client_ownership_probe_receipt.json). Continue at transport/error.rs character2,038.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_remote_session_review.py','remote_client_ownership_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'retained_burn_references_receipt.json','sha256':sha(HERE/'retained_burn_references_receipt.json')},
 'coverage':stats,'source_sha256':sources,'read_through':{p:c[p]['through'] for p in paths},'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[v[0] for v in new],'authority_entries':ids,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'all_inventory_live_membership_and_hashes_verified':{'files':len(i),'added':[],'removed':[],'changed':[]},
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only:39 Burn remote files complete plus2038-char transport partial.12 CPU cases using actual Tokio and explicit doubles; no actual remote/GPU/server run. Two owned exe/PDB outputs removed. Full live source membership/hashes unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
