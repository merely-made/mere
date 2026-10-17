"""One-shot complete product/host source and fresh native diagnostic checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

NAME = 'product_host_review_receipt.json'
assert not (HERE / NAME).exists()
old = load('capture_extension_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 946
assert all(sha(ROOT / n) == digest for n, digest in guards.items())
assert all(sha(HERE / n) == digest for n, digest in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == []
assert not (HERE / 'pending_transition.json').exists()
inv = load('source_inventory.json')['files']
cov = load('read_coverage.json')
names = [
    'mere_host_fixture.rs', 'product.rs', 'app.rs', 'local_edit.rs', 'local_edit/tests.rs',
    'local_intake.rs', 'local_intake/tests.rs', 'local_projection.rs', 'local_scene.rs',
    'browser_storage.rs', 'projection_compile.rs', 'host_presenter.rs', 'host_dataset_view.rs',
    'transfer_endpoint.rs', 'transfer_offer.rs', 'transfer.rs', 'web_tree/product/scenes.rs',
    'lib.rs', 'viewer_commands.rs', 'resident_host.rs', 'node_presentation.rs',
    'node_presentation_tests.rs', 'presentation_binding.rs', 'presentation_binding_tests.rs',
    'link_inspection.rs', 'presentation_editor.rs', 'presentation_editor_tests.rs',
    'access_inspection.rs', 'field_inspection.rs', 'link_inspection/tests.rs',
    'field_inspection/tests.rs', 'responsive_links.rs', 'responsive_links_tests.rs',
]
whole = ['ports/graphshell/src/' + n for n in names]
assert len(whole) == len(set(whole)) == 33
assert sum(inv[n]['chars'] for n in whole) == 714200
assert all(cov[n]['sha256'] == inv[n]['sha256'] == sha(ROOT / n)
           and cov[n]['through'] == inv[n]['chars'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1451 and stats['total_files'] == 3727
assert stats['read_chars'] == 19372651 and not stats['changed_since_inventory'] and not stats['pending_chunks']
terminal = load('capture_graphshell_terminal.json')
assert terminal['session_id'] == 87735 and terminal['exit_code'] == 0
assert terminal['build_finished'] == {'reason': 'build-finished', 'success': True}
assert terminal['stdout_sha256'] == sha(HERE / 'capture_graphshell_build.jsonl')
assert terminal['stderr_sha256'] == sha(HERE / 'capture_graphshell_build_stderr.log')
probes = [('current_capture_transaction', 6), ('product_transfer', 3), ('presentation_family', 2)]
for stem, count in probes:
    receipt = load(stem + '_probe_receipt.json')
    assert receipt['tests'] == count and receipt['test_exit'] == 0 and receipt['probe_build_exit'] == 0
    assert receipt['generated_output_removed']
    assert all(sha(ROOT / n) == digest for n, digest in receipt['source_sha256'].items())
    assert all(not Path(row['path']).exists() for row in receipt['removed_outputs'])
upstream = subprocess.check_output(['git', 'rev-parse', 'origin/main'], cwd=ROOT, text=True).strip()
delta = subprocess.check_output(['git', 'rev-list', '--left-right', '--count', 'HEAD...origin/main'], cwd=ROOT, text=True).strip()
upstream_status = {
    'observed_utc': datetime.now(timezone.utc).isoformat(), 'head': head,
    'fetched_origin_main': upstream, 'head_only_and_upstream_only_commits': delta,
    'merge': 'Deferred while other active Cargo builds use this shared checkout. No branch, stash, worktree, merge, reset or source update in this pass.',
    'new_upstream_commits': subprocess.check_output(['git', 'log', '--format=%H %s', 'HEAD..origin/main'], cwd=ROOT, text=True).splitlines(),
}
save('product_host_upstream_status.json', upstream_status)

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q246' and authority['entries'][-1]['id'] == 'A569'
for finding in notes['findings']:
    if finding['id'] in {'Q240', 'Q241', 'Q242'}:
        finding['native_follow_up'] = {
            'verification': NAME, 'diagnostic': 'current_capture_transaction_probe_receipt.json',
            'status': 'reproduced using fresh current Graphshell public APIs; six tests including controls passed',
            'limits': 'Synthetic in-memory fault backend, identities and URLs. No installed browser/history, physical disk failure or concurrent writer trial.',
        }
rows = [
    ('Q247', 'Local-file exclusion leaves explicit file locations in the exported graph',
     'ports/graphshell/src/product.rs:export_product_codicil/selected_facets; web_tree/product/scenes.rs:section; transfer.rs:prepare_transfer',
     'product_transfer_probe_receipt.json',
     'With include_local_file_locations=false, an ordinary host-created synthetic file:///C:/synthetic-audit/private-folder/example.txt address remains verbatim in the exported codicil. selected_facets removes LOCAL_FILE_FACET only; the graph snapshot retains its address. The UI labels this setting Include local file locations, and prepare_transfer states local locations are always removed. Browser-picked files use ni: identities and do not exercise this case.',
     'Define and enforce the exclusion over actual graph addresses and any local reference carriers before export. Reject or explicitly disclose an unportable file-address selection; silently rewriting an address could change Resource identity. This is a privacy/export-contract repair, not a blind string-redaction patch.'),
    ('Q248', 'Transfer receiver reports Completed for bytes inconsistent with the selected content identity',
     'ports/graphshell/src/transfer.rs:verify_content_facet/verify_manifest/apply_transfer',
     'product_transfer_probe_receipt.json',
     'Preparation verifies b"abc" against the graph content SHA-256/ni: identity. Replacing only its blob descriptor with the separately valid BLAKE3 address of same-length b"xyz" leaves the immutable selection integrity valid. apply_transfer verifies the new blob BLAKE3/length and returns Completed, retaining the original content identity while attaching and storing the different bytes. The ordinary b"abc" transfer is the passing consistency control. The receiver never reuses verify_content_facet on received bytes.',
     'Bind each received descriptor/role and verified byte payload to the immutable content facet using the same contract as preparation, before graph or authority mutation. Keep transport BLAKE3 verification and semantic SHA-256/ni: identity distinct. Trusted caller authorization was supplied; this does not demonstrate a custody or network admission bypass.'),
    ('Q249', 'Assertion rule binding labels Containment as Semantic',
     'ports/graphshell/src/presentation_binding.rs:bind_with_source Occurrence::Assertion',
     'presentation_family_probe_receipt.json',
     'The actual host creates a CollectionMember relation; the shared canonical predicate classifies it as Containment and the inspector preserves that predicate. Binding the same exact held handle supplies family="Semantic" to rules. A Cites assertion is correctly supplied as Semantic. The adapter hardcodes this label even though the inspector now carries exact assertions from all families.',
     'Derive the disclosed family through the canonical shared assertion/predicate classification and preserve unknown classification explicitly. Do not infer Semantic merely from the compatibility SemanticStatement name. Qualify all supported relation families at this adapter.'),
]
new_findings = []
for ident, title, source, diagnostic, evidence, repair in rows:
    new_findings.append({
        'id': ident, 'title': title, 'source': source, 'status': 'actual current public Rust counterexample reproduced',
        'verification': NAME, 'diagnostics': [diagnostic], 'evidence': evidence,
        'scope': 'Fresh current graphshell --lib --no-default-features --features web artifacts at recorded checkout; synthetic in-memory public APIs. No actual browser, installed profile, user file, physical disk fault, transport, custody or rendered UI trial.',
        'repair_triage': {'disposition': 'documented; production edits excluded by active objective', 'reason': repair},
    })
notes['findings'].extend(new_findings)
notes['candidates'].extend([
    {'title': 'Transfer storage stages have independent failure boundaries', 'source': 'ports/graphshell/src/transfer.rs:apply_transfer', 'verification': NAME,
     'note': 'Blob puts, imported graph edits, source/destination access records, host.persist, schema saves and receipt save are separate stages. Host persistence precedes final receipt/schema acknowledgement. Trace failure/retry UI and durable recovery before claiming one atomic transaction or promoting an additional false-success result. No fault-injected transfer-stage diagnostic ran.'},
    {'title': 'Copy replay uses deterministic Surface presence as its import guard', 'source': 'ports/graphshell/src/transfer.rs:apply_transfer import_required', 'verification': NAME,
     'note': 'Copy skips incoming graph/facet import when every deterministic destination ID exists, whereas Replicate always preflights and merges. Qualify same-transfer identity reuse, partially restored graph records and owner policy. Existing identical-manifest retry tests are narrower; no changed-copy-manifest counterexample ran.'},
    {'title': 'Host-dataset graph node identities omit SourceBinding', 'source': 'ports/graphshell/src/host_dataset_view.rs:host_dataset_view/GroupedHostDataset::view', 'verification': NAME,
     'note': 'Temporary graph URLs/UUIDs derive from occurrence_id alone. at_revision and HostPresenter::rebind refuse a different source binding, so a current cross-authority retarget is not established. Trace complete dataset replacement and selection/geometry restoration before promoting this as a product bug.'},
    {'title': 'Field rebuild eligibility repeats runtime shape classification', 'source': 'ports/graphshell/src/field_inspection.rs:coupling_read', 'verification': NAME,
     'note': 'The inspector separately classifies response/vector/scalar/lifecycle readiness, and labels it rebuild eligibility rather than evaluated force or spatial influence. Compare future runtime changes with this adapter; no current mismatch was established.'},
])
notes['source_qualifications'].append({
    'verification': NAME,
    'note': 'ImportReceipt, TransferReceipt and TransferOffer relations count graph.edges (Surface records). Fully read transfer tests explicitly pin this scope and separately verify Resource assertions survive replicate/copy. The count omission alone is not promoted as lost relation data. UI/API labeling may still need clearer stratum scope. Live metadata UI calls sync_canvas_metadata_from_graph; repository search finds no caller of the older label-based sync_canvas_metadata helper.',
})
authority_rows = [
    ('Product import owner', 'Imports compose resources through the shared snapshot merger and verify replay before live journal edits.', ['product_import_edits', 'pandect snapshot materialization', 'scratch replay'], 'Actual sharing; original preservation/collision tests were read, not rerun as a suite.'),
    ('Saved dynamics compatibility', 'SavedDynamics is opaque current authority; flat scene fields are compatibility readers.', ['SavedSceneV3', 'SavedSceneV2::into_v3', 'dynamics_spec'], 'Unknown definitions can be retained but are refused on executable open. Saving opaque data is not itself a runnable-success claim.'),
    ('Receipt count strata', 'Several relation counts intentionally measure Surface records.', ['ImportReceipt', 'TransferReceiptV1', 'TransferOfferV1'], 'Source tests explicitly qualify this scope and separately preserve Resource truth. Do not label the counts as total assertion coverage or data-loss proof.'),
    ('Local-location exclusion', 'Export exclusion filters one facet, while explicit file addresses remain in graph truth.', ['selected_facets', 'include_local_file_locations', 'prepare_transfer'], 'Q247 reproduces the contract gap with synthetic addresses; browser-picked ni: files are a narrower working path.'),
    ('Content integrity split', 'Preparation checks semantic content identity; reception checks transport blob integrity separately.', ['verify_content_facet', 'verify_manifest', 'apply_transfer'], 'Q248 shows that individually verified bytes and an intact selection can disagree. The receiver must share the content-binding owner, not just the hash-shaped descriptor vocabulary.'),
    ('Transfer stage acknowledgement', 'Transfer writes use several independent stores and acknowledgement points.', ['BlobStore::put', 'save_access_record', 'MereHost::persist', 'save_transfer_receipt'], 'Separate-stage failure/retry recovery remains a candidate. A completed API result is real only after its final receipt save; graph durability can precede that result.'),
    ('Transfer authorization boundary', 'ApplyTransferContext carries caller-supplied grant identity and revocation state.', ['TransferAuthorization', 'apply_transfer'], 'This trusted library composition boundary is not independent grant verification. No custody bypass is inferred from a caller constructing its own context.'),
    ('Copy versus replicate', 'Copy remaps Surface identity and keeps exact Resource claims; replicate merges both strata.', ['copied_product', 'remap_scene', 'product_import_edits'], 'Actual shared Resource truth survives copying. Deterministic all-node presence skips subsequent Copy graph intake and needs broader replay qualification.'),
    ('Transfer disclosure', 'A begin intent gates source resources and resume, while the snapshot advertises the operation.', ['TransferSourceEndpoint::invoke', 'resource', 'resume'], 'Accepted means disclosure enabled, not destination delivery completed. Shared instance/session/revision checks and BLAKE3 source blob verification are real.'),
    ('Offer routing and confidentiality', 'Personal offers filter device display while the admitted roster receives plaintext operations.', ['offer_address', 'offers_in', 'SyntheticAddressRule'], 'The module explicitly disclaims confidentiality from addressing. Friendly final device segments are presentation routing, not a proof of device identity.'),
    ('Metadata projection adapters', 'Current UI replays exact tag claims into Canvas, while a legacy label-based public helper remains.', ['sync_canvas_metadata_from_graph', 'sync_canvas_metadata', 'web_tree/product.rs'], 'Current call-site search adopts the exact-claim helper. The label helper has no repository caller; its presence alone is not a current UI provenance-loss result.'),
    ('Scene open graph versus view', 'Accepted graph installation and optional view restoration have explicit separate outcomes.', ['OpenedCodicil', 'install_accepted_codicil', 'scene_action_observed'], 'View error leaves Canvas attached to accepted graph. Save wording is staged for persistence acknowledgement; receiving whole UI lifecycle still needs full source review.'),
    ('Projection compiler adoption', 'Graphshell compiler module reexports shared Scenomise compiler and dataset owners.', ['pub use scenomise::projection', 'practice_compiler', 'local_projection'], 'Port code supplies recipes/card size and editor coordination, not a siloed alternate solver. Original compiler/adoption tests read only.'),
    ('Fold presentation versus truth', 'GroupedHostDataset reconstructs a temporary graph while retaining disclosed witnesses and shared folds.', ['HostDatasetFolds', 'GroupedHostDataset', 'HostPresenter'], 'Selection/folds are occurrence-local view state. Same-authority rebind and checkpoint checks are real; derived graph assertions do not become source authority.'),
    ('Canonical link inspection', 'Held/history and owner-bound lookup share the canonical link reader and exact assertion identities.', ['read_link_with_options', 'inspect_statement_at_owner', 'inspect_link_history'], 'The adapter now carries nonsemantic assertions, Surface/Resource owners and tombstones explicitly. It refuses conflicting exact-owner copies instead of choosing a bucket silently.'),
    ('Presentation family adapter drift', 'An assertion binder still hardcodes the Semantic family over an all-family inspector.', ['presentation_binding::bind_with_source', 'LinkRecord::statements', 'RelationKind::from_predicate_iri'], 'Q249 reproduces Containment misclassification with a Semantic control. A compatibility struct name is not a family authority.'),
    ('Presentation workspace storage', 'Rules use shared Backstep history and a Muniment compare-and-replace transaction.', ['PreparedSave::save', 'PresentationSession', 'Backend::transact'], 'Commit is installed only after acknowledged transaction; stale drafts and refused saves remain editable. This is a separately scoped workspace record, not a second graph truth store.'),
    ('Bound runtime disclosure', 'Node decisions use shared resolver with exact occurrence identity and explicit unavailable inputs.', ['NodeRuntime', 'BoundContext::resolve', 'ViewInputs'], 'Unavailable inputs suppress rules even under Not/Any. Hysteresis is scoped by view epoch/generation and occurrence; unsupported winners are disclosed rather than silently substituted.'),
    ('Field and Forme inspection', 'Inspector distinguishes authored selectors, boundary hits, rebuild eligibility and measured influence.', ['inspect_field', 'inspect_coupling', 'region_candidates', 'inspect_forme'], 'Held records and geometry discovery do not evaluate field expressions or mint a Numen owner for Forme. Eligibility shape classification is duplicated and needs drift tracking.'),
    ('Responsive link fixture evidence', 'Fixture evaluates real shared path geometry with exact graph witnesses and fixed endpoint controls.', ['ResponsiveLinksFixture', 'evaluate_link_appearances', 'SpringShareAvailability::Unavailable'], 'It explicitly withholds spring allocation claims and uses shared Numen/Pictograph evaluation. Read tests are not new headed/native force-acceptance evidence.'),
]
entries = [{'id': f'A{570+i:03d}', 'domain': domain, 'claim': claim,
            'implementations': implementations, 'assessment': assessment,
            'state': 'current source review; executed diagnostic scope named separately', 'verification': NAME}
           for i, (domain, claim, implementations, assessment) in enumerate(authority_rows)]
assert len(entries) == 20
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/src/personal_sync.rs', 'start': 0,
             'reason': 'Read the entire personal replication owner, then remaining Graphshell and all other unread Mere source.'}
notes['current_area'] = 'Complete product, local save/intake/scene, transfer, host disclosure and presentation/inspection adapters with original tests'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {
    'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME], 'production_edits_this_turn': False, 'next_read': next_read,
    'diagnostic_outcomes': 'Fresh Graphshell library build exit0. Eleven native public Rust checks passed: six capture, three product/transfer, two presentation family. Q240-Q242 now native reproduced; Q247-Q249 new reproduced findings. Original module test suites read only.',
    'pending_verification': None, 'upstream': upstream_status,
    'coverage_note': '33 whole source files714200 characters since previous checkpoint; all acknowledged source hashes current. Full Mere audit remains incomplete.',
}

addition = f'''

### Product, transfer and presentation authority review, 2026-10-17

This pass completely reads33 more source/test files714200 characters at {head}. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, without stale inventory hashes or pending chunks. Product/fixture composition, local edit/intake/scene owners, host datasets/presenters, transfer/source offers, rule workspace/runtime/binding and access/link/field inspection are included. The full Mere audit remains incomplete. The next whole file is personal_sync.rs; remaining Graphshell and every other unread repository area still count.

The original library build in session87735 is terminal exit0 with build-finished success=true. All6 prepared public capture tests now run against its fresh artifacts and pass: ordinary consistent capture, identical retry control, different-next-delivery after refusal, refused forget followed by another capture, segment sizes1 versus64 and other-application traversal ownership. Q240-Q242 are therefore native reproduced, not merely source-confirmed. A refused delivery's graph edits can become durable without matching ledger/trace/access authority; refused forget similarly diverges. The old failed retained-artifact attempt and its runner remain preserved. This is synthetic fault-backend evidence, not a real browser/profile, physical disk-failure or concurrent-writer receipt.

Q247: include_local_file_locations=false removes LOCAL_FILE_FACET only. An actual public export of the synthetic address file:///C:/synthetic-audit/private-folder/example.txt retains that location verbatim. The UI names the setting Include local file locations, and prepare_transfer says local locations are always removed. Picked files using ni: addresses are a working narrower path. This counterexample uses a synthetic address and no physical file. Repair must define the exclusion over graph identity/reference carriers, not blindly redact strings and silently change Resource identity.

Q248: receive-side content verification is weaker than preparation. A valid selection names b"abc" by SHA-256/ni: identity. Substituting only the manifest blob descriptor with a valid BLAKE3 address for same-length b"xyz" leaves the immutable selection intact. The actual receiver stores/attaches the different bytes and returns Completed while preserving the original content address. A normal b"abc" transfer is the successful consistency control. BLAKE3 and length checks genuinely pass; semantic content binding is missing. verify_content_facet is called at preparation but not on received bytes. Trusted caller authorization is supplied explicitly; this is not a custody or network admission bypass. Share that content-binding contract before destination mutations.

Q249: presentation_binding hardcodes family=Semantic for every exact assertion occurrence. A real CollectionMember relation has a canonical Containment predicate and is preserved by the shared inspector, but the actual binding supplies Semantic to rules. A Cites control is correctly Semantic. The underlying inspector now carries every family's canonical assertion, so retaining a compatibility SemanticStatement name does not justify the label. Derive family through the shared predicate/assertion classifier and preserve unknowns explicitly.

Eleven native diagnostics pass:6 capture,3 product/transfer and2 presentation family checks. They use exact fresh current libraries and synthetic in-memory stores/graphs. All audit-owned executables/PDB/import outputs are marker-verified and removed. These are diagnostics, not a rerun of all original tests or headed/browser acceptance. No isolated target, Cargo home or worktree was created. The stable ordinary Cargo target remains shared with its live build owners.

The full transfer tests explicitly define relations counts as Surface records. They separately verify Resource records/assertions survive replicate/copy. Consequently the earlier count concern is qualified as a protocol/reporting scope question, not promoted as lost data. Both product imports preflight composed Resource truth through shared snapshot/journal machinery. Copy remaps Surface IDs but preserves exact Resource identity and claims; scenes remap member roles and representations. Invalid opaque dynamics can be retained, while executable opens reject them. Those are real shared owners and deliberately distinct acceptance boundaries.

The current UI calls sync_canvas_metadata_from_graph, preserving exact tag handles and other authors. The older label-based sync_canvas_metadata helper remains public but has no repository caller in the current search. MereHost local edits retain pending state on write refusal and only report acknowledged saves after persistence. Optional scene view restoration reports its own error while Canvas remains attached to the accepted owner graph. The complete surrounding retained-UI lifecycle still needs source review before any broader acceptance claim.

Projection compilation reexports the Scenomise compiler rather than implementing another solver. Host folding uses shared HostDatasetFolds and retains original relationship witnesses; temporary graphs and selection are view state. Same-authority rebind/checkpoint checks are explicit. Access inspection retains missing associations without guessing from equal URLs. Link inspection uses shared canonical held/history reads with directed Surface/Resource owners; exact-owner lookup refuses conflicting retained copies. Presentation rules use Backstep history and Muniment transactional compare-and-replace before installing committed state. Node hysteresis stays per occurrence/view generation, and unavailable inputs suppress rules explicitly. Responsive path fixtures disclose fixed-endpoint geometry and state spring allocation unavailable, rather than claiming measured graph force.

Unpromoted candidates remain documented: independent transfer-stage failure/recovery boundaries, deterministic Copy presence guards, host occurrence UUIDs lacking SourceBinding, and separately implemented inspector rebuild-eligibility shape checks. None gained a new counterexample here. Offer addressing filters presentation, while the personal lane explicitly carries plaintext to the admitted roster; it is not confidentiality. A source endpoint's Accepted begin intent means disclosure is enabled, not that destination delivery succeeded.

Upstream was fetched: origin/main={upstream}, with {delta} local/upstream-only commits. Four upstream commits are newer than this audit checkout. They include a changed lockfile and Djinn source. Other live Cargo builds still use the shared checkout, so no merge was made during this pass. The evidence remains bound to the stated checkout rather than claiming it contains the fetched commits. Recheck build ownership and merge safely before the next basis transition; inspect changed source and refresh coverage through the existing transition rules.

All946 prior protected production/test hashes, prior cited audit evidence and the canonical document prefix remain preserved. No production or original test source was edited. A570-A589 record both implementation splits and verified shared ownership. The [product/host checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds complete coverage, fresh native evidence, upstream state, candidates and remaining limits. The full-source goal remains active.
'''
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
(HERE / 'product_host_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = [
    'record_product_host_review.py', 'capture_graphshell_terminal.json', 'capture_graphshell_build.jsonl',
    'capture_graphshell_build_stderr.log', 'capture_transaction_probe.rs', 'run_current_capture_transaction_probe.py',
    'current_capture_transaction_probe_receipt.json', 'current_capture_transaction_probe_compile.log',
    'current_capture_transaction_probe_output.log', 'product_host_upstream_status.json', 'product_host_diff_check.txt',
]
for stem in ['product_transfer', 'presentation_family']:
    evidence += [stem + '_probe.rs', 'run_' + stem + '_probe.py', stem + '_probe_receipt.json',
                 stem + '_probe_compile.log', stem + '_probe_output.log']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {
    'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'capture_extension_review_receipt.json', 'sha256': sha(HERE / 'capture_extension_review_receipt.json')},
    'kind': 'Complete product/host/presentation source review with fresh native diagnostics; full Mere audit incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 714200,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'new_findings': [f['id'] for f in new_findings], 'native_reproduced_findings': ['Q240', 'Q241', 'Q242', 'Q247', 'Q248', 'Q249'],
    'authority_entries': [e['id'] for e in entries], 'native_Rust_tests': 11,
    'original_full_module_test_suites_run': 0, 'production_edits_this_pass': False,
    'next_read': next_read, 'upstream': upstream_status,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {n: sha(HERE / n) for n in evidence},
    'completed_probe_outputs_removed': True, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Full audit incomplete. Native diagnostics use synthetic public API fixtures and supplied trusted authorization. No installed profile/browser/extension, user files, headed rendering, physical disk fault, transport or custody trial. Other shared build owners preserve their target and checkout basis.',
}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'whole_files': len(whole), 'whole_chars': 714200,
                  'new_findings': receipt['new_findings'], 'native_tests': 11,
                  'authority_entries': receipt['authority_entries'], 'guard_paths': len(guards), 'retained_bytes': budget}, indent=2))
