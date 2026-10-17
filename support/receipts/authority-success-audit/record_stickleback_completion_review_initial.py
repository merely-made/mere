"""Close exact Stickleback read coverage; preserve all earlier audit evidence."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
NAME = 'stickleback_completion_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def git(*args): return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
def save(n, value):
    with (HERE / n).open('x', encoding='utf-8') as f: json.dump(value, f, indent=2); f.write('\n')
assert not (HERE / NAME).exists()
old = load('stickleback_drop_review_receipt.json'); head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}; assert len(guards) == 2146
aux, runtime, shared = old['auxiliary_read_sha256'], old['runtime_input_sha256'], old['shared_working_file_sha256']
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime, **old['compile_source_sha256']}.items())
assert all(sha(HERE / p) == h for p, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']; prefix = doc.read_bytes(); assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute(): index = ROOT / index
index_hash = sha(index); assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2632 and stats['read_chars'] == 31223256
assert not stats['changed_since_inventory'] and load('pending_read.json') == []
all_files = [p for p in inv['files'] if p.startswith('crates/stickleback/')]
fresh = [p for p in all_files if p not in guards]
assert len(all_files) == 30 and len(fresh) == 16 and sum(inv['files'][p]['chars'] for p in fresh) == 295739
assert all(cov[p]['through'] == inv['files'][p]['chars'] and cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in all_files)
probes = {n: load(n + '_receipt.json') for n in ['stickleback_group_retry_probe', 'stickleback_prune_erase_probe']}
for n, probe in probes.items():
    assert all(v['exit'] == 0 for v in probe['results'].values()) and probe['original_tests'] == 0
    assert probe['runner_sha256'] == sha(HERE / ('run_' + n + '.py')) and probe['probe_sha256'] == sha(HERE / (n + '.rs'))
    assert probe['generated_output_removed'] and all(not Path(v['path']).exists() for v in probe['removed_outputs'])
    assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
    assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['libraries'].values())
assert [p['audit_cases'] for p in probes.values()] == [3, 2]
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q379' and authority['entries'][-1]['id'] == 'A1116'
findings = [
 {'id': 'Q380', 'title': 'A failed group-session save followed by same-session retry settles an update that was never persisted',
  'source': 'crates/stickleback/src/group_lane.rs:543-548,549,647-651',
  'status': 'reproduced through public lane/session APIs over freshly compiled complete current owner',
  'evidence': 'Real identities and genuine signed lane operations establish two group members. Applying an update advances the receiver, then an injected persistence callback returns Err. Retrying with that same live session returns success, classifies the update superseded and never calls its new save callback. Reopening the old saved session against the same lane skips the settled update and cannot decrypt the new epoch. Successful-first-save and reload-before-retry controls each save once, survive reopen and decrypt.',
  'scope': probes['stickleback_group_retry_probe']['qualification'], 'verification': NAME,
  'diagnostics': ['stickleback_group_retry_probe_receipt.json'],
  'repair_triage': {'disposition': 'bounded persistence-ordering repair candidate; active audit remains read-only',
    'reason': 'A settled marker must not outlive the session transition it certifies. Make failure rollback/reload obligations explicit, or require persistence before settling transitions already applied in memory. The documented same-session retry currently loses this obligation; preserve both restart controls.'}},
 {'id': 'Q381', 'title': 'Separate single and batch retention implementations disagree when pruning also erases a prefix payload',
  'source': 'crates/stickleback/src/store.rs:663,746-778; crates/stickleback/src/processor.rs:260,368',
  'status': 'reproduced through public processor and native-drop APIs over freshly compiled complete current owner',
  'evidence': 'A neutral policy explicitly authorizes a sequence2 checkpoint to prune two predecessors and erase the first payload. The single-operation path reports pruned_entries2/erased_payloads1 and leaves one operation pointer, but its log still contains the first signed header without a body plus the checkpoint. Prefix deletion is followed by a Put recreating that old log key. The native-drop batch applies the identical three signed operations under the same effects policy and leaves only the checkpoint, because its overlay retains the preceding deletion decision.',
  'scope': probes['stickleback_prune_erase_probe']['qualification'], 'verification': NAME,
  'diagnostics': ['stickleback_prune_erase_probe_receipt.json'],
  'repair_triage': {'disposition': 'bounded duplicate effect-planning repair candidate; no destruction policy chosen',
    'reason': 'Use a shared effect planner or otherwise make prefix deletion dominate payload stripping in both paths. Verify log keys, operation pointers and reported removal counts together for overlapping effects. This is a mechanics discrepancy under an explicitly supplied policy, not a ruling on what a domain may erase.'}}
]
notes['findings'].extend(findings)
b = 'crates/stickleback/src/'
rows = [
 ('Group-key lane admission shares signed-log mechanics but not all governance authority', 'GroupKeyPolicy invokes common structural processing and binds group, control, direct frames and Personae attestation.', [b+'group_lane.rs', b+'group_session.rs'], 'Session prekey registration explicitly requires host eligibility. Authenticated authorship and group addressing do not by themselves prove a constitutional role. Actual removed-member/control eligibility still needs upstream DCGKA and consumer tracing; no unauthorized-member exploit is claimed.'),
 ('A settled group frame certifies a separately persisted session transition', 'drain_ready mutates the session, calls persistence only for newly applied frames, then writes settled markers.', [b+'group_lane.rs', b+'group_session.rs'], 'Q380 proves the failed-save/same-live-session retry loses the persistence obligation. The function documentation explicitly permits that retry. Restarting from the durable old session before retry is a passing control, not the behavior of the failing same-session path.'),
 ('Session transition atomicity is local to the in-memory clone', 'Most GroupSession mutation entrances clone serialized state and install it after a successful transition.', [b+'group_session.rs'], 'This protects against partial local transition failure. It does not make host durable session state atomic with lane markers. Version/group/recipient/root bindings and strict carrier CBOR are real checks; secret serialized snapshots require sealed host storage and stayed only in memory in this diagnostic.'),
 ('Parked record release converts every processing error into an unpark decision', 'release_parked uses is_ok_and on process_batch_atomic and deletes the parked key after a non-insert result.', [b+'group_lane.rs', b+'processor.rs'], 'Source candidate: storage errors are collapsed alongside permanent continuity refusal, so a later successful delete may discard retryable data. Fault-injected storage execution has not run. Keep transient failure distinct from an authenticated permanent refusal before claiming safe release.'),
 ('Inventory summaries are bounded advisory metadata', 'One immutable snapshot supports a deterministic Cuckoo summary or exact pagination with digest/revision binding.', [b+'inventory/filter.rs', b+'inventory/mod.rs', b+'inventory/peers.rs', b+'inventory/tests.rs', b+'inventory/wire.rs'], 'Positive probabilistic membership may suppress useful offers; exact reconciliation is explicit. Saturation discards the tentative filter and falls back to exact pages. Clock, peer, entry, byte and candidate ceilings are real mechanics. Caller owns authenticated peer/namespace access, bytes, expiry and admission; complete inventory is not custody or completed delivery. Original tests were read, not freshly run.'),
 ('JoinedSpace shares the join ceremony and explicit shutdown lifetime', 'Join retains the session actor, live handle and shared drain; leave_and_wait aborts and awaits captured-store release.', [b+'joined_space.rs', b+'synced_space.rs'], 'Dropping aborts in field order but does not await completion; leave_and_wait is the stronger reopen boundary. lane_id canonicalizes a supplied kind/space, but caller-supplied lane IDs can still collide at the endpoint. Keep endpoint identity uniqueness distinct from shared ceremony code.'),
 ('Opaque stream completion is transport framing rather than domain acceptance', 'read_object checks prefix limit before allocation and requires EOF; one deadline covers all I/O including write shutdown.', [b+'object_stream.rs'], 'An application acknowledgement travels on the reverse direction under host ownership. Successful write/shutdown alone certifies neither decoding nor admission. Original source tests include pending FIN, slow reads, truncation and reverse acknowledgement, but were not freshly run.'),
 ('Single and batch storage effects remain separate implementations inside the shared adapter', 'The single path uses an ordered Vec of writes while batch import has a BTreeMap overlay and planned-operation index.', [b+'processor.rs', b+'store.rs'], 'Q381 proves observable divergence for overlapping prefix prune and payload erasure. A shared crate and shared Backend transaction are insufficient to make these two effect planners one authority. Batch pointer absence rechecks do not compare-and-swap the per-log frontier.'),
 ('Frontier serialization has a narrower boundary than the shared-store claim', 'process and process_with_writes take the author/log lock shared by MunimentStore clones.', [b+'processor.rs', b+'store.rs', b+'drop_io.rs', b+'group_lane.rs'], 'process_batch_atomic itself acquires no equivalent corpus locks; drop import and parked release call it directly. Independently constructed stores over a cloned backend also get independent lock tables. Document controlled concurrency followup, not an executed concurrent-fork acceptance claim. Backend atomic writes do not make prior frontier reads one transaction.'),
 ('Typed and AnyOperation log views share bytes while repeating traversal logic', 'Both LogStore implementations decode the same header/body blobs, but latest/heights/size/stream/prune logic remains duplicated.', [b+'store.rs'], 'These are compatibility views over one storage schema rather than separate graph truths. Their repeated logic is a maintenance seam. Public low-level store insertion and erasure helpers remain available, so policy-before-write is an ingress convention, not enforced by every store API. Q199 already documents the signed-cache/parsed-extension inconsistency; this pass does not count it again.'),
 ('Portable receipts certify an import transaction, not peer trust or durable custody', 'DropReceipt checks strict semantic CBOR plus an integrity-framed body; ReceiptPeer derives a caller-supplied authenticated identity scope.', [b+'receipt.rs', b+'drop_io.rs'], 'Receipt bytes do not authenticate their sender. Caller binds them to a carrier session and retention policy. Readers accept one declared frame rather than require generic reader EOF; object_stream has a stronger one-object EOF contract. No peer receipt is a local domain-admission authority.'),
 ('Quiet sync counters do not establish convergence', 'resync samples the accepted-operation counter until three quiet polls; SyncRound documents zero as already up to date.', [b+'synced_space.rs', b+'joined_space.rs'], 'Source proves only no accepted arrivals during that window. It does not consult syncing, peer connectivity, refusal or stream error state. drive skips subscription errors and receives only a boolean admission result. Overlapping round status and actual UI interpretation need consumer/network tracing; no live convergence diagnostic ran.'),
 ('Boundary fixtures demonstrate composition under chosen calls, not universal enforcement', 'The neutral ledger tests explicitly compose policy, authority and public native-drop processing.', ['crates/stickleback/tests/boundary.rs', b+'prune_proof.rs', b+'writer.rs'], 'The local/received fixture calls process twice and never drives a received transport. Its no-second-insertion-path wording is broader than that proof because raw store APIs remain public. The checkpoint fixture checks its caller-owned authority before separately invoking prune; the shared crate does not call that trait automatically. Writer binding resolves direct/attested subject identity and leaves membership permissions to domains.'),
 ('Stickleback source completion and diagnostic scope are separate receipts', 'All30inventory files and496048characters are source read and hash matched; five new public audit cases execute.', ['crates/stickleback/Cargo.toml', b+'lib.rs'], 'Sixteen newly completed files add295739characters. Both whole unchanged owners compile over17exact retained suppliers. Original tests0, live transport0, host data loss0, no production edits. Temporary owner libraries/executables/PDB/link outputs were marker checked and removed. Repository review remains incomplete.'),
 ('Turnstone ownership migration is a concrete consumer followup', 'with_graph_pane resolves pane graph and installs/stashes camera/selection, while GraphRuntimePool still Derefs to its active canvas.', ['../turnstone/src/app/runtime_pool.rs:385-453', '../turnstone/src/app/mod.rs:719-758', '../turnstone/src/app/node_arms.rs:1532-1589'], 'Targeted external source only, no Mere read credit or new Turnstone bug receipt. open_address visits through with_graph_pane but then resolves the returned graph-local node key through graph_runtimes.graph(), which uses the active cursor. graph_pane_focused_member also falls back from a pane selection result to shared canvas selection. Trace topology rebind, first-frame initialization and every caller before treating migration complete; user-mentioned Arrange/cross-window/restore failures were not independently reproduced in this pass.')
]
entries = [{'id': f'A{1117+i}', 'domain': domain, 'claim': claim, 'implementations': paths,
    'assessment': assessment, 'state': 'Stickleback source complete; repository review active', 'verification': NAME}
    for i, (domain, claim, paths, assessment) in enumerate(rows)]
authority['entries'].extend(entries)
completed = [{'prefix': p, 'files': 1, 'chars': inv['files'][p]['chars'], 'basis_commit': head, 'verification': NAME,
    'qualification': 'Whole file read and ACKed. All30Stickleback inventory files are now source complete; this does not extend to all consumers or original test execution.'} for p in fresh]
notes['completed_source_areas'].extend(completed)
followups = [
 ('Group persistence', 'Q380: preserve durable session/settled-marker ordering through failed save, same-session retry and reopen. No production repair in active read-only audit.'),
 ('Retention planner duplication', 'Q381: consolidate mechanics or deletion dominance without choosing governance/destruction semantics. Compare single and batch observable log/pointer state under overlapping effects.'),
 ('Parked release and ingress concurrency', 'Fault-inject backend apply failure during release_parked; verify staged data survives retry. Control batch-versus-single and independent store wrappers over the same backend before a concurrency finding. Storage quota/race/ownership redesign may exceed one hour; keep documented instead of delaying all-source reading.'),
 ('Sync observation wording', 'Treat resync zero as zero observed accepted arrivals, not proven up to date. Trace UI consumers, refusal visibility and overlapping rounds before stronger convergence claims.'),
 ('Turnstone migration priority supplied by user', 'Finish explicit graph/pane ownership through entire operations, including post-with_graph_pane lookups and pane selection reads. Audit active-cursor/Deref callers; distinguish Frisket topology, blueprint projection and Workspace ownership. User-mentioned Arrange/cross-window/first-frame failures are steering, not fresh reproductions. Full migration is broader than a one-hour audit fix and remains documented; no Turnstone source edits or read-coverage credit.')]
notes['review_followups'].extend({'area': area, 'detail': detail, 'verification': NAME} for area, detail in followups)
next_read = {'path': 'crates/system/luggage/Cargo.toml', 'start': 0, 'reason': 'Continue full-source inventory through Luggage release/signing/staging/install/update ownership and success paths.'}
notes['current_area'] = 'Stickleback30of30source files complete; group save retry and duplicate retention effects reproduced'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full repository source incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, *[n+'_receipt.json' for n in probes]], 'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'No owned process live. Remaining repository source unread. Original Stickleback suites, fault-injected parked release, ingress concurrency, live transport/convergence and full Turnstone ownership migration unexecuted. Earlier Q352 and metadata-timeout qualifications remain intact.',
    'diagnostic_outcomes': 'Fresh whole owners compile;3group retry cases and2single-versus-batch retention cases pass their diagnostic assertions. Original tests0. Generated owner libraries/consumer binaries/PDB/link outputs removed.'}
diff = subprocess.run(['git','diff','--check'], cwd=ROOT, capture_output=True, text=True); assert diff.returncode == 0
(HERE / 'stickleback_completion_diff_check.txt').write_text(diff.stdout+diff.stderr, encoding='utf-8')
relation = git('rev-list', '--left-right', '--count', 'HEAD...origin/main'); assert relation.split() == ['0','0']
turnstone = ROOT.parent / 'turnstone'
external = {p: sha(turnstone/p) for p in ['src/app/runtime_pool.rs','src/app/mod.rs','src/app/node_arms.rs']}
external_head = subprocess.check_output(['git','rev-parse','HEAD'], cwd=turnstone, text=True).strip()
addition = f'''

### Stickleback completed source review and ownership followup, 2026-10-17

Read the remaining16Stickleback files /295,739characters at unchanged Mere HEAD {head}. All30Stickleback inventory files /496,048characters are now fully read and hash matched. Repository coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no pending/stale credit; the full audit remains active and incomplete. Fetch succeeds and HEAD matches origin/main at this checkpoint.

Q380 reproduces a persistence retry losing the group session transition: a real signed update advances the live session, the save callback returns an injected refusal, then retrying that same session returns success and marks the update superseded without invoking save. Reopening the old saved session against the same lane skips the settled update and cannot decrypt the new epoch. Successful-first-save and reload-before-retry controls each persist once and decrypt after reopen. Secret state stayed only in memory. This is a save-callback failure case, not real disk failure or a demonstrated host data-loss incident.

Q381 reproduces divergent retained state from two effect planners in the shared store. An explicitly authorized checkpoint both prunes two predecessors and erases the first payload. The single-operation path reports2pruned/1erased but recreates the first signed header as a bodyless log entry without its operation pointer; the subsequent payload-strip Put overwrites a prior Delete. The native-drop batch applies the identical signed corpus under the same policy and leaves only the checkpoint. Its overlay handles deletion correctly. Shared crate ownership and atomic Backend writes have not removed this duplicated implementation. Repair concerns effect composition, not a new destruction-policy choice.

A1117-A1131 distinguish cryptographic group identity from domain membership authority; in-memory transition atomicity from durable marker ordering; parked-release swallowed errors from permanent refusal; bounded advisory inventory from custody; shared join lifetime from endpoint lane-ID uniqueness and awaited shutdown; transport EOF from application acknowledgement; single/batch effect duplication; process-local clone-shared locks from batch/frontier atomicity; typed/Any compatibility views over one schema; portable completion receipts from peer trust; quiet accepted-arrival counters from convergence; and neutral boundary tests from universal ingress enforcement. Q199's cached-header/parsed-extension split is retained as an existing finding, not recounted. Parked storage faults, batch concurrency and live convergence are documented followups, not executed conclusions.

User steering prioritizes unfinished graph/pane ownership migration over file splitting. Targeted external Turnstone source at {external_head} confirms with_graph_pane resolves the graph and installs/stashes pane camera/selection, while the pool's Deref still reads the active compatibility cursor. open_address visits through that boundary and then looks up the returned graph-local node key through the compatibility cursor; selection reads also retain a shared-canvas fallback. These are external source followups, with no Mere read credit, full Turnstone review, new bug reproduction or edits. User-mentioned Arrange/cross-window/first-frame failures are context rather than freshly executed evidence. Sustained migration work should carry explicit ownership through the entire operation and distinguish Frisket topology, blueprint projection and Workspace state; broad work remains documented under the audit's hour rule.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [three group retry cases](../../../support/receipts/authority-success-audit/stickleback_group_retry_probe_receipt.json), [two retention path cases](../../../support/receipts/authority-success-audit/stickleback_prune_erase_probe_receipt.json). Both complete unchanged owners and public consumers compiled over17exact retained suppliers; original tests0, live network0. Generated libraries/executables/PDB/link outputs removed. Prior evidence, protected/runtime/auxiliary hashes, shared WIP and Git index remain intact. Previous report bytes remain an exact prefix; diff check passes. No isolated target,Cargo home or worktree retained. Next source: Luggage manifest and release/install/update implementation.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
for n, value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
evidence = ['record_stickleback_completion_review.py','stickleback_completion_diff_check.txt']
for n in probes: evidence += ['run_'+n+'.py', n+'.rs', n+'_receipt.json', n+'_owner_compile.log', n+'_consumer_compile.log', n+'_output.log']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'origin_relation_after_fetch': relation,
 'kind': 'all30Stickleback source files read; five bounded public cases; full repository incomplete',
 'prior_receipt': {'path':'stickleback_drop_review_receipt.json','sha256':sha(HERE/'stickleback_drop_review_receipt.json')},
 'coverage': stats, 'completed_source_areas': completed, 'whole_current_files':fresh, 'source_sha256':{p:sha(ROOT/p) for p in fresh},
 'preserved_production_and_test_hashes': guards, 'auxiliary_read_sha256':aux, 'runtime_input_sha256':runtime, 'shared_working_file_sha256':shared,
 'external_targeted_source': {'root':str(turnstone),'basis_commit':external_head,'source_sha256':external,'qualification':'Targeted spans only; not full external file review, Mere read credit or executed Turnstone bugs.'},
 'new_findings':['Q380','Q381'], 'authority_entries':[e['id'] for e in entries], 'audit_authored_production_edits_this_pass':False,
 'diagnostics':{n+'_receipt.json':{'exits':{k:v['exit'] for k,v in p['results'].items()},'audit_cases':p['audit_cases'],'original_tests':0} for n,p in probes.items()},
 'pending_verification':notes['checkpoint']['pending_verification'],'next_read':next_read,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':index_hash,
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'retained_isolation':[],
 'limits':'No production changes. Whole-owner compilation uses exact retained dependency family; no fresh Cargo graph, original suite, live transport or full consumer migration proof. All prior qualifications preserved.'}
save(NAME,receipt)
assert len({**guards,**receipt['source_sha256']}) == 2162
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/p)==h for p,h in {**guards,**receipt['source_sha256'],**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
assert all(sha(HERE/p)==h for p,h in receipt['evidence_sha256'].items())
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],
 'audit_cases':5,'original_tests':0,'protected_sources':2162,'retained_bytes':budget,'next_read':next_read},indent=2))
