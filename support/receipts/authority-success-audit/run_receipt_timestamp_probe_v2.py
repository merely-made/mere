"""Compile an exact, hash-bound standalone source function; no Cargo rebuild."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
NAME = 'receipt_timestamp_probe_v2'
RECEIPT = HERE / (NAME + '_receipt.json')
assert not RECEIPT.exists()
source = ROOT / 'ports/graphshell/src/receipts/manifest.rs'
original = source.read_bytes()
text = original.decode('utf-8')
start = text.index('pub(crate) fn parse_rfc3339_ms(')
end = text.index('\n#[cfg(test)]', start)
function = text[start:end]
tests = r'''
#[cfg(test)]
mod audit_tests {
    use super::*;
    #[test]
    fn original_utc_control() {
        assert_eq!(parse_rfc3339_ms("2026-08-10T14:31:05.1234567Z"), Some(1_786_372_265_123));
        assert_eq!(parse_rfc3339_ms("not a time"), None);
    }
    #[test]
    fn positive_offset_is_silently_ignored() {
        let result = parse_rfc3339_ms("2026-08-10T14:31:05.123+05:30");
        assert_eq!(result, Some(1_786_372_265_123));
        assert_ne!(result, Some(1_786_352_465_123));
        println!("positive-offset found={result:?}, required=Some(1786352465123)");
    }
    #[test]
    fn negative_offset_is_ignored_or_unknown_depending_on_fraction() {
        assert_eq!(parse_rfc3339_ms("2026-08-10T14:31:05.123-05:30"), Some(1_786_372_265_123));
        assert_eq!(parse_rfc3339_ms("2026-08-10T14:31:05-05:30"), None);
    }
    #[test]
    fn malformed_unicode_fraction_panics() {
        assert!(std::panic::catch_unwind(|| parse_rfc3339_ms("2026-08-10T14:31:05.ééZ")).is_err());
    }
    #[test]
    fn overflowing_year_panics_in_checked_native_build() {
        assert!(std::panic::catch_unwind(|| parse_rfc3339_ms("9223372036854775807-08-10T14:31:05Z")).is_err());
    }
    #[test]
    fn invalid_february_date_is_accepted() {
        assert!(parse_rfc3339_ms("2026-02-30T14:31:05Z").is_some());
    }
}
'''
probe = HERE / (NAME + '.rs')
with probe.open('x', encoding='utf-8', newline='') as stream:
    stream.write(function + tests)
binary = TARGET / ('authority-success-' + NAME + '.exe')
assert TARGET.is_dir() and not list(TARGET.glob(binary.stem + '.*'))
command = ['rustc', '--edition=2024', '--test', str(probe), '-C', 'debuginfo=0', '-o', str(binary)]
built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=45)
with (HERE / (NAME + '_compile.log')).open('x', encoding='utf-8') as stream:
    stream.write(built.stdout + built.stderr)
removed = []
try:
    assert built.returncode == 0, built.stderr
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    ran = subprocess.run([str(binary), '--nocapture', '--test-threads=1'], cwd=ROOT,
                         capture_output=True, text=True, timeout=30)
    with (HERE / (NAME + '_output.log')).open('x', encoding='utf-8') as stream:
        stream.write(ran.stdout + ran.stderr)
    print(ran.stdout)
finally:
    for generated in sorted(TARGET.glob(binary.stem + '.*')):
        assert generated.resolve().parent == TARGET
        data = generated.read_bytes()
        assert ((generated.suffix == '.exe' and data.startswith(b'MZ')) or
                (generated.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))), str(generated)
        removed.append({'path': str(generated), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
        generated.unlink()
assert ran.returncode == 0, ran.stderr
assert source.read_bytes() == original
assert not list(TARGET.glob(binary.stem + '.*'))
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'tests': 6,
    'source_sha256': {'ports/graphshell/src/receipts/manifest.rs': hashlib.sha256(original).hexdigest()},
    'verbatim_function': {'char_start': start, 'char_end': end, 'sha256': hashlib.sha256(function.encode()).hexdigest()},
    'probe_sha256': hashlib.sha256(probe.read_bytes()).hexdigest(),
    'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'command': command, 'build_exit': built.returncode, 'test_exit': ran.returncode,
    'binary_sha256': digest, 'removed_outputs': removed, 'generated_output_removed': True,
    'scope': 'Six observations of the exact current parse_rfc3339_ms function extracted verbatim, standard library only. UTC and invalid-text controls; positive-offset mismeasurement, negative offset ignored with fractional seconds and refused without a fraction, malformed Unicode panic, checked-arithmetic overflow panic and invalid date acceptance. No full manifest/importer/graph/Cargo runtime executed. Source/original tests unchanged.'}
with RECEIPT.open('x', encoding='utf-8') as stream:
    stream.write(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'receipt': RECEIPT.name, 'tests': 6, 'generated_output_removed': True}))
