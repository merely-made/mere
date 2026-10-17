"""Exact pure numeric helpers: session fixture versus remote fixture."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
dest=HERE/'probe_numeric_validation_receipt.json';assert not dest.exists()
paths=['ports/distillery/probe/session-fixture/src/main.rs','ports/distillery/probe/remote-fixture/src/main.rs']
sources={n:sha(ROOT/n) for n in paths}
session=(ROOT/paths[0]).read_text(encoding='utf-8-sig')
remote=(ROOT/paths[1]).read_text(encoding='utf-8-sig')
extract=lambda s,start,end:s[s.index(start):s.index(end,s.index(start))]
a=extract(session,'fn max_abs(', 'fn decode(')
b=extract(remote,'fn max_abs_error(', 'fn l2_norm(')
probe=HERE/'probe_numeric_validation.rs';assert not probe.exists()
probe.write_text('use std::error::Error;\n'+a+b+'''
fn main() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let result = max_abs(&[value, 2.0], &[value, 0.0]).unwrap();
        assert_eq!(result, 2.0);
        assert!(max_abs_error(&[value, 2.0], &[value, 0.0]).is_err());
    }
    println!("OBSERVATION: session max_abs silently drops NaN differences, including equal infinities; remote max_abs_error refuses non-finite inputs");
    assert_eq!(max_abs(&[0.0, -1.0], &[0.0, -0.5]).unwrap(), 0.5);
    assert_eq!(max_abs_error(&[0.0, -1.0], &[0.0, -0.5]).unwrap(), 0.5);
    println!("CONTROL: finite inputs agree on the maximum difference");
    assert!(max_abs(&[1.0], &[]).is_err());
    assert!(max_abs_error(&[1.0], &[]).is_err());
    println!("CONTROL: both helpers refuse mismatched lengths");
    println!("3 checks passed");
}
''',encoding='utf-8')
binary=TARGET/'authority-success-probe-numeric-validation.exe';assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024',str(probe),'-C','debuginfo=0','-o',str(binary)]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'probe_numeric_validation_compile.log').write_text(built.stdout+built.stderr,encoding='utf-8')
assert built.returncode==0,built.stderr
digest=sha(binary);removed=[]
try:
    ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=15)
    (HERE/'probe_numeric_validation_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
    print(ran.stdout)
finally:
    for p in sorted(TARGET.glob(binary.stem+'.*')):
        assert p.resolve().parent==TARGET;data=p.read_bytes()
        if p.suffix=='.exe':assert data.startswith(b'MZ')
        elif p.suffix=='.pdb':assert data.startswith(b'Microsoft C/C++ MSF')
        elif p.suffix=='.lib':assert data.startswith(b'!<arch>\n') and binary.name.encode() in data
        elif p.suffix=='.exp':assert data[:2]==b'\x64\x86' and 0<struct.unpack_from('<H',data,2)[0]<128 and binary.name.encode() in data
        else:raise AssertionError('unrecognized generated output '+str(p))
        removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert ran.returncode==0 and '3 checks passed' in ran.stdout,ran.stderr
assert all(sha(ROOT/n)==h for n,h in sources.items())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,
 'verbatim_extracted_functions':{'session_max_abs':a,'remote_max_abs_error':b},
 'probe_source_sha256':sha(probe),'build_command':command,'build_exit':built.returncode,
 'native_binary_sha256':digest,'exit':ran.returncode,'checks':3,'removed_outputs':removed,
 'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),
 'scope':'Exact pure std-only helpers extracted verbatim from the current two source files and compiled natively. Synthetic float values, finite and length controls only. Session fixture separately checks adapted/independent logits for finiteness, so this is not proof that a complete model receipt passes NaNs. No actual model, GPU, browser, network, installed owner or original tests ran.'}
dest.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({'receipt':dest.name,'checks':3,'exit':ran.returncode}))
