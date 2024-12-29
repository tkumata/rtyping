mod domain;
mod presentation;

use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::Source;
use rodio::{source::SineWave, OutputStream};
use std::collections::HashMap;
use std::io::{self};
use std::io::{stdin, stdout, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style, terminal_size};

use domain::entity;
use presentation::bgm;
use presentation::cli;
use presentation::intro;

fn main() -> io::Result<()> {
    let args = cli::parse_args();

    // sine 波生成ストリーミング
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // スレッド間通信チャンネル
    let (mt_tx, mt_rx) = mpsc::channel(); // メイン -> タイマー
    let (tt_tx, tt_rx) = mpsc::channel(); // タイマー -> メイン
    let (bgm_tx, bgm_rx) = mpsc::channel();

    // 横幅を固定（例: 80）
    let fixed_width: u16 = 80;
    // 現在のターミナルサイズを取得
    let (width, _height) = terminal_size().unwrap_or((80, 24));
    // 使用する幅を固定幅と現在の横幅の大きい方にする
    let use_width = std::cmp::max(width, fixed_width);

    // サンプルテキスト
    let mut sample_contents = String::new();
    match entity::get_sample() {
        Ok(contents) => {
            sample_contents = contents;
        }
        Err(err) => {
            eprintln!("Failed to read file: {}", err);
        }
    }
    let text = sample_contents.as_str();

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
    intro::print_intro();

    // 目標単語列表示
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut inputs: Vec<String> = Vec::new(); // ユーザ入力保持 Vec 用意
    let mut incorrect_chars = 0; // 入力間違い文字数
    let target_string = generate_markov_chain(text, 3, args.level); // n-gram を使用して生成
    let target_str = &target_string;
    let line = "-".repeat(use_width as usize);
    print!("{}\r\n", line);
    print!("{}", termion::cursor::Save); // カーソル位置保存
    print!("{}\r\n", target_string);
    print!("{}\r\n", line);
    print!("{}", termion::cursor::Restore); // カーソル位置復元 (入力位置がここになる)
    print!("{}", termion::cursor::BlinkingBar); // カーソルをバーに変形
    io::stdout().flush().unwrap();

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
                print_timer(*timer_value);
                *timer_value += 1;
            }

            thread::sleep(Duration::from_secs(1));
        }

        print!(
            "\r\n\r\n{}{}⏰Time up. Press any key.{}\r\n",
            termion::cursor::Down(1),
            color::Fg(color::Red),
            style::Reset
        );
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

    print!("\r\n\r\nQuit.\r\n");

    // WPM 計算と表示
    let elapsed_timer = *timer.lock().unwrap() - 1;
    print!("{:<13}: {} sec\r\n", "⌚Total Time", elapsed_timer);
    print!("{:<13}: {} chars\r\n", "🔢Total Typing", inputs.len());
    print!("{:<13}: {} chars\r\n", "❌Misses", incorrect_chars);
    print!(
        "{:<13}: {}{:.2}{}\r\n",
        "🎯WPM",
        color::Fg(color::Green),
        calc_wpm(inputs.len(), elapsed_timer, incorrect_chars),
        style::Reset
    );
    print!("{}", termion::cursor::BlinkingBlock); // カーソルをブロックに変形

    bgm_tx.send(()).unwrap();
    Ok(())
}

fn print_timer(timer: i32) {
    print!("{}", termion::cursor::Save);
    print!("{}", termion::cursor::Goto(1, 3));
    print!("{}", termion::clear::CurrentLine);
    print!("Time: {} sec", timer);
    print!("{}", termion::cursor::Restore);

    io::stdout().flush().unwrap();
}

fn calc_wpm(inputs_length: usize, seconds: i32, incorrect: i32) -> f64 {
    (inputs_length as f64 - incorrect as f64) / (5.0 * seconds as f64 / 60.0)
}

// マルコフ連鎖関数
fn generate_markov_chain(text: &str, n: usize, level: usize) -> String {
    // サンプルテキストを単語に分割
    let words: Vec<&str> = text.split_whitespace().collect();

    // n-gram モデルを作成
    let mut markov_chain: HashMap<Vec<&str>, Vec<&str>> = HashMap::new();

    for i in 0..(words.len() - n) {
        let key = words[i..i + n].to_vec();
        let value = words[i + n];
        markov_chain.entry(key).or_insert_with(Vec::new).push(value);
    }

    // 初期状態としてランダムな開始単語を選ぶ
    let mut rng = thread_rng();
    let start_index = rand::Rng::gen_range(&mut rng, 0..words.len() - n);
    let mut current_state = words[start_index..start_index + n].to_vec();

    // 次の単語をランダムに選びながら生成
    let mut result = current_state.clone();
    for _ in 0..level {
        if let Some(next_words) = markov_chain.get(&current_state) {
            let next_word = next_words.choose(&mut rng).unwrap();
            result.push(*next_word);
            current_state.push(*next_word);
            current_state.remove(0); // 最初の単語を削除して次の状態に移動
        } else {
            break; // マッチするパターンが見つからない場合、終了
        }
    }

    // 結果を結合して文を返す
    result.join(" ")
}
