use clap::{ArgAction, Command, arg};

use crate::usecase::generate_sentence::GenerationSource;

pub struct CliArgs {
    pub timeout: i32,
    pub level: usize,
    pub freq: f32,
    pub sound: bool,
    pub source: GenerationSource,
}

pub struct UiHandler;

impl UiHandler {
    // ヘルプと引数処理
    pub fn parse_args() -> CliArgs {
        Self::parse_from(std::env::args())
    }

    pub fn parse_from<I, T>(args: I) -> CliArgs
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = Command::new("rtyping")
            .author("Tomokatsu Kumata")
            .about("R-Typing: A terminal-based typing app.")
            .arg(
                arg!(-t --timeout <TIMEOUT> "Seconds")
                    .default_value("60")
                    .value_parser(clap::value_parser!(i32)),
            )
            .arg(
                arg!(-l --level <LEVEL> "Number of words")
                    .default_value("30")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                arg!(--freq <FREQUENCY> "Frequency e.g, 880.0 or 480.0")
                    .default_value("80.0")
                    .value_parser(clap::value_parser!(f32)),
            )
            .arg(arg!(-s --sound "Enable BGM and typing sound"))
            .arg(
                arg!(--google "Use Google AI Studio for text generation")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("groq"),
            )
            .arg(
                arg!(--groq "Use Groq for text generation")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("google"),
            )
            .get_matches_from(args);

        let source = if matches.get_flag("google") {
            GenerationSource::Google
        } else if matches.get_flag("groq") {
            GenerationSource::Groq
        } else {
            GenerationSource::Local
        };

        CliArgs {
            timeout: *matches.get_one::<i32>("timeout").expect("expect number"),
            level: *matches.get_one::<usize>("level").expect("expect number"),
            freq: *matches.get_one::<f32>("freq").expect("expect frequency"),
            sound: matches.get_flag("sound"),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_google_flag() {
        let args = UiHandler::parse_from(["rtyping", "--google"]);
        assert_eq!(args.source, GenerationSource::Google);
    }

    #[test]
    fn parse_groq_flag() {
        let args = UiHandler::parse_from(["rtyping", "--groq"]);
        assert_eq!(args.source, GenerationSource::Groq);
    }
}
