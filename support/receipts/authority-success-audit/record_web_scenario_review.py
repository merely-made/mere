"""One-shot complete web-source and browser-adapter reading checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'web_scenario_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('community_host_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1108
assert all(sha(ROOT / n) == digest for n, digest in guards.items())
assert all(sha(HERE / n) == digest for n, digest in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
area = lambda n: n.startswith('ports/graphshell/web/') or n.startswith('ports/graphshell/src/web') or n == 'ports/graphshell/Cargo.toml'
whole = sorted(n for n, value in inv.items() if area(n) and n not in guards
               and cov.get(n, {}).get('sha256') == value['sha256'] and cov[n]['through'] == value['chars'])
assert len(whole) == 211 and sum(inv[n]['chars'] for n in whole) == 524630
web = sorted(n for n in inv if n.startswith('ports/graphshell/web/'))
assert len(web) == 245
assert all(cov.get(n, {}).get('sha256') == inv[n]['sha256'] and cov[n]['through'] == inv[n]['chars'] for n in web)
assert all(sha(ROOT / n) == inv[n]['sha256'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1789 and stats['total_files'] == 3727
assert stats['read_chars'] == 21781805 and not stats['changed_since_inventory'] and not stats['pending_chunks']
metadata = load('web_workspace_membership_receipt.json')
assert all(row['exit_code'] == 0 for row in metadata['rows'])
assert metadata['rows'][0]['graphshell_web'] == [] and metadata['rows'][0]['workspace_member_count'] == 104
assert metadata['rows'][1]['workspace_member_count'] == 1 and metadata['rows'][1]['graphshell_web'][0]['workspace_member']
assert all(sha(ROOT / n) == digest for n, digest in metadata['source_sha256'].items())
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q267' and authority['entries'][-1]['id'] == 'A673'
rows = [
    ('Q268', 'Component stylesheet positions navigation buttons outside the component',
     'ports/graphshell/web/styles.css:134-159,450-466; ports/graphshell/web/embed.html',
     'source-only selector containment defect',
     'The stylesheet that embed.html loads has bare nav button:nth-child selectors, including width/height and left/top/bottom declarations and narrower-viewport overrides. They match navigation buttons belonging to the embedding document as well as Graphshell. The existing embed fixture tests a host input and button inside a paragraph, not host navigation, so those cases do not cover this selector leak. Broad [hidden], .button-row and .file-picker rules also remain page-global; the definite finding here is the nav geometry selectors, not a claim that every generic rule causes a demonstrated visual regression.',
     'Contained selector scoping repair with a host navigation fixture at wide/narrow widths. No browser CSS/layout reproduction was run in this pass.'),
    ('Q269', 'Browser source advertises one shared workspace and a root gate that do not include it',
     'ports/graphshell/src/web.rs:9-21; ports/graphshell/web/Cargo.toml:[workspace]; Cargo.toml:exclude',
     'verified current Cargo metadata and source contradiction',
     'web.rs says graphshell-web remains a root workspace member sharing its lock and patch table, and recommends cargo check -p graphshell-web. The actual web manifest explicitly creates a standalone workspace and restates dependencies and patches. Locked/offline/no-deps Cargo metadata finds 104 root members with no graphshell-web, then one separate web-workspace member. A root workspace check therefore does not establish this browser crate was checked. No Wasm compilation or failure diagnostic for the recommended check command ran here.',
     'Contained source-document correction: identify the standalone workspace and run its target gate from its own directory so the committed Wasm configuration is discovered. Retain the separate dependency/patch maintenance obligation; do not claim one lockfile or root-only coverage.'),
    ('Q270', 'A failed browser rejoin consumes the credentials needed for another reconnect',
     'ports/graphshell/src/web_rtc_link.rs:reconnect/rejoin/RETIRED; ports/graphshell/src/webrtc_browser.rs:complete_rejoin',
     'source-only failed-reconnect ownership defect',
     'reconnect pops the only RetiredSession before starting asynchronous signaling, parks only its frames, then moves its ephemeral identity and delegation into rejoin. Any early error, such as a failed POST /offer, reaches the error callback without restoring the retired credentials. A second reconnect sees no retired session and refuses locally. Successful-reconnect scenarios exercise the one successful attempt, not transient failure followed by retry. This is loss of local retry capability, not proof that admission accepts invalid credentials.',
     'Preserve logical reconnect identity/delegation through a failed attempt and retire channel objects separately. Add failure-then-success and revocation/expiry controls. Actual browser/transport reproduction remains open.'),
    ('Q271', 'Remote callbacks and retirement slots are not bound to the connection they update',
     'ports/graphshell/src/web_rtc_link.rs:connect/join/spawn_pumps/reconnect; ports/graphshell/src/web_tree/remote.rs:RemoteHost implementation',
     'source-only connection-generation and ownership defect',
     'connect can start another join against the same TreeRemote, and adopt replaces its live session. Existing read pumps capture that shared host and call received/stream_closed without checking that their original connection is still current; the real TreeRemote implementation applies those callbacks to its current reservoir/live. A displaced join completion can likewise adopt later. Separately, RETIRED is one thread-local LIFO stack with no logical-host/session key, and reconnect pops whichever session retired last. This source path can misroute old callbacks or select unrelated reconnect state; no live cross-peer disclosure or authorization bypass was demonstrated.',
     'Hairy issue deferred: bind joins, pumps, closure notifications and retired credentials to explicit logical-host/connection identity, fence stale completion, and specify bounded callback retirement. This crosses carrier closure lifetime, reservoir state and browser-host adoption; document it rather than prolong this source review with a broad lifecycle redesign.'),
]
for ident, title, source, status, evidence, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source, 'status': status,
        'evidence': evidence, 'verification': NAME,
        'diagnostics': ['web_workspace_membership_receipt.json'] if ident == 'Q269' else [],
        'repair_triage': {'disposition': 'documented; no production changes', 'reason': repair}})
qualifications = {
    'Q253': 'The fully read Forme scenarios cover serial draft/save/reopen and saved undo. They do not run two independent Forme writers, so they do not contradict the prior native stale-overwrite reproduction.',
    'Q256': 'Presentation-rule scenarios bind exact inspected targets and verify draft/save/reopen on one current graph. They do not replace the active graph while retaining the rules pane, so the prior source ownership finding remains open.',
    'Q257': 'Projection save/undo/reopen scenarios await serial save results. They do not overlap an old save acknowledgement with incoming dataset/editor replacement, so they do not cover the pending-save identity gap.',
    'Q258': 'Forme draft scenario tests Escape then release and movement outside the candidate. Hosted gesture scenarios cover touches elsewhere. None qualifies native DOM touch/pointer cancellation through the Forme drop path; the prior source-only cancellation finding remains open.',
}
for finding in notes['findings']:
    if finding['id'] in qualifications:
        finding['scenario_coverage_qualification'] = {'verification': NAME,
            'scope': 'All current inventoried web scenarios read, not rerun.', 'detail': qualifications[finding['id']]}
arows = [
    ('Standalone browser build authority', 'The web manifest owns its own feature graph, lock and patch table.',
     ['web/Cargo.toml', 'web/.cargo/config.toml', 'src/web.rs', 'web_workspace_membership_receipt'], 'Q269 contradicts the source header claim of root membership. Metadata qualifies package membership only, not current transitive closure or browser compilation.'),
    ('Browser entry aliases', 'Public graphshell-view and graphshell-tree enter the retained tree.',
     ['web.rs::mount', 'loader.js::defineElements', 'index.html', 'tree.html'], 'Old GpuPresenter/tree scenario names no longer select two implementations when executed against current loader. Historical results remain tied to their original source; present paired runs must not be sold as that historical renderer comparison.'),
    ('Embedding boundary', 'The retained mount scopes roots and datasets, but the page stylesheet has global selectors.',
     ['web/styles.css', 'embed.html', 'appearance_embedded.scn', 'c4b2_embed.scn'], 'Q268 bare nav geometry selectors escape component ownership. The fixture covers host controls outside a nav and cannot qualify arbitrary host-page CSS isolation.'),
    ('Scenario execution authority', 'Current web_scenario encodes/publishes retained-lane results and captures.',
     ['web.rs::run_scenario/validate_scenario', 'web_scenario::mark/publish_capture', 'web_tree::TreeLane'], 'Mesquite/Taproot own stepping/parsing; this is a real extraction. Page-level DOM publication is not a second scenario executor. Reading scenario files is not executing them.'),
    ('Host dataset authority', 'Browser fetch disclosure passes strict parsing to projection_compile.',
     ['web_dataset::supplied/supplied_history', 'mount.js::loadHostDataset', 'tree_host_dataset*.scn'], 'Fetch refusal remains an explicit empty-graph refusal; no built-in practice dataset fallback. V1 practice and V2 history are distinct contracts.'),
    ('Appearance ownership', 'Browser evaluation samples Tabard roles in an isolated document.',
     ['web_appearance::Session', 'appearance.rs', 'appearance_modes.scn', 'appearance_embedded.scn'], 'Owned appearance stages evaluation and storage before publishing; embedded mounts read inherited roles rather than application storage. Candidate errors are placed in UI status; accept returning Ok on a handled refusal does not alone prove visible success.'),
    ('Practice workspace owner', 'PracticeHost composes the shared reducer and compiler over supplied comparison data.',
     ['web_practice::PracticeHost', 'practice_workspace', 'practice_disclosure', 'practice_*.scn'], 'One bounded localStorage proof slot saves a source-bound workspace and recipe; reopen checks shared source/revision compatibility. This is not the product graph store, a community authority, or a general native Woodshed adoption claim.'),
    ('Practice layout and motion', 'One Control record set and board own geometry, actions and encoded-axis constraints.',
     ['PracticeHost::controls/advance/board_pointer', 'PhysicsBoard', 'encoded_axes'], 'Retained realization uses shared records; both synthetic relation strokes and supplied source relations are drawn. Motion-off withdraws drag; direct board cancellation is release semantics, distinct from the Forme commit/cancel defect.'),
    ('RTC admission and framing', 'BrowserJoin delegates signed admission; browser session carries NDJSON through shared framing.',
     ['webrtc_browser::BrowserJoin', 'graphshell_host::webrtc_join', 'LineAssembler', 'BrowserFrames'], 'Signaling/fetch belongs to web_rtc_link. No duplicate cryptographic admission authority was found here. BrowserFrames separately owns an unbounded queue; no flooding or resource-ceiling qualification ran.'),
    ('Reconnect and callback lifetimes', 'Transport generations and logical-host identities are not yet one owner.',
     ['web_rtc_link::RETIRED/PARKED', 'spawn_pumps', 'TreeRemote::adopt/received/stream_closed'], 'Q270 loses retry credentials on failure; Q271 leaves stale callbacks and retirement selection unfenced. Parking closed frames deliberately avoids late browser callbacks into dropped closures; removing that workaround alone is not a safe repair.'),
    ('Display period and frame budget', 'Browser-specific worker/clock inputs feed the shared FrameBudget.',
     ['web_period_worker::PeriodWorker', 'web_speed', 'graphshell::frame_budget', 'p0_worker_*.scn', 'p6_tree_speed_*.scn'], 'Shared budget logic is not reimplemented here; browser clock grain, source selection and explicit planted controls remain local. Worker Blob URLs are not explicitly revoked in this adapter; no resource leak magnitude was measured.'),
    ('Timing and device ownership', 'FrameTiming records CPU stages and optional device timestamps around host submissions.',
     ['web_timing::FrameTiming', 'TreePage producer suspend', 'visibility_entry.html'], 'Hidden epochs survive pauses; unavailable GPU readback reports none. suspend releases device-bound marks while retaining CPU windows. Timestamp spans include submission idle; historical dual-presenter prose is stale on current aliases.'),
    ('Budget and physics evidence', 'Scenario signatures qualify selected graphs and measured steps, not arbitrary algorithm correctness.',
     ['p0_viewer_site_frames_21.scn', 'p5_tree_gpu_settle_2000.scn', 'physics_*.scn', 'p4_tree_physics_*.scn'], '21-node workload is explicitly synthetic; GPU settle checks dispatched steps/failures and CPU signature bounds. Law profile/count/framing checks and positive plants are not universal numerical, scheduling or hardware evidence.'),
    ('Rules and source-view separation', 'Presentation scenarios separate draft decisions, saved generations and enabled realization.',
     ['p4_tree_presentation_rules*.scn', 'p4_tree_presentation_live*.scn', 'p4_tree_node_ladder*.scn'], 'Unsupported winning Snapshot representation is checked as unsupported. Forme boundary ownership is distinct from guide visibility. Scenarios do not cover active-graph replacement or overlapping acknowledgements.'),
    ('Community and durable-view evidence', 'Accepted feedback, fresh owner facts and browser attention have separate assertions.',
     ['p4_tree_moot_resources*.scn', 'p4_tree_moot_resource_handlers*.scn', 'p6_tree_family_assertion*.scn', 'p7_tree_promoted_link*.scn'], 'Handler acceptance does not manufacture retention/hosting before refresh; promise remains availability-unverified. Hidden links/assertions preserve source facts; promoted copies retain their original assertions. These scripts were not rerun.'),
    ('Retained document and query evidence', 'Content sessions, frozen query outputs and cross-stratum reads keep named owners.',
     ['p4_tree_retained_content*.scn', 'query_views*.scn', 'clip_export_ready.scn'], 'Glyph-run/producer counts and playback refusal are bounded observations, not physical download/audio qualification. Query resource/surface counts and inspection hop costs are source/projection-specific.'),
]
entries = [{'id': f'A{674+i:03d}', 'domain': d, 'claim': claim, 'implementations': impl,
            'assessment': assessment, 'state': 'current whole-source review; scope explicitly qualified', 'verification': NAME}
           for i, (d, claim, impl, assessment) in enumerate(arows)]
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/docs/receipts/caller_owned_toolkits_20261013/.gitattributes', 'start': 0,
             'reason': 'Read remaining original Graphshell receipt tools, native examples/binaries/tests and all other unread Mere source; web/ source and browser adapter family are complete at this basis.'}
notes['current_area'] = 'All inventoried web/ source plus all web_* browser adapters and Graphshell manifest'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'web_workspace_membership_receipt.json'],
    'production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None,
    'diagnostic_outcomes': 'Two locked/offline/no-deps metadata commands succeeded. No new runtime tests, browser run or compilation.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
with (HERE / 'web_scenario_diff_check.txt').open('x', encoding='utf-8') as stream:
    stream.write(diff.stdout + diff.stderr)
evidence = ['record_web_scenario_review.py', 'web_scenario_diff_check.txt', 'web_workspace_membership_receipt.json']
evidence_hashes = {n: sha(HERE / n) for n in evidence}
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
addition = f'''

### Complete web-source and browser-adapter review, 2026-10-17

At {head}, this pass fully reads 211 additional files / 524,630 characters. Every one of the 245 inventoried source files under ports/graphshell/web is now complete, including scenarios, scripts, configuration, HTML and styles. The remaining thirteen web_* / webrtc_browser source modules and Graphshell's parent feature manifest are also completely read. Full coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. Full Mere review remains incomplete. All prior 1,108 protected source paths, twelve cited evidence hashes and the entire previous document prefix remain preserved.

Q268 is a source-only embedding violation: styles.css has unscoped nav button:nth-child geometry rules at wide and narrow widths. They also match the host page's navigation; the current embed fixture tests controls inside a paragraph, so it does not qualify nav isolation. Q269 is a verified authority/gate contradiction: web.rs says graphshell-web stays a root member sharing one lockfile and patch table. The actual manifest makes it standalone and restates dependencies/patches. Two successful locked/offline/no-deps metadata commands show 104 root members without graphshell-web, then one separate web-workspace member. No target compilation or recommended-command compiler failure was run.

Q270 is source-only: reconnect pops the retained session and moves its identity/delegation into asynchronous rejoin before signaling succeeds. An early signaling failure restores neither, so another reconnect has no retired identity to use. Q271 is a broader source-only ownership problem: joins/read pumps/close notifications capture a shared TreeRemote without a connection-generation guard, while adopt replaces its current live session. Old callbacks therefore reach the current session. The thread-local retirement stack also has no logical-host/session key. These findings establish missing ownership checks, not a demonstrated live cross-peer disclosure or cryptographic admission bypass. The callback/retirement repair is documented as larger work; parking browser frames deliberately prevents late callbacks into dropped closures and must not simply be removed.

A674-A689 distinguish real reuse from duplicated local ownership. The public element aliases now enter one retained tree, so old/new presenter script names cannot recreate a two-renderer comparison at current source. Historical results retain their original basis. Mesquite/Taproot own stepping/parsing; web_scenario only publishes retained-lane results and captured pixels. Dataset parsing, reducers, compiler, PhysicsBoard and FrameBudget are shared; browser fetch/configuration, source-bound practice localStorage, CSS role evaluation, clock/worker plumbing and transport lifecycle remain adapters with their own responsibilities.

The scenarios were completely read and not rerun. They explicitly distinguish synthetic workload size from actual site data, decision previews from enabled realization, accepted action receipts from refreshed custody/hosting facts, promises from verified availability, hidden view intent from source withdrawal, and promoted frozen copies from live assertions. They do not cover Q253 independent stale writers, Q256 active-graph rule rebinding, Q257 overlapped save acknowledgement or Q258 Forme touch/pointer cancellation; those prior findings retain their original runtime/source qualifications. No new browser, network, physical input, GPU, audio or disk qualification is claimed. Timing releases device marks on suspend, and hidden epochs survive frame pauses; GPU timestamps include idle between submissions.

The browser worker creates Blob URLs without an explicit revoke, and BrowserFrames owns an unbounded input queue. These are tracked local lifecycle/budget observations; no leak magnitude or flooding result was measured. The appearance adapter reports handled candidate refusal through UI status; an Ok return alone was not promoted to a false visible-success finding. The optional connect_remote export similarly warns on immediate refusal rather than claiming an authenticated link.

No production/original tests changed, no isolated target/Cargo home/worktree was created, and no audit-owned runtime job is pending. The dirty shared main basis remains unchanged while another Mere build is live; fetched upstream has not been merged. Next is remaining historical receipt source/tooling, native Graphshell consumers and every other unread Mere source. The [web source checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds complete file coverage, four findings, ownership map and metadata evidence. Goal remains active.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'community_host_review_receipt.json', 'sha256': sha(HERE / 'community_host_review_receipt.json')},
    'kind': 'Complete inventoried web-source and browser-adapter review; full Mere objective incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 524630,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'completed_subtree': {'prefix': 'ports/graphshell/web/', 'whole_files': len(web),
                          'chars': sum(inv[n]['chars'] for n in web)},
    'new_findings': [r[0] for r in rows], 'qualified_previous_findings': list(qualifications),
    'authority_entries': [e['id'] for e in entries], 'new_runtime_tests_run': 0,
    'read_only_metadata_commands_succeeded': 2, 'production_edits_this_pass': False,
    'pending_verification': None, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': evidence_hashes, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Q268/Q270/Q271 are source-only. Q269 verified through locked/offline/no-deps Cargo metadata; no compilation. Complete source reading is not scenario execution, headed/browser/network/audio/GPU/device or installed-profile qualification.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_whole_files': len(whole),
                  'new_chars': 524630, 'web_subtree_files_complete': len(web),
                  'new_findings': receipt['new_findings'], 'protected_paths': len(guards),
                  'retained_bytes': budget}, indent=2))
