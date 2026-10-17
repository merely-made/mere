#![allow(dead_code)]
struct InputBinding(Vec<usize>);impl InputBinding{fn shape(&self)->&[usize]{&self.0}}
#[derive(Clone,Copy)]struct Scheme{m:u32,n:u32,k:u32}
impl Scheme{fn elements_per_stage_along_m(&self)->u32{self.m}fn elements_per_stage_along_n(&self)->u32{self.n}fn elements_per_stage_along_k(&self)->u32{self.k}}
struct Builder{m:u32,n:u32,k:u32,stage_buffering:u32,tiling_scheme:Scheme}
struct Blueprint{check_k_bounds:bool,tiling_scheme:Scheme}
#[derive(Debug)]enum MatmulSetupError{InvalidConfig(Box<String>)}
struct Problem{k:usize}
struct GMM;struct Stages;
impl GMM{fn num_stages()->Stages{Stages}}
impl Stages{fn stage_buffering(&self)->u32{2}}
fn gemm(lhs:InputBinding,rhs:InputBinding)->(usize,usize,usize){
    let rank = rhs.shape().len();
    let lhs_shape = lhs.shape();
    let rhs_shape = rhs.shape();

    let m = lhs_shape.to_vec()[rank - 2];
    let n = rhs_shape.to_vec()[rank - 1];
    let k = lhs_shape.to_vec()[rank - 1];

(m,n,k)}
fn gemv(lhs:InputBinding,rhs:InputBinding)->(usize,usize,usize){
    let rank = rhs.shape().len();
    let m = lhs.shape().to_vec()[rank - 2];
    let n = rhs.shape().to_vec()[rank - 1];
    let k = lhs.shape().to_vec()[rank - 1];

(m,n,k)}
impl Builder{fn bounds(self)->(bool,bool,bool){
        let k_group = self.stage_buffering;

        let check_m_bounds = !self
            .m
            .is_multiple_of(self.tiling_scheme.elements_per_stage_along_m());
        let check_n_bounds = !self
            .n
            .is_multiple_of(self.tiling_scheme.elements_per_stage_along_n());
        let check_k_bounds = !self
            .k
            .is_multiple_of(self.tiling_scheme.elements_per_stage_along_k() * k_group);

(check_m_bounds,check_n_bounds,check_k_bounds)}}
fn group_admission(blueprint:&Blueprint,problem:&Problem)->Result<(),MatmulSetupError>{
        let k_group = blueprint.tiling_scheme.elements_per_stage_along_k()
            * GMM::num_stages().stage_buffering();
        if !blueprint.check_k_bounds && !(problem.k as u32).is_multiple_of(k_group) {
            return Err(MatmulSetupError::InvalidConfig(Box::new(format!(
                "k-bounds checks are disabled but k={} is not a multiple of the k-loop group {k_group}",
                problem.k
            ))));
        }

Ok(())}

fn operands(a:&[usize],b:&[usize])->(InputBinding,InputBinding){(InputBinding(a.to_vec()),InputBinding(b.to_vec()))}
type LaunchShape=fn(InputBinding,InputBinding)->(usize,usize,usize);
fn short_lhs_panics(f:LaunchShape){let(a,b)=operands(&[1,32],&[2,32,64]);assert!(std::panic::catch_unwind(||f(a,b)).is_err());}
fn long_lhs_misreads(f:LaunchShape){let(a,b)=operands(&[2,1,32],&[32,64]);let got=f(a,b);assert_eq!(got,(2,64,1));assert_ne!(got,(1,64,32));}
fn equal_rank_control(f:LaunchShape){let(a,b)=operands(&[2,1,32],&[2,32,64]);assert_eq!(f(a,b),(1,64,32));}
#[test]fn gemm_rank2_lhs_rank3_rhs_panics(){short_lhs_panics(gemm)}
#[test]fn gemv_rank2_lhs_rank3_rhs_panics(){short_lhs_panics(gemv)}
#[test]fn gemm_rank3_lhs_rank2_rhs_misreads_dimensions(){long_lhs_misreads(gemm)}
#[test]fn gemv_rank3_lhs_rank2_rhs_misreads_dimensions(){long_lhs_misreads(gemv)}
#[test]fn gemm_equal_rank_control(){equal_rank_control(gemm)}
#[test]fn gemv_equal_rank_control(){equal_rank_control(gemv)}
fn inferred(k:u32,buffering:u32)->Blueprint{let scheme=Scheme{m:1,n:1,k:32};let(_,_,check_k_bounds)=Builder{m:1,n:64,k,stage_buffering:buffering,tiling_scheme:scheme}.bounds();Blueprint{check_k_bounds,tiling_scheme:scheme}}
#[test]fn double_vecmat_single_stage_bounds_refuse_odd_stage_count(){let bp=inferred(96,1);assert!(!bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:96}).is_err());}
#[test]fn double_vecmat_even_stage_control(){let bp=inferred(128,1);assert!(!bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:128}).is_ok());}
#[test]fn explicit_double_stage_bounds_admit_tail_control(){let bp=inferred(96,2);assert!(bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:96}).is_ok());}
#[test]fn single_stage_nonmultiple_gets_bounds_control(){let bp=inferred(97,1);assert!(bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:97}).is_ok());}
