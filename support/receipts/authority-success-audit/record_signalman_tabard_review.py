"""Save complete Signalman and Tabard port reads without changing product code."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='signalman_tabard_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('pelt_completion_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
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
i=load('source_inventory.json')['files'];c=load('read_coverage.json');paths=[p for p in i if p.startswith(('ports/signalman/','ports/tabard/'))]
assert len(paths)==30 and sum(i[p]['chars'] for p in paths)==277225
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in paths)
assert load('pending_read.json')==[];sources={p:sha(ROOT/p) for p in paths}
probe=load('signalman_owner_probe_receipt.json');assert probe['exit']==0 and probe['passed']==4
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==2 and all(not Path(p['path']).exists() for p in probe['removed_outputs'])
assert not (HERE/'cpu-signalman-runtime/counter-fixture').exists()
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q416' and authority['entries'][-1]['id']=='A1253'
new=[
 ('Q417','First-owner counter reservations can successfully reuse a spent counter under concurrent calls',
  'ports/signalman/src/bin/mere-signalman-first-owner.rs:154-199',
  'reserve_next_counter reads last_used, computes next, then writes/syncs/renames a fixed sibling temporary file without serializing callers. Its never-reoffered claim and test cover only sequential calls. An unchanged-body probe schedules two actual filesystem reads of last_used=0, lets A write/sync/rename and return1, then lets B continue with its already-read snapshot. B also writes/syncs/renames and returns1; disk remains1. Both successful callers can offer the same controller counter. Shared temporary-file collision is not needed for this failure.',
  'Serialize the read/advance/durable commit across processes for each controller scope, or use an existing atomic reservation owner. Keep the lock over the complete operation, not only rename. Test concurrent reservation and interruption separately. Existing malformed-record and overflow checks remain real controls; board replay refusal prevents duplicate execution but cannot make both controller requests succeed.',
  'One extracted-body concurrency test with real filesystem/serde/sync/rename. A filesystem wrapper delays only return from actual reads to select a valid interleaving; it does not replace disk results or edit the reservation body. No board, radio, actual CLI process race or power-cut durability test.'),
 ('Q418','Host-side station stop and successful revoke cannot promptly interrupt an in-flight event wait',
  'ports/signalman/src/lib.rs:538-553,560-577; ports/signalman/src/lease.rs:166-180',
  'next_event owns the station Tokio mutex while awaiting the radio under the previously accepted grant window. stop closes the lease and aborts the watchdog, then waits for that same mutex; revoke likewise closes after successful authority revocation and waits. Neither signals the pending operation. Exact next_event/stop/revoke bodies with an hour-long accepted window and pending fake radio show both stop and successful revoke exceeding a10ms virtual timeout while the lease is already closed and the event task still owns the station. Cancelling that event task lets stop finish. Idle-stop control completes. The comment that host stop is immediate is therefore stronger than this cancellation implementation.',
  'Give the station runtime one cancellation owner that wakes in-flight operations when the local lease closes, then drops the radio without waiting for the old grant window. Cover sends, event waits and queueing, plus revocation errors, under the same boundary. This is a host-lifetime fix; remote delivery and board receipt remain separate.',
  'Two exact-method CPU cancellation cases and one idle-stop control, with actual Tokio mutex/time/timeout and paused virtual time. Radio, lease/authority success, clock and watchdog are deterministic doubles. No actual Postilion station, Djinn revoke, serial task or physical RF cessation was executed.')]
for ident,title,source,evidence,repair,scope in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,'status':'reproduced exact source-extracted host behavior; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['signalman_owner_probe_receipt.json']})
rows=[
 ('Wallet signing authority has a real custodian seam','StationAuthority has one production Djinn implementation; TestWallet is cfg(test) only.',
  ['ports/signalman/src/authority.rs','ports/signalman/src/lib.rs'],
  'Signalman requests scoped RetainedKeys and asks Djinn to sign/record grants and revoke. It opens no wallet. Station salts, controller salts and storage salts reuse Personae rather than duplicating derivation. Grant errors map pending/locked/no-wallet to WalletLocked. The production caller is still synchronous thread-per-request; invoking from async does not make that join nonblocking. TestWallet duplicates persistence setup only as an explicit custodian double.'),
 ('Connection custody and delegated station authority remain distinct lifetimes','Sennet and Tucket share StationCustodyLease; older Reticulum controller and station APIs retain other owner boundaries.',
  ['ports/signalman/src/authority/sennet.rs','ports/signalman/src/authority/tucket.rs','ports/signalman/src/bin/mere-signalman-first-owner.rs','ports/signalman/src/lease.rs','ports/signalman/src/head.rs'],
  'The Tucket alias is not a second watch implementation. Both checked opens verify master/scope and retain sticky lock/loss state with per-turn live/status checks. Public raw release APIs and separately held material leave guard retention to callers. Reticulum controller gets retained material without this watch; host station uses public roster/grant expiry, and unattended head deliberately owns a sealed delegate. Do not apply desktop-wallet lock semantics to unattended remote authority without a policy ruling. Watch registration/readiness and lock/unlock races remain followups.'),
 ('Replay reservation belongs to one durable scope owner','The CLI promises a counter is spent durably before board exchange; the library accepts caller-supplied counter.',
  ['ports/signalman/src/bin/mere-signalman-first-owner.rs','ports/signalman/src/lib.rs'],
  'Q417 is missing serialization in the CLI reservation, not a second board verifier. The library explicitly gives persistence to its caller. Init uses create_new for scope and refuses replacement; status loads existing scope, reserves before opening USB, and verifies signed status. Parent-directory crash durability and scope/counter pairing are separate untested followups.'),
 ('Control verification and persistence have explicit separate responsibilities','Receiver validates device/key/issuer/signature/expiry; head persists transitions before returning acknowledgement.',
  ['ports/signalman/src/control.rs','ports/signalman/src/head.rs'],
  'Actual controls include bounded strict-CBOR wire decoding, station signature plus command/device correlation in ack, identical-grant retry, strictly extending same-issuer renewal, permanent closure and revocation. Snapshot decoding is local trusted storage. Direct receiver callers must persist before sending ack; head wraps that requirement. transition clones prior receiver and rolls back memory after save failure. The effect of persistence refusal on a requested closure, independent restored heads for one path and competing provision calls needs failure/concurrency qualification; source read alone does not prove durable single ownership.'),
 ('Two station runtime wrappers duplicate operation lifetime machinery','SitedStation uses host grant/roster lease; RunningSitedStationHead uses sealed receiver and change watch.',
  ['ports/signalman/src/lib.rs','ports/signalman/src/head.rs','ports/signalman/src/lease.rs'],
  'Both keep Station behind an async mutex and implement parallel address/announce/send/event/management/stop/watchdog paths. Policies differ legitimately, but cancellation is siloed: head selects changes; host wrapper only old-deadline timeout, producing Q418. Both authorize before awaiting mutex; head subscribes after authorize and creates a relative timer after lock acquisition. Queued admission, change-before-subscribe and lock hold times need one shared runtime admission/cancellation boundary with separate grant-policy adapters. Full supplier/physical timing not tested here.'),
 ('Provisioning and custody receipts distinguish local acknowledgement from physical commissioning','Provision writes a sealed bench head, sends control locally, verifies ack and restores before success.',
  ['ports/signalman/src/bin/mere-signalman-provision.rs','ports/signalman/examples/sennet_custody_receipt.rs','ports/signalman/examples/tucket_custody_receipt.rs','ports/signalman/tests/dr_c_pending.rs'],
  'Provision has no physical transport call and prints local station identity only after local restore/authorize. Existing-head refusal precedes issue but is a check followed by replace-capable provision, requiring a concurrency owner. Scoped receipt examples check wrong-app release/sign/attest refusal, reconnect and lock/loss; they are not executed here. watch accepts any custody closure cause, so result custody_closed cannot prove a particular lock action. The ignored pending test requires an external keeper and does not establish current live custody evidence.'),
 ('Tabard host uses shared product and producer owners','Desktop retains WorkshopState and delegates scene production to shared SceneProducer.',
  ['ports/tabard/desktop/src/lib.rs','ports/tabard/desktop/src/reader.rs','ports/tabard/desktop/src/main.rs'],
  'No separate theme editing model is introduced. Each frame rebinds current Reader/Stylesheet source, mounts graph leaf, and registers bounded producer keys. FocusedTextSlot maps retained data-field to existing product text. Export destination belongs to host while validation/artifact/collision replacement stays in WorkshopState. Default library is product-local and explicit OS-string path bypasses discovery. Load errors happen before window construction. Close requests route to unsaved-work policy; exit_requested remains product-owned.'),
 ('Tabard receipt finalization has a stronger completion gate than a successful event loop','Scripted run requires completion Some(true), optional receipt and native example Done phase.',
  ['ports/tabard/desktop/src/lib.rs','ports/tabard/desktop/examples/native_shared_recovery.rs','ports/tabard/desktop/src/recovery_control.rs'],
  'Scripted completion is checked after host run, then receipt text is checked. This rules out simple event-loop-Ok => accepted-scenario claim. Recovery example independently asserts same owner/core, host isolation, physical image keys, fresh stamps/sequences, exact application Rc/state, whole-frame/producer pixels, native hide/show/resize, and injected IME path; it requires Done and preserves outputs with create_new. Controlled Device::destroy is explicitly qualified, and input is programmatic host injection rather than physical keyboard. No native example executed.'),
 ('Two controlled recovery qualifiers repeat capture/authority policy','Single-host RecoveryControl and two-independent-host example own separate checks and thresholds.',
  ['ports/tabard/desktop/src/recovery_control.rs','ports/tabard/desktop/examples/native_shared_recovery.rs','ports/tabard/desktop/scenarios/native_device_recovery.scn'],
  'Thresholds RGB3/alpha2/mean1 align, but identity/readiness, timeouts, baseline bounds and pixel helpers are separate. Example verifies nonblank/nonzero32MiB extents; single-host inspect checks len and cap but lacks explicit nonzero extent. Do not infer a real zero-size presented success without tracing shared capture admission. Native example caps individual frames, not total retained RGBA bytes; full output budget must be set by the lane owner. Source-only qualification, not a new GPU result.'),
 ('Scenario and harness acceptance stay separate from native/physical evidence','All desktop scenarios and original tests are completely read; none executed in this pass.',
  ['ports/tabard/desktop/scenarios/','ports/tabard/desktop/tests/usability.rs','ports/tabard/desktop/tests/workshop_routing.rs'],
  'Source checks actual product export capture vs later draft, collision-safe import, failed save/close preservation, caption routing and narrow wheel reachability. Harness substitutes destinations/file chooser and injects keys/clicks. Scenarios assert semantic state and named captures across fresh processes, but are scripts until run with current source/lock/binary receipts. Recovery device trigger is opt-in. Complete source reading does not establish current native/GPU/AT acceptance.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1254):
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'full current-local Signalman and Tabard port source read; ownership migrations and acceptance limits explicit','verification':NAME})
notes['review_followups'].extend([
 {'area':'Signalman runtime authority admission and persistence','detail':'Track older host-lease vs sealed-head duplicated runtime machinery, queued operations checked before async mutex, relative timeouts started after queueing, change-watch subscription gaps, failed closure persistence rollback, independent restores/provisioning of one record, and public raw released material lifetime. Q417/Q418 have bounded CPU reproductions; broader migration needs policy and supplier/transport evidence.','verification':NAME},
 {'area':'Tabard controlled recovery qualifier duplication','detail':'Align common capture/authority validation without collapsing distinct one-host and independent-two-host scopes. Inspect shared capture nonzero admission before asserting a zero-frame success; bind full output retention budget at lane level. Programmatic host input remains separate from physical input and AT.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2983 and stats['read_chars']==34930170 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Signalman13 and Tabard desktop17 files','files':30,'chars':277225,'verification':NAME,'basis_commit':head,'qualification':'Full current-local source/manifests/scenarios/tests/examples read.4 exact-method CPU cases with real filesystem/Tokio and explicit radio/authority/clock/watchdog doubles. No complete original suites, Djinn/serial/RF/native/GPU/physical input/AT runs.'})
next_read={'path':'scripts/check_port_boundaries.py','start':0,'reason':'Continue all remaining scripts, support receipt drivers and supplier patches/source; retain full source denominator.'}
pending='No audit-owned process running.4 CPU cases pass;2 exact run-owned exe/PDB removed from actual Cargo build/debug/deps. All Signalman and Tabard inventoried source read;755 source files remain. Fresh git fetch succeeded and origin/main unchanged; eight reviewed upstream commits still uninstalled because shared report overlaps. No production changes or isolation.'
notes['current_area']='Signalman and Tabard port source complete; scripts next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'signalman_owner_probe_receipt.json'])
addition=f'''

### Signalman and Tabard port source review, 2026-10-17

All13 Signalman files (168,279 characters) and all17 Tabard desktop files (108,946 characters) are completely read. Overall coverage: {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale/pending credit. The prior truncated Signalman emission was discarded without acknowledgement and fully reread. Fresh fetch succeeded: local/upstream remain {head}/{upstream}, eight previously reviewed upstream commits uninstalled because the shared report overlaps. The full audit remains incomplete, with755 source files still to read.

Q417: first-owner counter reservation has no serialization across the read/advance/sync/rename sequence. Two callers can both read0, then each successfully commit and return1. The unchanged-body probe uses actual filesystem writes, sync and rename, delaying only read returns to select this valid interleaving. This disproves the unqualified never-reoffered claim under concurrency; no actual board replay or power-cut test ran. Q418: host-side next_event holds its station mutex throughout the radio wait. stop and successful revoke close the lease, then wait behind that same operation without signalling it. Exact method bodies with real Tokio and a pending radio double both exceed the short virtual-time timeout; an idle-stop control completes. This is host cancellation evidence, not a physical RF cessation result.

A1254-A1263 record the ownership picture. Wallet signing stays in Djinn through a real trait seam, and Sennet/Tucket now share one sticky custody watch. Older controller, host-grant and unattended sealed-head lifetimes still differ. SitedStation and RunningSitedStationHead duplicate the station operation/watchdog machinery, with different cancellation paths. Queued authorization before mutex acquisition, change-watch subscription order, persistence rollback on closure, and multiple owners restoring/provisioning one record remain documented followups requiring broader concurrency/policy evidence. Control verification has real device/key/issuer/signature/expiry checks, and the head persists before returning an acknowledgement; those controls are not erased by the unresolved lifetime migration.

Tabard's desktop host uses WorkshopState and the shared SceneProducer directly, routes text/export/close decisions to existing product owners, and refuses incomplete scripted scenarios after the event loop. Its separate one-host and independent-two-host recovery qualifiers share thresholds but repeat capture/authority validation. The native example checks real owner/image/presentation/state/pixel identities and requires Done; its injected IME is programmatic input, and controlled Device::destroy is explicitly narrower than spontaneous driver loss. All scenarios and original tests were read, not executed. No current native/GPU/physical input/AT acceptance is claimed.

Four audit CPU cases pass. Two exact run-owned exe/PDB outputs were verified and removed; the initial cleanup lookup used legacy debug/deps, was corrected to Cargo's actual build/debug/deps, and is recorded. No production edit, separate target/home/worktree or live audit process. Shared reusable compiler dependencies remain. Prior report bytes, all protected sources/shared WIP and index are preserved. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [CPU probes](../../../support/receipts/authority-success-audit/signalman_owner_probe_receipt.json). Continue scripts and support suppliers.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_signalman_tabard_review.py','signalman_owner_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'pelt_completion_review_receipt.json','sha256':sha(HERE/'pelt_completion_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[v[0] for v in new],'authority_entries':ids,'all_signalman_inventory_source_read':True,'all_tabard_port_inventory_source_read':True,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.30 source files277225 chars;4 exact-method CPU cases with real filesystem/Tokio and explicit host doubles. No full product suites, radio/board/serial/native/GPU/physical input/AT.2 owned binaries removed; no isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
