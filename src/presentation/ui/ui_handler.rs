use clap::{arg, Command};
use std::io::{self};
use termion;
use termion::color;

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
    // ヘルプと引数処理
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
                arg!(--freq <FREQUENCY> "Frequency e.g, 880.0 or 480.0")
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
        let title = format!(
            r"
 ____     _____            _
|  _ \   |_   _|   _ _ __ (_)_ __   __ _ 
| |_) |____| || | | | '_ \| | '_ \ / _` |
|  _ <_____| || |_| | |_) | | | | | (_| |
|_| \_\    |_| \__, | .__/|_|_| |_|\__, |
               |___/|_|            |___/"
        );

        print!("{}", termion::clear::All);
        print!("{}", termion::cursor::Goto(1, Y_TITLE));
        println!("{}", title);
        println!("Press *ENTER* key to start.🚀");

        // ENTER 入力待ち
        let mut start: String = String::new();
        io::stdin()
            .read_line(&mut start)
            .expect("Failed to read line.");
    }

    // WPM 表示
    pub fn print_wpm(elapsed_timer: i32, length: usize, incorrects: i32) {
        let wpm = wpm::calc_wpm(length, elapsed_timer, incorrects);
        let result_text = format!(
            ",-----------------------------.\r
| 🏁 Result                   |\r
|-----------------------------|\r
| Total Time      : {elapsed_timer:<3} sec   |\r
| Total Typing    : {length:<3} chars |\r
| Total Misses    : {incorrects:<3} chars |\r
| Words Per Minute: {color}{wpm:<5.1}{reset} wpm |\r
`-----------------------------'\r",
            color = color::Fg(color::Green),
            reset = color::Fg(color::Reset)
        );

        print!("{}", termion::cursor::Goto(1, Y_QUIT));
        print!("{}", termion::clear::AfterCursor);
        println!("{}", result_text);
    }
}
