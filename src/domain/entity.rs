pub fn get_sample() -> String {
    include_str!("sample.txt").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sample() {
        // ID: ENT-001
        let content = get_sample();
        assert!(!content.is_empty(), "Sample text should not be empty");
    }
}
