use std::io::Write;
use std::io::{self};
use termion::terminal_size;

use crate::config::*;
use crate::usecase::generate_sentence;

pub struct SentenceHandler;

impl SentenceHandler {
    pub fn print_sentence(level: usize) -> String {
        // 横幅を固定
        let fixed_width: u16 = 80;

        // 現在のターミナルサイズを取得
        let (width, _height) = terminal_size().unwrap_or((80, 24));

        // 使用する幅を固定幅と現在の横幅の大きい方にする
        let use_width = std::cmp::max(width, fixed_width);

        // 枠
        let line = "-".repeat(use_width as usize);

        // 文章を生成依頼
        // Todo: unwrap() じゃなくてエラーハンドリングする。
        let target_string = generate_sentence::generate_sentence(level).unwrap();

        // 画面に反映
        print!("{}", termion::cursor::Goto(1, Y_TARGET));
        print!("{}\r\n", line);
        print!("{}", termion::cursor::Save); // カーソル位置保存
        print!("{}\r\n", target_string);
        print!("{}\r\n", line);
        print!("{}", termion::cursor::Restore); // カーソル位置復元 (入力位置がここになる)
        print!("{}", termion::cursor::BlinkingBar); // カーソルをバーに変形
        io::stdout().flush().unwrap();

        target_string
    }
}
