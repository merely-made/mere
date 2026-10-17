//! Exact config/core sources and unchanged production staging methods.
//! The Update facade cannot install anything; native pipeline is not linked.
#[path="C:/Users/mark_/Code/repos/mere/crates/intel/esp/src/infer/provider.rs"] mod portable;
mod infer { pub mod provider { pub use crate::portable::InferError; } }
#[path="C:/Users/mark_/Code/repos/mere/crates/intel/esp/src/infer/decoder/config.rs"] mod config;
#[path="C:/Users/mark_/Code/repos/mere/crates/system/luggage/src/error.rs"] mod error;
#[path="C:/Users/mark_/Code/repos/mere/crates/system/luggage/src/release.rs"] mod release;
pub use release::MANIFEST_NAME;
mod install {
    pub struct Update { pub version: String, pub format: crate::release::UpdateFormat,
        pub blake3: Option<String>, pub extract_path: std::path::PathBuf }
    impl Update {
        pub fn install(&self, _: Vec<u8>) -> crate::error::Result<()> {
            panic!("audit facade cannot invoke a real installer")
        }
    }
}
#[path="main_refresh_staging_production.rs"] mod staging;
#[path="main_refresh_comparison_projection.rs"] mod comparison;
fn directory(name: &str) -> std::path::PathBuf {
    let p=std::path::PathBuf::from(std::env::var_os("AUDIT_STAGING_DIRECTORY").unwrap()).join(name);
    assert!(!p.exists()); std::fs::create_dir(&p).unwrap(); p
}
fn record(path: &std::path::Path, bytes: &[u8]) {
    std::fs::write(path.join("staged.json"), serde_json::to_vec(&serde_json::json!({
        "version":"0.2.0", "format":"nsis", "blake3":blake3::hash(bytes).to_hex().to_string(),
        "extract_path":"audit-owned-unused-destination"
    })).unwrap()).unwrap();
}
#[test]
fn audit_staged_digest_replacement_after_reload() {
    let p=directory("staging-trust-reload"); let initial=b"initial audit payload";
    std::fs::write(p.join("staged-artifact"),initial).unwrap();record(&p,initial);
    assert_eq!(staging::StagedUpdate::load(&p).unwrap().unwrap().take_verified().unwrap(),initial);
    let replacement=b"unsigned substitute audit payload";
    std::fs::write(p.join("staged-artifact"),replacement).unwrap();
    assert!(staging::StagedUpdate::load(&p).unwrap().unwrap().take_verified().is_err());
    record(&p,replacement);
    assert_eq!(staging::StagedUpdate::load(&p).unwrap().unwrap().take_verified().unwrap(),replacement);
}
#[test]
fn audit_discard_returns_success_after_both_file_removals_fail() {
    let p=directory("staging-trust-discard"); let bytes=b"audit discard control";
    std::fs::write(p.join("staged-artifact"),bytes).unwrap();record(&p,bytes);
    let held=staging::StagedUpdate::load(&p).unwrap().unwrap();
    for name in ["staged-artifact","staged.json"] {
        let path=p.join(name);assert!(path.is_file());std::fs::remove_file(&path).unwrap();std::fs::create_dir(&path).unwrap();
    }
    assert!(held.discard().is_ok());
    assert!(p.join("staged-artifact").is_dir() && p.join("staged.json").is_dir());
}
#[test]
fn audit_reference_max_error_ignores_nan_mismatches() {
    let expected=vec![serde_json::json!(1.0),serde_json::json!(2.0)];
    assert_eq!(comparison::compare(&[1.0,2.0],&expected),0.0);
    assert!(comparison::compare(&[9.0,2.0],&expected)>2e-5);
    assert_eq!(comparison::compare(&[f32::NAN,2.0],&expected),0.0);
    assert_eq!(comparison::compare(&[f32::NAN,f32::NAN],&expected),0.0);
}
