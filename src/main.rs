mod domain;
mod presentation;
mod usecase;

use rodio::Source;
use rodio::{source::SineWave, OutputStream};
use std::io::{self};
use std::io::{stdin, stdout, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

use presentation::bgm;
use presentation::sentence_handler::SentenceHandler;
use presentation::ui::ui_handler::UiHandler;

fn main() -> io::Result<()> {
    let args = UiHandler::parse_args();

    // sine 波生成ストリーミング
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // スレッド間通信チャンネル
    let (mt_tx, mt_rx) = mpsc::channel(); // メイン -> タイマー
    let (tt_tx, tt_rx) = mpsc::channel(); // タイマー -> メイン
    let (bgm_tx, bgm_rx) = mpsc::channel();

    // 音の処理
    if args.sound {
        thread::spawn(move || loop {
            if bgm_rx.try_recv().is_ok() {
                break;
            }
            bgm::play_audio();
        });
    }

    // イントロを表示
    UiHandler::print_intro();

    // 目標文字列表示初期化 (イントロ表示後に初期化が必要)
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut inputs: Vec<String> = Vec::new(); // ユーザ入力保持 Vec 用意
    let mut incorrect_chars = 0; // 入力間違い文字数

    // 目標単語列表示
    let target_string = SentenceHandler::print_sentence(args.level);
    let target_str = &target_string;

    // タイマーの表示とカウント
    let timer = Arc::new(Mutex::new(0));
    let timer_clone = Arc::clone(&timer);
    let timer_thread = thread::spawn(move || {
        loop {
            if tt_rx.try_recv().is_ok() {
                return;
            }

            if let Ok(mut timer_value) = timer_clone.try_lock() {
                if *timer_value > args.timeout {
                    break;
                }
                UiHandler::print_timer(*timer_value);
                // print_timer(*timer_value);
                *timer_value += 1;
            }

            thread::sleep(Duration::from_secs(1));
        }

        // Print time up
        UiHandler::print_timeup();

        mt_tx.send(()).unwrap();
    });

    // ユーザ入力を監視する
    for evt in stdin.events() {
        // Todo: Change stdin.events
        if mt_rx.try_recv().is_ok() {
            break;
        }

        if let Ok(Event::Key(key)) = evt {
            match key {
                Key::Ctrl('c') | Key::Esc | Key::Char('\n') => {
                    print!("\r\n");
                    tt_tx.send(()).unwrap();
                    break;
                }
                Key::Backspace => {
                    if !inputs.is_empty() {
                        let l = inputs.len();
                        print!("{}", termion::cursor::Left(1));
                        print!("{}", target_str.chars().nth(l - 1).unwrap().to_string());
                        print!("{}", termion::cursor::Left(1));
                        inputs.pop();
                    }
                }
                Key::Char(c) => {
                    let l = inputs.len();

                    if target_str.chars().nth(l) == Some(c) {
                        print!("{}{}{}", color::Fg(color::Green), c, style::Reset);

                        // Produce a <FREQ> beep sound
                        let source =
                            SineWave::new(args.freq).take_duration(Duration::from_millis(200));
                        stream_handle.play_raw(source.convert_samples()).unwrap();
                    } else {
                        print!("{}{}{}{}", "\x07", color::Fg(color::Red), c, style::Reset);
                        incorrect_chars += 1;
                    }

                    inputs.push(String::from(c.to_string()));
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }
    }

    timer_thread.join().unwrap();

    // WPM 計算と表示
    UiHandler::print_wpm(*timer.lock().unwrap() - 1, inputs.len(), incorrect_chars);

    bgm_tx.send(()).unwrap();
    Ok(())
}
