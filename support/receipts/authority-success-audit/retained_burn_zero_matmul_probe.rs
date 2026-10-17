#![allow(dead_code,private_interfaces)]
type DType=();
#[derive(Clone)] struct Meta(Vec<usize>);
impl Meta { fn shape(&self)->&[usize]{&self.0} }
#[derive(Clone)] struct CubeTensor {meta:Meta, values:Vec<i32>, dispatched:bool}
fn t(shape:&[usize],fill:i32)->CubeTensor{CubeTensor{meta:Meta(shape.to_vec()),values:vec![fill;shape.iter().product()],dispatched:false}}
// Allocation double deliberately exposes the fact that the extracted prefix never fills it.
fn init_matmul_output(lhs:&CubeTensor,rhs:&CubeTensor,_dtype:DType)->CubeTensor{t(&[lhs.meta.shape()[0],rhs.meta.shape()[1]],23)}
pub fn matmul_autotune(
    lhs: CubeTensor,
    rhs: CubeTensor,
    out: Option<CubeTensor>,
    out_dtype: DType,
) -> CubeTensor {
    let output = out.unwrap_or_else(|| init_matmul_output(&lhs, &rhs, out_dtype));

    // Short-circuit: zero-sized matmul produces a zero-sized output.
    if lhs.meta.shape().iter().any(|&d| d == 0) || rhs.meta.shape().iter().any(|&d| d == 0) {
        return output;
    }

    CubeTensor { dispatched:true, ..output }
}
#[cfg(test)] mod cases {
 use super::*;
 #[test] fn zero_inner_dimension_leaves_supplied_nonempty_output_unchanged(){
  let result=matmul_autotune(t(&[2,0],0),t(&[0,3],0),Some(t(&[2,3],17)),());
  let empty_dot:i32=std::iter::empty::<i32>().sum();
  assert_eq!(empty_dot,0);assert_eq!(result.values,vec![17;6]);assert!(!result.dispatched);
  assert_ne!(result.values,vec![empty_dot;6]);
 }
 #[test] fn zero_inner_dimension_returns_allocator_double_without_initializing(){
  let result=matmul_autotune(t(&[2,0],0),t(&[0,3],0),None,());
  assert_eq!(result.values,vec![23;6]);assert!(!result.dispatched);
 }
 #[test] fn zero_rows_control_has_empty_output(){
  let result=matmul_autotune(t(&[0,2],0),t(&[2,3],0),Some(t(&[0,3],17)),());
  assert!(result.values.is_empty());assert!(!result.dispatched);
 }
 #[test] fn zero_columns_control_has_empty_output(){
  let result=matmul_autotune(t(&[2,3],0),t(&[3,0],0),Some(t(&[2,0],17)),());
  assert!(result.values.is_empty());assert!(!result.dispatched);
 }
 #[test] fn nonzero_dimensions_reach_dispatch_marker(){
  let result=matmul_autotune(t(&[2,1],0),t(&[1,3],0),Some(t(&[2,3],17)),());
  assert!(result.dispatched);
 }
}
