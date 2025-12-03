use std::io::{self};

use crate::usecase::generate_sentence;

pub struct SentenceHandler;

impl SentenceHandler {
    pub fn print_sentence(level: usize) -> Result<String, io::Error> {
        generate_sentence::generate_sentence(level)
    }
}
