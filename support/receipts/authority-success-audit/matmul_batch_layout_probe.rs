#![allow(dead_code)]
struct TensorBinding{shape:Vec<usize>,strides:Vec<usize>}
struct MatmulProblem{out_batches:Vec<usize>}
struct BatchLayoutLaunch{batch_shape:Vec<u32>,batch_strides:Vec<usize>}
impl BatchLayoutLaunch{
 fn new(batch_shape:Vec<u32>,batch_strides:Vec<usize>)->Self{Self{batch_shape,batch_strides}}
pub fn from_handle(handle: &TensorBinding, problem: &MatmulProblem) -> Self {
        let rank = handle.shape.len();
        let batch_shape = problem
            .out_batches
            .iter()
            .map(|shape| *shape as u32)
            .collect();
        let batch_strides = handle.strides[..rank - 2]
            .iter()
            .zip(&handle.shape[..rank - 2])
            .map(|(stride, shape)| if *shape == 1 { 0 } else { *stride })
            .collect();
        BatchLayoutLaunch::new(batch_shape, batch_strides)
    }
}
#[derive(Clone,Copy)]struct FastDivmod(u32);impl FastDivmod{fn div_mod(&self,x:u32)->(u32,u32){(x/self.0,x%self.0)}}
struct Sequence<T>(Vec<T>);impl<T:Clone> Sequence<T>{fn reversed(&self)->Self{Self(self.0.iter().rev().cloned().collect())}fn len(&self)->usize{self.0.len()}}
impl<T>std::ops::Index<usize> for Sequence<T>{type Output=T;fn index(&self,i:usize)->&T{&self.0[i]}}
trait Layout{type Coordinates;type SourceCoordinates;fn to_source_pos(&self,pos:Self::Coordinates)->Self::SourceCoordinates;}
struct BatchLayout{batch_shape:Sequence<FastDivmod>,batch_strides:Sequence<usize>}
impl From<BatchLayoutLaunch> for BatchLayout{fn from(v:BatchLayoutLaunch)->Self{Self{batch_shape:Sequence(v.batch_shape.into_iter().map(FastDivmod).collect()),batch_strides:Sequence(v.batch_strides)}}}
impl Layout for BatchLayout{type Coordinates=usize;type SourceCoordinates=usize;
fn to_source_pos(&self, pos: Self::Coordinates) -> Self::SourceCoordinates {
        let mut batch = pos as u32;
        let mut batch_offs = 0;
        let batch_shape = self.batch_shape.reversed();
        let batch_strides = self.batch_strides.reversed();

        
        for i in 0..batch_shape.len() {
            let (rem, local_pos) = batch_shape[i].div_mod(batch);
            batch = rem;
            batch_offs += local_pos as usize * batch_strides[i];
        }

        batch_offs
    }
}
fn payload(shape:Vec<usize>,strides:Vec<usize>,out_batches:Vec<usize>)->BatchLayoutLaunch{BatchLayoutLaunch::from_handle(&TensorBinding{shape,strides},&MatmulProblem{out_batches})}
#[test]fn unbatched_operand_against_batched_output_has_no_stride_for_its_batch_axis(){let p=payload(vec![2,3],vec![3,1],vec![4]);assert_eq!(p.batch_shape,vec![4]);assert!(p.batch_strides.is_empty());let l=BatchLayout::from(p);assert!(std::panic::catch_unwind(||l.to_source_pos(0)).is_err());}
#[test]fn shorter_batch_rank_has_fewer_strides_than_output_divisors(){let p=payload(vec![3,2,3],vec![6,3,1],vec![2,3]);assert_eq!(p.batch_shape,vec![2,3]);assert_eq!(p.batch_strides,vec![6]);let l=BatchLayout::from(p);assert!(std::panic::catch_unwind(||l.to_source_pos(0)).is_err());}
#[test]fn same_rank_nonbroadcast_control_maps_offsets(){let l=BatchLayout::from(payload(vec![4,2,3],vec![6,3,1],vec![4]));assert_eq!((0..4).map(|i|l.to_source_pos(i)).collect::<Vec<_>>(),vec![0,6,12,18]);}
#[test]fn same_rank_singleton_control_maps_zero(){let l=BatchLayout::from(payload(vec![1,2,3],vec![6,3,1],vec![4]));assert_eq!((0..4).map(|i|l.to_source_pos(i)).collect::<Vec<_>>(),vec![0,0,0,0]);}
#[test]fn unbatched_output_control_has_no_batch_axis_to_index(){let l=BatchLayout::from(payload(vec![2,3],vec![3,1],vec![]));assert_eq!(l.to_source_pos(0),0);}
#[test]fn same_rank_two_axis_broadcast_control(){let l=BatchLayout::from(payload(vec![2,1,2,3],vec![6,6,3,1],vec![2,3]));assert_eq!((0..6).map(|i|l.to_source_pos(i)).collect::<Vec<_>>(),vec![0,0,0,6,6,6]);}
