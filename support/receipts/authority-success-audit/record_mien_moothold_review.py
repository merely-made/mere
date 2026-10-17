"""Record complete Mien/Moothold reads and exactly scoped diagnostics."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='mien_moothold_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()

assert not (HERE/NAME).exists()
old=load('gemot_complete_review_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
aux,runtime=old['auxiliary_read_sha256'],old['runtime_input_sha256'];assert len(guards)==2013
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
shared=old['shared_working_file_sha256'];doc=ROOT/old['document']['path'];prefix=doc.read_bytes()
assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index
index_hash=sha(index);assert index_hash==old['git_index_sha256']
inv,cov=load('source_inventory.json'),load('read_coverage.json')
assert inv['commit']==head and load('pending_read.json')==[]
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2501 and stats['read_chars']==29941592
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
areas=[('crates/moot/mien/',11,125757),('crates/moot/moothold/',7,32778)]
fresh=[p for p in inv['files'] if any(p.startswith(a) for a,_,_ in areas)]
assert len(fresh)==18 and sum(inv['files'][p]['chars'] for p in fresh)==158535
assert all(cov[p]['through']==inv['files'][p]['chars'] and cov[p]['sha256']==sha(ROOT/p)==inv['files'][p]['sha256'] for p in fresh)
mien=load('mien_owner_probe_receipt.json');recip=load('moothold_reciprocity_probe_receipt.json')
assert mien['owner_build_exit']==mien['consumer_build_exit']==mien['run_exit']==0 and mien['checks']==6
assert recip['build_exit']==recip['run_exit']==0 and recip['checks']==8 and recip['original_tests']==5
for name,p,runner,probe in [('mien',mien,'run_mien_owner_probe.py','mien_owner_probe.rs'),('reciprocity',recip,'run_moothold_reciprocity_probe.py','moothold_reciprocity_probe.rs')]:
    assert p['generated_output_removed'] and p['runner_sha256']==sha(HERE/runner) and p['probe_sha256']==sha(HERE/probe)
    assert all(sha(ROOT/n)==h for n,h in p['source_sha256'].items())
    assert all(not Path(v['path']).exists() for v in p['removed_outputs'])
assert all(sha(Path(v['path']))==v['sha256'] for v in mien['libraries'].values())
assert sha(Path(recip['supplier']['path']))==recip['supplier']['sha256']
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q358' and authority['entries'][-1]['id']=='A1035'
base='crates/moot/mien/src/'
descriptions=[
('Q359','Standing judgments affect scores without the governance authorization the event contract assumes',
 base+'event.rs:14; '+base+'store.rs:74,271; '+base+'ledger.rs:219',
 'An unrelated identity signs an attested Censure targeting another root; ordinary admission accepts it and fold_moot lowers the target by30. The same signer signs a Pardon and restores30. Signer/root binding and signatures are genuine, but the public store/fold path has no governance-role provider. Store prose explicitly defers moderation to a later Moot policy, while event/ledger prose assumes an ingestion-time governance check. Current Gemot live Standing intake delegates directly to this store and does not provide that missing check.',
 'The executed owner is Mien with no external role provider or actual hosted community. Raw retention can be appropriate, but the unqualified fold applies the judgment as effective score; no deployed product gate bypass is claimed.',
 'Keep raw signed facts distinct from effective governance-authorized judgments. Define and verify the relevant current/historical Moot authority before score projection; do not repair out-of-order authority by silently discarding legitimate signed evidence.'),
('Q360','Unsigned Standing judgment weights reverse pardon and censure effects above the signed range',
 base+'ledger.rs:224,231,235; '+base+'event.rs:120',
 'Actual signed/attested Pardon and Censure events with weightu64::MAX pass admission. On an untouched target, Pardon yields-1 and Censure yields+1 because weight is cast to i64. Ordinary weight30 controls produce+30 and-30. The max-value cases do not rely on a debug overflow panic; the cast already reverses the intended direction.',
 'Four input combinations within one fresh public-store case, CPU/MemoryBackend. General accumulation/multiplication overflow, optimized-build differences and real moderation roles were not exercised.',
 'Admit a representable nonnegative range or use checked wider arithmetic with an explicit overflow outcome. Apply the same numeric contract to censure loopback and score accumulation; do not infer safe range from the signed transport.'),
('Q361','Asserted-time sorting reorders a valid Standing log and loses its completion',
 base+'store.rs:293,302,308; '+base+'ledger.rs:118,165',
 'An identity authors CommitmentMade at100 followed by a valid next-log-position CommitmentFulfilled at99. Both signed operations are retained and verify. Direct causal-order Ledger folding yields+10, but store fold sorts by asserted time first, ignores completion before creation and leaves the commitment lapsed at score-20. The forward-time completion control yields+10 through both paths.',
 'Same author, actual sequence/backlink continuity, sequential storage and explicit clock1000. This does not require packet reorder or a simulated external clock; host clock correction or asserted-time input could produce the backstep. No live network trial ran.',
 'Preserve causal dependencies/log order in deterministic materialization, or explicitly validate and report any required timestamp monotonicity. A deterministic wall-clock sort is not automatically a causal fold.'),
('Q362','Repeated vouches and self-vouching mint additional Standing without new follow-through',
 base+'ledger.rs:194,201,206',
 'After one signed governance-participation event gives a voucher score1, two fresh signed10000bp vouches to the same newcomer give that newcomer2 while the voucher remains1. The edge is deduplicated only for censure-loopback tracking; the boost is applied every time. A subsequent10000bp vouch by the voucher to itself doubles its own accrued score to2. All operations pass the normal attested author/store path.',
 'Fresh signed operations with separate authored times, not replaying one hash or bypassing store idempotence. This shows accrual behavior in Mien; it does not prove an actual host accepted invented governance participation or that an external policy has no vouch constraints.',
 'Define a bounded vouch lifecycle/identity and its replacement semantics; separate edge retention from boost entitlement, and decide whether self-vouching is valid. This is a reputation-policy repair to document rather than choosing economics during source audit.'),
('Q363','Standing accepts basis-point fractions beyond its documented10000 ceiling',
 base+'event.rs:87; '+base+'store.rs:87; '+base+'ledger.rs:206',
 'A signed Vouch with fraction_bpu16::MAX is admitted and lends6 from a voucher whose score is1. A10000bp control lends1. The event grammar documents0..=10000, but the field type permits65535 and current admission/projection does not reject the excess. Related config/lens u16 scaling inputs also lack a common validated fraction type.',
 'The executed case is specifically the Vouch wire field; depreciation, loopback and concord out-of-range variants were source-reviewed, not separately executed.',
 'Validate the documented basis-point range at the wire/config boundary and share one checked representation where semantics agree. Preserve operator choice within the admitted range rather than silently clamping malformed signed data.'),
('Q364','Shared Standing stores mix Moot scores when a raw author reuses the constant log id',
 base+'store.rs:50,151,271,293',
 'One raw author authors a governance-participation event in MootA and then one in MootB on the same store. The latter uses sequence1 because latest lookup is author/log0 without Moot identity. Both topics resolve that log, fold reads all its entries and discards the decoded _moot value. Both Moot projections therefore score2 despite only one event addressed to each. A separate-store control scores1forA and0forB.',
 'Public author_seed path sharing one real MemoryBackend across two Moot topics. Moot-scoped derived keys and one physical store per Gemot aggregate are separate isolation controls. This is not a demonstrated cross-directory product leak or physical peer transfer.',
 'Scope log identity and latest/continuity decisions by Moot where a shared store is supported, and defensively enforce decoded Moot address during projection. Merely topic-indexing a shared author/log does not isolate its contents; compatibility and carrier semantics need review.'),
]
findings=[]
for ident,title,source,evidence,scope,repair in descriptions:
    findings.append({'id':ident,'title':title,'source':source,'status':'reproduced through public APIs of unchanged freshly compiled complete current Mien',
                     'evidence':evidence,'scope':scope+' '+mien['qualification'],'verification':NAME,
                     'diagnostics':['mien_owner_probe_receipt.json'],'repair_triage':{'disposition':'documented under active read-only audit','reason':repair}})
findings.append({'id':'Q365','title':'Moothold reciprocity casts unsigned debt and tolerance into reversed signed decisions',
    'source':'crates/moot/moothold/src/reciprocity.rs:50,57',
    'status':'reproduced unchanged complete Reciprocity and MootId modules; original scoped tests also pass',
    'evidence':'Recording u64::MAX units of service makes the reported debt-1 and allows another request at zero tolerance. Conversely an empty ledger refuses a request at toleranceu64::MAX because that tolerance becomes-1. A normal50-unit debt with tolerance10 is correctly refused. Five original module tests and three audit cases pass.',
    'scope':recip['qualification'],'verification':NAME,'diagnostics':['moothold_reciprocity_probe_receipt.json'],
    'repair_triage':{'disposition':'documented numeric repair under read-only audit','reason':'Compare unsigned cumulative balances/tolerance with checked wide arithmetic, or explicitly admit a bounded representable domain. Keep debt direction and tolerance monotonic; do not silently use as i64 or leave cumulative addition overflow unspecified.'}})
notes['findings'].extend(findings)
rows=[
('Standing storage really shares the common processor, but that is structural admission',
 'Authoring and accept use one OperationProcessor/MunimentStore owner for signatures, continuity and indexed writes.',
 [base+'store.rs',base+'wire.rs'],
 'This is actual implementation reuse. Its own policy defers membership/governance meaning; shared cryptographic admission does not supply that absent authority. Q359 is the resulting effective-judgment gap, not a failure to verify the signer.'),
('Stable author binding uses the shared helper and a deliberate legacy rule',
 'Attested Standing uses stable_writer_subject and enforces event author; mismatched legacy claims remain retained but are skipped by fold.',
 [base+'wire.rs',base+'store.rs'],
 'Identity-backed authoring refuses another root before mutation. Legacy raw retention differs deliberately from effective accrual. Q364 involves valid matching raw authors across topics, not the already-filtered mismatched-root compatibility case.'),
('Per-Moot score authority is not established by topic lookup alone',
 'The constant author/log0 can be indexed under two Moot topics while fold reads its entire log.',
 [base+'store.rs'],
 'Q364 reproduces mixed scores on a shared backend. Derived per-Moot signer salts and physically separate aggregate stores mitigate different paths; no global cross-directory or live transport leak is inferred.'),
('Deterministic Standing order differs from causal order',
 'Store folding sorts authored time before author and sequence, while Ledger requires causal/DAG order.',
 [base+'store.rs',base+'ledger.rs'],
 'Q361 proves a valid two-operation same-author log can lose completion and accrue a lapse under a backward timestamp. Repeatability over the same corpus does not establish correct causal interpretation.'),
('Standing numeric domains are wider than the score representation and documented fractions',
 'Judgment weights areu64, scoresi64 and fractionsu16 without validated shared bounds.',
 [base+'event.rs',base+'ledger.rs',base+'composite.rs',base+'persona_chain.rs'],
 'Q360 andQ363 execute weight sign reversal and an out-of-range Vouch. Other raw add/multiply paths can exceed signed arithmetic, but those variants were not executed here. Integer arithmetic alone does not establish safe or cross-profile numerical behavior.'),
('Vouch edge identity and boost accrual have different deduplication rules',
 'vouchers_of deduplicates accountability edges, while every Vouch event adds another boost.',
 [base+'ledger.rs'],
 'Q362 executes repeated fresh same-edge operations and self-vouching, distinct from same-hash storage replay. A correction requires explicit entitlement/lifecycle semantics rather than copying the edge dedupe mechanically into every policy.'),
('The policy gate evaluates host-supplied facts rather than collecting them',
 'authorize reads score, vouched/member flags and recent_action_ms; it does not record an accepted action.',
 [base+'gate.rs','crates/moot/gemot/src/moot/group.rs','crates/moot/gemot/src/moot/service.rs'],
 'The pure gate is an intended reference evaluator. Gemot convenience paths pass MootGroup facts containing membership/write coverage with other fields defaulted, so they do not automatically consume Mien scores or maintain rate history. Trace real hosts before treating configured policy as an end-to-end enforced burst limit. Future timestamps count as age0 due to saturating subtraction; no gate-clock case ran.'),
('Reputation composition has explicit local policy and one-hop scope',
 'RepLens reads direct concord ledgers only, with schema-config equality when the viewer ledger exists.',
 [base+'composite.rs'],
 'No transitive traversal is implemented. Missing viewer config deliberately skips schema gating; CautiousImport imports negative standing at full weight even under a zero concord weight. Those are current source semantics to preserve/qualify, not automatic bugs from naming alone. The header still describes schema gating as federation-layer work although a local conditional check exists.'),
('Persona forest and identity-backed Standing accrual expose different root constructions',
 'persona_vault derives logical persona keys and links them, while author_for_identity attributes to the stable master root.',
 [base+'persona_vault.rs',base+'persona_chain.rs',base+'store.rs'],
 'Both are real paths, but their identifiers are not automatically interchangeable. Host integration must decide which root/parent source is canonical; no combined adapter failure was executed. Cycle handling guarantees termination, not a canonical shared root across a malformed cycle.'),
('Moothold repeats the current-revision admission versus corpus-fold pattern',
 'Moothold.fold chooses the lowest-hash eligible child, while accept retains only a child of the currently selected revision.',
 ['crates/moot/moothold/src/fold.rs','crates/moot/moothold/src/store.rs'],
 'This duplicates the constitutional preflight/fold shape fromQ355. Moothold is founder-only, so the distinct-elector no-fork counterexample does not transfer automatically. Its concurrent-child source test directly folds a forked same-author corpus; no Moothold-store convergence case ran here.'),
('Governed federation terms and reciprocal service accounting remain separate owners',
 'Signed Moothold events retain member terms/composition; Reciprocity is a caller-fed in-memory map.',
 ['crates/moot/moothold/src/event.rs','crates/moot/moothold/src/store.rs','crates/moot/moothold/src/reciprocity.rs'],
 'MemberTerms enforces10000bp for signed concord weights, unlike the raw RepLens/Vouch fields. The package has no signed reciprocity event log or automatic binding from governed tolerance to a request router. Q365 executes only arithmetic, not service enforcement or durable credit recovery.'),
('Source test names are not fresh network or durable receipts',
 'Mien has real-net catch-up/ticket tests and Moothold has a redb reopen test; neither suite ran in this pass.',
 [base+'sync.rs','crates/moot/moothold/src/store.rs'],
 'Fresh evidence consists of full current Mien compilation plus six public CPU cases, and scoped Reciprocity/MootId compilation plus five original tests and three audit tests. Old receive-only Standing prose is stale against current Gemot publish_standing. No whole Moothold library or fresh Cargo dependency graph was built.'),
]
entries=[{'id':f'A{1036+i}','domain':d,'claim':c,'implementations':p,'assessment':a,
          'state':'Mien and Moothold source complete; diagnostic scopes explicitly separated','verification':NAME} for i,(d,c,p,a) in enumerate(rows)]
authority['entries'].extend(entries)
completed=[]
for area,count,chars in areas:
    selected=[p for p in fresh if p.startswith(area)];assert len(selected)==count
    assert sum(inv['files'][p]['chars'] for p in selected)==chars
    item={'prefix':area,'files':count,'chars':chars,'basis_commit':head,'verification':NAME,
          'qualification':mien['qualification'] if '/mien/' in area else recip['qualification']+' All seven Moothold files source-reviewed; only the named two-module slice executed.'}
    completed.append(item);notes['completed_source_areas'].append(item)
notes['review_followups'].extend([
    {'verification':NAME,'area':'Governed moderation and active gate facts','detail':'Q359 requires authority-qualified score projection; current Gemot group facts default score/vouch/activity. Trace actual consumers and current/historical roles before claiming an end-to-end authorization or rate-limit guarantee.'},
    {'verification':NAME,'area':'Standing and reciprocity numeric admission','detail':'Q360,Q363,Q365 establish unsigned/signed and basis-point mismatches. Define admitted numeric domains and overflow outcomes across composition, depreciation and accumulation; the other arithmetic branches were source-reviewed only.'},
    {'verification':NAME,'area':'Standing root and shared-store isolation','detail':'Q364 raw-author topic mixing is distinct from attested root-binding correctness. Trace source-of-truth root/persona adapters and backend namespace migration before changing wire/log identities.'},
])
next_read={'path':'crates/moot/mooting/Cargo.toml','start':0,'reason':'Continue full unread Mooting recognition/electorate source, then remaining repository areas.'}
notes['current_area']='Mien and Moothold source complete; six Mien counterexamples and Reciprocity numeric reversal recorded'
notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':[NAME,'mien_owner_probe_receipt.json','moothold_reciprocity_probe_receipt.json'],
    'audit_authored_production_edits_this_turn':False,'next_read':next_read,
    'pending_verification':'No live diagnostic process. Whole Moothold/current Cargo graph, live network/disk/product gates unexecuted; priorQ352 remains pending.',
    'diagnostic_outcomes':'Six fresh public Mien cases pass over unchanged complete owner and8retained suppliers. Reciprocity/MootId scoped gate passes5original+3audit tests. All generated owner libraries, executables/PDB/link outputs removed.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'mien_moothold_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
addition=f'''

### Mien and Moothold source complete: Standing authority, arithmetic and isolation, 2026-10-17

Read18new complete files /158,535characters at unchanged HEAD {head}: all11Mien files /125,757characters and all7Moothold files /32,778characters. Repository coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale/pending credit. Full review remains active and incomplete.

Six public diagnostic cases freshly compile and execute the unchanged complete Mien production owner over8exact retained fingerprint-matched suppliers, using actual signed/attested operations, common admission and MemoryBackend. Q359: an unrelated signer can censure and pardon another root through store/fold without the governance check assumed by event/ledger prose. Raw retention is distinct from effective governance authority; no external-role or hosted gate bypass is claimed. Q360: u64::MAX Pardon gives-1 while Censure gives+1; ordinary30weight controls behave correctly. Q361: a valid same-author made-at100/completed-at99 log is sorted into completion-before-creation and scores-20 lapsed; direct causal fold and forward-time control score+10. Q362: two fresh10000bp vouches double a newcomer's boost while the voucher keeps its score, and self-vouch doubles the voucher's accrual. This is not same-hash replay. Q363: signed65535bp is admitted despite the documented10000ceiling and lends6from score1. Q364: a raw author/log0 reused across two Moot topics causes both projections to score2from one own event each; separate-store control yields1/0. Derived per-Moot signers and separate aggregate stores are distinct isolation controls.

Q365 is reproduced in the unchanged complete Reciprocity module plus actual Gemot MootId, compiled with namespace scaffolding and one retained serde supplier. u64::MAX service debt becomes-1 and allows another request at zero tolerance; maximum tolerance on an empty ledger becomes-1 and refuses. A normal debt/tolerance control works. All5original module tests and3audit tests pass. This is not whole Moothold, durable service credit or a real request router.

A1036-A1047 separate actual shared operation storage from missing moderation meaning, shared stable-author binding from deliberate legacy handling, topic indexing from log isolation, deterministic timestamp sort from causal meaning, documented numeric domains from primitive widths, vouch edge dedupe from boost accrual, host-supplied gate facts from collected rate history, explicit one-hop composition from schema/weight limits, persona roots from stable master accrual, the duplicated current-revision preflight/fold pattern in Moothold, signed federation terms from in-memory reciprocal accounting, and source-only network/durable tests from current execution. Moothold's founder-only model meansQ355's distinct-elector counterexample cannot simply be relabeled as its result. Its signed MemberTerms does validate10000bp; other raw fields do not share that bound. Current Gemot publishing also makes old receive-only Standing prose stale.

No numerical/GC/governance policy is chosen and no production code is edited. Repairs that need wider authority/lifecycle/namespace decisions are documented. No fresh Cargo dependency graph, original Mien suite, whole Moothold owner, disk, live LogSync, external-role or physical product acceptance is claimed. All generated owner rlib/executable/PDB/link outputs are removed; no process remains live.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [Mien public diagnostics](../../../support/receipts/authority-success-audit/mien_owner_probe_receipt.json) and [Reciprocity scoped tests](../../../support/receipts/authority-success-audit/moothold_reciprocity_probe_receipt.json). Prior evidence, protected source/runtime/auxiliary hashes, shared WIP and Git index remain intact; all previous document bytes remain an exact prefix. Diff check passes. No retained isolated target, Cargo home or worktree; retained receipts remain under100MB. Next: unread Mooting recognition/electorate owner.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
evidence=['record_mien_moothold_review.py','mien_moothold_diff_check.txt','mien_owner_probe.rs','run_mien_owner_probe.py',
 'mien_owner_probe_owner_compile.log','mien_owner_probe_consumer_compile.log','mien_owner_probe_output.log','mien_owner_probe_receipt.json',
 'moothold_reciprocity_probe.rs','run_moothold_reciprocity_probe.py','moothold_reciprocity_probe_compile.log',
 'moothold_reciprocity_probe_output.log','moothold_reciprocity_probe_receipt.json']
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Mien and Moothold source complete; public/scoped numeric-authority diagnostics; repository incomplete',
    'prior_receipt':{'path':'gemot_complete_review_receipt.json','sha256':sha(HERE/'gemot_complete_review_receipt.json')},
    'coverage':stats,'completed_source_areas':completed,'whole_current_files':fresh,'source_sha256':{p:sha(ROOT/p) for p in fresh},
    'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'preserved_production_and_test_hashes':guards,
    'shared_working_file_sha256':shared,'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
    'audit_authored_production_edits_this_pass':False,
    'diagnostics':{'mien_owner_probe_receipt.json':{'owner_build_exit':0,'consumer_build_exit':0,'run_exit':0,'checks':6,'original_tests':0,'audit_cases':6},
                   'moothold_reciprocity_probe_receipt.json':{'build_exit':0,'run_exit':0,'checks':8,'original_tests':5,'audit_cases':3}},
    'pending_verification':notes['checkpoint']['pending_verification'],'next_read':next_read,
    'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':index_hash,'retained_isolation':[],
    'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'limits':mien['qualification']+' '+recip['qualification']}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/p)==h for p,h in {**guards,**receipt['source_sha256'],**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],
                  'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
