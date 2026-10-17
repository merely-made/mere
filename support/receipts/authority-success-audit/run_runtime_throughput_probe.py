"""Exact roofline source and rank arithmetic; CPU only, no modeled GPU receipt."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubecl-runtime/src/throughput/'
paths=[base+p for p in ['roofline.rs','benchmarker.rs']]
roof,bench=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
rank=function(bench,'pub fn rank(');retarget=function(bench,'fn retarget(')
target=bench[bench.index('const TARGET_DURATION:'):bench.index(';',bench.index('const TARGET_DURATION:'))+1]
code=r'''#![allow(dead_code,private_interfaces)]
extern crate alloc;
mod roof {
'''+roof+r'''
#[cfg(test)]mod audit {
use super::*;
#[test]fn a_negative_normal_peak_panics_instead_of_being_ignored(){
 let negative=ResourceBound{amount:8,peak_per_s:-4.0};assert!((-4.0f64).is_normal());
 assert!(std::panic::catch_unwind(||negative.time_at_peak()).is_err());
}
#[test]fn a_positive_finite_peak_can_overflow_duration(){
 let slow=ResourceBound{amount:8,peak_per_s:1e-100};assert!(slow.peak_per_s.is_normal());
 assert!(std::panic::catch_unwind(||slow.time_at_peak()).is_err());
}
#[test]fn scoring_a_negative_peak_keeps_a_binding_the_docs_say_is_skipped(){
 let scores=score_resources(Duration::from_secs(1),&[ResourceBound{amount:8,peak_per_s:-4.0}]);
 let chosen=binding_achieved(&scores).unwrap();assert_eq!(chosen.fraction_of_peak,-2.0);
}
#[test]fn scoring_an_infinite_peak_keeps_a_binding_the_docs_say_is_skipped(){
 let scores=score_resources(Duration::from_secs(1),&[ResourceBound{amount:8,peak_per_s:f64::INFINITY}]);
 assert_eq!(binding_achieved(&scores).unwrap().fraction_of_peak,0.0);
}
#[test]fn scoring_a_zero_peak_control_is_unusable(){
 let scores=score_resources(Duration::from_secs(1),&[ResourceBound{amount:8,peak_per_s:0.0}]);
 assert!(binding_achieved(&scores).is_none());
}
}
}
mod ranking {
use std::time::Duration;
'''+target+r'''
const RANK_SAMPLES:usize=3;
struct KernelConfig{sample:Box<dyn Fn(usize)->Duration>,ops_count:usize,min_iterations:usize}
struct ThroughputValue{ops_count:usize,duration:Duration}
struct Ranked{value:ThroughputValue,iterations:usize}
struct ThroughputBenchmarker;
impl ThroughputBenchmarker{
'''+rank+'\n'+retarget+r'''
}
#[cfg(test)]mod audit{
use super::*;
#[test]fn ordinary_ranking_control_preserves_duration_per_iteration(){
 let config=KernelConfig{sample:Box::new(|n|Duration::from_nanos(1000*n as u64)),ops_count:1,min_iterations:1};
 let ranked=ThroughputBenchmarker::rank(&config,1000);assert_eq!(ranked.iterations,20_000);assert_eq!(ranked.value.duration,Duration::from_nanos(1000));
}
#[test]fn zero_minimum_control_is_clamped_to_one(){
 let config=KernelConfig{sample:Box::new(|_|TARGET_DURATION),ops_count:1,min_iterations:0};
 let ranked=ThroughputBenchmarker::rank(&config,0);assert_eq!(ranked.iterations,1);assert_eq!(ranked.value.duration,TARGET_DURATION);
}
#[cfg(target_pointer_width="64")]
#[test]fn a_usize_iteration_floor_multiple_of_u32_range_divides_by_zero(){
 let count=1usize<<32;
 let config=KernelConfig{sample:Box::new(|_|TARGET_DURATION),ops_count:1,min_iterations:count};
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||ThroughputBenchmarker::rank(&config,count))).is_err());
}
#[cfg(target_pointer_width="64")]
#[test]fn retargeting_can_leave_warmup_ceiling_and_normalize_by_wrapped_count(){
 let config=KernelConfig{sample:Box::new(|n|if n==1000 {Duration::from_nanos(1)}else{Duration::from_secs(20)}),ops_count:1,min_iterations:1};
 let ranked=ThroughputBenchmarker::rank(&config,1000);
 assert_eq!(ranked.iterations,20_000_000_000);
 assert_eq!(ranked.value.duration,Duration::from_nanos(7));
 let correct=Duration::from_secs_f64(20.0/ranked.iterations as f64);assert_eq!(correct,Duration::from_nanos(1));assert_ne!(ranked.value.duration,correct);
}
}
}
'''
rs=HERE/'runtime_throughput_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-runtime-throughput.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'runtime_throughput_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'runtime_throughput_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '17 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':17,'original_roofline_tests':8,'new_roofline_cases':5,'rank_cases':4,'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in {'rank':rank,'retarget':retarget,'TARGET_DURATION':target,'full_roofline':roof}.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_runtime_throughput_probe.py',rs.name,'runtime_throughput_compile.txt','runtime_throughput_run.txt']},
 'qualification':'Full unchanged roofline.rs compiled with8 original tests and5 new numeric policy cases. Exact rank/retarget bodies and target duration compiled with shape-equivalent config/result structs and deterministic CPU duration callbacks;2 ordinary controls and2 U64-count cases on64-bit host. No actual kernel iterations, hardware timing, GPU measurement, current probe-caller reachability, cache insertion or full backend/runtime build. Exact owned exe/PDB removed.'}
with (HERE/'runtime_throughput_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':17,'removed_outputs':len(removed)}))
