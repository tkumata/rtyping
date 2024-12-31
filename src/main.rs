mod config;
mod domain;
mod presentation;
mod usecase;

use rodio::Source;
use rodio::{source::SineWave, OutputStream};
use std::io::{self};
use std::io::{stdin, stdout, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

use presentation::bgm_handler::BgmHandler;
use presentation::sentence_handler::SentenceHandler;
use presentation::timer_handler::TimerHandler;
use presentation::ui::ui_handler::UiHandler;

fn main() -> io::Result<()> {
    // ヘルプと引数処理
    let args = UiHandler::parse_args();

    // sine 波生成ストリーミング
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // スレッド間通信チャンネル
    let (main_sender, main_receiver) = mpsc::channel();
    let (timer_sender, timer_receiver) = mpsc::channel();
    let (snd_sender, snd_receiver) = mpsc::channel();

    // BGM 処理
    if args.sound {
        let bgm_handler = BgmHandler::new(snd_receiver);
        bgm_handler.start();
    }

    // イントロを表示
    UiHandler::print_intro();

    // 目標文字列表示初期化 (イントロ表示後に初期化が必要)
    // Todo: この歪な依存関係を排除する。
    let stdin = stdin();
    // Raw mode に移行
    let mut stdout = stdout().into_raw_mode().unwrap();
    // ユーザ入力保持 Vec 用意
    let mut inputs: Vec<String> = Vec::new();
    // 入力間違い文字数
    let mut incorrects = 0;

    // 目標単語列表示
    let target_string = match SentenceHandler::print_sentence(args.level) {
        Ok(contents) => contents,
        Err(err) => {
            println!("Failed to generate sentence: {}", err);
            return Err(err);
        }
    };
    let target_str = &target_string;

    // タイマーの表示とカウント
    let timer = Arc::new(Mutex::new(0));
    let timer_clone = Arc::clone(&timer);
    let timer_handler = TimerHandler::new(timer_clone, timer_receiver, main_sender, args.timeout);
    timer_handler.start();

    // ユーザ入力を監視する
    // Todo: stdin.events を変える。
    for evt in stdin.events() {
        if let Ok(Event::Key(key)) = evt {
            match key {
                Key::Ctrl('c') | Key::Esc | Key::Char('\n') => {
                    if main_receiver.try_recv().is_ok() {
                        break;
                    }

                    timer_sender.send(()).unwrap();
                    break;
                }
                Key::Backspace => {
                    let l = inputs.len();
                    print!("{}", termion::cursor::Left(1));
                    print!("{}", target_str.chars().nth(l - 1).unwrap().to_string());
                    print!("{}", termion::cursor::Left(1));
                    inputs.pop();
                }
                Key::Char(c) => {
                    let l = inputs.len();

                    if target_str.chars().nth(l) == Some(c) {
                        print!("{}{}{}", color::Fg(color::Green), c, style::Reset);

                        // <FREQ> のビープ音を生成
                        let source =
                            SineWave::new(args.freq).take_duration(Duration::from_millis(100));
                        stream_handle.play_raw(source.convert_samples()).unwrap();
                    } else {
                        print!("{}{}{}{}", "\x07", color::Fg(color::Red), c, style::Reset);
                        incorrects += 1;
                    }

                    inputs.push(String::from(c.to_string()));
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }
    }

    // WPM 計算と表示
    UiHandler::print_wpm(
        (*timer.lock().unwrap() - 1).try_into().unwrap(),
        inputs.len(),
        incorrects,
    );

    snd_sender.send(()).unwrap();
    Ok(())
}
