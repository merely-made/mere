use std::error::Error;
fn max_abs(a: &[f32], b: &[f32]) -> Result<f32, Box<dyn Error>> {
    if a.len() != b.len() {
        return Err(format!("vector length mismatch: {} != {}", a.len(), b.len()).into());
    }
    Ok(a.iter()
        .zip(b)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0_f32, f32::max))
}

fn max_abs_error(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err(format!(
            "numerical comparison length mismatch: {} != {}",
            left.len(),
            right.len()
        ));
    }
    if !left.iter().chain(right).all(|value| value.is_finite()) {
        return Err("numerical comparison contains a non-finite value".to_string());
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max))
}


fn main() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let result = max_abs(&[value, 2.0], &[value, 0.0]).unwrap();
        assert_eq!(result, 2.0);
        assert!(max_abs_error(&[value, 2.0], &[value, 0.0]).is_err());
    }
    println!("OBSERVATION: session max_abs silently drops NaN differences, including equal infinities; remote max_abs_error refuses non-finite inputs");
    assert_eq!(max_abs(&[0.0, -1.0], &[0.0, -0.5]).unwrap(), 0.5);
    assert_eq!(max_abs_error(&[0.0, -1.0], &[0.0, -0.5]).unwrap(), 0.5);
    println!("CONTROL: finite inputs agree on the maximum difference");
    assert!(max_abs(&[1.0], &[]).is_err());
    assert!(max_abs_error(&[1.0], &[]).is_err());
    println!("CONTROL: both helpers refuse mismatched lengths");
    println!("3 checks passed");
}
