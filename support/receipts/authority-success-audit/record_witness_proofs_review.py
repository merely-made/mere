"""One-shot state-witness/proofs review and fully read Mesh namespace follow-up."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
 for value in values:
    at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
    if at is None:items.append(value)
    else:items[at]=value
old=load('load_contract_review_receipt.json');probe=load('witness_proofs_probe_receipt.json')
assert probe['tests']==4 and probe['test_exit']==0 and probe['generated_output_removed']
suite=(HERE/'witness_proofs_suite.log').read_text(encoding='utf-8-sig')
for count in [2,5,6,10]:assert f'{count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert suite.count('5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out')==2
namespace_suite=(HERE/'witness_proofs_namespace_suite.log').read_text(encoding='utf-8-sig')
assert '6 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out' in namespace_suite
tree=(HERE/'witness_normal_tree.log').read_text(encoding='utf-8-sig')
excluded=['heapless','hash32','byteorder','spin','lock_api','scopeguard','stable_deref_trait']
assert not any(line.split()[0] in excluded for line in tree.splitlines() if line.split())
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith(('crates/system/state-witness/','crates/system/proofs/')) or p=='crates/mesh/mesh/src/namespace.rs')
assert len(complete)==11 and sum(inventory[p]['chars'] for p in complete)==56519
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1164 and stats['read_chars']==15464067
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')

def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'reproduced through current Cargo-built public APIs; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'witness_proofs_review_receipt.json','diagnostic':'witness_proofs_probe_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test source','size':size,'reason':repair}}
findings=[
finding(194,'Out-of-order unique witness entries are falsely reported as duplicate labels',
 'crates/system/state-witness/src/witness.rs:TryFrom<Vec<Entry>>; Error::DuplicateLabel',
 'Witness::try_from accepts an owned list with distinct site:10 and site:9 labels and refuses it as DuplicateLabel(site:9), although that label occurs once. The list is invalid because natural order is reversed. Ordinary insertion sorts the same labels and framed reload succeeds. The Error documentation says DuplicateLabel means the witness already holds that label.',
 'Public list admission and current strict-framed roundtrip control. Invalid entries remain refused, so this is a misleading diagnostic rather than invalid-state admission, false equality or data loss.',
 'contained load-error distinction, preserving the ruled insert-error contract',
 'Distinguish ordering refusal from duplicate identity in load diagnostics. Preserve F124\'s existing insert errors and review consumer matches before choosing a new public load-error shape; do not silently sort malformed stored input or call distinct entries duplicates.'),
finding(195,'Mesh ambient blob presence and reads ignore the typed algorithm and declared length',
 'crates/mesh/mesh/src/namespace.rs:MunimentBlobSpace::has/get; crates/system/proofs/src/lib.rs:Digest::as_32 and BlobRef::verifies; Mesh JobSpec::validate',
 'After storing synthetic bytes, changing the BlobRef algorithm to Sha256 or increasing byte_len makes BlobRef::verifies false. Ambient MemoryBlobSpace::has still returns true and get returns those bytes, because only the raw32-byte array becomes a muniment BLAKE3 key. JobSpec validation also admits each reference through its width-only check. The actual JobNamespaceView then returns DigestMismatch before the resource sees either input. Stickleback\'s separate get_blob implementation explicitly checks BLAKE3 and reference.verifies.',
 'Current real Mesh/Proofs/Muniment in-memory APIs with valid, wrong-algorithm and wrong-length references. The namespace file is fully read; broader Mesh/spec and Stickleback/courier excerpts are not whole-file coverage. The resource-facing guard was executed and works: no permission bypass, unverified adapter input, peer exchange or durable user blob was demonstrated.',
 'contained typed-reference admission correction; shared store-adapter ownership review can exceed one-hour diversion',
 'Apply BlobRef semantics consistently at host readiness/lookup boundaries, distinguishing mere raw-key residency from a held valid typed reference. Preserve the existing resource-view verification. Reuse BlobRef::verifies and explicit supported-algorithm admission rather than treating as_32 as a verifier or creating another weaker store-specific hash rule.'),
]
findings[0]['impact_class']='misleading refusal diagnostic; invalid state still refused'
findings[1]['impact_class']='host lookup/readiness inconsistency; resource boundary correctly refuses'
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All8 state-witness and2 proofs files plus Mesh namespace read;26+2 original tests,6 filtered namespace tests and4 native observations pass',
  'verification':'witness_proofs_review_receipt.json'}
entries=[
entry(347,'Replay witness versus authenticated identity','State-witness is a labelled FNV equality aid, not a cryptographic digest.',
 ['Streaming postcard FNV flavor', 'Raw byte hash', 'Natural-order labelled fold with NUL separator and fixed-width digest', 'Typed first-divergence and tick trace'],
 'The implementation follows the explicitly narrow ruling. No signature, authorization, Merkle inclusion or adversarial collision-resistance claim should be inferred from a replay equality result. Proofs uses a separate cryptographic vocabulary for a different job; this is not two competing implementations of one commitment verifier.'),
entry(348,'FNV primitive copies versus extraction scope','F119 names State-witness as the owner and Isometry retires its copy.',
 ['Shared state-witness hash_bytes', 'Frozen copied isometer reference only in the independent test', 'Current FNV-signature sites in Cambium/Pictograph/Scenomise/linked-data/ESP/document-host', 'Explicit F119 reading leaves Mere\'s existing copies outside this move'],
 'Real repeated primitive code is tracked in witness_fnv_candidate_sites.log. Search hits include tests/constants and differing input conventions, so they are not a count of identical production contracts. The ruling did not claim all Mere FNV copies were retired. No blanket migration or family-name/publication decision is chosen here; current Isometry adoption was not checked.'),
entry(349,'Postcard-byte equality versus canonical application state','hash_value streams exactly the postcard encoding.',
 ['Flavor writes without allocating a full encoded buffer', 'Original tests compare against to_allocvec', 'Float sign/NaN bits are preserved', 'Ordered collection examples use BTreeMap/BTreeSet'],
 'The byte-level equivalence is real and freshly tested. Canonical collection iteration, type/schema identity and which state fields matter remain producer responsibilities. Raw bytes versus serde sequences deliberately differ by their length encoding; do not call this helper a canonicalizer of arbitrary Serialize values.'),
entry(350,'Label grammar versus structured identity','F127 explicitly chose plain String plus label(kind,parts).',
 ['Display parts appended with colon/slash delimiters', 'Witness rejects NUL and exact duplicate strings', 'Natural comparison only equates identical final strings', 'No escaping or typed path parts'],
 'The native formatter collision shows one a/b part equals two a,b parts. This is a producer grammar/admission limit, not a newly demonstrated current consumer collision: no Mere production caller of state-witness was found. Preserve the plain-string ruling rather than unilaterally introducing a new grammar.'),
entry(351,'Witness ordering and admission authority','Unique labels are naturally sorted and invalid stored lists are refused.',
 ['insert uses binary-search slot and preserves state on errors', 'TryFrom validates NUL/strict natural ordering', 'Serde loading passes through TryFrom', 'Divergence merge walks the same natural comparator'],
 'Actual invariant enforcement is present, not merely promised by derives. Q194 is the false duplicate diagnostic for a distinct out-of-order list. Digit-run comparison avoids numeric parse overflow, and the byte tie-break keeps09/9 distinct. No new comparator or equality defect was reproduced.'),
entry(352,'Tick trace currentness','Trace ticks strictly increase and comparison names the first differing or missing tick.',
 ['push validates increasing ticks', 'Deserialize routes through validated Vec conversion', 'Binary-search get', 'first_trace_divergence merges increasing tick sequences and delegates label comparison'],
 'The immutable private storage and validated constructors preserve the required order. Original planted-change, missing-tick and earlier-label-versus-later-gap controls pass. No runtime/clock or input-replay implementation is supplied; those are caller duties, not a success claim of Trace.'),
entry(353,'Framed witness persistence versus legacy tolerance','Witness/Trace use distinct checked schema/version headers.',
 ['Both delegate directly to Framing strict unframe', 'Magic/version1 pinned by fixed header tests', 'NUL-free labels prevent the tested payload/magic collision', 'JSON stores u64 digests as explicitly ruled F126'],
 'This is an actual adopter of the shared header helper. Original tests refuse unframed, wrong-schema/version, invalid witness and unordered trace data. Framing\'s declared trailing tolerance still applies; no full byte-exhaustion guarantee is made. Rust JSON roundtrip is not JavaScript precision qualification; the u64 choice is an explicit ruling.'),
entry(354,'Typed digest tag versus raw fixed-width view','Digest is algorithm-labelled and as_32 says it reads without discarding algorithm.',
 ['Digest equality includes alg and bytes', 'as_32 validates only length and returns an untagged array', 'p2panda_operation wraps caller-supplied32 bytes with a tag', 'Caller conversions manufacture raw backend hash types'],
 'The complete value keeps its algorithm, but the returned array does not carry one. The documentation is too broad if treated as an algorithm-safe conversion. Native controls show differently tagged digests yield the same array while full BlobRef verification rejects them. Q195 identifies an actual caller that mistakes width for sufficient identity admission.'),
entry(355,'Application commitment verification scope','Direct commitments retain scheme/domain/version/digest meaning.',
 ['Commitment::direct hashes caller canonical bytes with BLAKE3', 'verifies_direct checks scheme/domain/version/digest', 'Other scheme enum variants are vocabulary only', 'BlobRef and Commitment remain distinct outer Rust types'],
 'Native controls reject wrong expected domain, changed bytes and version. The crate does not canonicalize supplied bytes or implement every tagged Merkle/accumulator/PSI scheme. Tags are part of the complete object contract, not stand-alone authentication or existence proof for arbitrary caller-created digests.'),
entry(356,'Typed BlobRef verification versus backend residency','A content reference includes algorithm, bytes and declared length.',
 ['BlobRef::verifies checks all needed direct BLAKE3 facts', 'Mesh ambient has/get use raw digest key only', 'Stickleback get_blob explicitly checks algorithm and verifies bytes', 'Distillery courier conversion/tag also takes width-only as_32'],
 'Q195 is the actual ambient success-path mismatch; the resource view later refuses. Shared storage substrate does not imply shared typed-reference admission. Courier source is a candidate follow-up only, not a live delivery failure or accepted invalid peer transfer.'),
entry(357,'Host grant versus resource authority','JobNamespaceView is the only read/write capability passed to the resource.',
 ['Host-only grant builds named input and single output slots from assumed validated spec', 'read checks name, presence and BlobRef::verifies before returning', 'commit rejects wrong slot and oversized bytes before sink', 'No Backend/network/filesystem handle is exposed'],
 'Fresh original six namespace controls plus native invalid-algorithm/length controls establish the claimed resource boundary. Host authorization before grant remains an explicit precondition. No automatic job permission resolver is implemented by the view; sink durability is still the host callback\'s contract.'),
entry(358,'Spec shape admission versus usable content identity','JobSpec validation admits well-formed inputs.',
 ['Input names/duplicates validated', 'Digest width checked through as_32', 'Algorithm and declared byte length not checked by that path', 'Resource view eventually performs complete content verification'],
 'The native probe confirms bad typed references pass validate yet fail read. Validation is a shape gate, not a proof a supported/held reference will execute. Decide supported algorithm/input-size policy and keep preflight/reporting aligned without weakening the final content check.'),
entry(359,'Supplier equality evidence and dependency floor','Independent equality test and lean normal dependency graph qualify the helper.',
 ['260 fixed inputs compare against the frozen isometer reference; off-prime negative control', 'Original framed/JSON/invariant tests total26', 'Current normal tree excludes seven prior postcard-default artifacts', 'Manifest has only serde/postcard/framing runtime dependencies'],
 'The equality test proves present implementation versus its copied compatibility reference, not fresh external-repository adoption. The normal dependency cut is currently verified, matching the later plan correction. No fresh wasm32, physical replay or cross-machine determinism run was performed.'),
entry(360,'Reading scope versus tested dependency reach','Two leaf crates and one Mesh module are completely read.',
 ['State-witness8 files37331 chars', 'Proofs2 files5534 chars', 'Mesh namespace1 file13654 chars', 'Mesh unit filter runs6 and leaves117 unexecuted'],
 'Full-source objective remains incomplete. Current tests compile broader dependencies but that is not reading coverage or full Mesh acceptance. The candidate FNV site search and caller/design excerpts add ownership evidence only. Synthetic MemoryBackend probes created no user files, network or peer operations.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Algorithm-erasing blob conversions recur outside the ambient Mesh adapter',
  'source':'ports/distillery/src/mesh_host/courier.rs:blob_hash and tag; crates/mesh/mesh/src/spec.rs:validate; Stickleback get_blob control',
  'note':'Source excerpts show width-only as_32 conversion to transport hashes/tags. Q195 proves ambient Mesh has/get inconsistency, while JobNamespaceView correctly refuses bad references and Stickleback has explicit algorithm/full verification. No live courier delivery/retention mistake or peer admission was executed. Follow actual owners before sharing an admission helper; do not call all raw32 views verified references. Can exceed one-hour diversion.'},
 {'title':'Plain witness label parts need producer grammar guarantees',
  'source':'crates/system/state-witness/src/lib.rs:label; dynamics grammar plan F127',
  'note':'Native helper output collides for a single a/b part versus separate a,b parts. Plain String/helper was explicitly chosen; no current Mere production caller was found, and no Isometry label collision was tested. Document producer delimiter rules or validate at actual producer before choosing an escaping/schema migration. Not a new demonstrated application identity defect.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/mesh/mesh/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete state-witness8/8 and proofs2/2; complete Mesh namespace module as caller follow-up'
notes['next_area']='crates/mesh/mesh'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['witness_proofs_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'26 state-witness+2 proofs original tests pass;6 filtered namespace tests pass/117 not run;4 native observations pass. Normal witness tree excludes the seven prior default-feature artifacts.',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'11 complete files56519 chars added. Broader Mesh not claimed; spec/courier/Stickleback/design and FNV candidate search are separately qualified traces.'}
upsert(notes['source_qualifications'],[{'area':'Complete state-witness/proofs and Mesh namespace follow-up',
 'note':'All8 state-witness+2 proofs files and full Mesh namespace module56519 chars read. Default library/composition builds pass;26+2 original tests and6 namespace-filtered tests pass,117 Mesh tests unexecuted. Four generated current-library CPU observations pass: false ordering diagnostic, ambient typed-reference success mismatch with working resource guard, and qualified label/raw-digest boundaries. Witness normal tree confirms removal of seven prior postcard-default artifacts. No actual external Isometry adoption check, fresh wasm32, cross-machine replay, signatures, peer/network, browser, GPU or durable user files. Exact hashed artifacts used; probe binary/PDB removed; stable target reused, no isolation.'}],'area')

doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only state-witness, Proofs and Mesh namespace review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll 8 state-witness files,2 Proofs files and the full Mesh namespace module are read,11 files and56,519 characters added. Repository coverage is1,164/3,352 files and15,464,067/36,854,820 characters. Broader Mesh is incomplete. Full-source goal remains active; production and original tests/concurrent work are unchanged. Resume at '+next_path+'. Authority entries A347-A360 distinguish replay equality, explicit extraction scope, typed proof vocabulary, backend residency, resource verification and source/test limits.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='State-witness is correctly scoped to labelled non-cryptographic replay equality. Natural ordering, NUL/duplicate rejection, immutable private storage, serde TryFrom validation, framed schema/version checks, first label/tick divergence and bit-preserving value hashing are real implementations with fresh original controls. Raw byte and postcard sequence hashes intentionally differ. The helper does not canonicalize arbitrary producer collections or identify a Rust type/schema inside hash_value; producers own that stable shape and iteration. The plain String label helper is an explicit F127 ruling. One slash-bearing part collides with two path parts, as the generated observation shows, but no actual Mere producer or external Isometry label collision was found/tested. This remains a producer grammar boundary, not an invented cryptographic or application identity bypass.\n\n'
addition+='F119 chose State-witness as the shared FNV owner for Isometry adoption and explicitly left Mere\'s existing hand-written copies outside that move. Current candidate sites are retained in witness_fnv_candidate_sites.log, including Cambium, Pictograph, Scenomise, linked-data, ESP and document-host. These are repeated primitive sites, not automatically equivalent domain canonicalization contracts; the search also finds tests/constants. No completed-all-Mere-retirement claim was found and no blanket replacement is authorized by that ruling. The independent test intentionally copies the old isometer function and includes an off-prime negative control. Fresh equality against that frozen reference does not establish current external repository adoption; no such adoption was checked.\n\n'
addition+='Proofs is a vocabulary plus direct BLAKE3 verification, not a universal proof engine. Whole BlobRef/Commitment checks retain their required algorithm/length/domain/scheme/version facts. Other scheme variants are tags awaiting their actual owner verifier. as_32 checks width and returns an untagged array; the original Digest still carries its tag, but the array is not an algorithm-safe identity. Q195 traces an actual weaker lookup in Mesh, and Stickleback independently implements the stricter reference check over the same backing substrate. The existing JobNamespaceView then enforces full input verification before adapter access, so the host readiness/lookup inconsistency is explicitly separated from a resource authority bypass. JobSpec currently accepts the bad references through its width-only validation. Courier conversion/tag excerpts show a related follow-up boundary, not an executed delivery/retention failure.\n\n'
addition+='The namespace view exposes only named input reads and one bounded output slot after a host-owned grant. Fresh six original controls prove wrong names/size are refused before sink writes and lying input bytes are refused before adapters. The new invalid-algorithm/length controls confirm the same guard. Host authorization before grant and sink commit/durability are preconditions, not additional authority supplied by the view. No permission resolver, signed operation acceptance, network peer or user blob mutation was run here.\n\n'
addition+='Current library/composition builds pass. State-witness original suites pass26 tests (formats5,hash6,trace5,witness10), Proofs passes2, and the Mesh namespace filter passes6 with117 other Mesh tests unexecuted. Four generated native observations pass: two qualified findings and two explicit format/tag boundary checks. The current normal State-witness graph excludes heapless/hash32/byteorder/spin/lock_api/scopeguard/stable_deref_trait, confirming the later dependency-floor correction. Historical wasm32/clippy/cross-repository landing receipts remain historical; this pass makes no fresh wasm32 or cross-machine replay claim. Full Mesh source/suite is not qualified by a module test. All probe storage is synthetic MemoryBackend; no filesystem/backend user state, signatures, peer/network, browser or GPU ran. Executable/PDB removed; stable target reused, no isolated target, Cargo home or worktree. Evidence and source hashes are in the [witness/proofs review receipt](../../../support/receipts/authority-success-audit/witness_proofs_review_receipt.json).\n'
save('review_notes.json',notes);save('authority_map.json',authority)
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'witness_proofs_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['witness_proofs_probe.rs','run_witness_proofs_probe.py','witness_proofs_probe_receipt.json','witness_proofs_probe_build.log','witness_proofs_probe_output.log',
 'witness_proofs_library_build.jsonl','witness_proofs_library_build_stderr.log','witness_proofs_composition_build.jsonl','witness_proofs_composition_build_stderr.log',
 'witness_proofs_suite.log','witness_proofs_namespace_suite.log','witness_normal_tree.log','witness_normal_tree_stderr.log','witness_fnv_candidate_sites.log','witness_proofs_diff_check.txt']
traces=[ROOT/'crates/mesh/mesh/src/spec.rs',ROOT/'crates/mesh/mesh/src/lib.rs',ROOT/'crates/mesh/mesh/Cargo.toml',
 ROOT/'crates/stickleback/src/store.rs',ROOT/'ports/distillery/src/mesh_host/courier.rs',
 ROOT/'design_docs/mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete state-witness/proofs reading and complete Mesh namespace caller module; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'original_tests':{'state-witness':26,'proofs':2,'mesh_namespace_filter':6},'mesh_tests_filtered_out':117,'test_exits':0,
 'witness_normal_graph_excludes':excluded,'diagnostic_tests':4,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'excerpt_only_traces':{str(p):sha(p) for p in traces},
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production/original test changes.',
  'All8 state-witness,2 proofs and1 Mesh module read; not all Mesh source. Caller/design excerpts and search hits are not whole-file credit.',
  '26+2 original tests and6 filtered namespace tests pass;117 other Mesh tests unexecuted.',
  'Four generated native observations; wrong lookup references remain blocked by the executed JobNamespaceView guard.',
  'False duplicate diagnostic refuses invalid ordering; no invalid witness admission.',
  'Plain-label formatter collision and raw32 algorithm erasure are qualified boundaries; no current producer label collision or universal proof/authentication claim.',
  'F119 explicitly excludes Mere other FNV copies from its retirement scope; current Isometry adoption not checked.',
  'No fresh wasm32, cross-machine replay, signature verification, peer/network, browser/GPU or durable user files.',
  'Stable target reused, no retained isolated target/home/worktree; generated executable/PDB removed.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
save('witness_proofs_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
