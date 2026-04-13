pub fn calc_accuracy(typed_count: usize, incorrects: usize) -> f64 {
    if typed_count == 0 {
        return 0.0;
    }

    #[expect(clippy::cast_precision_loss)]
    let typed = typed_count as f64;
    #[expect(clippy::cast_precision_loss)]
    let incorrect = incorrects as f64;
    (typed - incorrect) / typed * 100.0
}

#[cfg(test)]
mod tests {
    #![expect(clippy::float_cmp)]
    use super::*;

    #[test]
    fn test_calc_accuracy_normal() {
        let result = calc_accuracy(100, 8);
        assert_eq!(result, 92.0);
    }

    #[test]
    fn test_calc_accuracy_with_misses() {
        let result = calc_accuracy(50, 5);
        assert_eq!(result, 90.0);
    }

    #[test]
    fn test_calc_accuracy_zero_typed_count() {
        let result = calc_accuracy(0, 0);
        assert_eq!(result, 0.0);
    }
}
