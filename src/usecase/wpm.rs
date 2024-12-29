use termion;
use termion::{color, style};

pub fn print_wpm(elapsed_timer: i32, length: usize, incorrect_chars: i32) {
    print!("{:<13}: {} sec\r\n", "⌚Total Time", elapsed_timer);
    print!("{:<13}: {} chars\r\n", "🔢Total Typing", length);
    print!("{:<13}: {} chars\r\n", "❌Misses", incorrect_chars);
    print!(
        "{:<13}: {}{:.2}{}\r\n",
        "🎯WPM",
        color::Fg(color::Green),
        calc_wpm(length, elapsed_timer, incorrect_chars),
        style::Reset
    );
    print!("{}", termion::cursor::BlinkingBlock); // カーソルをブロックに変形
}

fn calc_wpm(inputs_length: usize, seconds: i32, incorrect: i32) -> f64 {
    (inputs_length as f64 - incorrect as f64) / (5.0 * seconds as f64 / 60.0)
}
