// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Numerical retirement controls for the three launchers considered for patch retirement.
//!
//! CubeBackend is explicitly unfused; public operation wrappers dispatch
//! directly to the same numeric, float and integer launchers as the original
//! in-crate regression tests. These tests also work with the registry crate.
#![cfg(not(target_arch = "wasm32"))]

use burn_backend::{
    Shape, TensorData, TensorMetadata,
    ops::{FloatTensorOps, IntTensorOps},
};
use burn_cubecl::cubecl::{
    server::Handle,
    wgpu::{WgpuDevice, WgpuDeviceKind},
};
use burn_cubecl::{CubeBackend, CubeDevice, ops, tensor::CubeTensor};

// This predicate verifies fixture construction only. It does not claim that
// upstream performs the retired patch's six-field guard. All handles below
// come from real GPU allocations; no synthetic handle is submitted.
fn same_view(a: &Handle, b: &Handle) -> bool {
    a.memory.descriptor().id == b.memory.descriptor().id
        && a.offset_start == b.offset_start
        && a.offset_end == b.offset_end
        && a.stream == b.stream
        && a.size() == b.size()
        && a.service == b.service
}

#[derive(Clone, Copy, Debug)]
enum Case {
    BroadcastAlias,
    SameShapeAlias,
    Separate,
}

fn views(
    input: &CubeTensor,
    other: &CubeTensor,
    case: Case,
) -> (CubeTensor, CubeTensor, Vec<usize>) {
    let same_shape = matches!(case, Case::SameShapeAlias);
    let lhs = ops::reshape(input.clone(), Shape::new([3, 1]));
    let rhs = ops::reshape(
        if matches!(case, Case::Separate) {
            other.clone()
        } else {
            input.clone()
        },
        Shape::new(if same_shape { [3, 1] } else { [1, 3] }),
    );
    assert_eq!(
        same_view(&lhs.handle, &rhs.handle),
        !matches!(case, Case::Separate),
        "fixture must exercise the intended real handle identity"
    );
    (lhs, rhs, if same_shape { vec![3, 1] } else { vec![3, 3] })
}

fn assert_fresh(output: &CubeTensor, input: &CubeTensor, other: &CubeTensor, shape: &[usize]) {
    assert_eq!(output.shape(), Shape::from(shape));
    assert_ne!(
        output.handle.memory.descriptor().id,
        input.handle.memory.descriptor().id
    );
    assert_ne!(
        output.handle.memory.descriptor().id,
        other.handle.memory.descriptor().id
    );
}

fn float_case(case: Case, atan2: bool) {
    let device = CubeDevice::Wgpu(WgpuDevice::new(WgpuDeviceKind::DiscreteGpu(0)));
    let values = [-2.0_f32, 1.0, 5.0];
    let others = [3.0_f32, -4.0, 2.0];
    let input = <CubeBackend as FloatTensorOps<CubeBackend>>::float_from_data(
        TensorData::new(values.to_vec(), [3]),
        &device,
    );
    let other = <CubeBackend as FloatTensorOps<CubeBackend>>::float_from_data(
        TensorData::new(others.to_vec(), [3]),
        &device,
    );
    // Keep both uploaded tensors alive across the operation: the result must
    // be fresh and neither retained allocation may be changed in place.
    let (lhs, rhs, shape) = views(&input, &other, case);
    let output = if atan2 {
        <CubeBackend as FloatTensorOps<CubeBackend>>::float_atan2(lhs, rhs)
    } else {
        ops::numeric::sub(lhs, rhs)
    };
    assert_fresh(&output, &input, &other, &shape);
    let actual = ops::into_data_sync(output).to_vec::<f32>().unwrap();
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
    assert_eq!(ops::into_data_sync(input).to_vec::<f32>().unwrap(), values);
    assert_eq!(ops::into_data_sync(other).to_vec::<f32>().unwrap(), others);
}

fn integer_case(case: Case) {
    let device = CubeDevice::Wgpu(WgpuDevice::new(WgpuDeviceKind::DiscreteGpu(0)));
    let values = [2_i32, 7, 13];
    let others = [3_i32, 6, 10];
    let input = <CubeBackend as IntTensorOps<CubeBackend>>::int_from_data(
        TensorData::new(values.to_vec(), [3]),
        &device,
    );
    let other = <CubeBackend as IntTensorOps<CubeBackend>>::int_from_data(
        TensorData::new(others.to_vec(), [3]),
        &device,
    );
    // Keep both uploaded tensors alive across the operation: the result must
    // be fresh and neither retained allocation may be changed in place.
    let (lhs, rhs, shape) = views(&input, &other, case);
    let output = ops::numeric::bitwise_xor(lhs, rhs);
    assert_fresh(&output, &input, &other, &shape);
    let actual = ops::into_data_sync(output).to_vec::<i32>().unwrap();
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
    assert_eq!(ops::into_data_sync(input).to_vec::<i32>().unwrap(), values);
    assert_eq!(ops::into_data_sync(other).to_vec::<i32>().unwrap(), others);
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
