pub fn calc_wpm(inputs_length: usize, seconds: i32, misses: i32) -> f64 {
    (inputs_length as f64 - misses as f64) / (5.0 * seconds as f64 / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_wpm_normal() {
        // ID: WPM-001
        // 50文字, 60秒, 0ミス -> 10 WPM
        let result = calc_wpm(50, 60, 0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_calc_wpm_with_misses() {
        // ID: WPM-002
        // 50文字, 60秒, 5ミス -> 9 WPM
        let result = calc_wpm(50, 60, 5);
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_calc_wpm_short_time() {
        // ID: WPM-003
        // 10文字, 10秒, 0ミス -> 12 WPM
        // (10 - 0) / (5 * 10 / 60) = 10 / (50/60) = 10 / 0.8333... = 12.0
        let result = calc_wpm(10, 10, 0);
        assert_eq!(result, 12.0);
    }

    #[test]
    fn test_calc_wpm_zero_seconds() {
        // ID: WPM-004
        // 時間0の場合は無限大になる（Rustのf64仕様）
        let result = calc_wpm(10, 0, 0);
        assert!(result.is_infinite());
    }
}
