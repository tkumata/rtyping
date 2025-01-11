pub fn calc_wpm(inputs_length: usize, seconds: i32, misses: i32) -> f64 {
    (inputs_length as f64 - misses as f64) / (5.0 * seconds as f64 / 60.0)
}
