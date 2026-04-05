use std::io::{self};

use crate::config::ProviderConfig;
use crate::usecase::generate_sentence;
use crate::usecase::generate_sentence::GenerationSource;

pub struct SentenceHandler;

impl SentenceHandler {
    pub fn print_sentence(
        text_scale: usize,
        source: GenerationSource,
        provider_config: Option<ProviderConfig>,
    ) -> Result<String, io::Error> {
        generate_sentence::generate_sentence(text_scale, source, provider_config)
    }
}
