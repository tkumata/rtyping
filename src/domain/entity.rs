use std::io;

pub fn get_sample() -> Result<String, io::Error> {
    let contents = include_str!("sample.txt").to_string();
    Ok(contents)
}
