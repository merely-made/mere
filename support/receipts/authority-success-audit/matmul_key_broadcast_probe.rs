#![allow(dead_code)]
use std::{cell::Cell,rc::Rc};
#[derive(Clone,Debug)]enum ValidationResult{Pass,Fail,Skipped,Error}
#[derive(Clone,Debug)]enum TestOutcome{Validated(ValidationResult),CompileError}
#[derive(Clone,Debug)]struct Strategy{baseline:TestOutcome,broadcast:TestOutcome,calls:Rc<Cell<usize>>}
impl std::fmt::Display for Strategy{fn fmt(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{write!(f,"stub strategy")}}
enum MatrixLayout{ColMajor,RowMajor}
#[derive(Clone,Copy)]enum Broadcast{BatchLhs,BatchRhs}
impl Broadcast{fn label(self)->&'static str{"model axis"}}
mod cubecl{pub fn test_device()->Device{Device}pub struct Device;impl Device{pub fn client(self){}}}
fn make_problem(_:Broadcast,zero_stride:bool,_:MatrixLayout,_:MatrixLayout)->bool{zero_stride}
fn run_with_strides(_:(),broadcast:bool,strategy:Strategy)->TestOutcome{if broadcast{strategy.calls.set(strategy.calls.get()+1);strategy.broadcast}else{strategy.baseline}}
fn dimensions(lhs_logical:&[usize],rhs_logical:&[usize])->(usize,usize,usize){
        let ndims = lhs_logical.len();
        let m = lhs_logical[ndims - 2];
        let k = lhs_logical[ndims - 1];
        let n = rhs_logical[ndims - 1];
(m,n,k)}
pub(crate) fn passed(outcome: &TestOutcome) -> bool {
    matches!(outcome, TestOutcome::Validated(ValidationResult::Pass))
}

pub(crate) fn assert_batch_broadcast(strategy: Strategy) {
    use MatrixLayout::{ColMajor as C, RowMajor as R};
    let client = cubecl::test_device().client();
    for broadcast in [Broadcast::BatchLhs, Broadcast::BatchRhs] {
        let baseline = run_with_strides(
            client.clone(),
            make_problem(broadcast, false, R, C),
            strategy.clone(),
        );
        if !passed(&baseline) {
            continue;
        }
        let out = run_with_strides(
            client.clone(),
            make_problem(broadcast, true, R, C),
            strategy.clone(),
        );
        assert!(
            passed(&out),
            "{strategy}: {} batch broadcast gave {out:?}",
            broadcast.label()
        );
    }
}

#[test]fn key_lhs_rank3_rhs_rank2_panics(){assert!(std::panic::catch_unwind(||dimensions(&[2,1,32],&[32,64])).is_err());}
#[test]fn key_lhs_rank2_rhs_rank3_misreads_n(){assert_eq!(dimensions(&[1,32],&[2,32,64]),(1,32,32));assert_ne!(dimensions(&[1,32],&[2,32,64]),(1,64,32));}
#[test]fn equal_rank2_key_control(){assert_eq!(dimensions(&[1,32],&[32,64]),(1,64,32));}
#[test]fn equal_rank3_key_control(){assert_eq!(dimensions(&[2,1,32],&[2,32,64]),(1,64,32));}
fn strategy(baseline:TestOutcome,broadcast:TestOutcome)->(Strategy,Rc<Cell<usize>>){let calls=Rc::new(Cell::new(0));(Strategy{baseline,broadcast,calls:calls.clone()},calls)}
#[test]fn wrong_numeric_baseline_silently_skips_broadcast(){let(s,c)=strategy(TestOutcome::Validated(ValidationResult::Fail),TestOutcome::Validated(ValidationResult::Fail));assert_batch_broadcast(s);assert_eq!(c.get(),0);}
#[test]fn baseline_compile_error_also_skips(){let(s,c)=strategy(TestOutcome::CompileError,TestOutcome::Validated(ValidationResult::Fail));assert_batch_broadcast(s);assert_eq!(c.get(),0);}
#[test]fn passing_controls_execute_both_broadcast_axes(){let(s,c)=strategy(TestOutcome::Validated(ValidationResult::Pass),TestOutcome::Validated(ValidationResult::Pass));assert_batch_broadcast(s);assert_eq!(c.get(),2);}
#[test]fn passing_baseline_wrong_broadcast_still_panics(){let(s,c)=strategy(TestOutcome::Validated(ValidationResult::Pass),TestOutcome::Validated(ValidationResult::Fail));assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||assert_batch_broadcast(s))).is_err());assert_eq!(c.get(),1);}
