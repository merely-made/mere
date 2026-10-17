"""Record full chrome/comms reads and exactly scoped portable diagnostics."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
NAME = 'shell_chrome_comms_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    with (HERE / n).open('x', encoding='utf-8') as f: json.dump(value, f, indent=2); f.write('\n')
def git(*args): return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
assert not (HERE / NAME).exists()
old = load('servitor_review_receipt.json'); head = git('rev-parse', 'HEAD'); assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}; assert len(guards) == 2110
aux, runtime = old['auxiliary_read_sha256'], old['runtime_input_sha256']
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in old['evidence_sha256'].items())
assert all(sha(Path(p)) == h for p, h in old['external_contract_sha256'].items())
shared = old['shared_working_file_sha256']; doc = ROOT / old['document']['path']; prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute(): index = ROOT / index
index_hash = sha(index); assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2602 and stats['read_chars'] == 30727208
assert inv['commit'] == head and load('pending_read.json') == [] and not stats['changed_since_inventory']
areas = [('crates/shell/chrome/', 12, 103871), ('crates/shell/comms/', 10, 88398)]
fresh = [p for p in inv['files'] if any(p.startswith(a) for a, _, _ in areas)]
assert len(fresh) == 22 and sum(inv['files'][p]['chars'] for p in fresh) == 192269 and not set(fresh).intersection(guards)
assert all(cov[p]['through'] == inv['files'][p]['chars'] and cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in fresh)
route, units, chrome = load('comms_routing_probe_receipt.json'), load('comms_core_unit_retry_receipt.json'), load('chrome_navigation_probe_receipt.json')
assert route['audit_cases'] == 2 and route['original_core_tests'] == 0
assert {k: v['exit'] for k, v in route['results'].items()} == {'owner_compile': 0, 'unit_compile': 1, 'consumer_compile': 0, 'output': 0}
assert units['build_exit'] == units['run_exit'] == 0 and units['original_core_tests'] == 20
assert chrome['build_exit'] == chrome['run_exit'] == 0 and chrome['original_tests'] == 32 and chrome['audit_cases'] == 2
for name, receipt, has_probe in [('comms_routing_probe', route, True), ('comms_core_unit_retry', units, False), ('chrome_navigation_probe', chrome, True)]:
    assert receipt['runner_sha256'] == sha(HERE / ('run_' + name + '.py'))
    if has_probe: assert receipt['probe_sha256'] == sha(HERE / (name + '.rs'))
    assert all(sha(ROOT / p) == h for p, h in receipt['source_sha256'].items())
    assert all(sha(Path(v['path'])) == v['sha256'] for v in receipt['libraries'].values())
    assert receipt['generated_output_removed'] and all(not Path(v['path']).exists() for v in receipt['removed_outputs'])
assert units['prior_receipt']['sha256'] == sha(HERE / 'comms_routing_probe_receipt.json')
assert sha(Path(units['native_library']['path'])) == units['native_library']['sha256']
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q375' and authority['entries'][-1]['id'] == 'A1087'
c, m = 'crates/shell/chrome/src/', 'crates/shell/comms/src/'
rows = [
('Q376', 'Merged comms inbox loses adapter ownership when multiple backends share a protocol',
 m + 'comms.rs:53,59,84,127; ' + m + 'model.rs:68',
 'Comms accepts two same-protocol adapters and lists both inboxes, but adapter_for always finds the first by protocol. With distinct thread keys, reading/sending the second listed thread returns NotFound; reversing registration makes it reachable. With the same protocol/key on two owners, the second titled row reads the first owner data and the supplied fake send returns the first owner ID. ConversationId carries no adapter/account identity.',
 route['qualification'], ['comms_routing_probe_receipt.json', 'comms_core_unit_retry_receipt.json'],
 'Either enforce one adapter per protocol at registration with a visible refusal, or preserve adapter/account identity through conversation IDs, failures, selection, drafts and every operation. Do not try every backend until one sends: that can author through the wrong identity. Multi-account address migration is a wider issue to document under the audit budget.'),
('Q377', 'Suggestion resolution silently replaces an explicitly tagged search provider with DuckDuckGo',
 c + 'suggest.rs:185; ' + c + 'nav.rs:47; ' + c + 'omnibar.rs:101',
 'Actual resolve_match returns the same DuckDuckGo URL for SearchQuery values explicitly tagged Bing, Google or DuckDuckGo, because the match discards provider and calls the separate NavTarget::Search hardcoded resolver. Three complete unchanged modules and all32original module tests run; the provider case demonstrates the discarded input.',
 chrome['qualification'] + ' The local suggestions producer intentionally emits only DuckDuckGo today. This is the public shared-match resolver accepting and silently overriding another provider tag, not an observed live host changing a configured search engine or sending a query.', ['chrome_navigation_probe_receipt.json'],
 'Resolve using the advertised provider or return an explicit unsupported result for values this host cannot handle. Keep the default choice at configuration/construction and share URL generation, rather than encoding a second implicit provider inside resolution.'),
('Q378', 'Configurable command classification is not carried into the suggestion path',
 c + 'nav.rs:89; ' + c + 'suggest.rs:49,58,66',
 'With command sigil colon, classify_with(:home, colon) returns Command, but suggestions(:home) offers a web SearchQuery. Conversely classify_with(>home, colon) returns Search while suggestions(>home) returns no rows. suggestions calls default classify twice and has no custom-sigil input, so the promised configurable navigation decision cannot be shared by this path.',
 chrome['qualification'] + ' This executes classification/suggestion disagreement only. No live host wiring or actual command text leaving through a provider request is claimed.', ['chrome_navigation_probe_receipt.json'],
 'Pass the same configured classification or navigation policy into suggestion generation and activation, retaining a convenience default wrapper if useful. Do not independently guess command mode in each surface; user-configured sigils must not silently become search text.')]
findings = []
for ident, title, source, evidence, scope, diagnostics, repair in rows:
    findings.append({'id': ident, 'title': title, 'source': source, 'status': 'reproduced unchanged portable library code with stated boundary/supplier scopes',
        'evidence': evidence, 'scope': scope, 'diagnostics': diagnostics, 'verification': NAME,
        'repair_triage': {'disposition': 'documented under active read-only full-source audit', 'reason': repair}})
notes['findings'].extend(findings)
authority_rows = [
('Chrome authority bundles are mutable views, not access-control decisions',
 'GraphSearchAuthorityMut/FocusAuthorityMut reborrow portable runtime-owned fields.',
 [c + 'authorities.rs', c + 'frame_model.rs'],
 'Named methods centralize some state changes, but all fields remain public and raw mutation is possible. This is a borrowing/ownership seam, not a permission proof. Focus ownership is copied as a bool and host time is explicit. Do not equate this use of authority with Servitor authorization.'),
('View-model extraction is real but its migration prose is stale',
 'FrameViewModel and portable OverlayStrokePass live together in this crate.',
 [c + 'frame_model.rs', c + 'host_intent.rs', c + 'lib.rs', 'crates/shell/chrome/Cargo.toml'],
 'The header still says the aggregate remains shell-side and HostIntent lives in kernel, contradicted by these definitions. Old graphshell/Servo/iced paths describe historical consumers; package source alone does not prove they remain current or provide live runtime enforcement.'),
('Focus fade documentation disagrees with the tested curve',
 'EaseOut is documented as1-p squared in one spelling, but implementation/test use(1-p)^2.',
 [c + 'frame_model.rs', c + 'frame_model/tests.rs'],
 'More precisely the variant doc says1-(p^2), yielding0.75 at half, while code/test say0.25. Distinct-node/elapsed/zero-duration gates are explicit; no headed animation or fresh focus test ran. This is a documented source contradiction, not a measured accessibility failure.'),
('Host intents are a parallel portable vocabulary with caller-owned application',
 'A small HostIntent enum carries chrome decisions through FrameHostInput.',
 [c + 'host_intent.rs', c + 'frame_model.rs'],
 'The enum deliberately differs from the larger host GraphIntent surface. Serialization tests round-trip current values rather than pin historical bytes. Unknown/disabled logged-no-op behavior is described as runtime policy, not implemented by this data-only crate; no current host write path or unavailable-action success was executed.'),
('Navigation history and URL heuristics are distinct from document acquisition authority',
 'History owns a linear URL back/forward path; URL parsing/canonicalization happens later except relative-link joining.',
 [c + 'nav.rs'],
 'Reload preserves forward entries; new branches truncate them. Host heuristics are intentionally incomplete (including contains:// acceptance and limited authority-less schemes), and the nonempty-label prose exceeds the simple endpoint-dot check. Command.resolve returns raw expression as a supposedly harmless fallback; callers must branch before navigation. No backend or command execution ran.'),
('Suggestion defaults become a second policy when resolving shared tagged values',
 'The suggestion producer and NavTarget resolver both hardcode DuckDuckGo; classification also independently uses the default sigil.',
 [c + 'nav.rs', c + 'suggest.rs', c + 'omnibar.rs'],
 'Q377/Q378 execute the shared-input drift. Local default-only production is explicitly recognized; no live configured-host violation is inferred. Ranking shares the OmnibarMatch shape, not an authoritative provider/configuration decision.'),
('Suggestion ranking is limited to the current linear history',
 'URL frequency and newest index rank exact/prefix/substring host hits above path matches.',
 [c + 'suggest.rs', c + 'nav.rs'],
 'This is not a durable all-visits frecency store. Direct-navigation and history dedupe exact strings without canonical URL equivalence, and the six-row cap includes a direct row. Parsed host extraction here uses url::Url, unlike the document-host hand parser audited earlier. Unsupported graph variants produce empty labels/None by the helper explicit scope.'),
('Mailbox generation reuse is real, with separate status projection',
 'ProviderSuggestionMailbox uses kernel AsyncRequestState plus a private request generation.',
 [c + 'omnibar.rs'],
 'Stale resolve rejection is supplied by the shared state machine. clear_pending clears query/result/deadline but retains status; generation wrapping and cross-mailbox identity remain host-adapter concerns. Fresh module tests cover ordinary sequencing/debounce, not actual asynchronous providers or lifetime overflow.'),
('Toolbar drafts reuse one type while settings mirror app projections',
 'ToolbarDraft is an alias of ToolbarEditable; host settings mirror their read-side values.',
 [c + 'toolbar.rs', c + 'frame_model/settings.rs'],
 'The alias is actual shared data ownership. Public POD copies and settings defaults do not apply or validate persisted user settings. Complete initial load status means idle chrome in this model, not evidence a document successfully loaded.'),
('Comms shares a model but protocol is insufficient backend identity',
 'One Comms owner merges all adapters and routes per-conversation reads/sends/delivery by protocol.',
 [m + 'comms.rs', m + 'model.rs', m + 'adapter.rs'],
 'Q376 executes unreachable and aliased same-protocol backends. Identities are one per adapter but ConversationId/AdapterFailure carry no account/adapter discriminator. Sequential inbox awaits expose immediate errors but have no per-backend timeout; the two failure domains are separate.'),
('Delivery facts deliberately avoid promoting carriage into recipient success',
 'DeliveryStatus names queued/carrier/propagation/direct/ACK facts and defaults to Unknown.',
 [m + 'model.rs', m + 'adapter.rs', m + 'pane.rs'],
 'Default delivery_status verifies message existence and returns Unknown, not a read receipt. Free-form pane send_status can say anything supplied by a host, so a Delivered doc example is not execution of such a claim. Current fake sends are explicit doubles with deterministic IDs and no authoring/store mutation.'),
('Murm adapter reuses owner reads but generic messages omit history diagnostics',
 'CabalSink abstracts real local/synced owners; exact conversation address includes protocol,cabal and channel.',
 [m + 'murm_adapter.rs', m + 'adapter.rs'],
 'Source validates channels, rejects empty/subject-bearing text, preserves causal order and labels observed authors rather than membership. Public history retains missing-history diagnostics; generic messages returns only text rows, so its caller must fetch diagnostics separately. Delete/moderation is explicitly not folded. One cabal read failure fails its whole adapter list. Real-owner channel tests are source-read only in this pass.'),
('Misfin adapter remains a synchronous receive projection with incomplete read-state meaning',
 'One mailbox groups received records by resolved sender address or fingerprint; send returns Unsupported.',
 [m + 'misfin_adapter.rs', m + 'model.rs'],
 'Gemmail display blurb is claimed text separate from the server-resolved key. unread counts every listed record despite model wording0when backend has no read tracking; trace MailboxStore.list/read state before a broader unread bug claim. messages checks correspondent key without protocol unlike Murm exact lookup. Store list is synchronous inside async trait methods. No optional Misfin feature/store test or outbound delivery ran.'),
('Pane request and draft guards are current local ownership mechanisms',
 'ThreadRequest uses Arc identity for the latest request; acknowledge_sent compares the exact submitted draft.',
 [m + 'pane.rs'],
 'Original core tests cover selection/reload/new-pane stale results, newer edits and parked drafts. Deriving Clone copies a pending request Arc into a cloned pane, so the another-pane guarantee needs clone/identity semantics qualification. clear_identity clears thread/drafts but leaves inbox/failures, while set_identity does not reset compose state. These are source followups, not executed live account leaks.'),
('Source completeness and executable feature scope remain separate',
 'All22chrome/comms source files are read; tests cover portable comms core and three chrome modules.',
 ['crates/shell/chrome/Cargo.toml', 'crates/shell/comms/Cargo.toml'],
 '20original core tests plus32original navigation module tests pass;4audit cases run across two harnesses. First comms unit link omitted a Windows import-lib path; owner/consumer and2routing cases already passed, then only unit compilation/tests were retried with that path. Full Chrome crate/focus suite, optional adapter tests, browser/network/radio and fresh Cargo graph remain unexecuted.')]
entries = [{'id': f'A{1088 + i}', 'domain': d, 'claim': claim, 'implementations': paths, 'assessment': assessment,
            'state': 'chrome and comms source complete; portable execution scopes explicitly limited', 'verification': NAME}
           for i, (d, claim, paths, assessment) in enumerate(authority_rows)]
authority['entries'].extend(entries)
completed = []
for area, count, chars in areas:
    paths = [p for p in fresh if p.startswith(area)]
    assert len(paths) == count and sum(inv['files'][p]['chars'] for p in paths) == chars
    item = {'prefix': area, 'files': count, 'chars': chars, 'basis_commit': head, 'verification': NAME,
            'qualification': chrome['qualification'] if '/chrome/' in area else route['qualification'] + ' All20original portable core tests pass in the separate unit retry.'}
    completed.append(item); notes['completed_source_areas'].append(item)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Adapter/account route identity', 'detail': 'Q376 requires either checked one-per-protocol registration or explicit backend identity. Preserve the address through failures, pane selection and drafts; do not use send fallback as discovery.'},
    {'verification': NAME, 'area': 'One configured navigation policy', 'detail': 'Q377/Q378 are scoped public helper disagreements; current in-repo search found no live external caller for these entry points. Connect a real host before claiming user query disclosure or ignored deployed preference.'},
    {'verification': NAME, 'area': 'Comms projection and pane lifecycle candidates', 'detail': 'Trace Misfin list/read state, lost generic Murm diagnostics, cloned ThreadRequest identity and retained inbox on identity clear. No live wrong-account delivery, full optional adapter acceptance or account-switch leak was executed.'}])
next_read = {'path': 'crates/stickleback/Cargo.toml', 'start': 0, 'reason': 'Continue full unread Stickleback storage/authority/inventory source, then remaining repository areas.'}
notes['current_area'] = 'Chrome and comms source complete;52scoped original tests and4audit cases recorded'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'comms_routing_probe_receipt.json', 'comms_core_unit_retry_receipt.json', 'chrome_navigation_probe_receipt.json'],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'No owned process live. Full Chrome/focus, optional Misfin/Murm adapters, actual host/account/provider/network/radio and fresh Cargo graph unexecuted. Earlier Q352 and metadata timeout remain qualified.',
    'diagnostic_outcomes': 'Portable Comms owner and2public in-memory routing cases pass. First unit link lacks Windows import-lib path; separate retry passes20original core tests. Three unchanged Chrome modules pass32original+2audit tests. All generated owner/unit/consumer binaries,rlib,PDB/link outputs removed.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True); assert diff.returncode == 0
with (HERE / 'shell_chrome_comms_diff_check.txt').open('x', encoding='utf-8') as f: f.write(diff.stdout + diff.stderr)
addition = f'''

### Chrome and comms source complete: configured navigation and adapter identity, 2026-10-17

Read22new complete files /192,269characters at unchanged HEAD {head}: chrome12 /103,871 and comms10 /88,398. Coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, without stale/pending credit. Full source audit remains active and incomplete.

Q376 reproduces the mismatch between a merged multi-adapter inbox and first-by-protocol dispatch. Two same-protocol adapters both list conversations, but the second unique thread returnsNotFound for read/send; reversing registration changes reachability. A shared protocol/key produces two titled rows but selecting the second reads/fake-sends through the first owner. This runs unchanged portable Comms and its supplied InMemoryAdapter; fake sends return deterministic IDs and do not author/deliver mail. No real account or live message was exercised. Backend/account identity must survive routing or duplicate protocol registration must be visibly refused; trying successive sends is not safe owner discovery.

Q377 reproduces resolve_match discarding Bing/Google provider tags and returning DuckDuckGo through the separate hardcoded NavTarget resolver. Q378 reproduces custom colon-command classification disagreeing with suggestions using default greater-than: colon-home becomes a search row, while greater-than-home loses its search rows. The suggestion producer deliberately supports default/DuckDuckGo today; these are public shared-input/configuration disagreements, not observed live user queries or deployed preference violations. Three complete unchanged modules nav/omnibar/suggest pass32original tests plus2audit cases over exact retained kernel/url suppliers; full Chrome/focus tests did not run.

Portable Comms core passes20original unit tests, with optional Misfin/Murm adapters disabled. The first direct unit link omitted a retained Windows import-library search path; that attempt is preserved. Its portable owner/consumer and2routing cases already passed. Only original unit compilation/tests were retried with the missing native path. The total fresh execution is52scoped original tests and4audit cases, not a full package/all-feature Cargo receipt.

A1088-A1102 distinguish mutable chrome state bundles from permission decisions; extracted FrameViewModel/HostIntent from stale ownership prose; documented1-p^2 focus curve from tested(1-p)^2; portable intent vocabulary from actual host handlers; URL heuristics/history from document acquisition; duplicate provider/default-sigil decisions; linear history ranking from durable frecency; shared request-state generation from status projection; alias-shared toolbar drafts from mirrored settings; protocol from adapter/account identity; explicit delivery facts from recipient/read success; real Murm owner reads from generic text-only diagnostic loss; receive-only Misfin projection from unread/read-state uncertainty; local thread/draft guards from pane-clone/identity-switch obligations; and source completeness from executable feature scope. Misfin unread/protocol checking, clone-shared request identity, retained inbox after identity clear and blocking store calls remain source followups with their exact limits.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [Comms routing and preserved first unit link](../../../support/receipts/authority-success-audit/comms_routing_probe_receipt.json), [core unit retry](../../../support/receipts/authority-success-audit/comms_core_unit_retry_receipt.json), and [Chrome module tests](../../../support/receipts/authority-success-audit/chrome_navigation_probe_receipt.json). No production edits. Prior evidence, protected source/runtime/auxiliary hashes, shared WIP and Git index remain intact; every prior report byte remains an exact prefix. Diagnostic libraries/executables/PDB/link outputs removed; no owned process live. Diff check passes. No isolated target, Cargo home or worktree retained; evidence remains below100MB. Next: unread Stickleback source.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
for n, value in [('review_notes.json', notes), ('authority_map.json', authority)]: (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
evidence = ['record_shell_review.py', 'shell_chrome_comms_diff_check.txt', 'comms_routing_probe.rs', 'run_comms_routing_probe.py',
    'comms_routing_probe_receipt.json', 'comms_routing_probe_owner_compile.log', 'comms_routing_probe_unit_compile.log',
    'comms_routing_probe_consumer_compile.log', 'comms_routing_probe_output.log', 'run_comms_core_unit_retry.py',
    'comms_core_unit_retry_receipt.json', 'comms_core_unit_retry_compile.log', 'comms_core_unit_retry_output.log',
    'chrome_navigation_probe.rs', 'run_chrome_navigation_probe.py', 'chrome_navigation_probe_receipt.json',
    'chrome_navigation_probe_compile.log', 'chrome_navigation_probe_output.log']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'kind': 'chrome and comms source complete;52scoped original tests and4audit cases; repository incomplete',
    'prior_receipt': {'path': 'servitor_review_receipt.json', 'sha256': sha(HERE / 'servitor_review_receipt.json')},
    'coverage': stats, 'completed_source_areas': completed, 'whole_current_files': fresh, 'source_sha256': {p: sha(ROOT / p) for p in fresh},
    'preserved_production_and_test_hashes': guards, 'auxiliary_read_sha256': aux, 'runtime_input_sha256': runtime, 'shared_working_file_sha256': shared,
    'new_findings': [f['id'] for f in findings], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False,
    'diagnostics': {'comms_routing_probe_receipt.json': {'owner_compile_exit': 0, 'unit_compile_exit': 1, 'consumer_compile_exit': 0, 'run_exit': 0, 'original_core_tests': 0, 'audit_cases': 2},
                    'comms_core_unit_retry_receipt.json': {'build_exit': 0, 'run_exit': 0, 'original_core_tests': 20, 'audit_cases_this_attempt': 0},
                    'chrome_navigation_probe_receipt.json': {'build_exit': 0, 'run_exit': 0, 'original_tests': 32, 'audit_cases': 2}},
    'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'retained_isolation': [], 'limits': route['qualification'] + ' ' + chrome['qualification']}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'],
    'original_tests': 52, 'audit_cases': 4, 'protected_sources': len({**guards, **receipt['source_sha256']}), 'retained_bytes': budget}, indent=2))
