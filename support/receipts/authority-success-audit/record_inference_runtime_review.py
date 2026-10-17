"""One-shot inference runtime audit checkpoint; retain pending test identity."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'inference_runtime_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
assert not (HERE / NAME).exists()
old = load('resident_routes_personal_facts_review_receipt.json')
probe = load('inference_crash_probe_receipt.json')
assert probe['tests'] == 3 and probe['test_exit'] == 0 and probe['generated_output_removed']
assert load('inference_crash_probe_import_cleanup.json')['remaining_probe_outputs'] == []
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-inference-crash-probe.*'))
whole = ['ports/djinn/tests/inference_route.rs', 'ports/djinn/tests/inference_gpu_reclaim.rs',
    'crates/intel/esp/src/infer/actor.rs', 'ports/distillery/src/inference.rs',
    'ports/distillery/src/inference/durable.rs', 'ports/distillery/src/inference/gpu.rs',
    'ports/distillery/src/inference/recall.rs', 'ports/distillery/src/inference_stage.rs',
    'ports/distillery/src/inference_stage/fallback.rs', 'ports/distillery/src/inference_stage/lane.rs',
    'ports/distillery/src/inference_stage/discovery.rs', 'ports/distillery/src/inference_stage/automatic.rs',
    'ports/distillery/tests/inference_durable.rs', 'ports/distillery/tests/inference_layered.rs',
    'ports/distillery/tests/inference_stages.rs']
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
assert len(whole) == 15 and sum(inv[n]['chars'] for n in whole) == 417348
assert all(sha(ROOT / n) == inv[n]['sha256'] == cov[n]['sha256']
           and cov[n]['through'] == inv[n]['chars'] for n in whole)
assert load('pending_read.json') == []
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1265 and stats['total_files'] == 3722, stats
assert stats['read_chars'] == 16966577 and stats['total_chars'] == 39342110, stats
assert not stats['changed_since_inventory'] and stats['pending_chunks'] == 0
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(preserved) == 775 and all(sha(ROOT / n) == h for n, h in preserved.items())
assert all(sha(ROOT / n) == h for n, h in probe['source_sha256'].items())
assert all(sha(Path(lib['path'])) == lib['sha256'] for lib in probe['libraries'].values())
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip() == old['basis_commit']
failed = (HERE / 'djinn_inference_route_current_tests.log').read_text(encoding='utf-8-sig')
assert 'memory allocation of ' in failed and 'error: could not compile' in failed
live = load('inference_route_serial_live_processes.json')
assert any(p['Name'] == 'cargo.exe' and '--test inference_route' in p['CommandLine']
           and '-j 1' in p['CommandLine'] for p in live)
serial = (HERE / 'djinn_inference_route_serial_tests.log').read_text(encoding='utf-8-sig')
serial_results = [line for line in serial.splitlines() if line.startswith('test result:')]
test_state = {'session_id': 4064, 'log': 'djinn_inference_route_serial_tests.log',
    'last_observed': 'live cargo/rustc processes and exec session confirmed before checkpoint',
    'process_snapshot': 'inference_route_serial_live_processes.json',
    'results_visible_at_checkpoint': serial_results,
    'instruction': 'Poll this same session or verify recorded processes before any restart. A timed observation is not a terminal outcome.'}

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q216' and authority['entries'][-1]['id'] == 'A462'
finding = {'id': 'Q217',
    'title': 'Inference actor disconnection is not folded into run failure and later control operations falsely report success',
    'source': 'ports/distillery/src/inference.rs:bridge/ask_internal/cancel/pause/resume/close; crates/armillary/src/actor.rs:ActorHandle::command/join; crates/intel/esp/src/infer/actor.rs:provider construction',
    'status': 'two native public lane counterexamples reproduced; documented, not repaired',
    'evidence': 'A gated provider-construction panic disconnects the actor after an admitted ask has been journaled. A later ask returns Closed, proving disconnection, but has already inserted and journaled another Queued run. The earlier run remains Queued with no load_error. Cancel and pause return success after their command delivery fails, and close returns success while both table and in-memory journal retain queued work. A second public lane opened on a retained user pause returns success from resume, appends a continuation and commits Queued even though the actor is already disconnected. The bridge simply exits on update-channel closure, and ActorHandle::join discards the actor panic result.',
    'scope': 'Synthetic, intentionally gated construction panics and a public in-memory RunJournal. No GPU, real model, disk journal, resident process, broker or network was used. An ordinary returned InvalidConfig load error is the passing control: that actor remains alive, reports load_error and makes its ask Failed. Some GPU/stage generation wrappers already catch panics; this finding does not claim that every backend failure crashes the actor.',
    'repair_triage': {'disposition': 'documented in read-only audit', 'size': 'shared actor-disconnect and durable run-transition handling',
        'reason': 'Fold unexpected update-channel closure into a visible failure for unfinished runs, reject dead actor commands, and reconcile committed queued/resume state when delivery fails. Preserve valid retained checkpoints. A checked join result or explicit actor completion channel is needed to distinguish panic from clean shutdown; do not change every actor consumer or choose durable recovery semantics speculatively.'},
    'verification': NAME, 'diagnostic': 'inference_crash_probe_receipt.json'}
notes['findings'].append(finding)
notes['candidates'].append({'title': 'Vault lock can wait indefinitely after an actor disconnect leaves queued or executing state behind',
    'source': 'ports/distillery/src/inference.rs:InferenceLane::set_locked',
    'note': 'Q217 reproduces stranded Queued records and ignored failed Pause delivery. set_locked(true) loops until every record is nonexecuting and neither Running nor Queued, with no disconnected-actor exit. This lock wait is source-qualified only; the audit did not invoke a hanging lock or claim a measured resident shutdown hang.'})
notes['candidates'].append({'title': 'Headed and GPU fixture output directories can retain old control/result files',
    'source': 'ports/djinn/tests/inference_route.rs:serve_trained_model_for_headed_turnstone; inference_gpu_reclaim.rs:reclaim',
    'note': 'The headed fixture reads restart.request/stop.request and conditionally writes finished/cancelled artifacts; the GPU reclaim writer merges into an existing receipt. Neither establishes fresh output itself. A harness can supply a fresh directory, and final headed assertions also inspect current snapshots. No false completed headed/GPU acceptance was reproduced; do not infer freshness from file existence without checking the owning qualification driver.'})
rows = [
    ('Inference actor ownership', 'One shared lane owns the actor and materialized table while RunJournal owns durable commits.', ['InferenceLane Shared Arc', 'ESP checkpoint actor', 'RunJournal synchronous append barrier'], 'Clones share one actor/table. Checkpoint commits publish only after journal append; this is real sharing. Unexpected actor disconnection remains Q217 rather than a second authority implementation.'),
    ('Actor death versus accepted controls', 'The shared actor API reports command delivery, but lane control paths discard it.', ['ActorHandle::command bool', 'InferenceLane cancel/pause/resume', 'bridge update recv closure'], 'Two native public counterexamples prove successful controls and queued persistence after the receiver is gone. A rejected late ask also leaves a queued record. No broker or installed runtime was exercised.'),
    ('Join and durable state truth', 'A successful close should describe actual actor termination and run transitions.', ['ActorHandle::join discarded thread result', 'InferenceLane::close', 'bridge disconnect'], 'Q217 close succeeds after a synthetic actor panic without reconciling its queued records. Lock waiting on those records is an additional source-qualified consequence, not an executed hang.'),
    ('Ordinary model load failure', 'Returned provider errors remain visible and do not select a fallback model.', ['ESP provider Result retained in command loop', 'InferUpdate::LoadFailed/Failed', 'native ordinary-error control'], 'The native control becomes Failed with load_error. Reading and probing rule out the broader claim that a normal model-load error strands every ask; Q217 concerns an actual disconnected actor.'),
    ('Private journal and epoch authority', 'The host supplies current group admission, while the journal owns signed per-ask logs.', ['RunGroupProtection', 'PersonaRunJournal Policy/Delta/fold', 'Djinn personal group adapter'], 'The journal cannot create a group. Signed domain/run addresses, authenticated envelopes and same-run parent admission precede group insertion. Retired history and new-epoch execution remain distinct; no new cryptographic breach was demonstrated.'),
    ('Source evidence versus derived materialization', 'Signed operation logs own retained state; typed Eidetic material is an addressed projection.', ['PersonaRunJournal::fold', 'materialize_evidence', 'resolve_passage/resolve_evidence_blob'], 'No plaintext fallback or independently folded second run owner was found here. Concurrent continuations are explicit errors instead of a silent chosen winner. Recovery requires an actual policy decision if a branch selection workflow is absent.'),
    ('Recall source and citation authority', 'Recall borrows current source heads and builds a disposable lexical index.', ['RecallCorpus/PageTextCorpus', 'RecallIndex', 'ObservedSources', 'RecallEvidence::validate'], 'Snapshot passages keep exact versions/selectors; a single projection observes each current head once. Model markers and full verbatim matches are distinct citation bases; stale or absent owners remove citations without rewriting retained supplied evidence.'),
    ('Duplicate recall limit validation', 'The shared RecallConfig validator and RecallIndex repeat the same checks.', ['inference_protocol/recall.rs:RecallConfig::validate', 'inference/recall.rs:RecallIndex::build'], 'The four nonzero-limit checks are independently written with the same error text. Current behavior agrees, so this is a concrete consolidation seam, not a reproduced divergence or a second index implementation.'),
    ('Live publication ownership', 'A connected publisher session owns its supplied corpus and disconnect removes its live head.', ['InferenceAskEndpoint::publish/Drop', 'PublishedRecall owner map', 'PublishedCorpus'], 'The session prevents another live publisher replacing that owner and checks page-text version bytes. Durable passages remain retained evidence, while current source availability is a live publication fact. No application-private source store is reopened.'),
    ('Stage admission and tensor drain', 'The same mesh lease and key-group owners govern every stage call.', ['StageService authorize_claim/admit', 'StageResource execute_for', 'StageRuntime/RunRegistration', 'StageChain'], 'Claims are rebound to signed job input and current roots before/after input and before output. Detached blocking work retains guards until actual runtime destruction. Source reading does not establish current GPU or multi-device acceptance.'),
    ('Automatic planning versus durable posts', 'Only the serialized authority tick authors stage jobs.', ['StagePlanner::factory/plan', 'PostStages bounded channel', 'drive_posts', 'prompt-free discovery/reservations'], 'Discovery and reservation use public model/session metadata; the actor does not become a separate mesh writer. Expired/abandoned post requests are checked before posting, and each active call still uses supervised lease admission.'),
    ('Exact-session fallback and repeated adapters', 'Fallback shares decoder checkpoint guards rather than inventing a continuation format.', ['StageFallbackProvider', 'LibraryStages', 'ESP GenerationCheckpoint'], 'Stage retirement drops tensors before constructing same-session streaming fallback and records placement at committed token/draw boundaries. The two modules separately implement similar checkpoint-to-fragment wrappers; that duplicated adapter is distinct from their shared continuation owner.'),
    ('Self-comparison fixture qualification', 'The layered-model fixture seeds qualification metadata to exercise BF16 runtime selection.', ['tests/inference_layered.rs half branch', 'baseline_tokens/format_tokens both result.token_ids', 'logit_rmse/logit_max_abs literal zero'], 'This is self-comparison setup, not independent numerical format qualification. The test name and receipt-shaped metadata must not be cited as measured BF16-versus-baseline quality. The same fixture has a stale non-journaled comment despite supplying Some(journal). No numerical qualification test ran here.'),
    ('Inference acceptance boundaries', 'Each integration fixture names the engine/device/process boundary it can actually test.', ['Djinn inference_route/gpu_reclaim', 'Distillery inference_durable/layered/stages'], 'The source includes actual local carrier, seeded continuation, process-kill/reopen and supervised-stage tests, with GPU/headed cases gated separately. This checkpoint reads those tests; its fresh broader route attempt failed during compilation due allocation failure, and the serial retry remains separately tracked. Only the three native crash observations/control ran successfully here.'),
]
entries = []
for number, (domain, claim, implementations, assessment) in enumerate(rows, 463):
    entries.append({'id': f'A{number:03}', 'domain': domain, 'claim': claim,
        'implementations': implementations, 'assessment': assessment,
        'state': 'current source review; executed scope named in assessment', 'verification': NAME})
authority['entries'].extend(entries)
next_read = {'path': 'ports/distillery/src/authority.rs', 'start': 0,
    'reason': 'Continue through remaining Distillery authority/supervisor/receipt code; poll existing serial inference test session4064 before any restart.'}
notes['current_area'] = 'Complete shared inference actor/lane/journal/recall/stage source and named integration fixtures'
notes['next_area'] = 'Remaining Distillery authority, supervisor, receipt and consumer code'
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats, 'finding_count': 217,
    'authority_entries': 476, 'new_receipts': [NAME], 'next_read': next_read,
    'production_edits_this_turn': False, 'diagnostic_outcomes': 'Fresh Distillery inference-model library build and3 native observations/control pass. Broader Djinn route build exited101 on memory allocation failure before test execution; single-job retry tracked by existing session4064.',
    'pending_verification': test_state, 'coverage_note': '15 additional whole current files417348 chars. No stale acknowledged hash/pending chunks; full Mere inventory remains incomplete.'}
notes['source_qualifications'].append({'area': 'Shared inference runtime and receipts',
    'note': 'Q217 uses gated construction panics and a public in-memory journal, not real model/backend failure. Returned model error is a successful control. Numerical BF16 self-comparison is fixture setup, not independent quality evidence. GPU/headed/staged/process-crash tests were read, not run. Original broader route gate has only a compilation failure; retry result is not assumed.'})
doc = ROOT / old['document']['path']
original = doc.read_bytes()
assert hashlib.sha256(original).hexdigest() == old['document']['sha256']
addition = '''

### Shared inference runtime, journal and stage review, 2026-10-17

This checkpoint reads 15 additional whole current files, 417,348 characters: the two remaining Djinn inference integration fixtures; ESP's inference actor; Distillery's inference lane, durable journal, recall and GPU adapter; the complete stage service and its automatic/discovery/fallback/lane modules; and the durable, layered and stage integration fixtures. Current whole-source coverage is 1,265 of 3,722 files and 16,966,577 of 39,342,110 characters. No stale acknowledged hash or pending read chunk remains. The complete repository objective is still active.

Q217: a disconnected inference actor is not folded into failed run state. A fresh native probe gates an intentional provider-construction panic after a public ask has been admitted and journaled. A later ask returns Closed, demonstrating actual command-channel disconnection, but that rejected ask has already inserted and journaled another queued record. The earlier run remains queued with no load_error. Cancel and pause return success despite failed command delivery; close also returns success and retains queued table/journal state. A second public lane loaded on a user pause returns success from resume and journals a queued continuation after its actor is disconnected. The bridge exits on update-channel closure, while Armillary join discards the actor's panic result. This is a false-success and stranded-state defect across the shared lifecycle seam.

The three native checks include an ordinary returned InvalidConfig error as the control. That actor remains alive, reports load_error and makes the ask Failed. The finding therefore concerns disconnection, not every normal model-load failure. The fixtures use a public in-memory RunJournal and synthetic gated construction panics; no GPU, real model, resident process, broker, network or disk journal ran. GPU/stage generation wrappers already catch some panics. set_locked(true) has an additional source-qualified risk: it waits for queued/running state to disappear while failed Pause delivery cannot change that state. No hanging lock was invoked. Repair should share explicit actor-disconnect handling, reconcile durable transitions when command delivery fails, and expose checked completion without discarding valid checkpoints. This is documented rather than widening the read-only audit into speculative recovery policy.

There is substantial actual sharing here. Inference clones share one actor/table; synchronous checkpoint append is the durable publication barrier. The private journal borrows host-supplied group admission and authenticates domain/run addresses, writer proofs and same-run parents before group insertion. Typed Eidetic evidence is a derived content-addressed projection of those signed logs. Concurrent continuations remain visible errors rather than a silent winner. Recall borrows its source owner, retains exact passage versions/selectors, and uses one set of current-head observations per projection. Absent/stale heads remove current citations without rewriting retained evidence. Publications are owned by a live publisher session and disconnected heads are withdrawn.

Duplicated implementations remain tracked alongside those owners. RecallIndex::build repeats the same four nonzero-limit checks as RecallConfig::validate instead of calling the shared validator; current behavior agrees. LibraryStages and StageFallbackProvider separately adapt checkpoint text prefixes into streamed fragments. The layered-model BF16 fixture assigns identical result token IDs to baseline and formatted output and writes literal zero logit errors. That is self-comparison setup for runtime selection, not independently measured format quality. Its non-journaled comment is stale because the lane now receives Some(journal). These qualifications do not fabricate a numerical failure or turn every repeated adapter into a second durable authority.

Stage admission reuses current mesh leases, signed inputs and key-group readers before/after input and before output. Detached blocking work retains its runtime/busy/GPU guards through destruction. Automatic discovery and reservations carry public model metadata; only the authority's serialized drive_posts writes mesh jobs. Fallback checks the exact model/session/capability and resumes from committed token/draw state after retiring the staged provider. The reviewed integration sources contain stronger local transport, process-kill/reopen and stage checks, with separate GPU/headed gates, but those checks are not new runtime proof in this checkpoint. External fixture output freshness remains qualified: headed control/result files and merged GPU receipts need their owning driver's fresh-directory evidence.

Fresh verification: the inference-model library build passes and the three native crash observations/control pass. The first broader Djinn inference-route Cargo attempt exited 101 during compilation with memory-allocation failures, before tests ran. Its failed log is retained. A single-job retry uses the same approved shared target and remains tracked by exec session 4064 and a captured live process snapshot; poll that existing handle/process before any restart. Its result is not assumed. The probe executable/PDB and verified generated import/export linker files were removed; no isolated target, Cargo home or worktree was created. Shared WIP and the entire prior document prefix were preserved. Authority entries A463-A476 and the [runtime checkpoint](../../../support/receipts/authority-success-audit/inference_runtime_review_receipt.json) bind the exact sources, compact evidence and remaining scope. Continue at Distillery authority.rs.
'''
doc.write_bytes(original + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'inference_runtime_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['inference_crash_probe.rs', 'run_inference_crash_probe.py',
    'inference_crash_probe_build.jsonl', 'inference_crash_probe_build_stderr.log',
    'inference_crash_probe_compile.log', 'inference_crash_probe_output.log',
    'inference_crash_probe_receipt.json', 'inference_crash_probe_import_cleanup.json',
    'djinn_inference_route_current_tests.log', 'inference_route_serial_live_processes.json',
    'record_inference_runtime_review.py', 'inference_runtime_diff_check.txt']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': old['basis_commit'],
    'kind': 'Shared inference runtime full-file source audit; Mere full-source goal incomplete',
    'production_edits_this_pass': False, 'coverage': stats, 'whole_current_files': whole,
    'whole_current_chars': 417348, 'new_findings': ['Q217'],
    'authority_entries': [row['id'] for row in entries],
    'source_sha256': {n: sha(ROOT / n) for n in whole},
    'trace_sha256': {n: sha(ROOT / n) for n in ['crates/armillary/src/actor.rs', 'ports/distillery/src/inference_protocol/recall.rs']},
    'preserved_production_and_test_hashes': preserved,
    'prior_receipt': {'path': 'resident_routes_personal_facts_review_receipt.json', 'sha256': sha(HERE / 'resident_routes_personal_facts_review_receipt.json')},
    'next_read': next_read, 'native_observations': 3, 'native_probe_exit': 0,
    'fresh_library_build_exit': 0, 'first_route_gate': {'build_exit': 101, 'tests_executed': False, 'cause': 'compiler memory allocation failures'},
    'pending_verification': test_state,
    'document': {'path': old['document']['path'], 'sha256': sha(doc),
        'preserved_prefix_bytes': len(original), 'preserved_prefix_sha256': hashlib.sha256(original).hexdigest()},
    'evidence_sha256': {n: sha(HERE / n) for n in evidence},
    'retained_isolation': [], 'probe_outputs_removed': True,
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'No completed full-source or whole-runtime claim. Only public crash/error native fixtures ran successfully; no new GPU, staged, headed, physical multi-device, installed or disk-journal receipt. Serial original integration gate is tracked separately, not declared passed.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': ['Q217'],
                  'authority_entries': receipt['authority_entries'], 'retained_bytes': budget}, indent=2))
