"""Native Graphshell example and receipt-owner source checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'native_receipt_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('receipt_tooling_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1408
assert all(sha(ROOT / n) == digest for n, digest in guards.items())
assert all(sha(HERE / n) == digest for n, digest in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
area = lambda n: (n.startswith('ports/graphshell/examples/') or n.startswith('ports/graphshell/src/receipts/')
                  or n in ['ports/graphshell/src/bin/receipt_ingest.rs', 'ports/graphshell/src/bin/h7_sync_peer.rs'])
whole = sorted(n for n in inv if area(n) and n not in guards
               and cov.get(n, {}).get('sha256') == inv[n]['sha256'] and cov[n]['through'] == inv[n]['chars'])
assert len(whole) == 11 and sum(inv[n]['chars'] for n in whole) == 101058
assert all(sha(ROOT / n) == inv[n]['sha256'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1889 and stats['total_files'] == 3727
assert stats['read_chars'] == 22300395 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probe = load('receipt_timestamp_probe_v2_receipt.json')
assert probe['tests'] == 6 and probe['build_exit'] == probe['test_exit'] == 0 and probe['generated_output_removed']
assert all(sha(ROOT / n) == digest for n, digest in probe['source_sha256'].items())
assert sha(HERE / 'receipt_timestamp_probe_v2.rs') == probe['probe_sha256']
assert sha(HERE / 'run_receipt_timestamp_probe_v2.py') == probe['runner_sha256']
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-receipt_timestamp_probe*'))
initial = load('receipt_timestamp_probe_initial_failure.json')
assert all(sha(HERE / n) == digest for n, digest in initial['evidence_sha256'].items())
cross = ['ports/djinn/src/personal_sync.rs', 'crates/eidetic/muniment/src/redb_backend.rs', 'crates/eidetic/muniment/src/blob.rs']
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q273' and authority['entries'][-1]['id'] == 'A699'
rows = [
    ('Q274', 'Receipt dry run writes its database and artifacts before saying nothing was written',
     'ports/graphshell/src/bin/receipt_ingest.rs:106-125; ports/graphshell/src/receipts/ingest.rs:117-135; crates/eidetic/muniment/src/redb_backend.rs:49-60; crates/eidetic/muniment/src/blob.rs:84-89',
     'source-only complete CLI and owner trace',
     'run creates the store parent, opens RedbBackend, and awaits ingest_directory before inspecting args.dry_run. RedbBackend::open creates/materializes/commits a database table, and each artifact goes through BlobStore::put to a committed backend write. Only graph-events.json and inbox deposit are skipped when the CLI prints dry run: nothing written. This is a definite mutation/reporting contradiction from the current source; no actual user database or CLI invocation was used.',
     'Contained repair: choose an ephemeral store or validation-only ingest path before any durable open/write, or explicitly define and disclose a staging-only mode rather than promising nothing written.'),
    ('Q275', 'Unknown receipt provenance survives parsing but is discarded from the replicated facts',
     'ports/graphshell/src/receipts/manifest.rs:113-149; ports/graphshell/src/receipts/ingest.rs:178-193,259-269',
     'source-only complete parser-to-events trace',
     'ReceiptManifest flattens unknown fields into extra and promises they are preserved untouched so newer-lane provenance is not lost on ingest. run_facet serializes a fixed list and omits extra; the other emitted facet contains artifacts only, and the source manifest itself is not a stored blob. The test unknown_manifest_fields_survive only parses and checks extra, so it never crosses the lossy conversion. For example a gpu field is retained in the temporary Rust value and then absent from graph events.',
     'Contained repair: preserve a versioned/namespaced extra-provenance object in the run facet, with collision rules, and test the events after actual ingest rather than only the parser.'),
    ('Q276', 'An empty producer digest silently disables receipt artifact verification',
     'ports/graphshell/src/receipts/manifest.rs:100-109,153-155; ports/graphshell/src/receipts/ingest.rs:121-140',
     'source-only artifact-integrity contract contradiction',
     'The manifest parser accepts an empty sha256 string. ingest_manifest checks a mismatch only when the string is nonempty, then stores the received bytes and writes their newly computed digest into receipt.artifacts. The module claims it verifies the producing machine digest rather than trusting arrived bytes, but this path has no producer digest to compare. The declared artifact.bytes is also unused for verification. Host BLAKE3 restaging checks bytes against the locally produced event hash, so it cannot recover the missing original producer check.',
     'Contained repair: validate a producer SHA-256 format and required size, or explicitly distinguish an unverified legacy import in the graph/card. A computed local hash is not a successful producer-digest comparison.'),
    ('Q277', 'Receipt timestamp parsing is neither total nor faithful to the claimed RFC3339 format',
     'ports/graphshell/src/receipts/manifest.rs:199-258',
     'six native observations of verbatim current function; full importer not executed',
     'The exact current parse_rfc3339_ms function panics on a non-ASCII fractional string because [..3] can split UTF-8. A huge i64 year also panics in checked native arithmetic. A +05:30 offset is silently ignored; a fractional -05:30 offset is likewise ignored while an integral-second negative offset returns None. 2026-02-30 is accepted. Thus ran_at_ms does not always turn unsupported input into the promised unknown=0 and can emit a wrong observation time. The original UTC/invalid-text control passes. Initial probe expected all negative offsets to refuse; that expectation failed on the fractional case and is preserved, with a distinct corrected six-case probe.',
     'Contained repair: use a validated shared date parser or a strict checked UTC-only grammar, reject unsupported offsets explicitly, validate calendar/time ranges and avoid byte slicing or unchecked arithmetic. Preserve original timestamp text alongside explicit parse status.'),
    ('Q278', 'Unknown receipt results receive a Failed badge',
     'ports/graphshell/src/receipts/card.rs:49-50,71-78,95-100,298-302',
     'source-only card result inconsistency',
     'Missing or invalid exit_code produces Result=unknown but passed=false, so the badge branch adds Failed. The sparse-run test checks the Result value and never checks badges. A reader skimming the badge receives a failure claim that the run facet does not establish. Normal zero/nonzero results are rendered separately and correctly.',
     'Contained repair: derive Result and badge from one three-state outcome, with an Unknown badge or an omitted outcome badge for unknown.'),
    ('Q279', 'A non-UTF8 inbox file aborts intake for all healthy pending receipts',
     'ports/graphshell/src/receipts/intake.rs:128-157; ports/djinn/src/personal_sync.rs:1117-1130',
     'source-only library-to-live-host intake trace',
     'pending skips JSON syntax errors but std::fs::read_to_string uses ?, so one .json file containing invalid UTF-8, an unreadable file or a directory named *.json aborts the entire scan. The host logs the scan failure and retries the same batch ten seconds later, never receiving the healthy pending entries. This contradicts the documented isolation of malformed handoffs. Even valid-UTF8 malformed JSON is silently skipped without a per-file diagnostic returned to the caller, despite the comment saying the caller logs it. No host or filesystem fault injection ran.',
     'Contained repair: isolate and report each unreadable/unparseable entry, continue scanning valid files, and leave or quarantine failed handoffs without an endless whole-inbox failure.'),
    ('Q280', 'Receipt artifact names can resolve outside their selected source directory',
     'ports/graphshell/src/receipts/manifest.rs:101-109; ports/graphshell/src/receipts/ingest.rs:116-121; ports/djinn/src/personal_sync.rs:1087-1092',
     'source-only manifest-path confinement defect',
     'Artifact name is an arbitrary deserialized string and ingest reads dir.join(name) without rejecting rooted paths, parent traversal or escaping links. The host later repeats receipt.source.join(name). An absolute path replaces the joined base on the current platform; ../ paths escape it. Consequently a supplied remote receipt can cause import of a local file outside the chosen receipt directory, especially when Q276 allows an empty producer digest. The host checks that later staged bytes match the graph hash, which binds content but does not confine the chosen source path. No sensitive files were accessed and no peer disclosure was executed.',
     'Validate confined relative artifact names and resolve symlink/reparse escape under the selected directory before read. Keep local privileged inbox editing distinct from accepting untrusted producer paths. This needs a deliberate path policy, not a blanket claim that content hashing provides source confinement.'),
]
for ident, title, source, status, evidence, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source, 'status': status,
        'evidence': evidence, 'verification': NAME,
        'diagnostics': ['receipt_timestamp_probe_v2_receipt.json'] if ident == 'Q277' else [],
        'repair_triage': {'disposition': 'documented; no production changes', 'reason': repair}})
arows = [
    ('Receipt graph authoring', 'The resident Djinn host authors deposited receipt events as one signed turn.',
     ['receipts::ingest', 'receipts::intake', 'djinn::personal_sync::spawn_receipt_intake'],
     'Real single-writer boundary: importer stores blobs/produces events; resident host stages captures then authors, and only successful authoring clears inbox. Not a second replication engine. Failed mark_applied can append again next poll by design; graph idempotence and operation-log idempotence differ.'),
    ('Receipt source provenance', 'Manifest, replicated run facet and card are separate representations.',
     ['ReceiptManifest::extra', 'run_facet', 'receipt_card'],
     'Q275 drops unknown provenance crossing manifest-to-events; parse-only preservation tests miss it. Dirty checkout count is actually displayed in both commit text and badge. Q278 uses inconsistent unknown/failure states.'),
    ('Receipt digest and path ownership', 'SHA-256 checks producer claims; BLAKE3 names arrived blobs; directory policy chooses readable inputs.',
     ['ingest_manifest', 'ManifestArtifact', 'stage_captures'],
     'Hash functions are intentionally different roles, not duplicate competing digest authority. Q276 silently omits producer comparison; Q280 has no source-path confinement. Host restaging checks its event hash before authoring but cannot repair either original claim.'),
    ('Receipt time ownership', 'Run times originate in supplied manifest text, not a new ingest clock.',
     ['ReceiptManifest::ran_at_ms', 'parse_rfc3339_ms', 'BlobAvailabilityObservation'],
     'Clock-free determinism is real. Hand-written RFC3339 conversion introduces Q277 panic and wrong-time cases; deterministic does not mean validated. The exact extracted function was executed; full store or graph ingest was not.'),
    ('Receipt staging and dry run', 'CLI durable staging precedes optional event-file and inbox output.',
     ['receipt_ingest::run', 'RedbBackend::open', 'BlobStore::put'],
     'Q274 says nothing written after durable mutation. Source claims about authoring remain true: this CLI does not sign a graph turn. A failed later artifact can leave earlier content-addressed blob puts without producing events; no atomic rollback is promised.'),
    ('Inbox isolation and capture declarations', 'Inbox files are parsed into events and captures are read from their artifact facets.',
     ['pending', 'captures_in', 'mark_applied'],
     'Q279 whole-scan I/O/encoding failure differs from skipped JSON syntax errors. captures_in silently skips malformed artifact declarations while arbitrary availability events remain in the event vector; validated importer output supplies valid hashes, so no general authorization-bypass claim is made for locally edited inbox files.'),
    ('Hex decoding duplication', 'Card and intake each decode a 64-character content hash locally.',
     ['card::parse_hash', 'intake::parse_hex32', 'muniment::Hash::from_hex'],
     'Two similar byte-pair decoders return different local types, alongside an existing store hash parser. This is small duplicated parsing policy, not a new cryptographic implementation or demonstrated divergent accepted digest. Track a shared checked hash conversion rather than claiming universal single authority.'),
    ('Caller-owned toolkit evidence', 'One booted GPU supplies egui and Iced adapters in a native headless example.',
     ['examples/caller_owned_toolkits.rs', 'ContentProducer', 'GraphPane', 'embedding::{egui,iced}'],
     'The example passes the same device/queue to both paths, reads real GPU pixels and asserts clip containment/content. Generation=2 rehydration is simulated on the same handles; qualification explicitly leaves actual device loss, IME/AT and audio open. No native benchmark was rerun.'),
    ('Projection and clip examples', 'Small examples call shared compiler, dynamics receipt and Fleece export owners.',
     ['projection_compile_perf.rs', 'projection_dynamics_receipt.rs', 'retained_content_fixture.rs'],
     'Compiler timing explicitly excludes GPU/headed FPS, measures full compile and retained label refresh separately. Synthetic selected exports delegate Fleece; no second extraction implementation. Source reading does not prove current compilation or physical downloads.'),
    ('H7 physical receipt scope', 'The H7 binary composes a shared replica, admission function and Stickleback lane.',
     ['h7_sync_peer.rs', 'PersonalGraphReplica', 'accept_into', 'JoinedSpace'],
     'Checks durable local edits then selected two-peer convergence, tags/access/conflict/blob-device observations. Seed labels and blob-presence events are fixture facts, not actual blob transfer proof. Source read only; no peer process, store or network launched.'),
]
entries = [{'id': f'A{700+i:03d}', 'domain': d, 'claim': claim, 'implementations': impl,
            'assessment': assessment, 'state': 'whole-source or explicitly bounded cross-owner trace', 'verification': NAME}
           for i, (d, claim, impl, assessment) in enumerate(arows)]
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/src/bin/c4_webrtc_host.rs', 'start': 0,
             'reason': 'Read remaining native Graphshell bins, scenarios, tests and unread shared modules, then all other unread Mere source.'}
notes['current_area'] = 'Native Graphshell examples, receipt importer/types/card/intake and H7 peer'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'receipt_timestamp_probe_initial_failure.json', 'receipt_timestamp_probe_v2_receipt.json'],
    'production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None,
    'diagnostic_outcomes': 'First parser probe 5/6 passed, with one incorrect harness expectation; preserved. Distinct corrected probe passes six cases. Generated EXE/PDB removed. No full importer/Cargo/host/GPU/peer test.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
with (HERE / 'native_receipt_diff_check.txt').open('x', encoding='utf-8') as stream:
    stream.write(diff.stdout + diff.stderr)
evidence = ['record_native_receipt_review.py', 'native_receipt_diff_check.txt',
    'run_receipt_timestamp_probe.py', 'receipt_timestamp_probe.rs', 'receipt_timestamp_probe_compile.log',
    'receipt_timestamp_probe_output.log', 'receipt_timestamp_probe_initial_failure.json',
    'run_receipt_timestamp_probe_v2.py', 'receipt_timestamp_probe_v2.rs', 'receipt_timestamp_probe_v2_compile.log',
    'receipt_timestamp_probe_v2_output.log', 'receipt_timestamp_probe_v2_receipt.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
addition = f'''

### Native examples and receipt-owner review, 2026-10-17

At {head}, eleven further files / 101,058 characters are completely read: all four Graphshell native examples, all five receipt modules, receipt_ingest and h7_sync_peer. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. All prior 1,408 protected source paths, eight evidence hashes and the preceding document prefix are preserved. Full Mere source review is still incomplete.

Q274 is a source-proven CLI contradiction: dry-run creates/opens a durable redb database and writes artifact blobs before printing nothing written. Q275 loses newer producer provenance: extra fields survive parsing but run_facet drops them from the replicated events; the preservation test only exercises parsing. Q276 accepts an empty producer SHA-256 and replaces the missing verification with a computed local digest, while declared artifact byte size is not checked. Q280 separately leaves artifact names unconstrained: absolute paths, parent traversal or escaping links can read beyond the selected receipt directory. These are source-only paths; no user database, local private file, remote producer or peer disclosure was exercised.

Q277 is runtime-qualified for the exact current timestamp function, extracted verbatim and hash-bound, using standard-library-only native tests. UTC control passes. A positive offset is ignored; a fractional negative offset is also ignored, while an integral negative-offset timestamp refuses. Unicode fractions can panic at a UTF-8 byte boundary, huge years panic under checked arithmetic, and February 30 is accepted. The first probe had five passing cases and one incorrect expectation that all negative offsets refuse. That failed original probe and output remain, followed by a distinct corrected six-case passing probe. This is not execution of the complete manifest importer or durable graph host. The probe EXE/PDB were removed from the stable approved target after exit.

Q278 renders missing exit_code as Result=unknown but adds Failed to badges. Q279 propagates one unreadable/non-UTF8 *.json entry out of pending(), so the resident host retries the same failed scan and healthy receipts do not reach its authoring loop. Ordinary JSON syntax errors are skipped, but the function gives the caller no per-file diagnostic even though its comment says the caller logs them. These findings are source-only and their existing narrow tests do not exercise the reported path.

A700-A709 distinguish importer staging, authoring, producer verification, local hashing, path policy and presentation. The resident Djinn host really stages captures before authoring one signed event turn, then marks applied only after success. This is a real single-writer boundary, not a second sync engine; host restaging cannot restore discarded provenance or an absent producer digest. Two small local hex decoders duplicate parsing beside muniment's hash parser, but no divergent accepted hash was shown. The caller-owned toolkit example passes one booted device/queue through both adapters and reads pixels; its generation bump simulates resource rehydration without physical device loss, which its qualification explicitly disclaims. Compiler timing, dynamics and retained clip examples delegate their named shared owners. H7 checks selected sync convergence, not physical blob bytes, and was not run here.

No production or original test changes, full Cargo build, browser/GPU/peer run, actual importer invocation or isolated output was made. The dirty shared main and live build remain preserved. The [native receipt checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds all source hashes, seven findings, ten authority entries and exact diagnostic limits. Next is remaining native Graphshell source and then every other unread Mere file. Goal remains active.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'receipt_tooling_review_receipt.json', 'sha256': sha(HERE / 'receipt_tooling_review_receipt.json')},
    'kind': 'Native Graphshell examples and receipt owner review; full Mere objective incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 101058,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'cross_owner_source_sha256': {n: sha(ROOT / n) for n in cross},
    'cross_owner_scope': 'Referenced source spans checked; no additional whole-file reading claimed by these cross references.',
    'new_findings': [r[0] for r in rows], 'authority_entries': [e['id'] for e in entries],
    'exact_function_native_cases_passed': 6, 'initial_diagnostic_failure_preserved': True,
    'full_importer_or_host_runs': 0, 'production_edits_this_pass': False,
    'pending_verification': None, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {n: sha(HERE / n) for n in evidence}, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Q277 only has exact-function native verification. Six other findings are source-only. Source inspection of four examples and the H7 peer is not compilation, physical input, GPU, network or complete importer execution.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_whole_files': len(whole),
    'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'],
    'protected_paths': len(guards), 'retained_bytes': budget}, indent=2))
