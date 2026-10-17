"""One-shot personal-sync, carriage and retained-host source review checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

NAME = 'personal_sync_review_receipt.json'
assert not (HERE / NAME).exists()
old = load('product_host_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 979
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
names = [
    'personal_sync.rs', 'personal_sync/assertions.rs', 'carriage.rs', 'retained_policy.rs',
    'retained_content.rs', 'retained_content/export.rs', 'retained_content/fixture.rs',
    'retained_content/producer.rs', 'retained_content/tests.rs', 'retained_content/view.rs',
    'canvas_reader.rs', 'canvas_reader/tests.rs', 'remote_board.rs', 'promoted_links.rs',
    'promoted_links/tests.rs', 'session_item.rs', 'speed_bar.rs', 'forme_workspace.rs',
    'forme_session.rs', 'forme_session_lifecycle_tests.rs',
]
whole = ['ports/graphshell/src/' + n for n in names]
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
assert len(whole) == len(set(whole)) == 20
assert sum(inv[n]['chars'] for n in whole) == 463265
assert all(cov[n]['sha256'] == inv[n]['sha256'] == sha(ROOT / n)
           and cov[n]['through'] == inv[n]['chars'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1471 and stats['total_files'] == 3727
assert stats['read_chars'] == 19835916 and not stats['changed_since_inventory'] and not stats['pending_chunks']
assert not (HERE / 'personal_sync_probe_receipt.json').exists()
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-personal-sync-probe.*'))
assert not (HERE / 'personal_sync_build_terminal.json').exists()
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command',
    "@(Get-CimInstance Win32_Process -Filter \"Name='cargo.exe'\" | Where-Object { $_.CommandLine -like '*build -p graphshell --lib --no-default-features --features personal-sync*' } | Select-Object ProcessId,ParentProcessId,CreationDate) | ConvertTo-Json -Compress"], text=True))
assert live and any(p['ProcessId'] == 22716 for p in live)
save('personal_sync_build_checkpoint_live.json', {
    'observed_utc': datetime.now(timezone.utc).isoformat(), 'session_id': 60676,
    'processes': live, 'stdout_bytes': (HERE / 'personal_sync_build.jsonl').stat().st_size,
    'scope': 'Exact current personal-sync-feature build independently confirmed live; original exec session was polled. This is not a terminal build/test result.',
})
pending = {
    'session_id': 60676,
    'command': 'cargo --config build.build-dir=C:/t/cargo-targets/mere/build build -p graphshell --lib --no-default-features --features personal-sync --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --message-format=json',
    'stdout': 'personal_sync_build.jsonl', 'stderr': 'personal_sync_build_stderr.log',
    'state': 'Live original build, now emitting dependency artifacts after its initial wait; no prepared personal-sync tests ran.',
    'next_step': 'Poll the same session60676. Only after its authoritative terminal success, write personal_sync_build_terminal.json with session, exit, build-finished and final log hashes. Then run run_personal_sync_probe.py once. Retain failures and do not restart merely after an observation timeout.',
    'prepared_probe': 'personal_sync_probe.rs', 'prepared_runner': 'run_personal_sync_probe.py', 'prepared_tests': 5,
}
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q249' and authority['entries'][-1]['id'] == 'A589'
rows = [
    ('Q250', 'Personal sync authors tags into legacy Surface metadata instead of current Resource tagging',
     'ports/graphshell/src/personal_sync.rs:apply_event AddTag/RemoveTag; graph-kernel/src/graph/apply.rs:ReplayInsertNodeTagById; resource_tags.rs:node_content_tags',
     'AddTag/RemoveTag replay through the deliberately legacy node.tags setters. Ordinary InsertNodeTag/RemoveNodeTag use shown Resource assertions. AddNode supplies a shown Resource, so current node_content_tags reads Resource labels and does not fall back to the synchronized raw Surface tags. The sync convergence/fingerprint and Redb tests inspect node.tags directly, retaining a passing legacy representation. The prepared native test compares both reads on one actual public replica; it has not executed.',
     'Version new sync tag authoring around the shared exact Resource/tag-concept contract and attribution, while retaining a byte-compatible reader for old signed events. Do not reinterpret stored signatures or blindly mirror labels into a second owner.'),
    ('Q251', 'Personal access and blob-observation IDs silently keep the first conflicting fact',
     'ports/graphshell/src/personal_sync.rs:apply_event AppendAccess/ObserveBlobAvailability; collect_conflicts',
     'Both records use BTreeMap entry(record_id).or_insert_with. Admission checks portability/nonempty device but not same-ID fact equality. Later causally ordered changed records are accepted and then discarded; collect_conflicts only compares concurrent operations and cannot report this case. Identical duplicate delivery can be valid idempotence, but different times/devices/availability facts disappear under the same outcome. The prepared changed-time/identical-duplicate access checks have not executed. Blob observation behavior is source-only.',
     'Share an immutable record-identity comparison across intake and fold. Preserve duplicate idempotence while rejecting or explicitly retaining conflicting source facts. Keep append-only authority and derived availability state distinct; do not turn first-wins suppression into an unqualified chronology claim.'),
    ('Q252', 'An admitted exact assertion with missing endpoints aborts the whole personal projection',
     'ports/graphshell/src/personal_sync.rs:author/accept_into/materialize; personal_sync/assertions.rs:assert_statement',
     'Admission validates event identity, writer and causal metadata, but not endpoint presence. assert_statement later returns Excluded when either endpoint is absent and materialize propagates it for the entire fold. Thus author can acknowledge a stored operation that leaves projection unusable; adding endpoints in a later operation does not help because replay reaches the earlier error first. The five-test prepared public diagnostic includes missing-endpoint/later-node and prior-endpoint controls; none ran yet.',
     'Define endpoint-deletion/concurrency semantics at the deterministic fold boundary, then enforce or explicitly represent unresolved assertions. A valid unrelated operation should not need to bypass one permanently failing graph event. This is a larger admission/materialization consistency seam, not a blind skip-on-error patch.'),
]
new = []
for ident, title, source, evidence, repair in rows:
    new.append({'id': ident, 'title': title, 'source': source,
                'status': 'source-confirmed; prepared public native counterexamples not executed',
                'verification': NAME, 'diagnostics': [], 'evidence': evidence,
                'scope': 'Complete current personal-sync and assertion-helper source/tests read. Kernel adapter paths traced at unchanged reviewed hashes. No new native test, browser/profile, network, encryption distribution or physical storage-failure trial completed this pass.',
                'repair_triage': {'disposition': 'documented; production edits excluded by active objective', 'reason': repair}})
notes['findings'].extend(new)
notes['candidates'].extend([
    {'title': 'Forme save has no stored-baseline comparison across independent editing sessions', 'source': 'ports/graphshell/src/forme_session.rs:PreparedSave::save; forme_workspace.rs:save', 'verification': NAME,
     'note': 'Both use Backend::put for a whole workspace/history slot. Tokens prevent a foreign completion from changing one live session, and FormeWriteFence prevents this process reopening an owner while its own retired writer remains pending. Neither compares the stored baseline across independent tabs/processes. PresentationSession uses transactional compare-and-replace for the analogous workspace. A stale independent Forme proposal can overwrite acknowledged layout/history at this public API; actual UI multiwriter ownership and native counterexample remain unqualified.'},
    {'title': 'Carriage admission snapshots held-slot order and time in its policy', 'source': 'ports/graphshell/src/carriage.rs:CarriageAdmissionPolicy', 'verification': NAME,
     'note': 'The policy receives owned held-slot state and now_ms. Sequential reuse can outlive that snapshot; inspect production construction/reuse and deletion rechecks before claiming current stale-lease or stale-prune admission. The grammar delegates prune/erasure to Stickleback and intentionally separates issuer signature from transport writer identity.'},
    {'title': 'Remote generic board replacement retains matching physics IDs across scoped identities', 'source': 'ports/graphshell/src/remote_board.rs:sync_scoped/sync_reading', 'verification': NAME,
     'note': 'sync_reading explicitly clears the board on identity/epoch change, while sync_scoped changes its reconciliation key and calls PhysicsBoard::sync with reusable instance IDs. Trace generic caller ownership and retention expectations before promoting prior simulated positions/roles as a product retarget. Existing scoped test checks item counts; reading-mode test actually verifies new owner positions.'},
    {'title': 'Retained content semantics keeps keys across session generations', 'source': 'ports/graphshell/src/retained_content/producer.rs:ContentProducer::keys/semantics', 'verification': NAME,
     'note': 'The stable key map grows for new (instance,generation,node) tuples and actions are cleared separately. Check generation churn and producer lifetime/replacement before promoting a resource-growth bug. Stale action, hidden-state and session/revision guards are real.'},
])
notes['source_qualifications'].append({
    'verification': NAME,
    'note': 'The earlier broad legacy nonsemantic retraction concern is ruled out for current PersonalGraphReplica: assertions::retract_legacy handles every non-Traversal selector through exact authored assertion withdrawals and preserves peers. It returns false only for legacy Traversal-family clearing. Method-version/evidence-order normalization in IdentityIndex is explicit and tested. Keyless sealed retention requires header writer attestation; the existing raw-key refusal test does not cover attested retention. Some NoKey/policy comments still describe the pre-retention behavior.',
})
arows = [
    ('Shared causal substrate', 'Personal sync uses Stickleback admission, stores and causal helpers with a product-specific fold.', ['OperationProcessor', 'MunimentStore', 'causal_projection', 'PersonalGraphEvent'], 'Real sharing at the storage/admission boundary; the event grammar and materialization remain Graphshell-local.'),
    ('Signed compatibility', 'Added header fields omit default values so old signed encodings survive.', ['PersonalGraphExt', 'PersonalEncryption', 'to_operation/from_operation'], 'Encoding compatibility is explicitly tested in source. No new signing or encryption runtime trial ran.'),
    ('Stored versus readable', 'Attested sealed operations may be held without keys and counted unreadable.', ['PersonalGraphPolicy::admit NoKey', 'load_records', 'SyncProjection::unreadable'], 'Retention is not materialization. Raw unattested sealed operations are refused; stale comments are not the live branch.'),
    ('Identity duplication', 'Header and body each carry attestation/parents; different passes consume them.', ['header_subject', 'stable_subject', 'PersonalGraphRecord'], 'Header enables keyless attribution, body enables legacy projection. Consistency/adversarial attestation pairing needs its own evidence, not a guessed bypass.'),
    ('Assertion identity index', 'A Graphshell index validates exact ID/pair/author/reason before the canonical kernel writer.', ['IdentityIndex::observe', 'assert_statement', 'try_assert_persisted_relation_statement'], 'The index and kernel are separate validation seams. Method version/evidence-order normalization is deliberate and tested; missing endpoints remain Q252.'),
    ('Legacy retraction ownership', 'Non-Traversal legacy selectors withdraw only the signed author\'s exact assertions.', ['retract_legacy', 'apply_imported_semantic_withdrawal_for_pair'], 'Current helper preserves other authors across all canonical families. A family-shaped wire event alone is not evidence of clearing every peer claim.'),
    ('Tag migration gap', 'Sync tags still use legacy Surface replay setters while live local tags use Resource claims.', ['PersonalGraphEvent::AddTag', 'ReplayInsertNodeTagById', 'node_content_tags'], 'Q250 is a source-confirmed active representation split. Existing convergence checks read the legacy field and miss the current content-reader result.'),
    ('Immutable fact replay', 'Access and blob observation maps deduplicate by ID without fact comparison.', ['AppendAccess', 'ObserveBlobAvailability', 'collect_conflicts'], 'Q251 separates exact duplicate replay from accepted conflicting facts. Concurrent-conflict reporting does not cover causally later reuse.'),
    ('Admission and materialization', 'Exact endpoint checks occur after operations are stored.', ['accept_into', 'materialize', 'assert_statement'], 'Q252 can block whole projection before later endpoint additions are folded. The admission success means storage, not successful graph projection.'),
    ('Reader-set authority', 'Admitted writers and current readers are distinct sets.', ['SyncRoster', 'fold_readers', 'AdmitReader/RetireReader'], 'Only current readers change readership after bootstrap; last-reader retirement is refused by the fold. Key reconciliation/rotation remains the consumer\'s separate owner.'),
    ('Carriage issuer authority', 'A lease binds graph, blinded slot, issue, expiry and payload under a trusted issuer signature.', ['lease_message', 'verify_lease', 'CarriageAdmissionPolicy'], 'Transport writer may be a relay; this is explicit rather than a second issuer. One per-slot log bounds protocol prune and payload erasure.'),
    ('Carriage retention engine', 'Carriage reuses shared protocol pruning and exposes a pure purge proposal.', ['Admission::prune_before_current', 'erasing_payloads', 'propose_carriage_purge'], 'It has no epoch checkpoint projection, so it does not reuse that unrelated gate. Actual executor/state-refresh qualification still requires consumers.'),
    ('Held document owner', 'ContentHost routes caller-supplied bytes through Pelt and reuses verified clip readers.', ['HeldContent', 'open_clip', 'retained_clip::read_store'], 'No ambient destination fetch in this composition. Generic HeldContent requires admitted bytes from its caller; it is not an independent content verifier.'),
    ('Clip intake preflight', 'Clip packages are validated in staging before live import and graph reference creation.', ['import_clip', 'Fleece::import_clip_package', 'persist_intake'], 'The live store and reference persistence remain distinct stages with explicit retry errors. Read source tests do not constitute a new custody acceptance receipt.'),
    ('Retained action identities', 'Producer actions bind current content session, accessibility revision and exact target.', ['ContentProducer::semantics/act', 'ContentSlot::input', 'ClipExportScope'], 'Hidden/retired session and stale revision checks are real. Rendering uses the embedding core; no GPU/headed proof ran this pass.'),
    ('Input ownership', 'Retained bounds and content view control graph-versus-reader input.', ['painted_overlays', 'graph_point', 'content_view', 'filling_content_view'], 'Half-open bounds, cancel and wheel-containment tests were fully read. Existing real-Genet tests are not freshly rerun native/browser evidence.'),
    ('Accessibility routes', 'Viewer drag/pin routes are separate local actions over shared Canvas/PhysicsBoard.', ['CanvasLocalActions', 'BoardLocalActions', 'canvas_act'], 'Endpoint domain truth is not changed by local arrangement. The read headless harness includes planted missing/dead action controls; no OS screen reader ran.'),
    ('Promoted link truth', 'Frozen link Resources use kernel capture and exact-source drift comparison.', ['freeze_link_with_nonce', 'open_frozen_link', 'compare'], 'Coverage limitations become unavailable rather than unchanged. Personal comment/withdrawal checks retain source and author context; original tests were read only.'),
    ('Forme layout storage', 'Forme keeps workspace/history in one slot but uses its own blind-put protocol.', ['FormeSession::PreparedSave', 'FormeWriteFence', 'FormeWorkspace::save'], 'Request tokens and same-process retired-write fences are real but narrower than the rule editor\'s transactional stale-baseline guard. Independent stale saves remain a candidate.'),
    ('Forme graph lifecycle', 'Layout projects recorded member identities against the actual completed graph owner.', ['project_region', 'FormeLoadGate', 'reconcile_completed_graph lifecycle tests'], 'Missing cells can remain authored in live drafts while graph undo/redo masks them; persistent load reconciliation deliberately prunes missing members/history. Layout undo does not snapshot graph content.'),
]
entries = [{'id': f'A{590+i:03d}', 'domain': d, 'claim': c, 'implementations': impl,
            'assessment': a, 'state': 'current complete source review; no new runtime test receipt', 'verification': NAME}
           for i, (d, c, impl, a) in enumerate(arows)]
assert len(entries) == 20
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/src/forme_workspace/tests.rs', 'start': 0,
             'reason': 'Finish workspace tests and remaining host/retained UI ownership source, then every other unread Mere area.'}
notes['current_area'] = 'Full personal sync/assertion grammar, carriage, retained content, accessibility, promoted links and Forme session/lifecycle source'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME], 'production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': pending,
    'diagnostic_outcomes': 'No new runtime tests completed. Five public personal-sync checks prepared; exact current feature build remains live in original session60676.',
    'coverage_note': '20 whole current files463265 chars. Read source exceeds half of inventoried characters; full Mere scope remains incomplete.'}
addition = f'''

### Personal sync, retained content and Forme source review, 2026-10-17

This pass completely reads20 current files463265 characters at {head}, including the entire153229-character personal_sync.rs and its25069-character assertion helper, their embedded tests, carriage, retained content and all its submodules/tests, Canvas accessibility adapters/tests, remote board, promoted links/tests, session card/speed helpers, and Forme workspace/session/lifecycle source. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, without stale hashes or pending chunks. More than half the inventoried characters are read; the full Mere objective is still incomplete. Next is forme_workspace/tests.rs, not a claim that all Graphshell or Forme source is finished.

Q250 is source-confirmed: PersonalGraphEvent AddTag/RemoveTag still writes node.tags through the explicitly legacy Surface replay setters. Live InsertNodeTag/RemoveNodeTag uses shown Resource assertions. Because synchronized AddNode supplies a shown Resource, current node_content_tags reads Resource labels rather than falling back to those raw tags. The convergence/fingerprint and Redb tests inspect node.tags directly and preserve a passing legacy representation. The prepared public native comparison has not run. A migration must version new authoring through the shared tag-concept/assertion contract while retaining old signed bytes, not rewrite stored signatures or maintain another label-derived authority.

Q251 is source-confirmed: access and blob-observation maps use first entry by record_id with no same-ID fact equality check. A causally later changed fact is admitted and silently discarded, while collect_conflicts only examines concurrent operations. Identical duplicates can be valid idempotence; changed time/device/availability under one ID is a different case. Prepared changed-time and identical-duplicate access controls remain unexecuted; blob-observation behavior is source-only. Share immutable record identity checks rather than claiming complete chronology or current availability from first-wins suppression.

Q252 is source-confirmed: exact relation operations are stored before endpoint validity is known. assert_statement later errors on a missing endpoint and materialize propagates the error for the whole projection. Adding endpoints in a later operation cannot fix the earlier replay failure. Admission success here means stored, not successfully projected. The prepared public probe includes missing-endpoint/later-node and valid-prior-endpoint controls, not yet executed. Endpoint deletion and concurrent-edit semantics need a coherent deterministic owner; blindly ignoring failed events would select a policy without evidence.

The personal-sync-feature library build remains live in original session60676, with exact Cargo processes independently confirmed and its artifact log growing after the initial wait. No new native tests completed this pass. personal_sync_probe.rs and run_personal_sync_probe.py prepare5 public tests and require an authoritative terminal build receipt before execution. Poll this same handle; after actual success save final build-finished/exit/log hashes to personal_sync_build_terminal.json and run the prepared diagnostic once. An observation timeout is not termination. Do not restart the build or relabel Q250-Q252 as native reproductions. No isolated target, Cargo home or worktree was created; the ordinary stable Mere target remains shared with its live build owners.

Personal sync genuinely shares Stickleback OperationProcessor, causal helpers, MunimentStore and LogSync composition. Its local event fold is separate product semantics. Exact assertion IDs preserve pair, stable writer root, reason and causal withdrawal knowledge; method-version/evidence-order normalization is deliberate and tested. Current legacy non-Traversal retraction uses source-authored exact withdrawals and preserves other writers, ruling out the earlier broad clearing suspicion for that path. Legacy Traversal remains event-shaped. Attested sealed operations can be retained and counted unreadable without keys; raw unattested sealed operations are refused. Some NoKey/policy comments still describe the older refusal behavior. Header and body attestation/parents have distinct uses; no adversarial consistency bypass was demonstrated.

Reader membership is a deterministic fold separate from writer admission. Only current readers change it after bootstrap, and the final reader is retained. This source does not itself qualify key rotation/distribution or actual revoked-device exclusion. Carriage separates trusted issuer lease signatures from relay transport signatures, binds exact graph/slot/issue/expiry/payload facts, and delegates destructive supersession to shared per-slot protocol prune/erasure. Purge proposal is pure and blocked without held state. Production state-refresh/reuse of its snapshot policy remains a candidate, not a newly demonstrated deletion bug.

Retained content uses Pelt/Genet with caller-admitted bytes and verified Fleece/Eidetic clip readers. Staging validates packages before live import; graph reference persistence is a separate stage with retry wording. Hidden sessions refuse input/frames, producer actions recheck session/revision/target, and export scopes bind persona/graph/access/content session. Input tests cover actual Genet cancel behavior and bounded wheel custody, but none were rerun here and no browser/GPU/OS screen reader was used. The accessibility adapter uses shared Canvas/PhysicsBoard local actions; the original headless harness includes planted missing/dead-action controls, not a universal desktop acceptance claim.

Promoted links use kernel frozen Resources and exact-source drift comparison; incomplete coverage returns unavailable rather than unchanged. The full tests preserve independent captures, personal comments, exact withdrawals, and receiver coverage. Remote reading mode explicitly clears prior bodies and follows owner slots; generic sync_scoped reuse across different identity tokens still needs caller/physics-retention qualification before becoming a product bug.

Forme has another save protocol: whole-record Backend::put, while PresentationSession uses transactional compare-and-replace for a comparable workspace. Forme tokens prevent a foreign completion from changing one live session and FormeWriteFence prevents that process reloading an owner ahead of its own retired write. Neither proves a stored-baseline guard across independent editing sessions. Stale-save overwrite remains a documented API/caller-ownership candidate with no native/multitab trial. Forme lifecycle tests do correctly distinguish actual graph ownership, graph undo/redo masking, failed-save retained edits, accepted graph versus optional view error, and parked live physics. Workspace geometry and arrangement validation/source ownership remain separate from graph truth. Remaining workspace tests and retained UI consumers are still unread.

A590-A609 track these actual shared owners, copied adapters and qualification boundaries alongside the three new source findings. All979 protected prior production/test hashes, cited prior evidence and the canonical document prefix remain preserved. No production/original-test code was edited. The four fetched upstream commits remain unmerged while other builds use the shared checkout; this evidence stays bound to the stated basis. The [personal-sync checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds coverage, source findings, candidates and the exact pending build. The full-source goal remains active.
'''
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
(HERE / 'personal_sync_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_personal_sync_review.py', 'personal_sync_probe.rs', 'run_personal_sync_probe.py',
            'personal_sync_build_checkpoint_live.json', 'personal_sync_diff_check.txt']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'product_host_review_receipt.json', 'sha256': sha(HERE / 'product_host_review_receipt.json')},
    'kind': 'Complete personal-sync/retained-host/Forme source review; runtime diagnostics pending; full Mere objective incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 463265,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'new_findings': [f['id'] for f in new], 'source_only_findings': [f['id'] for f in new],
    'authority_entries': [e['id'] for e in entries], 'new_native_tests_run': 0,
    'production_edits_this_pass': False, 'pending_verification': pending, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {n: sha(HERE / n) for n in evidence}, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Source-confirmed defects and read original tests. Five prepared public checks not executed; feature build live. No installed profile, network, encryption-distribution, headed rendering, physical disk or native storage-failure trial. Shared WIP and source basis preserved.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'whole_files': 20, 'whole_chars': 463265,
                  'new_source_findings': receipt['source_only_findings'], 'authority_entries': receipt['authority_entries'],
                  'guard_paths': len(guards), 'pending_session': 60676, 'retained_bytes': budget}, indent=2))
