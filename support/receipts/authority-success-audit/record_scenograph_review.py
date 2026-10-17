"""Record source coverage, contained repairs and unresolved ownership boundaries."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n, value):
    (HERE/n).write_text(json.dumps(value, indent=2)+'\n', encoding='utf-8')
def upsert(items, additions, key):
    for value in additions:
        at = next((i for i, item in enumerate(items) if item.get(key) == value[key]), None)
        if at is None:
            items.append(value)
        else:
            items[at] = value

preserved = load('scenotime_review_receipt.json')['preserved_production_and_test_hashes']
assert all(sha(ROOT/p) == digest for p, digest in preserved.items())
assert load('pending_read.json') == []
green, red = load('scenograph_green_gates.json'), load('scenograph_red_gates.json')
assert len(green['gates']) == 4 and all(g['exit'] == 0 for g in green['gates'])
assert [g['test_results'] for g in green['gates']] == [
    [['ok', '48', '0']], [['ok', '52', '0']],
    [['ok', '26', '0'], ['ok', '3', '0'], ['ok', '9', '0']],
    [['ok', '212', '0'], ['ok', '7', '0'], ['ok', '3', '0'], ['ok', '4', '0']]]
assert all(g['exit'] != 0 and g['test_results'] == [['FAILED', '0', '1']] for g in red['gates'])
for result in (green, red):
    assert all(sha(HERE/g['log']) == g['sha256'] for g in result['gates'])
assert all(sha(ROOT/p) == digest for p, digest in green['source_sha256'].items())
rows = (HERE/'scenograph_red_rows.log').read_text(encoding='utf-8-sig', errors='replace')
assert 'left: 7' in rows and 'right: 6' in rows and 'test result: FAILED. 0 passed; 1 failed;' in rows
client_log = HERE/'scenograph_green_client.log'
client_text = client_log.read_text(encoding='utf-8-sig', errors='replace')
client_counts = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;', client_text)
assert client_counts == [('ok', '79', '0'), ('ok', '1', '0'), ('ok', '5', '0'), ('ok', '2', '0'), ('FAILED', '4', '1')]
assert 'a_fold_free_trace_writes_back_byte_identical ... FAILED' in client_text
client_total = sum(int(passed) for _, passed, _ in client_counts)
emitted = json.loads(next(line for line in client_text.splitlines() if line.startswith('  left: '))[8:])
original = json.loads(next(line for line in client_text.splitlines() if line.startswith(' right: '))[8:])
extra_defaults = ',"link_key":null,"marks":{"forward":false,"reverse":false,"opposed":false},"strength":null'
assert emitted.replace(extra_defaults, '') == original
wire_paths = ['crates/cambium/scenes/sceno/src/scene.rs',
              'crates/cambium/scenes/scenotime/tests/fixtures/mer3ly_projection_trace.json',
              'crates/graphshell/graphshell-client/tests/site_fold_parity.rs']
assert all((ROOT/p).read_bytes().replace(b'\r\n', b'\n') == subprocess.check_output(['git', 'show', 'HEAD:'+p], cwd=ROOT) for p in wire_paths)
save('scene_wire_identity_diagnostic.json', {'original_bytes': len(original.encode()), 'emitted_bytes': len(emitted.encode()),
    'added_default_groups': emitted.count(extra_defaults), 'removing_only_new_default_groups_restores_exact_fixture': True,
    'working_tree_matches_head_after_line_ending_normalization': wire_paths,
    'source_sha256': {p: sha(ROOT/p) for p in wire_paths},
    'scope': 'Executed existing native trace test; serializer, fixture and test are unchanged from merged HEAD. No separate clean-checkout full-package run or product archive mutation claimed.'})
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
paths = [p for p in inventory if p.startswith('crates/cambium/scenes/scenograph/')]
assert len(paths) == 14
extra = ['crates/cambium/scenes/scenomise/Cargo.toml',
         'crates/cambium/scenes/scenomise/src/arrangement.rs',
         'crates/cambium/scenes/scenomise/src/authoring.rs',
         'crates/cambium/scenes/scenomise/src/backdrops.rs',
         'crates/cambium/scenes/scenomise/src/opened_link.rs',
         'crates/script/rhai/tests/scenograph_rules.rs',
         'crates/graphshell/graphshell-client/src/remote.rs',
         'crates/graphshell/graphshell-client/src/remote/tests.rs']
for p in paths + extra:
    assert coverage[p]['sha256'] == inventory[p]['sha256'] == sha(ROOT/p), p
    assert coverage[p]['through'] == inventory[p]['chars'], p

notes, authority = load('review_notes.json'), load('authority_map.json')
findings = [
    {'id': 'Q070', 'status': 'reproduced before repair; repaired and package-tested',
     'title': 'Inspector compatibility copies could reverse the default opened-link layout',
     'source': 'crates/cambium/scenes/scenograph/src/link.rs:OpenedLinkLayout::default_for',
     'related_sources': ['crates/cambium/scenes/scenograph/src/link.rs:LinkReading::rest_summary', 'crates/canvas/pictograph/src/canvas/link_reading.rs:legacy_data', 'ports/graphshell/src/web_tree/link_inspection.rs:opened_layout'],
     'evidence': 'Two dated assertions and one portable Traversal member selected Timeline. Adding only its inspector compatibility summary selected List before repair. The producer emits both representations; rest_summary already excludes the duplicate. The regression failed before the implementation changed.',
     'scope': 'Compiled shared model and current Graphshell caller/producer source trace. No headed browser claim.',
     'repair': 'LinkBucket.unrepresented_legacy_summaries is the shared rule for summaries without a portable member of the same family. The layout counts only these fallback summaries, preserving summary-only readings and real undated-member majorities.',
     'verification': 'scenograph_review_receipt.json',
     'repair_triage': {'disposition': 'contained repair completed', 'reason': 'Compatibility copies do not represent independent members; no graph/source identity change.'}},
    {'id': 'Q071', 'status': 'reproduced on real Rhai before repair; repaired and package-tested',
     'title': 'Boolean composition could turn a failed script into a matching presentation rule',
     'source': 'crates/cambium/scenes/scenograph/src/presentation.rs:evaluate',
     'related_sources': ['crates/cambium/scenes/scenograph/src/presentation/scripts.rs:run', 'crates/script/rhai/tests/scenograph_rules.rs:composed_script_failures_leave_the_whole_rule_unmatched'],
     'evidence': 'run collapsed script errors, budget exhaustion and non-bool results to false. Not inverted the false into true; Any with Always could also apply a failed rule. Real Rhai missing_function() under Not reproduced the failure. SE99 expressly requires the whole failed rule to remain unmatched and traced.',
     'scope': 'Presentation effects, not source admission or source authorization. Tests execute native Rhai through the actual ValueEngine trait.',
     'repair': 'Evaluation keeps Result separate from the boolean through All/Any/Not and maps an error to unmatched only at the rule boundary. Eager sibling evaluation and per-script budgets remain. Tests cover three failure kinds in five compositions, valid false under Not, fallback rule selection and failure traces.',
     'verification': 'scenograph_review_receipt.json',
     'repair_triage': {'disposition': 'contained repair completed', 'reason': 'Implements an existing explicit ruling; no new failure policy or script effect.'}},
    {'id': 'Q072', 'status': 'reproduced before repair; repaired and package-tested',
     'title': 'Opened-link compilation rendered compatibility details as extra independent members',
     'source': 'crates/cambium/scenes/scenomise/src/opened_link.rs:collect_entries',
     'related_sources': ['crates/cambium/scenes/scenograph/src/link.rs:LinkBucket::unrepresented_legacy_summaries'],
     'evidence': 'collect_entries independently emitted both each portable legacy member and each raw compatibility summary. The regression added only a summary for an existing member to a six-member fixture and produced seven entries before repair.',
     'scope': 'Native shared list/Timeline compilation. Original disclosure, exact assertions, parallel buckets, member references and inspector data stay available; no product UI execution.',
     'repair': 'The compiler reuses the same fallback-summary iterator as default layout selection. List and Timeline controls each retain six entries; summary-only readings still produce the existing read-only summary row.',
     'verification': 'scenograph_review_receipt.json',
     'repair_triage': {'disposition': 'contained repair completed', 'reason': 'Removes a duplicate projection without deleting compatibility data or inventing assertion identity.'}},
    {'id': 'Q073', 'status': 'existing native compatibility test failed; documented, not repaired',
     'title': 'The fold-free trace byte-identity claim no longer survives relation-field serialization',
     'source': 'crates/cambium/scenes/sceno/src/scene.rs:RoutedRelation',
     'related_sources': ['crates/graphshell/graphshell-client/tests/site_fold_parity.rs:a_fold_free_trace_writes_back_byte_identical', 'crates/cambium/scenes/scenotime/tests/fixtures/mer3ly_projection_trace.json'],
     'evidence': 'The client all-features gate passes 91 tests and fails the existing exact-byte trace test. Reserialization adds 15 groups of default link_key/marks/strength fields, increasing the fixture from 10391 to 11741 bytes. Removing only these added groups restores the original bytes exactly. Serializer, fixture and failing test are unchanged from merged HEAD.',
     'scope': 'Existing native compatibility test, not an observed user archive rewrite or capture lookup failure. No separate clean-checkout package execution. The file still decodes/replays; exact emitted identity differs.',
     'verification': 'scenograph_review_receipt.json', 'diagnostic': 'scene_wire_identity_diagnostic.json',
     'repair_triage': {'disposition': 'deferred broad compatibility repair; one-hour diversion limit',
         'reason': 'Changing default omission affects canonical bytes emitted for existing scenes, traces and captures. Decide versioned canonical encoding versus explicit historical encoding/identity retention, inspect capture-address consumers and qualify old/new records. Do not regenerate the golden or relax byte equality just to claim a green gate.'}},
]
upsert(notes['findings'], findings, 'id')
q66 = next(v for v in notes['findings'] if v['id'] == 'Q066')
q66.update({'status': 'previously reproduced; bounded wording repair now package-tested',
            'repair': 'RemoteSession reports updated rather than diff for Outcome::Changed(true). The two scripted board tests are named for the Snapshot replies they actually supply. No exact wire-mode claim is inferred from the boolean outcome.',
            'repair_verification': 'scenograph_review_receipt.json',
            'repair_triage': {'disposition': 'contained repair completed', 'reason': 'A generic truthful update label avoids introducing a new outcome/protocol API solely for UI wording.'}})
# Historical read-only checkpoints do not override the directly authorized repair policy.
for v in notes['findings']:
    triage = v.get('repair_triage', {})
    if triage.get('disposition') == 'documented; active goal excludes production edits':
        triage['disposition'] = 'open; prior read-only checkpoint, reassess under contained-repair policy'
q69 = next(v for v in notes['findings'] if v['id'] == 'Q069')
q69['repair_triage']['disposition'] = 'deferred broad repair; one-hour diversion limit'
q69['repair_triage']['scope_reason'] = 'A complete repair spans shared geometry, snapshot/diff admission, portable capture and trace reopening, and downstream solvers. Keep this larger consumer/serialization pass documented rather than fixing one NaN field and claiming all portable scenes valid. Finite admission can be separated from any new negative/degenerate-geometry policy.'
entries = [
    {'id': 'A072', 'domain': 'Legacy member and inspector compatibility-copy counting',
     'claim': 'A raw inspector summary is not an additional portable member.',
     'implementations': ['Pictograph legacy_data emits a Traversal member plus optional summary', 'Rest summary counts portable legacy members', 'Default layout previously counted every member plus summary', 'Scenomise previously emitted every member plus summary'],
     'assessment': 'Q070/Q072 demonstrated real divergence. Default layout and compilation now share unrepresented_legacy_summaries, preserving raw-only fallback and original inspector payload. This is a local shared rule, not proof that all inspector/projection paths use one enumerator.',
     'state': 'two regressions failed before repair and pass afterward', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A073', 'domain': 'Script failure and boolean condition truth',
     'claim': 'SE99 requires a failed script to leave its whole rule unmatched, traced.',
     'implementations': ['ValueEngine yields typed value/error', 'Script runner previously flattened error into false', 'All/Any/Not composed only booleans', 'Rule boundary now isolates Result errors'],
     'assessment': 'Q071 repaired the lost distinction. No script backend is linked into Scenograph itself; the real Rhai integration exercises the supplied-engine seam. Host registered functions remain host policy; globals are not a sandbox guarantee.',
     'state': 'real-engine failure/composition matrix and existing suites pass', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A074', 'domain': 'Public source revision versus process-local witness',
     'claim': 'Durable reproducibility evidence and runtime freshness have different owners.',
     'implementations': ['PublicSourceRevision persists in provenance', 'RuntimeSourceBinding carries an opaque witness without serde/Clone/Debug', 'bind_runtime checks declared source/evidence but delegates verification to the host', 'Recorded RuntimeVerified marker discards the witness'],
     'assessment': 'Source equality and evidence shape are real checks. The marker is not a replayable witness or proof of current authorization; public witness construction does not claim otherwise.',
     'state': 'all Scenograph source read, binding tests pass', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A075', 'domain': 'Portable recipe validation versus arrangement and dynamics admission',
     'claim': 'Portable authoring models own local shape; compiler/host catalogs own interpretation.',
     'implementations': ['Draft.to_definition validates required fields and provenance', 'Authored.validate reuses draft validation for every declared source', 'DynamicsSlot canonicalizes a JSON-object envelope, not all physics terms', 'Arrangement option rows share Family/registry declarations in arrangement.rs'],
     'assessment': 'The layer separation is explicit. Public ProjectionDefinition fields/serde and raw serializers do not themselves establish validated executable input. Full compiler/custom-registry admission is ahead in the source review; the option single-declaration claim is only traced through arrangement helpers so far.',
     'state': 'model and arrangement-helper source read; remaining compiler review open', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A076', 'domain': 'View attention, seed positions and suggestion settings',
     'claim': 'View-local presentation state does not become graph/source authority.',
     'implementations': ['AttentionBook separates selection/foreground pins/retention/suggestion', 'SeedBook supplies view-local seed positions with host-held reset protection', 'SuggestionSettings provides host-configurable source filters and budgets, off by default', 'Upstream product scene-state persistence is separate consumer work'],
     'assessment': 'These are distinct local concerns, not competing durable graph owners. This pass reads the newly merged suggestion module but does not qualify upstream saved-scene persistence merely because SeedBook is understood.',
     'state': 'model tests pass; complete product persistence/consumer review remains ahead', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A077', 'domain': 'Projection editor draft history and save acknowledgment',
     'claim': 'Shared authoring state persists only through a host-provided sink.',
     'implementations': ['ProjectionEditor reuses ProjectionDraft validation', 'Backstep owns draft undo/dirty state', 'Workbench owns panel arrangement separately', 'save marks clean only after sink success'],
     'assessment': 'The editor holds no graph, endpoint or persistence authority handle. Sink success is the host adapter contract, not proof of disk durability. Raw mark_saved/load_stored are explicit host callbacks and need consumer qualification.',
     'state': 'authoring source fully read and feature-enabled package gate passes', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A078', 'domain': 'Disclosed scene facts and domain-specific projection judgments',
     'claim': 'Backdrop compilation supplies geometry without acquiring source/application authority.',
     'implementations': ['SceneInput.parse enforces a byte cap and strict nested record keys', 'validate checks bounded contours, placed coverage anchors and instrument pitch consistency', 'compile_metre compares source-disclosed rhyme tails inside supplied groups', 'compile_fretboard links repeated SourceRef values; terrain retains disclosed coverage basis'],
     'assessment': 'No network, pronunciation inference, RF prediction or durable graph write is performed. The geometry-only shorthand still includes domain-specific equality/join judgments and diagnostics. Shared sequence geometry is reused, but specialized metre/instrument/terrain DTOs and compilation remain in this module; this is not a fully generic source-neutral compiler claim.',
     'state': 'backdrops.rs fully read; template implementation/consumer review ahead', 'verification': 'scenograph_review_receipt.json'},
    {'id': 'A079', 'domain': 'Backward decoding versus canonical byte identity',
     'claim': 'The old fold-free trace test claims byte-identical reserialization.',
     'implementations': ['RoutedRelation defaulted fields preserve decoding', 'Derived serialization now writes those defaults explicitly', 'SceneTrace rebuilds current typed records', 'The existing exact-byte fixture gate detects the difference'],
     'assessment': 'Q073 demonstrates that decoder compatibility does not imply canonical-byte or content-address compatibility. The unchanged native gate fails; the broader encoded-identity policy remains open.',
     'state': 'one existing client compatibility test fails; exact added-field diagnostic recorded', 'verification': 'scenograph_review_receipt.json'},
]
upsert(authority['entries'], entries, 'id')
upsert(notes['candidates'], [
    {'title': 'Public projection definition construction can bypass draft validation', 'source': 'crates/cambium/scenes/scenograph/src/lib.rs:ProjectionDefinition', 'note': 'Public fields/serde and to_json_bytes are raw model operations; Draft.to_definition and authored binding validate explicitly. Trace Scenomise/host admission before claiming invalid definitions become executable or durable successes.'},
    {'title': 'Dataset column types and row scalar shape have no model-local admission', 'source': 'crates/cambium/scenes/scenograph/src/dataset.rs', 'note': 'Typed column declarations coexist with arbitrary row values and native nonfinite numbers. Identify compiler use and refusal boundaries before calling this a source/schema enforcement failure.'},
    {'title': 'Multi-cell reflection cycles require compiler resolution qualification', 'source': 'crates/cambium/scenes/scenograph/src/swatch.rs:Facet::validate', 'note': 'Facet.validate rejects self/dangling references but not a cycle among two or more cells. Its stated local contract is narrower; inspect compiler traversal/bounds before reporting an infinite resolution path.'},
    {'title': 'Script budgets bound each leaf rather than an entire occurrence resolve', 'source': 'crates/cambium/scenes/scenograph/src/presentation.rs:evaluate', 'note': 'All/Any evaluate every child and each script receives 100000 steps. Rule/condition count limits do not imply a shared per-frame budget. This is an unmeasured responsiveness concern requiring host occurrence count and performance evidence, not an observed hang.'},
    {'title': 'Remote object label counts allocated table slots rather than active items', 'source': 'crates/graphshell/graphshell-client/src/remote.rs:RemoteSession::label', 'note': 'label uses tables.items.len(), which can include tombstones. Sparse admitted-scene behavior needs a direct label/control test before promoting a new misleading-count finding.'}
], 'title')
upsert(notes['source_qualifications'], [
    {'area': 'Scenograph full source', 'note': 'All 14 current files fully read, including the newly merged suggestion settings and every script/default-feature branch. Post-repair files were reread at their new hashes. Source binding is not source verification; dynamics JSON envelope validation is not full host schema admission.'},
    {'area': 'Scenomise partial source', 'note': 'Manifest, arrangement.rs, authoring.rs, backdrops.rs and opened_link.rs fully read. Full catalog/compiler/template and solver source remain unread, irrespective of package-test success.'},
    {'area': 'Scenograph repair execution', 'note': 'Three new regressions failed before implementation repair. Final gates pass 48 default Scenograph, 52 scripts-enabled Scenograph, 38 real Rhai and 226 authoring-enabled Scenomise test executions. Client all-features gate passes 91 and fails one existing byte-identity fixture test, Q073; the remote wording tests pass. Native package evidence only; no headed browser, wasm or product durability claim.'}
], 'area')
if 'crates/cambium/scenes/scenograph' not in notes['completed_areas']:
    notes['completed_areas'].append('crates/cambium/scenes/scenograph')
stats = json.loads(subprocess.check_output(['python', str(HERE/'read_coverage.py'), 'status'], cwd=ROOT))
next_read = {'path': 'crates/cambium/scenes/scenomise/src/catalog.rs', 'start': 0}
notes['basis_commit'] = green['basis_commit']
notes['objective'] = 'Full-source audit of Mere with contained, directly authorized in-place fixes; record duplicated authority and misleading success alongside individual bugs. Preserve concurrent work. Document broad/policy-dependent repairs beyond the one-hour diversion limit.'
notes['current_area'] = 'Scenograph complete; Scenomise partial source review with bounded repairs'
notes['next_area'] = 'Continue Scenomise catalog/compiler/solvers, then remaining Mere source'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete', 'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': ['scenograph_review_receipt.json'], 'next_read': next_read, 'diagnostic_outcomes': 'Q070-Q072 repaired after failing regressions; Q066 wording corrected. Four scene/script gates pass 364 test executions; client gate adds 91 passing and one existing failing byte-identity test, Q073.', 'workspace': 'Reused stable C:/t/cargo-targets/mere; no isolated outputs, Cargo home or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: Scenotime full read, Q069 documented', 'classification_of_previous_turn': 'progress', 'production_edits_this_turn': True, 'completed_areas_this_pass': ['crates/cambium/scenes/scenograph/'], 'coverage_checkpoint': stats, 'next_read': next_read, 'user_instruction': notes['repair_policy']['user_instruction'], 'authorization_note': 'Direct user authorization to fix in place and seek no-brainers takes precedence over the older stored-goal no-production-edits wording. Historical no-edit triage is no longer a reason to leave bounded repairs untouched.', 'coverage_note': 'Tests do not credit unread source. All edited source files reread at final hashes.'}
assert len({v['id'] for v in notes['findings']}) == len(notes['findings'])
assert len({v['id'] for v in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = f'''

## Full-source audit and contained repairs, 2026-10-16

**Status:** Scenograph's 14 current source/manifest/example files are fully read.
Scenomise's manifest, arrangement helpers, authoring reducer, disclosed-backdrop model and opened-link
compiler are read; its remaining compiler/solver source is still under review.
The whole Mere audit remains active. Upstream main through `434aa49c` was merged
without conflicts at `4523cb6f`; concurrent dirty work was preserved.

Three defects have bounded repairs (Q070–Q072). Default link layout counted a
portable legacy member and its inspector summary twice: adding the summary
alone reversed Timeline to List. The opened-link compiler similarly turned six
members into seven scene items. Both now use
`LinkBucket::unrepresented_legacy_summaries`: a summary is a fallback row/count
only when its family has no portable member in that bucket. Original inspector
data and summary-only readings remain available; distinct buckets stay distinct.
This establishes one shared compatibility-copy rule for these two consumers,
not one authority for every link inspector or graph writer.

The script runner flattened runtime errors, exhausted budgets and non-bool
returns to false, allowing `Not` or `Any` to apply a failed rule. SE99 already
requires that whole rule to remain unmatched. Errors now survive boolean
composition until the rule boundary; the trace retains the failure reason and
other rules still resolve. Real Rhai tests cover all three failure kinds under
five compositions, plus genuine false negation and a working fallback rule.

Source ownership qualifications are recorded beside the bugs. Public source
revisions are durable provenance; runtime witnesses remain host-verified and
are discarded on recording. `RuntimeVerified` is not replayable freshness.
DynamicsSlot validates a canonical JSON-object envelope, leaving physics terms
to the binding host. Raw public definitions/serializers do not themselves prove
that a compiler or store admitted the definition. Option declarations are shared
through the arrangement helpers, with complete compiler/custom-registry use
still to be reviewed. View attention, seeds and suggestions retain their local
owners; newly merged product persistence is not qualified by these model tests.

The broader Q069 finite-geometry/wire-validity repair remains documented under
the user's one-hour audit-diversion limit. Its done-condition is consistent
finite admission across snapshots, committed diffs, portable captures and traces,
with successful reopening controls and downstream solver coverage. A one-field
patch would leave the misleading portable-success claim intact. Negative and
degenerate geometry policy can remain separate from finite-value rejection.

All three new regressions failed before the repairs. Final gates pass 48 default
Scenograph, 52 script-enabled Scenograph, 38 real Rhai and 226 authoring-enabled
Scenomise test executions. The separate all-feature client gate passes
{client_total} tests, including the remote wording repair (Q066), and fails one
existing exact-byte trace test (Q073). It adds 15 groups of default relation
fields on reserialization, increasing the old trace from 10391 to 11741 bytes;
removing only those groups restores the original bytes exactly. Serializer,
fixture and failing test match merged HEAD. Decoding compatibility is not byte
identity. A versioned canonical-encoding/identity review is deferred under the
one-hour limit; neither serializer defaults nor the golden were changed merely
to turn this gate green. These are native package
receipts; headed browser, wasm and whole-product acceptance are not claimed.
The [audit receipt](../../../support/receipts/authority-success-audit/scenograph_review_receipt.json)
binds source, logs, coverage, deferred work and preservation checks.
'''
doc_record = {}
for path, text in [(doc, addition),
    ('design_docs/mere_docs/technical_architecture/2026-10-15_transport_agnosticism_note.md', '''

### Audit wording repair, 2026-10-16

Q066's remote resume receipt now reports `updated` rather than inferring `diff`
from a changed-result boolean. That outcome can represent a complete replacement
snapshot. The two scripted-board tests now name the Snapshot replies they
actually supply. This bounded repair makes the label truthful; it does not add
wire-mode provenance to Outcome or repair the separate unexpected-close and
resource-correlation findings. The all-feature client gate passes 91 tests,
including these remote tests, and fails the existing exact-byte trace test
(Q073), whose default-field reserialization changes are documented separately.
Native results are recorded in
the [audit receipt](../../../support/receipts/authority-success-audit/scenograph_review_receipt.json).
''')]:
    destination = ROOT/path
    current = destination.read_bytes()
    marker = text.splitlines()[2]
    # Idempotent recording; never overwrite another lane's earlier content.
    if marker.encode() not in current:
        original = current
        destination.write_bytes(original + text.encode('utf-8'))
        doc_record[path] = {'preserved_prefix_sha256': hashlib.sha256(original).hexdigest(), 'preserved_prefix_bytes': len(original), 'sha256': sha(destination)}
    else:
        doc_record[path] = {'sha256': sha(destination), 'existing_append_retained': True}

diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE/'scenograph_review_diff_check.txt').write_text(diff.stdout+diff.stderr, encoding='utf-8')
assert diff.returncode == 0
assert all(sha(ROOT/p) == digest for p, digest in preserved.items())
evidence = ['run_scenograph_gates.py', 'scenograph_red_gates.json', 'scenograph_green_gates.json', 'scenograph_red_rows.log', 'scenograph_green_client.log', 'scene_wire_identity_diagnostic.json', 'scenograph_review_diff_check.txt']
evidence += [g['log'] for r in (red, green) for g in r['gates']]
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': notes['basis_commit'], 'kind': 'Complete Scenograph source review, partial Scenomise source review and four contained repairs',
           'new_findings': ['Q070', 'Q071', 'Q072', 'Q073'], 'prior_finding_repaired': 'Q066', 'authority_entries': [v['id'] for v in entries], 'production_edits_this_pass': True,
           'source_files_in_complete_area': len(paths), 'source_chars_in_complete_area': sum(inventory[p]['chars'] for p in paths), 'coverage': stats, 'next_read': next_read,
           'source_sha256': {p: sha(ROOT/p) for p in paths+extra}, 'preserved_production_and_test_hashes': preserved,
           'upstream_integration': {'before': 'b567660d67c4d900f1d638345c89715e1f36c575', 'fetched_main': '434aa49c', 'merge': '4523cb6f4b3998a27dc474ba30a7f1df537c5ec3', 'conflicts': False},
           'regression_controls': {'new_pre_fix_failures': 3, 'final_scene_script_test_executions': 364, 'client_passed': client_total, 'client_failed': 1, 'client_test_results': client_counts},
           'client_command': 'cargo test -p graphshell-client --lib --tests --all-features --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --config build.build-dir=C:/t/cargo-targets/mere/build',
           'documents': doc_record, 'retained_isolation': [], 'coverage_limits': ['Whole Mere source review remains incomplete.', 'Package test success is not source-reading credit.', 'No headed browser, wasm, actual network teardown or product durability qualification.'],
           'diff_check': 0, 'evidence_sha256': {p: sha(HERE/p) for p in evidence}}
save('scenograph_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'findings': len(notes['findings']), 'authorities': len(authority['entries']), 'test_executions': 364+client_total, 'preserved_files': len(preserved), 'next_read': next_read}))
