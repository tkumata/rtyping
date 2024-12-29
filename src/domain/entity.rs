use std::fs;
use std::io;

pub fn get_sample() -> Result<String, io::Error> {
    let file_path = "src/domain/sample.txt";

    match fs::read_to_string(file_path) {
        Ok(contents) => Ok(contents),
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            Err(err)
        }
    }
}
