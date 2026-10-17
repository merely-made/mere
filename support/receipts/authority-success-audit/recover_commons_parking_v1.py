from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent
TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='commons_parking_v1_failure_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/NAME).exists()
outputs=sorted(TARGET.glob('authority-success-commons-parking.*'))
assert len(outputs)==6
removed=[]
for p in outputs:
    assert p.resolve().parent==TARGET
    data=p.read_bytes()
    assert (p.suffix=='.o' and data.startswith(b'\x64\x86')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)})
receipt={'build_exit':1,'run_exit':None,'executed_checks':0,'reason':'Windows linker LNK1181 for windows.0.52.0.lib; harness omitted build-script native search paths. Cleanup marker rejected valid compiler COFF object suffix, stopping the runner before its normal receipt.','runner_sha256':sha(HERE/'run_commons_parking_probe.py'),'probe_sha256':sha(HERE/'commons_parking_probe.rs'),'compile_log_sha256':sha(HERE/'commons_parking_probe_compile.log'),'removed_outputs':removed,'cleanup_basis':'Owned exec session41985 reached terminal exit1; all six outputs remain in the approved stable target, with distinct audit prefix and validated COFF/PDB markers. Hashes recorded before removing.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
for item in removed:
    p=Path(item['path']);assert sha(p)==item['sha256'];p.unlink()
assert not list(TARGET.glob('authority-success-commons-parking.*'))
print(json.dumps({'failure_receipt':NAME,'removed_outputs':len(removed),'remaining_outputs':0}))
