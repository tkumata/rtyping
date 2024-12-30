use clap::{arg, Command};
use std::io::Write;
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

    pub fn print_intro() {
        print!("{}", termion::clear::All);
        print!("{}", termion::cursor::Goto(1, Y_TITLE));
        print!(
            "{}🦀 R-Typing ⌨️{}\r\n",
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

    pub fn print_timer(timer: i32) {
        print!("{}", termion::cursor::Save); // 入力中の位置を保存
        print!("{}", termion::cursor::Goto(1, Y_TIMER));
        print!("{}", termion::clear::CurrentLine);
        print!("Time: {} sec", timer);
        print!("{}", termion::cursor::Restore); // 入力中の位置に戻す

        // フラッシュ
        io::stdout().flush().unwrap();
    }

    pub fn print_wpm(elapsed_timer: i32, length: usize, incorrect_chars: i32) {
        print!("{}", termion::cursor::Goto(1, Y_QUIT));
        print!("{}", termion::clear::AfterCursor);
        print!("{:<width$}: {} sec\r\n", "Total Time", elapsed_timer, width = SCORE_TITLE_WIDTH);
        print!("{:<width$}: {} chars\r\n", "Total Typing", length, width = SCORE_TITLE_WIDTH);
        print!("{:<width$}: {} chars\r\n", "Misses", incorrect_chars, width = SCORE_TITLE_WIDTH);
        print!(
            "{:<width$}: {}{:.2}{} wpm\r\n",
            "Word Per Minute",
            color::Fg(color::Green),
            wpm::calc_wpm(length, elapsed_timer, incorrect_chars),
            style::Reset,
            width = SCORE_TITLE_WIDTH
        );
    }

    pub fn print_timeup() {
        print!("{}", termion::cursor::Goto(1, Y_QUIT));
        print!("{}", termion::clear::AfterCursor);
        print!(
            "{}⏰Time up. Press any key.↩️{}\r\n",
            color::Fg(color::Red),
            style::Reset
        );
    }
}
