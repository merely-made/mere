"""Keep failed attempts immutable; correct the harness's equal-size premise."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
probe=(HERE/'commons_parking_probe.rs').read_text(encoding='utf-8')
old='let one=bytes(1);assert_eq!(one,bytes(2));\n    let status=race(parking::ParkingLimits{max_records:10,max_bytes:one});\n    assert_eq!(status.parked,2);assert_eq!(status.parked_bytes,one*2);'
new='let first=bytes(1);let second=bytes(2);let cap=first.max(second);\n    let status=race(parking::ParkingLimits{max_records:10,max_bytes:cap});\n    assert_eq!(status.parked,2);assert_eq!(status.parked_bytes,first+second);\n    assert!(status.parked_bytes>cap);'
assert probe.count(old)==1
with (HERE/'commons_parking_probe_v3.rs').open('x',encoding='utf-8') as f:f.write(probe.replace(old,new))
runner=(HERE/'run_commons_parking_probe_v2.py').read_text(encoding='utf-8')
assert runner.count("NAME='commons_parking_probe_v2'")==1
assert runner.count("'commons_parking_probe.rs'")==2
runner=runner.replace("NAME='commons_parking_probe_v2'","NAME='commons_parking_probe_v3'").replace("'commons_parking_probe.rs'","'commons_parking_probe_v3.rs'")
with (HERE/'run_commons_parking_probe_v3.py').open('x',encoding='utf-8') as f:f.write(runner)
print('Created distinct v3 harness and runner; original and v2 evidence unchanged.')
