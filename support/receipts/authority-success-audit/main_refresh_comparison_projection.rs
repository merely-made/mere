pub fn compare(logits: &[f32], expected: &[serde_json::Value]) -> f32 {
let max = logits
                    .iter()
                    .zip(expected)
                    .map(|(a, b)| (*a - b.as_f64().unwrap() as f32).abs())
                    .fold(0.0f32, f32::max);
                max
}
