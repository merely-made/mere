#![allow(dead_code,private_interfaces)]
use std::{cmp::max,ops::Deref};
type Shape=Vec<usize>;
#[derive(Debug,Clone,PartialEq)]struct Strides(Vec<usize>);impl Strides{fn new(s:&[usize])->Self{Self(s.to_vec())}}impl From<Vec<usize>>for Strides{fn from(v:Vec<usize>)->Self{Self(v)}}impl Deref for Strides{type Target=[usize];fn deref(&self)->&[usize]{&self.0}}
enum StridedLayout{RowMajor,ColMajor,Explicit(Vec<usize>)}
impl StridedLayout{
pub fn compute_strides(&self, shape: &Shape) -> Strides {
        let n = shape.len();
        match self {
            StridedLayout::RowMajor => {
                assert!(n >= 1, "RowMajor requires at least 1 dimension");
                let mut strides = vec![0; n];
                strides[n - 1] = 1;
                for i in (0..n - 1).rev() {
                    strides[i] = strides[i + 1] * shape[i + 1];
                }
                Strides::new(&strides)
            }
            StridedLayout::ColMajor => {
                assert!(n >= 2, "ColMajor requires at least 2 dimensions");
                let mut strides = vec![0; n];
                strides[n - 2] = 1;
                strides[n - 1] = shape[n - 2];
                for i in (0..n - 2).rev() {
                    strides[i] = strides[i + 1] * shape[i + 1];
                }
                Strides::new(&strides)
            }
            StridedLayout::Explicit(strides) => {
                assert!(
                    strides.len() == n,
                    "Explicit strides must have the same rank as the shape"
                );
                strides.clone().into()
            }
        }
    }
}
pub fn broadcast_batches(lhs: &[usize], rhs: &[usize]) -> Option<Shape> {
    let max_len = max(lhs.len(), rhs.len());
    let lhs_padded = std::iter::repeat_n(1, max_len - lhs.len()).chain(lhs.iter().cloned());
    let rhs_padded = std::iter::repeat_n(1, max_len - rhs.len()).chain(rhs.iter().cloned());

    lhs_padded
        .zip(rhs_padded)
        .map(|(l, r)| {
            if l != r && l != 1 && r != 1 {
                None
            } else {
                Some(max(l, r))
            }
        })
        .collect()
}
#[derive(Debug)]struct HostDataVec(Vec<f64>);impl HostDataVec{fn get_f64(&self,i:usize)->f64{self.0[i]}}impl From<(Vec<f64>,u8)>for HostDataVec{fn from(v:(Vec<f64>,u8))->Self{Self(v.0)}}
#[derive(Debug)]struct HostData{data:HostDataVec,shape:Shape,strides:Strides}
impl HostData{
pub fn get_f64(&self, index: &[usize]) -> f64 {
        self.data.get_f64(self.strided_index(index))
    }
fn strided_index(&self, index: &[usize]) -> usize {
        let mut i = 0usize;
        for (d, idx) in index.iter().enumerate() {
            i += idx * self.strides[d];
        }
        i
    }
}
struct Progress;impl Progress{fn set_total(&self,_v:u64){}fn bump(&self){}}
struct GlobalDtypes{out:u8}
struct MatmulProblem{m:usize,n:usize,k:usize,lhs_batches:Shape,rhs_batches:Shape,out_batches:Shape,out_shape:Shape,global_dtypes:GlobalDtypes}
impl MatmulProblem{
pub fn num_batches(&self) -> usize {
        self.out_batches.iter().product()
    }
}
pub fn matmul_cpu_reference(
    lhs: &HostData,
    rhs: &HostData,
    problem: &MatmulProblem,
    progress: Option<&Progress>,
) -> HostData {
    let m = problem.m;
    let n = problem.n;
    let k = problem.k;

    let out_shape = problem.out_shape.clone();
    let rank = out_shape.len();
    let num_batches = problem.num_batches();

    if let Some(p) = progress {
        p.set_total((num_batches * m * n) as u64);
    }

    let mut out = vec![0.0f64; num_batches * m * n];

    let mut batch_index = vec![0usize; rank - 2];
    let mut lhs_index = vec![0usize; rank];
    let mut rhs_index = vec![0usize; rank];
    let mut out_index = vec![0usize; rank];

    let lhs_batches = &problem.lhs_batches;
    let rhs_batches = &problem.rhs_batches;
    let out_batches = &problem.out_batches;

    for batch_flat in 0..num_batches {
        let mut t = batch_flat;
        for d in (0..rank - 2).rev() {
            batch_index[d] = t % out_batches[d];
            t /= out_batches[d];
        }

        for d in 0..rank - 2 {
            lhs_index[d] = if d < lhs_batches.len() && lhs_batches[d] != 1 {
                batch_index[d]
            } else {
                0
            };
            rhs_index[d] = if d < rhs_batches.len() && rhs_batches[d] != 1 {
                batch_index[d]
            } else {
                0
            };
            out_index[d] = batch_index[d];
        }

        for i in 0..m {
            lhs_index[rank - 2] = i;
            out_index[rank - 2] = i;

            for j in 0..n {
                rhs_index[rank - 1] = j;
                out_index[rank - 1] = j;

                let mut sum = 0.0f64;
                for kk in 0..k {
                    lhs_index[rank - 1] = kk;
                    rhs_index[rank - 2] = kk;

                    sum += lhs.get_f64(&lhs_index) * rhs.get_f64(&rhs_index);
                }

                let out_linear = batch_flat * (m * n) + i * n + j;
                out[out_linear] = sum;
                if let Some(p) = progress {
                    p.bump();
                }
            }
        }
    }

    let strides = StridedLayout::RowMajor.compute_strides(&out_shape);

    let data = HostDataVec::from((out, problem.global_dtypes.out));

    HostData {
        data,
        shape: out_shape,
        strides,
    }
}
#[derive(Clone,Copy)]enum MatrixLayout{RowMajor,ColMajor}
struct Metadata{shape:Vec<usize>,strides:Strides}
impl Metadata{fn rank(&self)->usize{self.shape.len()}fn strides_mut(&mut self)->&mut[usize]{&mut self.strides.0}}
struct TensorHandle{metadata:Metadata}
type Client=();
struct Device;impl Device{fn client(&self)->Client{()}}
fn make_uniform(_c:&Client,shape:[usize;3],_d:u8,_s:u64,_lo:f32,_hi:f32)->TensorHandle{let shape=shape.to_vec();let strides=StridedLayout::RowMajor.compute_strides(&shape);TensorHandle{metadata:Metadata{shape,strides}}}
struct Dtypes{lhs_global:u8,rhs_global:u8}
struct GemmBench{b:usize,m:usize,n:usize,k:usize,lhs_layout:MatrixLayout,rhs_layout:MatrixLayout,device:Device,dtypes:Dtypes}
trait Benchmark{type Input;fn prepare(&self)->Self::Input;}
impl Benchmark for GemmBench{type Input=(TensorHandle,TensorHandle);
fn prepare(&self) -> Self::Input {
        let client = self.device.client();
        let tl = matches!(self.lhs_layout, MatrixLayout::ColMajor);
        let tr = matches!(self.rhs_layout, MatrixLayout::ColMajor);

        let mut lhs = make_uniform(
            &client,
            [self.b, self.m, self.k],
            self.dtypes.lhs_global,
            0,
            0.0,
            1.0,
        );
        if tl {
            let len = lhs.metadata.rank();
            lhs.metadata.strides_mut().swap(len - 2, len - 1);
        }

        let mut rhs = make_uniform(
            &client,
            [self.b, self.k, self.n],
            self.dtypes.rhs_global,
            1,
            0.0,
            1.1,
        );
        if tr {
            let len = rhs.metadata.rank();
            rhs.metadata.strides_mut().swap(len - 2, len - 1);
        }

        (lhs, rhs)
    }
}
fn host(shape:Vec<usize>,v:Vec<f64>)->HostData{let strides=StridedLayout::RowMajor.compute_strides(&shape);HostData{data:HostDataVec(v),shape,strides}}
fn problem(lhs:Shape,rhs:Shape,out:Shape)->MatmulProblem{let out_shape=out.iter().copied().chain([1,1]).collect();MatmulProblem{m:1,n:1,k:2,lhs_batches:lhs,rhs_batches:rhs,out_batches:out,out_shape,global_dtypes:GlobalDtypes{out:0}}}
fn bench(m:usize,k:usize,n:usize,lhs_layout:MatrixLayout,rhs_layout:MatrixLayout)->GemmBench{GemmBench{b:1,m,k,n,lhs_layout,rhs_layout,device:Device,dtypes:Dtypes{lhs_global:0,rhs_global:0}}}
fn span(t:&TensorHandle)->usize{t.metadata.shape.iter().zip(t.metadata.strides.iter()).map(|(d,s)|(d-1)*s).sum::<usize>()+1}
#[test]fn zero_and_singleton_broadcast_loses_empty_dimension(){assert_eq!(broadcast_batches(&[0],&[1]),Some(vec![1]));assert_eq!(broadcast_batches(&[1],&[0]),Some(vec![1]));}
#[test]fn implicit_leading_singleton_also_loses_empty_dimension(){assert_eq!(broadcast_batches(&[],&[0]),Some(vec![1]));}
#[test]fn ordinary_broadcast_control(){assert_eq!(broadcast_batches(&[2,1],&[3]),Some(vec![2,3]));}
#[test]fn conflicting_broadcast_control(){assert_eq!(broadcast_batches(&[0],&[2]),None);}
#[test]fn equal_zero_broadcast_control(){assert_eq!(broadcast_batches(&[0],&[0]),Some(vec![0]));}
#[test]fn rectangular_lhs_column_stride_view_exceeds_its_allocation(){let(l,_)=bench(2,3,2,MatrixLayout::ColMajor,MatrixLayout::RowMajor).prepare();assert_eq!(l.metadata.shape,vec![1,2,3]);assert_eq!(l.metadata.strides.0,vec![6,1,3]);assert_eq!(span(&l),8);assert_eq!(l.metadata.shape.iter().product::<usize>(),6);}
#[test]fn rectangular_rhs_column_stride_view_exceeds_its_allocation(){let(_,r)=bench(2,2,3,MatrixLayout::RowMajor,MatrixLayout::ColMajor).prepare();assert_eq!(r.metadata.shape,vec![1,2,3]);assert_eq!(r.metadata.strides.0,vec![6,1,3]);assert_eq!(span(&r),8);}
#[test]fn tall_column_stride_view_aliases_distinct_logical_indices(){let(l,_)=bench(3,2,2,MatrixLayout::ColMajor,MatrixLayout::RowMajor).prepare();let s=&l.metadata.strides;assert_eq!(2*s[1],s[2]);assert_eq!(l.metadata.shape,vec![1,3,2]);}
#[test]fn square_column_stride_control(){let(l,r)=bench(3,3,3,MatrixLayout::ColMajor,MatrixLayout::ColMajor).prepare();assert_eq!(span(&l),9);assert_eq!(span(&r),9);assert_eq!(l.metadata.strides.0,vec![9,1,3]);}
#[test]fn row_layout_control(){let(l,r)=bench(2,3,4,MatrixLayout::RowMajor,MatrixLayout::RowMajor).prepare();assert_eq!(span(&l),6);assert_eq!(span(&r),12);}
#[test]fn cpu_reference_same_rank_control(){let p=problem(vec![],vec![],vec![]);let lhs=host(vec![1,2],vec![2.,3.]);let rhs=host(vec![2,1],vec![4.,5.]);assert_eq!(matmul_cpu_reference(&lhs,&rhs,&p,None).data.0,vec![23.]);}
#[test]fn cpu_reference_same_rank_broadcast_control(){let p=problem(vec![1],vec![2],vec![2]);let lhs=host(vec![1,1,2],vec![2.,3.]);let rhs=host(vec![2,2,1],vec![4.,5.,6.,7.]);assert_eq!(matmul_cpu_reference(&lhs,&rhs,&p,None).data.0,vec![23.,33.]);}
#[test]fn cpu_reference_valid_rank_mismatch_panics_in_host_stride_lookup(){let p=problem(vec![],vec![2],vec![2]);let lhs=host(vec![1,2],vec![2.,3.]);let rhs=host(vec![2,2,1],vec![4.,5.,6.,7.]);assert!(std::panic::catch_unwind(||matmul_cpu_reference(&lhs,&rhs,&p,None)).is_err());}
