use std::io::Write;
use std::io::{self};
use termion::{style, terminal_size};

use crate::config::*;
use crate::usecase::generate_sentence;

pub struct SentenceHandler;

impl SentenceHandler {
    pub fn print_sentence(level: usize) -> Result<String, io::Error> {
        // 現在のターミナルサイズを取得
        let (width, _height) = terminal_size().unwrap_or((FIXED_WIDTH, 14));

        // 使用する幅を固定幅と現在の横幅の大きい方にする
        let use_width = std::cmp::max(width, FIXED_WIDTH);

        // 枠
        let line = WINDOW_LINE.repeat(use_width as usize);

        // 文章を生成依頼
        match generate_sentence::generate_sentence(level) {
            Ok(contents) => {
                // 画面に反映
                print!("{}", termion::clear::All);
                print!("{}", termion::cursor::Goto(1, Y_TARGET));
                print!("{}\r\n", line);
                print!("{}", termion::cursor::Save); // カーソル位置保存
                print!("{}\r\n", contents);
                print!("{}\r\n", line);
                print!("{}", termion::cursor::Restore); // カーソル位置復元 (入力位置がここになる)
                print!("{}", termion::cursor::BlinkingBar); // カーソルをバーに変形
                io::stdout().flush().unwrap();
                Ok(contents)
            }
            Err(err) => {
                println!("{}", style::Reset);
                println!("Failed to generate strings: {}", err);
                Err(err)
            }
        }
    }
}
