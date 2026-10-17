//! Public testkit API diagnostics. Every path is a generated audit fixture.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use djinn_testkit::guard::{Guard, SpawnKind, isolated_env};
use djinn_testkit::walls::WallSpec;
use djinn_testkit::{HarnessError, Run, RunConfig, verify};

fn base() -> PathBuf {
    PathBuf::from(std::env::var_os("MERE_AUTHORITY_AUDIT_FIXTURES").unwrap())
}
fn config(case: &str) -> RunConfig {
    let mut config = RunConfig::new(case, case, std::env::current_exe().unwrap());
    config.receipts_root = base().join("receipts");
    config.quiet = None;
    // These diagnostics launch no Djinn or SSH client and touch no user roots.
    config.walls = WallSpec { installed: vec![], standard_endpoints: vec![], canaries: vec![] };
    config
}

#[test]
fn evidence_control_is_retained_and_checked() {
    let run = Run::begin(config("evidence-control")).unwrap();
    let dir = run.dir().to_path_buf();
    let path = dir.join("required.txt");
    std::fs::write(&path, "control evidence").unwrap();
    run.evidence(path);
    let receipt = run.finish_record();
    assert!(receipt.outcome.passed);
    assert_eq!(receipt.evidence.len(), 1);
    let checked = verify(&dir).unwrap();
    assert!(checked.ok());
    assert_eq!(checked.checked, 1);
    println!("CONTROL requested=1 retained=1 checked=1");
}

#[test]
fn missing_requested_evidence_still_reports_passed() {
    let run = Run::begin(config("missing-evidence")).unwrap();
    let dir = run.dir().to_path_buf();
    run.evidence(dir.join("missing-required.txt"));
    let receipt = run.finish_record();
    assert!(receipt.outcome.passed);
    assert!(receipt.evidence.is_empty());
    let checked = verify(&dir).unwrap();
    assert!(checked.ok());
    assert_eq!(checked.checked, 0);
    println!("OBSERVED requested=1 retained=0 outcome=passed verifier=ok checked=0");
}

#[test]
fn unwritable_receipt_path_still_reports_passed() {
    let run = Run::begin(config("write-failure")).unwrap();
    let dir = run.dir().to_path_buf();
    std::fs::create_dir(dir.join("receipt.json")).unwrap();
    let receipt = run.finish_record();
    assert!(receipt.outcome.passed);
    assert!(dir.join("receipt.json").is_dir());
    assert!(dir.join("summary.md").is_file());
    assert!(verify(&dir).is_err());
    println!("OBSERVED outcome=passed receipt-file-absent=true summary-written=true verifier=error");
}

#[test]
fn rejected_resident_name_has_already_written_outside_the_run() {
    let run = Run::begin(config("pre-guard-write")).unwrap();
    let escaped = base().join("outside-run-root");
    assert!(!escaped.exists());
    let mut resident = run.resident(escaped.to_str().unwrap());
    let lock = resident.app_dir().join("lock.toml");
    assert!(!lock.starts_with(run.dir()));
    let refused = resident.start(&[]);
    assert!(matches!(refused, Err(HarnessError::Refused(_))));
    assert!(resident.pid().is_none());
    assert!(lock.is_file());
    assert!(std::fs::read_to_string(lock).unwrap().contains("session_lock = false"));
    drop(resident);
    let receipt = run.finish_record();
    assert!(receipt.outcome.passed);
    println!("OBSERVED spawn-refused=true child-spawned=false lock-settings-written-outside-run=true");
}

#[cfg(windows)]
#[test]
fn case_aliases_pass_the_guard_and_override_the_child_environment() {
    let root = base().join("guard-run");
    let token = "authority-audit-alias";
    let app = format!(r"\\.\pipe\djinn-{token}-app");
    let browser = format!(r"\\.\pipe\djinn-{token}-browser");
    let guard = Guard { run_dir: root.clone(), run_token: token.into(),
        standard_endpoints: vec![r"\\.\pipe\openssh-ssh-agent".into()], owned_buses: BTreeSet::new() };
    let original = isolated_env(&root.join("side"), &app, &browser);
    guard.check(SpawnKind::Command, &original, &[], &BTreeSet::new()).unwrap();
    let outside = base().join("environment-decoy").display().to_string();
    let standard = r"\\.\pipe\openssh-ssh-agent";
    let mut bad_root = original.clone();
    bad_root.insert("LOCALAPPDATA".into(), outside.clone());
    assert!(guard.check(SpawnKind::Command, &bad_root, &[], &BTreeSet::new()).is_err());
    let mut bad_socket = original.clone();
    bad_socket.insert("SSH_AUTH_SOCK".into(), standard.into());
    assert!(guard.check(SpawnKind::Command, &bad_socket, &[], &BTreeSet::new()).is_err());
    let mut aliased = original;
    aliased.insert("localappdata".into(), outside.clone());
    aliased.insert("ssh_auth_sock".into(), standard.into());
    aliased.insert("MERE_AUDIT_EXPECTED_LOCAL".into(), outside);
    guard.check(SpawnKind::Command, &aliased, &[], &BTreeSet::new()).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .env_clear().envs(&aliased)
        .args(["--exact", "read_only_environment_child", "--ignored", "--nocapture"])
        .output().unwrap();
    assert!(output.status.success(), "{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(!Path::new(aliased.get("localappdata").unwrap()).exists());
    println!("OBSERVED canonical-names-refused=true case-aliases-admitted=true child-selected-outside-root-and-standard-agent=true no-endpoint-opened=true");
}

#[cfg(windows)]
#[test]
#[ignore = "only reads two environment variables in a child; never opens a root or endpoint"]
fn read_only_environment_child() {
    assert_eq!(std::env::var("LOCALAPPDATA").unwrap(), std::env::var("MERE_AUDIT_EXPECTED_LOCAL").unwrap());
    assert_eq!(std::env::var("SSH_AUTH_SOCK").unwrap(), r"\\.\pipe\openssh-ssh-agent");
}
