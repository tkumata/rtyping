use clap::{arg, Command};
use std::io::{self};
use termion;
use termion::{color, style};

use crate::config::*;
use crate::usecase::wpm;

pub struct CliArgs {
    pub timeout: i32,
    pub level: usize,
    pub freq: f32,
    pub sound: bool,
}

pub struct UiHandler;

impl UiHandler {
    pub fn parse_args() -> CliArgs {
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
                arg!(--freq <FREQUENCY> "Frequency e.g, 800.0 or 480.0")
                    .default_value("800.0")
                    .value_parser(clap::value_parser!(f32)),
            )
            .arg(arg!(-s --sound "Enable BGM"))
            .get_matches();

        CliArgs {
            timeout: *matches.get_one::<i32>("timeout").expect("expect number"),
            level: *matches.get_one::<usize>("level").expect("expect number"),
            freq: *matches.get_one::<f32>("freq").expect("expect frequency"),
            sound: matches.get_flag("sound"),
        }
    }

    // イントロ表示
    pub fn print_intro() {
        print!("{}", termion::clear::All);
        print!("{}", termion::cursor::Goto(1, Y_TITLE));
        print!(
            "{}>>> R-Typing <<<{}\r\n",
            color::Fg(color::LightBlue),
            style::Reset
        );
        print!("Press *ENTER* key to start.🚀\r\n");

        // ENTER 入力待ち
        let mut start: String = String::new();
        io::stdin()
            .read_line(&mut start)
            .expect("Failed to read line.");
    }

    // WPM 表示
    pub fn print_wpm(elapsed_timer: i32, length: usize, incorrects: i32) {
        print!("{}", termion::cursor::Goto(1, Y_QUIT));
        print!("{}", termion::clear::AfterCursor);
        print!(
            "{:<width$}: {} sec\r\n",
            TOTAL_TIME,
            elapsed_timer,
            width = SUMMARY_TITLE_WIDTH
        );
        print!(
            "{:<width$}: {} chars\r\n",
            TOTAL_TYPE,
            length,
            width = SUMMARY_TITLE_WIDTH
        );
        print!(
            "{:<width$}: {} chars\r\n",
            TOTAL_MISSES,
            incorrects,
            width = SUMMARY_TITLE_WIDTH
        );
        print!(
            "{:<width$}: {}{:.2}{} wpm\r\n",
            WORD_PER_MINUTE,
            color::Fg(color::Green),
            wpm::calc_wpm(length, elapsed_timer, incorrects),
            style::Reset,
            width = SUMMARY_TITLE_WIDTH
        );
    }
}
