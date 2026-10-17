"""Complete current Gemot source coverage, public-owner diagnostics and limits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'gemot_complete_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a): return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()

assert not (HERE/NAME).exists()
old = load('gemot_constitution_review_receipt.json')
head = git('rev-parse','HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'],**old['source_sha256']}
aux,runtime = old['auxiliary_read_sha256'],old['runtime_input_sha256']
assert len(guards) == 2000
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT/old['document']['path']; prefix = doc.read_bytes()
assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index = Path(git('rev-parse','--git-path','index'))
index = index if index.is_absolute() else ROOT/index
index_hash = sha(index); assert index_hash==old['git_index_sha256']
inv,cov = load('source_inventory.json'),load('read_coverage.json')
assert inv['commit']==head and load('pending_read.json')==[]
stats = json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2483 and stats['read_chars']==29783057
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
area = 'crates/moot/gemot/'
files = [p for p in inv['files'] if p.startswith(area)]
fresh = old['partial_source_areas'][0]['remaining']
assert len(files)==51 and len(fresh)==14
assert sum(inv['files'][p]['chars'] for p in files)==777720
assert sum(inv['files'][p]['chars'] for p in fresh)-25978==310157
assert all(cov[p]['through']==inv['files'][p]['chars'] and cov[p]['sha256']==sha(ROOT/p)==inv['files'][p]['sha256'] for p in files)
probe = load('gemot_owner_probe_receipt.json')
assert probe['owner_build_exit']==probe['consumer_build_exit']==probe['run_exit']==0
assert probe['checks']==4 and probe['original_tests']==0 and probe['generated_output_removed']
assert probe['runner_sha256']==sha(HERE/'run_gemot_owner_probe.py')
assert probe['probe_sha256']==sha(HERE/'gemot_owner_probe.rs')
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
assert all(not Path(v['path']).exists() for v in probe['removed_outputs'])
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-gemot-owner.*'))
assert not Path('C:/t/cargo-targets/mere/libauthority_success_gemot_owner.rlib').exists()

notes,authority = load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q355' and authority['entries'][-1]['id']=='A1023'
common = {'verification':NAME,'diagnostics':['gemot_owner_probe_receipt.json']}
new_findings = [
    {**common,'id':'Q356','title':'Tulpa effective adoption cites one signed proposal but returns another proposal\'s artifact',
     'source':area+'src/moot/tulpa.rs:191,204,251,263',
     'status':'reproduced through public TulpaStore using unchanged freshly compiled complete current Gemot owner',
     'evidence':'Two accepted adoption proposals name the same Tulpa/version with different artifact references. The version map keeps the artifact from the lowest proposal id, while the effective candidate is selected by authored time and operation hash. The result names the later proposal and its signed operation, whose Adopt action carries the new artifact, but returns the earlier artifact. Both recognition decisions are accepted; all four signed proposal/endorsement facts remain. The distinct-version control returns the artifact its effective proposal actually names.',
     'scope':probe['qualification'],
     'repair_triage':{'disposition':'documented under active read-only audit',
         'reason':'Bind effective artifact provenance to the canonical adoption that establishes the version, or explicitly reject/mark conflicting version-to-artifact bindings. Keep rollback citations distinct from original adoption provenance. Do not silently pair the latest signed citation with another proposal\'s bytes.'}},
    {**common,'id':'Q357','title':'Self-declared Tulpa electorate reaches adopted aggregate state without community membership evidence',
     'source':area+'src/moot/service.rs:1102,1596; '+area+'src/moot/tulpa.rs:138,149,171,419',
     'status':'reproduced public aggregate source path; downstream product consequences require consumer trace',
     'evidence':'A Moot is founded under one identity, with actual membership empty. A different signer supplies a proposal whose RecognitionContext lists only itself and then signs its own endorsement. Both aggregate commands return Ok; the snapshot still reports zero actual members and the original constitutional founder, but Tulpa.decisions says accepted and Tulpa.adopted contains the outsider-selected artifact. The projection validates the proposal\'s supplied recognition policy and evaluates its supplied electorate; neither aggregate recording nor fold binds that electorate/revision to retained constitutional or membership authority. Freezing those asserted bytes is not proof that the community selected them.',
     'scope':probe['qualification']+' The executed fixture has founded constitutional law and zero membership records; it is not a physical peer attack, a one-member fixture, or a demonstrated downstream executable activation.',
     'repair_triage':{'disposition':'document high-effect recognition authority provenance issue; do not choose governance policy during audit',
         'reason':'Preserve raw signed proposals and endorsements, but distinguish proposer-asserted recognition from community-authorized adoption. Define and verify the governing electorate/policy revision at the projection seam before exposing adopted as effective community state. Product consumers and historical/frozen-frontier semantics need review before selecting a repair.'}},
    {'id':'Q358','title':'Configured Moot availability and erasure bounds do not participate in checkpoint or prune decisions',
     'source':area+'src/moot/service.rs:145,1193; '+area+'src/moot/records/retention.rs:69,217,274; '+area+'src/moot/records/store.rs:125',
     'status':'source-confirmed across complete current Gemot source; bound-matrix runtime probe not executed',
     'evidence':'MootRetentionSettings.resolve copies availability and erasure into MootRetentionPolicy. The complete Gemot source contains no subsequent read of those policy fields or their promised_floor/history_ceiling values in production decisions. Checkpoint validation checks version, Moot, single signer, revision, commitments and frontier; pruning checks the prune flag and checkpoint reference before asking the shared processor to retire the author prefix. The ForAgeMs/Forever/UntilCheckpoint settings therefore carry configuration data without implementing a time/bound decision here, despite the retention owner\'s statement that policy decides what the checkpoint may retire.',
     'scope':'Complete current Gemot source review and exact targeted reference check. The whole library compiled for other cases, but no varied-bound/clock/physical deletion diagnostic ran. This finding does not choose semantics for any KeepBound value or prove a host failed to impose an external rule.',
     'verification':NAME,
     'repair_triage':{'disposition':'document inert high-effect configuration and preserve user-owned retention policy ruling',
         'reason':'Define the intended availability-versus-erasure decision and its owner, then either enforce it at the checkpoint/prune boundary or expose these values as unenforced host inputs. Do not invent a destruction/GC policy or claim physical retention guarantees from the currently carried fields.'}},
]
notes['findings'].extend(new_findings)
rows = [
    ('The complete Gemot source has one aggregate API over distinct lane owners',
     'All51inventoried files are now read, including seven-lane joining and the full aggregate service.',
     [area+'src/moot/service.rs',area+'src/moot/lanes.rs',area+'src/moot/mod.rs'],
     'MootLanes actually centralizes lane setup, identifiers, typed publishing and per-lane status. Local store authoring does not publish by itself. Older module prose still says Gemot owns no network session and capability gating is a later milestone, despite current joined-lane/capability implementations; source-assembly and transitive-dependency claims need precise scope.'),
    ('Signed facts and authorized fauna/collection views are explicitly separate',
     'Raw share/record commands retain facts; authorized fauna and collections intersect current grants/delegations with group/admission inputs.',
     [area+'src/moot/service.rs',area+'src/moot/records/roster.rs',area+'src/moot/records/collection.rs'],
     'Bypassing an authorized convenience method to retain an untrusted fact is not itself a projection bypass. The lower roster authorized_fauna checks its supplied AuthorityProvider; the aggregate adds membership/admission. Callers must choose the intended surface. This is a source-confirmed division, not one identical meaning for every method named authorize.'),
    ('Collection causal selection and contribution identity have dedicated implementations',
     'Collections cite original Shared operation identities, preserve selected versus effective sets, and merge causal edits with hash ties.',
     [area+'src/moot/records/collection.rs',area+'src/moot/service.rs'],
     'A collection fork binds canonical membership and parent commitment, while current authority controls effective display. Invalid fork commitments remain pending but a declaration can still produce an empty-root view; trace intended pending/effective semantics before treating that as a separate defect. The recipe is distinct from Commons graph causal folding; shared identity alone does not make one algorithm the authority for both.'),
    ('Tulpa artifact resolution and effective-candidate resolution disagree',
     'The version map picks an artifact by proposal-id traversal while candidate selection uses authored time and operation hash.',
     [area+'src/moot/tulpa.rs'],
     'Q356 proves a signed provenance mismatch through the current public owner. The returned operation/proposal identify new bytes, while the returned artifact names old bytes under the same version. Distinct-version control passes.'),
    ('Frozen recognition bytes are not verified community authority',
     'Tulpa recognizes the electorate and policy carried in the proposal without grounding it in retained Moot membership/governance.',
     [area+'src/moot/tulpa.rs',area+'src/moot/service.rs',area+'src/moot/constitution/event.rs'],
     'Q357 reproduces self-elected adoption in a founded aggregate with zero members. The optional constitutional Standing filter helper is not called by this projection. This is an authority provenance gap at the exposed adopted snapshot, not proof of downstream code activation or a ruling on which historical electorate policy is correct.'),
    ('Typed capability encoding actually reuses the Servitor canonical owner',
     'TypedMootAuthorization parses Cap and canonicalizes through cap_path before certificate matching.',
     [area+'src/moot/typed_authorization.rs','crates/servitor/src/delegation.rs'],
     'The shared encoder escapes slash/percent inside logical segments and maps facet dot segments onto slash hierarchy. The suspected facet mismatch is not present. TypedMootAuthorization intentionally requires delegated coverage, while MootAuthority also answers direct roots; Delegate mode is explicitly refused. Shared vocabulary does not imply identical root or mode semantics.'),
    ('Quorum command success means a stored vote before it means accepted replacement law',
     'Moot.amend returns Ok after retaining a supporter even when the accepted revision/rules remain unchanged.',
     [area+'src/moot/constitution/governance.rs',area+'src/moot/service.rs'],
     'A fresh public aggregate case confirms the prior source observation with threshold2 and one eligible signature. The returned snapshot is honest, but replace/amend/atomic-switch wording needs retained-vote versus accepted-change status. This result is recorded as an outcome limitation rather than inventing a policy that should reject valid pending signatures.'),
    ('Moot retention carries bounds without interpreting them',
     'AvailabilityPolicy and ErasurePolicy survive configuration resolution but have no subsequent production reads in Gemot.',
     [area+'src/moot/records/retention.rs',area+'src/moot/records/store.rs',area+'src/moot/service.rs'],
     'Q358 is source-confirmed inert configuration. Revision, single-signer authority, commitment and monotone-frontier checks are implemented; age/forever/checkpoint retention-bound meaning is not. No new GC/destruction policy is chosen, and external host enforcement was not traced in this pass.'),
    ('Aggregate evidence parsing genuinely shares one structural owner',
     'evidence_section centralizes uniqueness, subject, version and exact re-encoding checks for three critical evidence sections.',
     [area+'src/moot/service.rs'],
     'Constitution evidence is required; delegation/domain sections default to empty for compatibility. Import then mutates distinct stores in dependency order before object import, so a later error can follow earlier durable changes. This is not a single atomic aggregate transaction; the older proof explicitly admits that limit.'),
    ('Historical object import intentionally differs from live intake',
     'MootPolicy permits historical checkpoint authority/prunes for aggregate records and refuses them for live intake.',
     [area+'src/moot/records/store.rs',area+'src/moot/records/retention.rs'],
     'A dedicated source test explicitly pins this divergence. Do not label the difference a duplicate authority bug merely because one operation gets two answers. Retained checkpoint validation preserves past authority; full constitution-history provenance and multi-store admission remain separate obligations. The original test was read, not run.'),
    ('Snapshot and checkpoint construction read several independently changing views',
     'Moot.snapshot reads governance, membership, roster, checkpoints, delegation and social projections sequentially; build_checkpoint separately reads operations and roster.',
     [area+'src/moot/service.rs',area+'src/moot/records/store.rs'],
     'One returned struct is not by itself one transactional authority cut. The snapshot/frontier consistency question needs host serialization or an explicit shared snapshot generation; no concurrency/failure diagnostic ran. The caller serialization contract in hosting cannot automatically establish it for every public method.'),
    ('Broader proof names do not extend the four fresh diagnostic cases',
     'Current source includes real-network and durable tests, while this pass freshly executes only CPU/MemoryBackend public cases.',
     [area+'src/moot/service.rs',area+'src/moot/records/sync.rs',area+'src/moot/lane_coexistence.rs'],
     'The protected-drop unit fixture uses an XOR TestProtector, proving adapter wiring rather than cryptographic confidentiality. Live lane tests distinguish late catch-up from post-join publication and report separate lane counters. None of those original suites, disk or network cases were rerun here; full-owner compilation over retained suppliers is not a fresh Cargo graph.'),
]
entries = [{'id':f'A{1024+i}','domain':d,'claim':c,'implementations':p,'assessment':a,
            'state':'Gemot source complete; current whole-owner compilation and four public CPU cases; other tests source-only','verification':NAME}
           for i,(d,c,p,a) in enumerate(rows)]
authority['entries'].extend(entries)
completed = {'prefix':area,'files':51,'chars':777720,'basis_commit':head,'verification':NAME,
             'qualification':'Every inventoried Gemot file fully read at matching current hashes. Whole unchanged production library freshly compiled over15exact retained suppliers and four public diagnostic cases passed; original tests and current Cargo dependency graph not run.'}
notes['completed_source_areas'].append(completed)
for prior in notes.get('partial_source_areas',[]):
    if prior.get('prefix')==area: prior['completed_by']=NAME; prior['remaining']=[]
notes['review_followups'].extend([
    {'verification':NAME,'area':'Tulpa recognition and artifact provenance','detail':'Q356 requires internally consistent citation/artifact resolution. Q357 requires a governed recognition source before treating proposer-supplied electorate bytes as community adoption; preserve raw facts and user-owned governance semantics.'},
    {'verification':NAME,'area':'Configured retention bounds','detail':'Q358: current fields are copied but not read by production decisions. Establish the intended owner/semantics before any GC or destruction change; no varied-bound runtime or external host acceptance is claimed.'},
    {'verification':NAME,'area':'Moot command and snapshot authority cuts','detail':'Full service review confirms sequential multi-store reads and import writes. Trace actual host serialization, checkpoint snapshot/frontier consistency and post-persistence snapshot errors before promoting broader atomicity claims.'},
])
next_read = {'path':'crates/moot/mien/Cargo.toml','start':0,'reason':'Continue full unread Mien/Standing source and its actual scoring/admission owners, then remaining Moot packages.'}
notes['current_area']='Gemot complete; Tulpa provenance and self-declared recognition reproduced; inert retention bounds source-confirmed'
notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':[NAME,'gemot_owner_probe_receipt.json'],'audit_authored_production_edits_this_turn':False,'next_read':next_read,
    'pending_verification':'Q352 full-owner key replacement remains unexecuted; Q358 bound matrix and downstream Q357 consumers not executed; no process live',
    'diagnostic_outcomes':'Whole current Gemot owner and public consumer compiled;4CPUcases passed: distinct-version control, same-version citation/artifact mismatch, self-elected zero-member aggregate adoption, pending-quorum Ok with unchanged law. Generated owner rlib/executable/PDB removed.'}
diff = subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True); assert diff.returncode==0
with (HERE/'gemot_complete_diff_check.txt').open('x',encoding='utf-8') as out:out.write(diff.stdout+diff.stderr)
addition = f'''

### Gemot source complete: Tulpa provenance and recognition authority, 2026-10-17

Finished14files, including the previously partial service, for310,157additional characters at unchanged HEAD {head}. All51Gemot inventory files /777,720characters are fully read at matching hashes. Repository coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale/pending credit. The full source audit remains active and incomplete.

Q356 reproduces an internally inconsistent Tulpa adoption. Two accepted signed proposals bind the same Tulpa/version to different artifacts. The selected effective proposal/operation names the later artifact, but the returned adopted artifact is taken from the earlier lowest-proposal-id binding. The diagnostic retrieves and decodes the cited signed proposal to compare its actual artifact with the returned value. Both recognition decisions are accepted and all four facts remain. A distinct-version control returns a consistent citation/artifact pair.

Q357 reproduces the recognition provenance gap through the aggregate. A Moot is constitutionally founded under one identity but has zero membership records. Another signer supplies an electorate containing only itself, proposes an artifact and self-endorses. Both commands succeed and the aggregate snapshot reports adopted while still showing zero actual members and the original founder. The projection freezes and evaluates asserted recognition bytes without grounding them in retained community authority. This is not a demonstrated downstream executable activation or physical peer attack, and the fixture is explicitly zero-member. Selecting the correct governed/historical electorate is a larger design issue; raw facts should remain distinct from effective adoption.

Q358 is source-confirmed: configured availability/erasure bounds are copied into MootRetentionPolicy but never read by Gemot's production checkpoint/prune decisions. Implemented checks cover revision, single-signer authority, commitments and monotone frontier, while ForAgeMs/Forever/UntilCheckpoint do not select behavior here. No varied-bound/clock/physical-deletion case was run; this does not choose any GC/destruction semantics or prove external hosts lack their own enforcement.

The unchanged complete current Gemot production library and a public consumer were freshly compiled over15exact retained suppliers from one named Cargo fingerprint family. Four sequential CPU/MemoryBackend cases pass: distinct-version control, Q356, Q357 and pending quorum. The latter confirms aggregate amend returns Ok with unchanged accepted rules/revision when only one of two signatures is retained; its snapshot is honest, but command/prose status must distinguish a stored vote from accepted replacement law. This is not a fresh Cargo dependency graph or execution of original package, disk, live LogSync, headed, protected transport or tensor tests. Generated owner rlib/executable/PDB/link output is removed.

A1024-A1035 record actual lane-assembly ownership, raw versus authorized projections, causal collection citation, separate Tulpa resolution paths, asserted versus governed electorate, correct shared facet canonicalization, quorum result scope, inert retention configuration, shared evidence parser versus non-atomic import, intentionally different historical/live prune intake, sequential snapshot cuts and proof scope. The earlier facet suspicion is resolved: Servitor's canonical encoder turns logical facet segments into the expected slash hierarchy with segment escaping. The object wire uses shared stable_writer_subject; membership still has its local binding recipe. Protected-drop TestProtector is an XOR fixture, not confidentiality evidence. Source docs still contain stale M1/ownership milestone wording despite current capability and lane implementations.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [whole-owner diagnostic](../../../support/receipts/authority-success-audit/gemot_owner_probe_receipt.json). No production edits or Git index changes. Prior evidence, protected source/runtime/auxiliary hashes and shared WIP remain intact; all earlier document bytes are an exact prefix. Diff check passes. No live process, retained isolated target, Cargo home or worktree; evidence remains under100MB. Next: unread Mien/Standing owner,11files /125,757characters.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
evidence=['record_gemot_complete_review.py','gemot_complete_diff_check.txt','gemot_owner_probe.rs','run_gemot_owner_probe.py',
          'gemot_owner_probe_owner_compile.log','gemot_owner_probe_consumer_compile.log','gemot_owner_probe_output.log','gemot_owner_probe_receipt.json']
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,
    'kind':'Gemot source complete; current owner compiled; Tulpa provenance/recognition diagnostic; repository incomplete',
    'prior_receipt':{'path':'gemot_constitution_review_receipt.json','sha256':sha(HERE/'gemot_constitution_review_receipt.json')},
    'coverage':stats,'completed_source_areas':[completed],'whole_current_files':fresh,'source_sha256':{p:sha(ROOT/p) for p in fresh},
    'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'preserved_production_and_test_hashes':guards,
    'shared_working_file_sha256':shared,'new_findings':['Q356','Q357','Q358'],'authority_entries':[e['id'] for e in entries],
    'audit_authored_production_edits_this_pass':False,
    'diagnostics':{'gemot_owner_probe_receipt.json':{'owner_build_exit':0,'consumer_build_exit':0,'run_exit':0,'checks':4,'original_tests':0,'audit_cases':4,'generated_output_removed':True}},
    'pending_verification':notes['checkpoint']['pending_verification'],'next_read':next_read,
    'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':index_hash,'retained_isolation':[],
    'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
    'limits':probe['qualification']+' Q358 is source-confirmed only; full repository remains incomplete.'}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/p)==h for p,h in {**guards,**receipt['source_sha256'],**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],
                  'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
