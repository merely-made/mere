"""Persist the complete client source review and bounded public-API evidence."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n, value): (HERE/n).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
notes, authority = load('review_notes.json'), load('authority_map.json')
old = load('host_review_receipt.json')
probe = load('client_review_probe_receipt.json')
preserved = old['preserved_production_and_test_hashes']
assert all(sha(ROOT/p)==digest for p,digest in preserved.items())
assert probe['test_exit']==0 and probe['tests']==8 and probe['generated_native_output_removed']
assert all(sha(ROOT/p)==digest for p,digest in probe['source_sha256'].items())
assert load('pending_read.json')==[]
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
client_paths = [p for p in inventory if p.startswith('crates/graphshell/graphshell-client/')]
assert len(client_paths)==15
assert all(sha(ROOT/p)==inventory[p]['sha256']==coverage[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in client_paths)
def finding(id,title,source,related,evidence,scope,repair):
    return {'id':id,'status':'reproduced through current public client API; documented, not repaired','title':title,'source':source,'related_sources':related,
      'evidence':evidence,'scope':scope,'verification':'client_review_receipt.json','diagnostic':'client_review_probe_receipt.json',
      'repair_triage':{'disposition':'documented; active goal excludes production edits','reason':repair}}
findings = [
 finding('Q062','Presentation resolution accepts a different advertised resource and keeps requesting the missing one',
  'crates/graphshell/graphshell-client/src/core.rs:519',
  ['crates/graphshell/graphshell-client/src/core.rs:286','crates/graphshell/graphshell-client/src/lib.rs:369','crates/graphshell/graphshell-client/src/session.rs:273'],
  'Pending::Resolve remembers session/instance, not its resource request. on_response admits any response ClientState considers advertised and valid, then resolves again. A valid two-item fixture requests item 0; six replies containing the distinct advertised item 1 are accepted and cached, each producing another request for item 0 with the operation still pending. The requested-resource control resolves normally. The blocking adapter drive loop has no additional progress/retry bound. Pending::MediaResource does explicitly compare the requested session/hash.',
  'Current compiled public SessionCore/ClientState on a validated generated snapshot and content-addressed bytes. No actual network or browser denial-of-service run. Hash and session advertisement checks still work; this is reply correlation/liveness failure, not serving an unadvertised resource or foreign endpoint secret.',
  'Retain the outstanding ResourceRequest and reject a mismatching session/hash before caching or advancing. Distinguish successful additional resources from a repeated non-progressing answer, and test the same rejection through both adapters.'),
 finding('Q063','Frozen snapshot traversal ignores the explicit item order used by the remote reader',
  'crates/graphshell/graphshell-client/src/frozen.rs:232',
  ['crates/graphshell/graphshell-client/src/lib.rs:482','crates/graphshell/graphshell-client/src/session.rs:171'],
  'freeze_snapshot_with_details enumerates sparse table slots directly, while the remote accessibility tree and blocking resolve_all use active_items_in_order. A validated snapshot with item_order [1,0] is read [1,0] remotely and [0,1] frozen; dense-order control agrees. Sparse identity preservation is real, but does not require ignoring the explicit order. Existing sparse test covers tombstones/repeated sources with unreordered slots.',
  'Public native snapshot/freezing/accessibility data APIs. No live screen reader, browser DOM or OS accessibility bridge executed. Demonstrates reading-order divergence on valid source data, not renumbering of identities.',
  'Share ordered active-item traversal while retaining InstanceId values, and verify reordered/tombstoned snapshots across interactive, frozen and accessible outputs.'),
 finding('Q064','Remote accessibility exposes items whose own visible flag is false',
  'crates/graphshell/graphshell-client/src/lib.rs:482',
  ['crates/graphshell/graphshell-client/src/frozen.rs:283','crates/graphshell/graphshell-client/src/remote.rs:advertised_actions'],
  'accessibility_tree_with filters fold-hidden items but never checks item.visible. A validated two-item snapshot makes item 1 invisible; SceneSnapshot::is_shown is false and FrozenScene omits it, but the remote accessibility tree includes both items. Remote advertised_actions derives its surface from these children. Existing fold parity tests exercise fold hiding with all own flags true, rather than this branch.',
  'Current compiled public client functions, generated validated snapshot. No OS accessibility or product action execution. The item was already disclosed in the scene, so this is presentation/action-surface divergence, not a new secret-disclosure or endpoint authorization bypass.',
  'Apply the scene visibility rule consistently to flat accessible children and offered actions. Preserve explicit folded-member disclosure separately; agree on the precedence of an own visibility flag inside folds.'),
 finding('Q065','Unexpected remote closure leaves live mount status and a lost request can block rediscovery',
  'crates/graphshell/graphshell-client/src/remote.rs:418',
  ['crates/graphshell/graphshell-client/src/remote.rs:443','crates/graphshell/graphshell-client/src/remote.rs:474','crates/graphshell/graphshell-client/src/driver.rs:disconnect','ports/graphshell/src/web_rtc_link.rs:110','ports/graphshell/src/web_tree/remote.rs:278'],
  'RemoteSession::disconnect clears pending wire state and marks mounted scenes disconnected, but channel_closed only updates strings/link_busy. Diagnostic of a mounted idle session shows outer closed status with MountedScene.status still Live. During initial discovery, channel_closed leaves driver awaiting and in_flight true; rejoined then fails with still waiting on request and queues no discovery. Intentional-disconnect control marks the mount disconnected and sends rediscovery. Existing unexpected-close test checks only the outer status string. Targeted port source routes stream termination to this method without the intentional disconnect step.',
  'Public native RemoteSession plus encoded discovery/mount replies; targeted current port call-site excerpts inspected, full port review remains ahead. No actual browser channel teardown or DTLS rejoin executed. Reproduces lost-request recovery failure and contradictory client status, not a general claim that every reconnect fails.',
  'Give all terminal link paths a common state transition that cancels lost operations, clears or qualifies old queued traffic and marks mounts disconnected, while preserving the cached scene for re-admission. Exercise unexpected close and link failure both idle and mid-request; preserve the intentional disconnect control.')
]
findings.append(finding('Q066','Resume receipt calls a full replacement snapshot a diff',
 'crates/graphshell/graphshell-client/src/remote.rs:RemoteSession::finish',
 ['crates/graphshell/graphshell-client/src/core.rs:Pending::Resume','crates/graphshell/graphshell-client/src/remote/tests.rs:Board::answer'],
 'ClientState distinguishes Current, Applied and Resynchronize but SessionCore reduces Current/Applied to Outcome::Changed(true), without preserving whether the reply carried diffs or a full snapshot. RemoteSession labels any true result diff. A public mounted-session diagnostic receives a revision notice and a real ResumeReply::Snapshot at revision 2, applies the replacement, and reports diff · 1 → 2. The scripted board test likewise answers resumes with Snapshot while its test name/assertion says by diff.',
 'Current public RemoteSession driven by encoded NDJSON and validated generated snapshots; no browser or real network. Snapshot recovery itself succeeds correctly. This is false evidence about the recovery mode, not failure to apply a valid snapshot. The separate host real-endpoint test genuinely checks a diff; it was read in the preceding pass, not executed here.',
 'Preserve the actual resume mode through Outcome and render Current, Diff and Snapshot honestly, or use a generic updated label that makes no diff claim. Test snapshot and diff replies independently rather than asserting only the formatted string.'))
entries = [
 {'id':'A058','domain':'Outstanding resource request versus cache advertisement admission','claim':'One SessionCore drives resource resolution over blocking and line adapters.',
  'implementations':['Shared SessionCore is real protocol reuse','Pending::Resolve forgets its requested hash','ClientState accepts any valid advertised session resource','MediaResource path has a separate stricter request/response comparison'],
  'assessment':'Q062 separates cache admissibility from answering this request. A shared cache validator cannot supply missing response correlation or progress ownership.','state':'non-progress on repeated advertised wrong-resource replies reproduced','verification':'client_review_receipt.json'},
 {'id':'A059','domain':'Sparse instance identity versus authoritative reading order','claim':'The frozen form preserves the same scene identity and semantics as other readers.',
  'implementations':['Scene snapshot carries explicit item_order independently of table slots','Remote accessibility and retained adapter use active_items_in_order','Frozen snapshot reader directly enumerates table slots','Sparse test preserves identity but does not reorder'],
  'assessment':'Q063 shows duplicated traversal algorithms disagree despite shared snapshot storage. Preserving stable ids and honoring explicit order are compatible requirements.','state':'valid reordered snapshot gives different reader sequences','verification':'client_review_receipt.json'},
 {'id':'A060','domain':'Item visibility and fold disclosure across client readers','claim':'Every Graphshell reader interprets shared scene/fold facts consistently.',
  'implementations':['FoldReading supplies shared labels/counts/members','Remote flat accessibility filters fold effect only','Frozen reader filters own invisible flag except folded members','Snapshot is_shown combines own flag and fold hiding'],
  'assessment':'Q064 shows real fold vocabulary reuse with divergent final visibility admission. Fold-member display precedence and flat-item visibility are separate obligations.','state':'invisible un-folded item included remotely and excluded frozen','verification':'client_review_receipt.json'},
 {'id':'A061','domain':'Terminal transport state versus protocol pending state and mount status','claim':'RemoteSession owns common reconnect sequencing for all hosts.',
  'implementations':['Intentional disconnect calls driver.disconnect and clears RemoteOp','Unexpected channel_closed changes outer status/link_busy only','Driver request id and core pending operation survive that unexpected close','Port stream_closed calls channel_closed/link_failed'],
  'assessment':'Q065 shows differently owned lifecycle fields disagree after the same physical loss of a link. Source sharing across hosts propagates the omission rather than eliminating it.','state':'live mount and blocked rediscovery reproduced; actual channel teardown not run','verification':'client_review_receipt.json'},
 {'id':'A062','domain':'Shared resume state machine and retained compatibility algorithm','claim':'SessionCore owns protocol sequencing; session.rs is the blocking adapter.',
  'implementations':['RetainedEndpointSession drives SessionCore for its operations','Standalone resume_after_notice retains its own bounded request/apply/retry loop','resume_request_for_notice duplicates core resume entry logic','Both use ClientState.apply_resume and the same RESUME_ATTEMPTS constant'],
  'assessment':'This is an explicit compatibility implementation, documented as retained for direct-carrier hosts. Targeted current repository search finds no direct resume_after_notice caller, while resume_request_for_notice remains used by a host test. Track manual duplication as obsolete/unshared maintenance surface, not as a proven divergent runtime behavior or unused public API across external consumers.','state':'full source read; compatibility duplication and current in-repo caller search recorded','verification':'client_review_receipt.json'}
]
entries.append({'id':'A063','domain':'Resume reply kind, semantic outcome and diagnostic label','claim':'Remote receipts distinguish diff recovery from an already-current session.',
 'implementations':['ResumeReply carries Current, Diffs or Snapshot','ClientState.apply_resume reports application success','SessionCore reduces mode to a Changed bool','RemoteSession names true as diff; scripted test asserts that label after Snapshot'],
 'assessment':'Q066 is an evidence-authority loss: successful application is real, but the lossy outcome cannot justify the more specific displayed claim.','state':'Snapshot application reported as diff reproduced','verification':'client_review_receipt.json'})
def upsert(items,added,key):
    for entry in added:
        at = next((i for i,x in enumerate(items) if x.get(key)==entry[key]),None)
        if at is None: items.append(entry)
        else: items[at]=entry
upsert(notes['findings'],findings,'id')
upsert(authority['entries'],entries,'id')
notes['candidates'] = [x for x in notes['candidates'] if x.get('title') != 'Presentation resolve retries after applying a response without retaining the requested hash']
upsert(notes['candidates'],[
 {'title':'Frozen fold disclosure uses table order and can resurrect own-hidden members','source':'crates/graphshell/graphshell-client/src/frozen.rs:283','note':'freeze_parts places members in folded buckets while iterating table items, not in the fold members order, and explicitly includes own-invisible items when a fold hides them, contradicting the module statement that own-invisible items remain omitted. FoldReading.hidden retains the fold order. Need governing fold/visibility precedence and valid overlap inputs qualified before broad parity claims; Q063/Q064 independently establish ordinary ordered/flat visibility mismatches.'},
 {'title':'Revoked client resource admission and persistence remain separately callable','source':'crates/graphshell/graphshell-client/src/lib.rs:369','note':'mark_revoked purges cached resources conditionally; apply_resource has no status guard, and persist_session checks retention/time/protection rather than mounted status. Public mutable client APIs may intentionally rely on composing authority; trace delayed replies, persistence owners and retained-authority callers before claiming a post-revocation product leak or persisted-cache purge guarantee.'},
 {'title':'Resuming diffs unconditionally ends with Live despite a diff carrying a terminal status','source':'crates/graphshell/graphshell-client/src/lib.rs:599','note':'apply_resume clones/validates transactionally, applies each diff including its status, then sets final mounted status Live. Empty diffs also conclude Applied with the previous ack. Trace the ResumeReply producer/status contract and actual use before classifying this as a false authority-success path; pure ClientState does not authenticate a reply.'},
 {'title':'Remote failure does not retire all wire and outbox state','source':'crates/graphshell/graphshell-client/src/remote.rs:474','note':'fail clears RemoteOp and changes status/form strings but does not clear driver inflight, queued notices or outbox. link_failed calls it; unexpected close has the qualified Q065 diagnostic. Trace late callbacks and writer ownership before extending Q065 to failed-line replay or every failure path.'},
 {'title':'Opening the first bounded action can leave an older draft behind on failure','source':'crates/graphshell/graphshell-client/src/remote.rs:ActionForm::open_first_bounded','note':'Missing mount/tree/form updates status and returns without clearing previous draft/target. Public form transfer between RemoteSessions is intentional and documented. Qualify caller expectations and stale endpoint checks before claiming an unintended old-target invocation.'},
 {'title':'Remote object count counts sparse slots instead of active items','source':'crates/graphshell/graphshell-client/src/remote.rs:315','note':'label uses mounted.scene.tables.items.len(), including tombstones; scene exposes active_item_count. Current generated test scenes have no tombstones. Source-confirmed count overstatement on sparse input; defer a separate presentation-only diagnostic until consumer reporting review.'}
],'title')
upsert(notes['source_qualifications'],[
 {'area':'Graphshell client complete source','note':'All 15 inventoried files fully read at current hashes, including tests and manifests. One SessionCore genuinely serves retained and line adapters; cache bytes are session-scoped; unknown/unadvertised/hash/size-invalid resources are refused. Five new qualified defects are reproduced without network or production edits.'},
 {'area':'Frozen, portable capture and local action evidence','note':'Frozen source tests use real ScriptedDom parsing and taproot selector resolution, but explicitly say no driven live-app surface exists there. AccessKit tests traverse an in-memory UxTree, not an OS screen reader. Portable-projection shelfmark helpers are explicitly test-only host examples, not shipped verification. LocalActions intentionally executes a supplied host handler without sending endpoint traffic, including equal intent names; this is a typed local action boundary, not automatically an endpoint authorization bypass.'},
 {'area':'Client persistence and transient projections','note':'apply_diff and apply_resume use scratch copies to preserve old acknowledged state on invalid replay. Restore validates scene/resources and marks restored status Stale. EditableText offers must be memory-only and purge-on-revocation. Injected store protection is a host assertion; no real durable encrypted store, multi-instance lifecycle or delayed-reply revocation scenario ran this pass.'},
 {'area':'Graphshell remote action aggregation','note':'advertised_actions intentionally keeps the first target per intent and tests state one-per-intent rather than per card. Treat this as an explicit surface contract until consumer requirements establish otherwise. Board test resume returns a Snapshot despite one test name saying by diff; graphshell-host remote_session_live separately asserts an actual diff, source-read last pass, not executed here.'}
],'area')
if 'crates/graphshell/graphshell-client' not in notes['completed_areas']: notes['completed_areas'].append('crates/graphshell/graphshell-client')
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
next_path=next(p for p in sorted(inventory) if p.startswith('crates/chirograph/') and (coverage.get(p,{}).get('sha256')!=inventory[p]['sha256'] or coverage.get(p,{}).get('through',0)<inventory[p]['chars']))
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Shared Graphshell endpoint, host and client fully read; portable protocol/capture/codec next'
notes['next_area']='crates/chirograph, then remaining Mere source'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':['client_review_receipt.json'],'diagnostic_outcomes':'Fresh production client build passed. Eight public-API diagnostic tests passed while confirming five unresolved findings and three controls. No full client test suite or browser/OS accessibility gate.',
 'next_read':next_read,'workspace':'Reused C:/t/cargo-targets/mere; standalone client probe exe/PDB removed. No isolated target, Cargo home or worktree.'}
notes['audit_continuation']={'previous_turn':'progress: completed Graphshell host, Q056-Q061 and ten bounded diagnostics','classification_of_previous_turn':'progress',
 'production_edits_this_turn':False,'completed_areas_this_pass':['crates/graphshell/graphshell-client/'],'coverage_checkpoint':stats,'next_read':next_read,
 'user_instruction':'Continue full source audit, preserve concurrent work and do not change production code.','coverage_note':'Client source complete; whole repository incomplete. Diagnostic success means defects/controls reproduced, not repairs.'}
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes)
save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'client_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
doc='design_docs/mere_docs/technical_architecture/2026-10-15_transport_agnosticism_note.md'
evidence=['graphshell_client_build_output.jsonl','client_review_probe.rs','run_client_review_probe.py','client_review_probe_build_output.txt','client_review_probe_output.txt','client_review_probe_receipt.json']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'previous_turn_classification':'progress','kind':'Complete client source reading and bounded native public-API defect diagnostics; no repairs',
 'findings':[x['id'] for x in findings],'authority_entries':[x['id'] for x in entries],'complete_areas':['crates/graphshell/graphshell-client/'],
 'client_files':len(client_paths),'client_chars':sum(inventory[p]['chars'] for p in client_paths),'coverage':stats,'next_read':next_read,'production_edits_this_pass':False,
 'preserved_production_and_test_hashes':preserved,'client_source_sha256':{p:sha(ROOT/p) for p in client_paths},'governing_document_sha256':{doc:sha(ROOT/doc)},
 'build':{'exit':0,'command':'cargo build -p graphshell-client --lib --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --config build.build-dir=C:/t/cargo-targets/mere/build --message-format=json'},
 'diagnostic_test_count':8,'diagnostic_test_exit':probe['test_exit'],
 'coverage_limits':['Existing client unit/integration tests read, not run this pass.','No browser, network, live accessibility bridge or installed resident scenario.','Targeted port call sites do not count as full port source coverage.','No whole-repository completion claim.'],
 'generated_output_removed':probe['generated_native_output_removed'],'retained_isolation':[],'diff_check':{'exit':diff.returncode,'log':'client_review_diff_check.txt'},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence}}
save('client_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authorities':len(authority['entries']),'next_read':next_read,'preserved_hashes':len(preserved),'diff_check':diff.returncode}))
