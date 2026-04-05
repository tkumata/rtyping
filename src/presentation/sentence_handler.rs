use std::io::{self};

use crate::config::ProviderConfig;
use crate::usecase::generate_sentence;
use crate::usecase::generate_sentence::GenerationSource;

pub struct SentenceHandler;

impl SentenceHandler {
    pub fn print_sentence(
        level: usize,
        source: GenerationSource,
        provider_config: Option<ProviderConfig>,
    ) -> Result<String, io::Error> {
        generate_sentence::generate_sentence(level, source, provider_config)
    }
}
