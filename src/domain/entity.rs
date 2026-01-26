use std::io;

pub fn get_sample() -> Result<String, io::Error> {
    let contents = include_str!("sample.txt").to_string();
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sample() {
        // ID: ENT-001
        match get_sample() {
            Ok(content) => {
                assert!(!content.is_empty(), "Sample text should not be empty");
            }
            Err(e) => {
                panic!("Failed to get sample text: {}", e);
            }
        }
    }
}
