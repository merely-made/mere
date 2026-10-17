// Namespace scaffolding only; both business/data modules are unchanged source.
extern crate self as gemot;
pub mod moot {
    #[path="C:/Users/mark_/Code/repos/mere/crates/moot/gemot/src/moot/id.rs"]
    mod id;
    pub use id::MootId;
}
#[path="C:/Users/mark_/Code/repos/mere/crates/moot/moothold/src/reciprocity.rs"]
mod reciprocity;
use moot::MootId;
use reciprocity::Reciprocity;
fn ids() -> (MootId,MootId) { (MootId([1;32]),MootId([2;32])) }
#[test]
fn ordinary_debt_control() {
    let (a,b)=ids();let mut r=Reciprocity::new();r.record(a,b,50);
    assert_eq!(r.balance(a,b),50);assert!(!r.may_request(b,a,10));
}
#[test]
fn max_unsigned_debt_is_treated_as_credit() {
    let (a,b)=ids();let mut r=Reciprocity::new();r.record(a,b,u64::MAX);
    assert_eq!(r.balance(a,b),-1);assert!(r.may_request(b,a,0));
}
#[test]
fn max_tolerance_refuses_even_zero_debt() {
    let (a,b)=ids();let r=Reciprocity::new();
    assert_eq!(r.balance(a,b),0);assert!(!r.may_request(b,a,u64::MAX));
}
