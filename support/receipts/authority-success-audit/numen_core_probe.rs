use numen::{FieldDef, FieldId, FieldRegistry, ScalarField, VectorField, eval_scalar, grad_scalar};

fn close(actual: (f32, f32), expected: (f32, f32), tolerance: f32) {
    assert!((actual.0 - expected.0).abs() < tolerance, "{actual:?} != {expected:?}");
    assert!((actual.1 - expected.1).abs() < tolerance, "{actual:?} != {expected:?}");
}

#[test]
fn linear_dynamic_normal_omits_its_derivative_and_sample_changes_the_answer() {
    let mut registry = FieldRegistry::new();
    let field = ScalarField::Linear { normal: Box::new(VectorField::Coord), offset: 0.0 };
    assert_eq!(eval_scalar(&field, &registry, 3.0, 4.0, 0.0), 25.0);
    let direct = grad_scalar(&field, &registry, 3.0, 4.0, 0.0);
    assert_eq!(direct, (3.0, 4.0)); // Wrong: gradient of x*x+y*y is (2*x,2*y).
    let id = registry.insert_scalar("same", field.clone());
    let sampled = grad_scalar(&ScalarField::Sample(id), &registry, 3.0, 4.0, 0.0);
    close(sampled, (6.0, 8.0), 0.005);
    close(grad_scalar(&ScalarField::linear(2.0, -3.0, 1.0), &registry, 3.0, 4.0, 0.0), (2.0, -3.0), 0.0001);
    println!("Dynamic Linear value25: direct gradient{direct:?}, registry-sampled equivalent{sampled:?}, expected(6,8); constant-normal control correct.");
}

#[test]
fn gaussian_dynamic_center_produces_a_gradient_for_a_constant_function() {
    let registry = FieldRegistry::new();
    let field = ScalarField::Gaussian {
        center: Box::new(VectorField::Add(Box::new(VectorField::Coord), Box::new(VectorField::const_vec(1.0, 0.0)))),
        sigma: 10.0,
    };
    let value = eval_scalar(&field, &registry, 3.0, 4.0, 0.0);
    assert_eq!(value, eval_scalar(&field, &registry, 100.0, -40.0, 0.0));
    let direct = grad_scalar(&field, &registry, 3.0, 4.0, 0.0);
    assert!(direct.0 > 0.009 && direct.0 < 0.011 && direct.1 == 0.0);
    let h = 0.01;
    let finite = ((eval_scalar(&field, &registry, 3.0+h, 4.0, 0.0) - eval_scalar(&field, &registry, 3.0-h, 4.0, 0.0))/(2.0*h),
        (eval_scalar(&field, &registry, 3.0, 4.0+h, 0.0) - eval_scalar(&field, &registry, 3.0, 4.0-h, 0.0))/(2.0*h));
    close(finite, (0.0, 0.0), 0.0001);
    let fixed = ScalarField::gaussian_at(1.0, 0.0, 10.0);
    let fixed_value = eval_scalar(&fixed, &registry, 3.0, 4.0, 0.0);
    close(grad_scalar(&fixed, &registry, 3.0, 4.0, 0.0), (-2.0/100.0*fixed_value, -4.0/100.0*fixed_value), 0.00001);
    println!("Gaussian center=p+(1,0): constant value{value}, direct gradient{direct:?}, finite difference{finite:?}; fixed-center control correct.");
}

#[test]
fn locally_minted_ids_overwrite_an_explicit_zero_identity() {
    let id = FieldId::from_uuid(Default::default());
    let mut registry = FieldRegistry::new();
    registry.insert_with_id(id, FieldDef::Scalar(ScalarField::Const(7.0)));
    assert_eq!(eval_scalar(&ScalarField::Sample(id), &registry, 0.0, 0.0, 0.0), 7.0);
    let minted = registry.insert_scalar("new", ScalarField::Const(9.0));
    assert_eq!(minted, id);
    assert_eq!(registry.len(), 1);
    assert_eq!(eval_scalar(&ScalarField::Sample(id), &registry, 0.0, 0.0, 0.0), 9.0);
    let mut control = FieldRegistry::new();
    let a = control.insert_scalar("a", ScalarField::Const(7.0));
    let b = control.insert_scalar("b", ScalarField::Const(9.0));
    assert_ne!(a, b);
    assert_eq!(control.len(), 2);
    assert_eq!(control.insert_scalar("a", ScalarField::Const(11.0)), a);
    assert_eq!(control.len(), 2);
    println!("Explicit zero ID value7 is silently replaced by unrelated new named field value9; ordinary fresh-name and same-name-update controls behave as documented.");
}
