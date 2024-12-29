use clap::{arg, Command};
use std::io::Write;
use std::io::{self};
use termion;
use termion::{color, style};

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
            .about("R-Typing: A terminal-based typing practice app.")
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

    pub fn print_intro() {
        print!(
            "{}{}{}🦀 R-Typing - Rust Typing Program ⌨️{}\r\n",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            color::Fg(color::LightBlue),
            style::Reset
        );
        print!("🚀Press *ENTER* key to start.\r\n");

        let mut start: String = String::new();

        io::stdin()
            .read_line(&mut start)
            .expect("Failed to read line.");
    }

    pub fn print_wpm(elapsed_timer: i32, length: usize, incorrect_chars: i32) {
        print!("{:<13}: {} sec\r\n", "⌚Total Time", elapsed_timer);
        print!("{:<13}: {} chars\r\n", "🔢Total Typing", length);
        print!("{:<13}: {} chars\r\n", "❌Misses", incorrect_chars);
        print!(
            "{:<13}: {}{:.2}{}\r\n",
            "🎯WPM",
            color::Fg(color::Green),
            wpm::calc_wpm(length, elapsed_timer, incorrect_chars),
            style::Reset
        );
        print!("{}", termion::cursor::BlinkingBlock); // カーソルをブロックに変形
    }

    pub fn print_timeup() {
        print!(
            "\r\n\r\n{}{}⏰Time up. Press any key.{}\r\n",
            termion::cursor::Down(1),
            color::Fg(color::Red),
            style::Reset
        );
    }

    pub fn print_timer(timer: i32) {
        print!("{}", termion::cursor::Save);
        print!("{}", termion::cursor::Goto(1, 3));
        print!("{}", termion::clear::CurrentLine);
        print!("Time: {} sec", timer);
        print!("{}", termion::cursor::Restore);

        io::stdout().flush().unwrap();
    }
}
