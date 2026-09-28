//! Direct launcher regressions: no fusion backend can replace these operations.
use super::{binary, binary_float, binary_int};
use crate::{ops::base, tensor::CubeTensor};
use burn_backend::{Shape, TensorData, TensorMetadata};
use cubecl::wgpu::{WgpuDevice, WgpuRuntime};

#[derive(Clone, Copy, Debug)]
enum Case {
    BroadcastAlias,
    SameShapeAlias,
    Separate,
}

fn views(
    input: &CubeTensor<WgpuRuntime>,
    other: &CubeTensor<WgpuRuntime>,
    case: Case,
) -> (CubeTensor<WgpuRuntime>, CubeTensor<WgpuRuntime>, Vec<usize>) {
    let same_shape = matches!(case, Case::SameShapeAlias);
    let lhs = base::reshape(input.clone(), Shape::new([3, 1]));
    let rhs = base::reshape(
        if matches!(case, Case::Separate) {
            other.clone()
        } else {
            input.clone()
        },
        Shape::new(if same_shape { [3, 1] } else { [1, 3] }),
    );
    assert_eq!(
        lhs.handle.is_same_allocation(&rhs.handle),
        !matches!(case, Case::Separate),
        "fixture must exercise the intended real handle identity"
    );
    (lhs, rhs, if same_shape { vec![3, 1] } else { vec![3, 3] })
}

fn assert_fresh(
    output: &CubeTensor<WgpuRuntime>,
    input: &CubeTensor<WgpuRuntime>,
    other: &CubeTensor<WgpuRuntime>,
    shape: &[usize],
) {
    assert_eq!(output.shape(), Shape::from(shape));
    assert!(
        !output
            .handle
            .memory
            .is_same_allocation(&input.handle.memory)
    );
    assert!(
        !output
            .handle
            .memory
            .is_same_allocation(&other.handle.memory)
    );
}

fn float_case(case: Case, atan2: bool) {
    let device = WgpuDevice::DiscreteGpu(0);
    let values = [-2.0_f32, 1.0, 5.0];
    let others = [3.0_f32, -4.0, 2.0];
    let input = base::from_data::<WgpuRuntime>(TensorData::new(values.to_vec(), [3]), &device);
    let other = base::from_data::<WgpuRuntime>(TensorData::new(others.to_vec(), [3]), &device);
    let (lhs, rhs, shape) = views(&input, &other, case);
    let output = if atan2 {
        binary_float::launch_binop_float::<WgpuRuntime, binary_float::ArcTan2Op>(lhs, rhs)
    } else {
        binary::launch_binop::<WgpuRuntime, binary::SubOp>(lhs, rhs)
    };
    assert_fresh(&output, &input, &other, &shape);
    let actual = base::into_data_sync(output).to_vec::<f32>().unwrap();
    let right = if matches!(case, Case::Separate) {
        others
    } else {
        values
    };
    let mut expected = Vec::new();
    for row in 0..3 {
        for col in 0..shape[1] {
            let r = right[if shape[1] == 1 { row } else { col }];
            expected.push(if atan2 {
                values[row].atan2(r)
            } else {
                values[row] - r
            });
        }
    }
    assert_eq!(actual.len(), expected.len());
    assert!(actual.iter().chain(&expected).all(|v| v.is_finite()));
    println!("{case:?} atan2={atan2}: expected={expected:?} actual={actual:?}");
    for (index, (a, e)) in actual.iter().zip(&expected).enumerate() {
        if atan2 {
            assert!((a - e).abs() < 1.0e-6, "index {index}: {a} != {e}");
        } else {
            assert_eq!(a, e, "index {index}");
        }
    }
    assert_eq!(base::into_data_sync(input).to_vec::<f32>().unwrap(), values);
    assert_eq!(base::into_data_sync(other).to_vec::<f32>().unwrap(), others);
}

fn integer_case(case: Case) {
    let device = WgpuDevice::DiscreteGpu(0);
    let values = [2_i32, 7, 13];
    let others = [3_i32, 6, 10];
    let input = base::from_data::<WgpuRuntime>(TensorData::new(values.to_vec(), [3]), &device);
    let other = base::from_data::<WgpuRuntime>(TensorData::new(others.to_vec(), [3]), &device);
    let (lhs, rhs, shape) = views(&input, &other, case);
    let output = binary_int::launch_binop_int::<WgpuRuntime, binary_int::BitwiseXorOp>(lhs, rhs);
    assert_fresh(&output, &input, &other, &shape);
    let actual = base::into_data_sync(output).to_vec::<i32>().unwrap();
    let right = if matches!(case, Case::Separate) {
        others
    } else {
        values
    };
    let mut expected = Vec::new();
    for row in 0..3 {
        for col in 0..shape[1] {
            expected.push(values[row] ^ right[if shape[1] == 1 { row } else { col }]);
        }
    }
    assert_eq!(actual.len(), expected.len());
    println!("{case:?} xor: expected={expected:?} actual={actual:?}");
    assert_eq!(actual, expected);
    assert_eq!(base::into_data_sync(input).to_vec::<i32>().unwrap(), values);
    assert_eq!(base::into_data_sync(other).to_vec::<i32>().unwrap(), others);
}

macro_rules! gpu_case {
    ($name:ident, $body:expr) => {
        #[test]
        #[ignore = "requires a discrete WGPU device; run explicitly with --ignored"]
        fn $name() {
            $body
        }
    };
}

gpu_case!(sub_broadcast_alias, float_case(Case::BroadcastAlias, false));
gpu_case!(
    sub_same_shape_alias,
    float_case(Case::SameShapeAlias, false)
);
gpu_case!(sub_separate, float_case(Case::Separate, false));
gpu_case!(
    atan2_broadcast_alias,
    float_case(Case::BroadcastAlias, true)
);
gpu_case!(
    atan2_same_shape_alias,
    float_case(Case::SameShapeAlias, true)
);
gpu_case!(atan2_separate, float_case(Case::Separate, true));
gpu_case!(xor_broadcast_alias, integer_case(Case::BroadcastAlias));
gpu_case!(xor_same_shape_alias, integer_case(Case::SameShapeAlias));
gpu_case!(xor_separate, integer_case(Case::Separate));
