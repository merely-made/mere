"""Complete current Graphshell source subtree and inspect its shared route seam."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
NAME='graphshell_completion_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert not (HERE/NAME).exists()
old=load('native_receipt_review_receipt.json')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert len(guards)==1419 and all(sha(ROOT/n)==d for n,d in guards.items())
assert all(sha(HERE/n)==d for n,d in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest()==old['document']['sha256']
assert load('pending_read.json')==[] and not (HERE/'pending_transition.json').exists()
inv,cov=load('source_inventory.json')['files'],load('read_coverage.json')
graphshell=[n for n in inv if n.startswith('ports/graphshell/')]
assert len(graphshell)==528 and sum(inv[n]['chars'] for n in graphshell)==4259545
assert all(cov.get(n,{}).get('sha256')==inv[n]['sha256'] and cov[n]['through']==inv[n]['chars'] for n in graphshell)
extra=['crates/cambium/mere-overmap/src/route.rs','crates/cambium/mere-overmap/src/route_session.rs']
whole=sorted(n for n in [*graphshell,*extra] if n not in guards
    and cov.get(n,{}).get('sha256')==inv[n]['sha256'] and cov[n]['through']==inv[n]['chars'])
assert len(whole)==43 and sum(inv[n]['chars'] for n in whole)==461503
assert all(sha(ROOT/n)==inv[n]['sha256'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1932 and stats['read_chars']==22761898
assert stats['total_files']==3727 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probe=load('chronicle_mount_probe_v3_receipt.json')
assert probe['build_exit']==probe['run_exit']==0 and probe['checks']==5 and probe['generated_output_removed']
assert sha(HERE/'chronicle_mount_probe_v3.rs')==probe['probe_sha256']
assert sha(HERE/'run_chronicle_mount_probe_v3.py')==probe['runner_sha256']
assert all(sha(ROOT/n)==d for n,d in probe['source_sha256'].items())
assert all(sha(Path(lib['path']))==lib['sha256'] for lib in probe['libraries'].values())
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-chronicle-mount-probe.*'))
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q280' and authority['entries'][-1]['id']=='A709'
rows=[
 ('Q281','G5 returns success after a peer closes before completing its request script',
  'ports/graphshell/src/bin/g5_peer/connect.rs:run_session/connect; g5_peer/main.rs:main',
  'source-only premature-success exit path',
  'run_session prints that the endpoint closed without answering, breaks the script and returns Ok with whatever responses arrived. An immediate EOF after first admission yields an empty vector; connect returns Ok before the interruption and remaining sessions. An EOF later likewise does not require the expected response count, ids or outcomes. Therefore exit zero alone does not establish the rehearsal was completed. A deliberate grant refusal is also an expected observation in this tool, so this finding concerns unexpected EOF and incomplete scripts, not the mere existence of a normal reported refusal.',
  'Separate observation mode from a qualification mode that asserts expected request ids, complete response sequences and declared granted/revoked outcomes. Reuse the existing SessionDriver or checked H6 request loop rather than another loosely checked wire loop.'),
 ('Q282','C4 concurrency coverage is claimed by a test that closes each visitor before opening the next',
  'ports/graphshell/tests/c4_host_signaling.rs:join_and_count/the_fixture_binary_serves_a_projection_over_its_own_signaling',
  'source-only acceptance-scope mismatch; no product serialization proved',
  'The test module says it proves a second visitor is served rather than queued behind the first. join_and_count sends Close, drops its line stream, awaits the pump and closes the carrier before returning. The test awaits that entire first call before beginning the second. Both visitors passing therefore proves repeat sequential admission, not overlapping live sessions; a host that serialized visitors until close could also satisfy this ordering. The host itself spawns service tasks, and this review does not establish that actual concurrency is broken.',
  'Keep the first mounted session open, mount/use a second concurrently within a bounded deadline, then verify both still serve independent requests before closing them.'),
 ('Q283','Native Mere viewer duplicates the shared route state machine and silently accepts absent scoped material',
  'ports/graphshell/src/bin/graphshell_mere.rs:Connection::read/take/spawn_worker; crates/cambium/mere-overmap/src/route_session.rs:Task::Resolve/Task::Media; route.rs:session_graph_request',
  'source-only live duplicated policy and demonstrated branch divergence',
  'The browser uses MereRouteSession with SessionDriver, and the shared type already implements the native LineDriven seam. The native binary independently sequences snapshots, resources, merge choices, application/session/epoch checks, saved view, archive state and intents through Connection and a worker. If an explicit session card lacks its scoped media, the shared route rejects it. The native session_graph_request helper returns None and Connection::read skips the entire validation block, retaining previous saved_view/read_only and returning a Reading. Thus sharing rendering and pure route helpers does not make validation or lifecycle single-owned. This is a source-path counterexample, not a live peer exploit or wrong-file-store mutation.',
  'Hairy seam documented: make the native transport drive the shared route state machine, retaining native wake/UI adaptation; specify current application/session/material identity and purge semantics. Do not extend the audit with a broad worker/connection rewrite. A bounded refusal for explicit missing material can be repaired separately.'),
 ('Q284','Chronicle receipt fields commit rejected snapshot and resume attempts as though applied',
  'ports/graphshell/src/distillery_w1.rs:ChronicleMount::mount/resume/receipt',
  'verified fresh current module over hash-bound retained dependency libraries',
  'mount sets snapshot_revision before ClientState accepts the snapshot; resume sets resumed_by_diff from the reply variant before apply_resume succeeds or reports Resynchronize. The public receipt then mixes those attempted values with the unchanged acknowledged scene. Three native counterexamples show wrong-session resume returning Err while resumed_by_diff=true at revision 1, a missing-base diff returning Resynchronize with the same false flag, and a rejected revision-999 snapshot stamped over acknowledged revision 1. A normal applied diff reaches revision 2 as its positive control. The shared ClientState correctly preserves the old acknowledged scene; the adapter bookkeeping creates the misleading report.',
  'Commit receipt metadata after successful application, distinguish resynchronization from applied replay, and retain attempt/error observations separately from acknowledged state.'),
 ('Q285','Chronicle accepts a self-validating resource different from the one it requested',
  'ports/graphshell/src/distillery_w1.rs:ChronicleMount::refresh_details; crates/graphshell/graphshell-client/src/core.rs:Pending::MediaResource',
  'verified fresh current module over hash-bound retained dependency libraries',
  'refresh_details checks response.has_valid_address but does not compare response.resource or response.session with its requested pair. A supplied endpoint can therefore return another correctly hashed PortableCard, and its values enter the requested instance frozen details. A native synthetic endpoint advertising PortableCard and returning a different self-validating card produces the substituted text in frozen_html. This parallels the previously reported shared presentation-resolution request-binding gap Q062; it is a separate direct endpoint adapter path, while Pending::MediaResource already checks scope, requested address and content hash. No actual remote admission or authorization bypass was exercised.',
  'Check both response scope and the exact requested hash before decoding, preferably through the shared checked resource path. A self-consistent content hash does not prove it is the requested content.'),
]
for ident,title,source,status,evidence,repair in rows:
 notes['findings'].append({'id':ident,'title':title,'source':source,'status':status,
  'evidence':evidence,'verification':NAME,
  'diagnostics':['chronicle_mount_probe_v3_receipt.json'] if ident in ['Q284','Q285'] else [],
  'repair_triage':{'disposition':'documented; no production changes','reason':repair}})
arows=[
 ('Native WebRTC fixture','C4 delegates admission, catalog routing and service to graphshell_host.',
  ['c4_webrtc_host.rs','admit_webrtc_catalog','ResidentProjectionHost'],
  'Loopback HTTP is fixture signaling with a fixed host-selected route, fresh host identity and bounded invitation terms. Chronicle/resident routes explicitly refuse nudge. ReleaseRef fixture identity is a signed claim, not executable trust. No fixture was launched.'),
 ('C4 qualification scope','Native str0m test drives real HTTP, door admission and mounted protocol state.',
  ['tests/c4_host_signaling.rs','SessionDriver'],
  'Source includes a foreign-invite reason check and real mounted card counts. Q282 limits the claimed concurrency proof to serial visitors. This is not a browser, TURN, physical input or current rerun receipt.'),
 ('Rehearsal identity copies','G5 and H6 duplicate environment hashing, grant issuance, profile and transport setup.',
  ['g5_peer/identity.rs','h6_transfer_peer/identity.rs','connect/dial/serve'],
  'These are two local rehearsal implementations, not two cryptographic admission owners: both call shared Personae/Insigne/Notochord/transport. Shared owner seed lets the client mint a grant; both disclaim real grant-delivery qualification. Their request validation has drifted.'),
 ('Rehearsal result authority','G5 primarily prints observations; H6 destination verifies its declared sequence.',
  ['g5_peer/connect.rs','h6_transfer_peer/destination.rs'],
  'Q281 incomplete G5 exchanges still exit zero. H6 checks request ids, result variants, addressed blobs, transferred graph facts, resumed acknowledgement and declared revocation outcome. H6 uses independent memory-backed destination state, not a durable reopen proof. Neither was run here.'),
 ('Native Mere route duplication','The native viewer and MereRouteSession independently orchestrate disclosed Mere state.',
  ['graphshell_mere::Connection','MereRouteSession','web_tree::remote'],
  'Q283 traces a real strict-refusal divergence. Pure route helpers are shared; native state sequencing, worker notices, saved focus/layout and merge material are separate. Shared LineDriven support makes this a consolidation seam, not a need for another transport-specific authority.'),
 ('Chronicle ownership','ChronicleMount receives a product-owned endpoint and applies its disclosed scene to ClientState.',
  ['distillery_w1.rs','distillery_chronicle_host.rs','Distillery ChronicleObserver'],
  'No duplicate job-board owner was found in the mount. Q284/Q285 are adapter bookkeeping and resource binding defects. Existing real-host test uses an in-memory carrier with shared admission, not a running Distillery daemon or cross-device proof.'),
 ('Chronicle evidence limits','The original current module was compiled fresh for five local checks.',
  ['chronicle_mount_probe_v3_receipt.json'],
  'One positive control, three rejected/unapplied-state counterexamples and one wrong-resource counterexample pass. Retained dependency libraries are exact bytes from a previous successful build, not a new full-current transitive Cargo gate. Initial path failure and a four-pass/one fixture failure remain preserved.'),
 ('Appearance source and storage','Tabard owns theme parsing/resolution; Graphshell owns its settings envelope and roles.',
  ['appearance.rs','web_appearance'],
  'Choice/import uses shared validation; accept writes candidate settings before replacing the valid state. Authored CSS stays last and defaults are explicit, while embedded rendering has its separate inherited-role path. These tests were read, not rerun.'),
 ('Frame period and budget','One FrameBudget estimates periods and supplies the shared canvas StepBudget.',
  ['frame_budget.rs','web_period_worker','web_speed'],
  'Recorded and planted tests cover loaded/missing/even worker intervals, coarse clocks, rate switches and fallback. Inference is bounded to 60-360 Hz and cannot infer hidden refreshes absent worker data. Source reading is not new display or timing measurement.'),
 ('Practice disclosure and reducer','Woodshed supplies comparison evidence; Graphshell validates its fixed literal disclosure.',
  ['practice_disclosure.rs','practice_workspace.rs','web_practice'],
  'Pitch partitions are recomputed only as consistency checks on the named method. Reducer retains full canonical evidence and source/revision, bounded history and runtime config; it does not compute new music material. Fixed source tuple is explicitly not authentication. No Woodshed product adoption or execution claimed.'),
 ('Projection comparison composition','Arrangement/Scope and Arrangement/Dynamics share compiler and facet composition.',
  ['projection_compare.rs','projection_dynamics_compare.rs','projection_dynamics_receipt.rs'],
  'Some family enumeration/grid assembly is repeated across distinct axis contracts, but motion comes from shared ProjectionDynamics. Snapshot cap and live working-cell state are separate; refused cells remain reported and nonworking scenes stay frozen. Raw float words are observations, not tolerance-based cross-platform proof by themselves.'),
 ('Query link projection','Shared query definitions mutate through the host journal; presented_links only groups display rows.',
  ['query_views.rs','query_view::save_spec/freeze_view','LinkKey/LinkMarks'],
  'Canonical assertion ids and graph scope deduplicate lifted rows; Surface/Resource strata remain separate, shown bindings are excluded, withdrawals do not produce resting strokes, and legacy traversal summaries are counted separately. This is a projection, not a second stored relation authority. Existing grouping tests were read, not rerun.'),
 ('Receipt generators and legacy reboot checker','Small CLIs call shared view/session/policy owners; the reboot script targets the legacy installation.',
  ['g1_receipt.rs','g4_sessions.rs','n4_policy_receipt.rs','h4f_agent_probe.rs','personae_vault.rs','verify-reboot-windows.ps1'],
  'Vault CLI delegates custody and checks explicit SSH enrollment confirmation; the probe verifies a real requested signature if executed. Reboot checker still names graphshell-device-host and Graphshell paths, matching retained legacy install references rather than proving modern Djinn restart. N4 uses PID-named temporary state; none of these effectful helpers was launched.'),
]
entries=[{'id':f'A{710+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,
 'state':'whole-source review with source/runtime limits','verification':NAME} for i,(d,c,im,a) in enumerate(arows)]
authority['entries'].extend(entries)
notes.setdefault('review_followups',[]).append({'verification':NAME,'area':'Mere generic record values',
 'detail':'route::add_node_facets prepends flattened owner facet fields to PortableCard values, while both native and shared saved-focus code search the first label Address. Inspect the typed owner view contract and current consumer tests before promoting a field-label collision to a live wrong-focus finding. No probe or owner action was performed.'})
next_read={'path':'crates/cambium/mere-overmap/','start':0,
 'reason':'Finish shared Mere-overmap source/tests and continue all remaining Mere source. Upstream fetch succeeded without changes; merge feasibility checked next.'}
notes['current_area']='Complete inventoried Graphshell subtree plus shared Mere route helper/state machine'
notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress',
 'goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,'chronicle_mount_probe_receipt.json','chronicle_mount_probe_v2_receipt.json','chronicle_mount_probe_v3_receipt.json'],
 'production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Fresh original module probe: initial include-path compile failure retained; v2 has four passing checks and a fixture advertisement mistake; distinct v3 passes five checks. All generated executables/PDB removed.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
assert diff.returncode==0,diff.stdout+diff.stderr
with (HERE/'graphshell_completion_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_graphshell_completion_review.py','graphshell_completion_diff_check.txt']
for suffix in ['','_v2','_v3']:
 evidence.extend(['chronicle_mount_probe'+suffix+ext for ext in ['.rs','_compile.log','_receipt.json']])
 evidence.append('run_chronicle_mount_probe'+suffix+'.py')
 if suffix:evidence.append('chronicle_mount_probe'+suffix+'_output.log')
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget<100_000_000
addition=f'''

### Complete Graphshell subtree and shared route seam, 2026-10-17

At {head}, every one of the 528 inventoried files / 4,259,545 characters under ports/graphshell is now completely read. This pass adds 41 native source/scenario/test/helper files, plus the two shared Mere-overmap route modules: 43 files / 461,503 characters. Current full coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. All prior 1,419 protected source paths, twelve evidence hashes and the previous document prefix remain preserved. The full Mere review remains incomplete.

Q281 is source-only: G5 returns Ok after premature EOF, including before the second/third session are attempted. Logs can reveal the incomplete exchange, but exit zero cannot qualify the declared full rehearsal. H6's copied request loop is stricter: it checks ids, variants, hashes, transfer facts and expected revocation. Both reuse shared cryptographic owners and explicitly disclose the self-issued rehearsal grant; they do not qualify real grant distribution.

Q282 is a test-coverage contradiction: C4's module says it proves a second visitor is served rather than queued behind the first, but its first helper closes the session/carrier before the second helper starts. It proves sequential reuse. The actual host spawns service tasks; no production serialization defect is claimed. Q283 is a more substantial live duplication: native graphshell_mere::Connection independently sequences the same route/material/view/merge state that MereRouteSession owns for the browser, despite the shared adapter already supporting LineDriven native pumps. Their validation differs: an explicit session card without scoped media is refused by the shared adapter, while native read skips validation and retains earlier saved-view/read-only fields. This is source-only; broader consolidation is documented as a larger seam rather than implemented during the audit.

Q284 and Q285 have scoped native counterexamples. The original current distillery_w1.rs module is compiled fresh by path against exact retained library bytes from a successful previous build. Normal replay advances revision 1 to 2. A refused wrong-session diff and an unapplied missing-base diff leave acknowledgement at 1 but stamp resumed_by_diff=true. A refused malformed snapshot stamps snapshot_revision=999 over acknowledged revision 1. The shared ClientState correctly preserves the old scene; Chronicle's adapter mutates its receipt fields too early. Separately, a correctly self-hashed but unrequested PortableCard enters requested-instance frozen details because refresh_details checks content hash validity without comparing the requested scope/address. These are adapter defects, not proof of remote admission bypass or a second Distillery job-board authority.

The first diagnostic compile failed on an audit include-path mistake. Its receipt/source/log remain. V2 confirmed four observations, then failed because the substitution fixture advertised NativeGlyph and never exercised the PortableCard-only path. V3 corrects that audit advertisement and passes one positive control plus four counterexamples. Every generated probe EXE/PDB was removed. No full-current transitive Cargo build, original module test suite, Distillery daemon, native app, browser, GPU, network or physical rehearsal was run.

A710-A722 track real reuse, repeated setup and distinct projection contracts. Tabard definitions are shared; application role/settings mapping is local and storage precedes state acceptance. FrameBudget has one worker/main inference owner and explicit fallback bounds. Practice preserves source-bound canonical disclosure and bounded navigation rather than recomputing product music. Arrangement and Dynamics matrices repeat some assembly but use shared compilers, dynamics and facet composition. Query links are a display projection over canonical assertions with explicit legacy traversal summaries, not a second stored ledger. Small receipt generators delegate shared views; personae-vault delegates custody. The reboot helper is still tied to the named legacy Graphshell installation and does not qualify current Djinn restart.

One follow-up remains qualified rather than promoted: flattened facet values are prepended to PortableCard fields, while saved-focus paths find the first Address label. The typed owner contract and consumer tests need inspection before asserting a live wrong-focus outcome. Source reading uncovered the possible collision; no owner action or data mutation was performed.

No production or original test changes, isolated Cargo target/home or worktree were created. The earlier shared Mere build handles are now absent. Fresh git fetch succeeds and origin/main remains 175c4148, twelve ahead of this nine-ahead local main. The 42 upstream-changed paths overlap neither dirty tracked paths nor untracked paths; merge feasibility can now be checked without moving shared edits aside. This [Graphshell checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds complete reading coverage, five findings, thirteen ownership entries and diagnostic limits. Goal remains active.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,
 'prior_receipt':{'path':'native_receipt_review_receipt.json','sha256':sha(HERE/'native_receipt_review_receipt.json')},
 'kind':'Complete current Graphshell source subtree and shared route seam; full Mere incomplete',
 'coverage':stats,'whole_current_files':whole,'whole_current_chars':461503,
 'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,
 'completed_subtree':{'prefix':'ports/graphshell/','whole_files':528,'chars':4259545},
 'new_findings':[r[0] for r in rows],'authority_entries':[e['id'] for e in entries],
 'current_module_native_checks_passed':5,'prior_failed_attempts_preserved':True,
 'production_edits_this_pass':False,'pending_verification':None,'next_read':next_read,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),
   'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'retained_isolation':[],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Three new findings source-only; two current adapter paths compiled against exact retained dependency bytes, not a new full-current transitive gate. No actual product/carrier/browser/physical run.'}
save(NAME,receipt)
print(json.dumps({'receipt':NAME,'coverage':stats,'additional_files':len(whole),'completed_graphshell_files':528,
 'findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'retained_bytes':budget},indent=2))
