use std::io::{self};
use termion;
use termion::{color, style};

pub(crate) fn print_intro() {
    print!(
        "{}{}{}🦀 R-Typing - Typing Practice Program ⌨️{}\r\n",
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
