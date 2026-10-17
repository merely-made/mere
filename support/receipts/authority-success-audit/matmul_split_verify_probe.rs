fn rejects(have:f32,want:f32)->bool{(have - want).abs() > want.abs() * 1e-3 + 1e-3}

#[test]fn nan_output_is_not_rejected(){assert!(!rejects(f32::NAN,8.0));}
#[test]fn wrong_finite_output_is_rejected(){assert!(rejects(80.0,8.0));}
#[test]fn equal_finite_output_control(){assert!(!rejects(8.0,8.0));}
#[test]fn infinite_output_is_rejected(){assert!(rejects(f32::INFINITY,8.0));}
