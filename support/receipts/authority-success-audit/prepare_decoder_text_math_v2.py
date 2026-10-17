"""Retain v1 failure; correct the sampling hypothesis and output marker set."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
removed=[]
for p in sorted(TARGET.glob('authority-success-decoder-text-math.*')):
 assert p.resolve().parent==TARGET;data=p.read_bytes()
 assert (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.lib' and data.startswith(b'!<arch>\n')) or (p.suffix=='.exp' and data.startswith(b'd\x86'))
 removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p),'marker':'PDB MSF, COFF AMD64 or ar archive by extension'});p.unlink()
v1={'build_exit':0,'run_exit':101,'passed':14,'failed':1,'failure':'Audit hypothesis expected unique maximum token1 to be lost at tiny temperature, but original sampler selected token1. This invalidates that counterexample. Check tied maxima separately.','cleanup_failure':'Original runner recognized executable/PDB only. Actual tokenizer-linked test also creates COFF .exp and ar .lib. Executable was removed before marker refusal, but its digest was not persisted because final receipt was never reached. Remaining three outputs now verified and removed.','removed_outputs':removed,'generated_output_removed':not list(TARGET.glob('authority-success-decoder-text-math.*')),'evidence_sha256':{n:sha(HERE/n) for n in ['run_decoder_text_math_probe.py','prepare_decoder_text_math_probe.py','decoder_text_math_probe.rs','decoder_text_math_source.json','decoder_text_math_cases.rs','decoder_text_math_probe_compile.log','decoder_text_math_probe_output.log']}}
with (HERE/'decoder_text_math_v1_failure_receipt.json').open('x',encoding='utf-8') as f:json.dump(v1,f,indent=2);f.write('\n')
s=(HERE/'decoder_text_math_probe.rs').read_text()
first=s.index('    #[test] fn accepted_subnormal_temperature_selects_the_wrong_token()')
last=s.index('    #[test] fn nan_expected_logit_is_ranked_first()',first)
replacement='''    #[test] fn accepted_subnormal_temperature_biases_tied_maxima() {
        let t=1e-40_f32; assert!(t.is_finite() && t>0.0);
        for top_p in [None,Some(0.9)] {
            let mut tiny=[0usize;2]; let mut control=[0usize;2];
            for seed in 0..128 {
                let mut s=sample::Sampler::new(t,top_p,seed);
                tiny[s.sample(&[3.0,3.0]) as usize]+=1;
                let mut c=sample::Sampler::new(0.01,top_p,seed);
                control[c.sample(&[3.0,3.0]) as usize]+=1;
            }
            assert_eq!(tiny,[0,128]);
            assert!(control[0]>32 && control[1]>32, "{control:?}");
        }
    }
'''
with (HERE/'decoder_text_math_probe_v2.rs').open('x',encoding='utf-8') as f:f.write(s[:first]+replacement+s[last:])
r=(HERE/'run_decoder_text_math_probe.py').read_text().replace("NAME='decoder_text_math_probe'","NAME='decoder_text_math_probe_v2'").replace("binary=TARGET/'authority-success-decoder-text-math.exe'","binary=TARGET/'authority-success-decoder-text-math-v2.exe'")
r=r.replace("or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))", "or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.lib' and data.startswith(b'!<arch>\\n')) or (p.suffix=='.exp' and data.startswith(b'd\\x86'))")
with (HERE/'run_decoder_text_math_probe_v2.py').open('x',encoding='utf-8') as f:f.write(r)
print(json.dumps({'v1_failed_hypothesis_retained':True,'v1_outputs_removed':len(removed),'v2_prepared':True}))
