use clap::{arg, Command};
use rand::Rng;
use rodio::Source;
use rodio::{source::SineWave, OutputStream};
use std::fs;
use std::io;
use std::io::{stdin, stdout, BufReader, Cursor, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

fn main() -> io::Result<()> {
    let matches = Command::new("rtyping")
        .author("Tomokatsu Kumata")
        .about("Typing Practis Program")
        .arg(
            arg!(-t --timeout <TIMEOUT> "Seconds")
                .default_value("60")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            arg!(-l --level <LEVEL> "Number of words")
                .default_value("9")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            arg!(--freq <FREQUENCY> "Frequency e.g, 800.0 or 480.0")
                .default_value("800.0")
                .value_parser(clap::value_parser!(f32)),
        )
        .arg(arg!(-s --sound "Enable BGM"))
        .get_matches();

    let timeout: i32 = *matches.get_one::<i32>("timeout").expect("expect number");
    let level: usize = *matches.get_one::<usize>("level").expect("expect number");
    let sound: bool = matches.get_flag("sound");
    let freq: f32 = *matches.get_one::<f32>("freq").expect("expect frequency");

    // イントロを表示
    print_intro();

    // 音の処理
    if sound {
        thread::spawn(|| loop {
            play_audio();
        });
    }

    // 初期化
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    let timer = Arc::new(Mutex::new(0));
    let timer_clone = Arc::clone(&timer);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // 打鍵用ストリーミング
    let mut inputs: Vec<String> = Vec::new(); // ユーザ入力保持 Vec 用意
    let mut incorrect_chars = 0; // 入力間違い文字数

    let (tx_mt, rx_mt) = mpsc::channel(); // メイン -> タイマー
    let (tx_tt, rx_tt) = mpsc::channel(); // タイマー -> メイン

    let target_string = load_words(level); // 目標単語列
    let target_str = &target_string;

    println!("{}\r", target_string); // 目的単語列を表示
    println!("{}", termion::cursor::Up(2)); // 入力位置を調整

    // タイマーの表示とカウント
    let timer_thread = thread::spawn(move || {
        loop {
            if rx_tt.try_recv().is_ok() {
                return;
            }
            {
                let mut timer_value = timer_clone.lock().unwrap();
                if *timer_value > timeout {
                    break;
                }
                print_timer(*timer_value);
                *timer_value += 1;
            }
            thread::sleep(Duration::from_secs(1));
        }

        println!(
            "\r{}==> {}Time up. Press any key.{}\r",
            termion::cursor::Down(1),
            color::Fg(color::Red),
            style::Reset
        );
        tx_mt.send(()).unwrap();
    });

    // ユーザ入力を監視する
    for evt in stdin.events() {
        if rx_mt.try_recv().is_ok() {
            break;
        }

        match evt.unwrap() {
            Event::Key(Key::Ctrl('c')) | Event::Key(Key::Esc) | Event::Key(Key::Char('\n')) => {
                println!("\r");
                tx_tt.send(()).unwrap();
                break;
            }
            Event::Key(Key::Backspace) => {
                if !inputs.is_empty() {
                    let l = inputs.len();
                    print!("{}", termion::cursor::Left(1));
                    print!("{}", target_str.chars().nth(l - 1).unwrap().to_string());
                    print!("{}", termion::cursor::Left(1));
                    inputs.pop();
                }
            }
            Event::Key(Key::Char(c)) => {
                let l = inputs.len();

                if target_str.chars().nth(l) == Some(c) {
                    print!("{}{}{}", color::Fg(color::Green), c, style::Reset);

                    // Produce a 440Hz beep sound
                    let source = SineWave::new(freq).take_duration(Duration::from_millis(200));
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

    timer_thread.join().unwrap();

    println!("Quit.\r");

    // WPM 計算と表示
    let elapsed_timer = *timer.lock().unwrap() - 1;
    println!("Total Time: {} sec\r", elapsed_timer);
    println!("Total Types: {} chars\r", inputs.len());
    println!("Incorrect Types: {} chars\r", incorrect_chars);
    println!(
        "WPM: {}{:.2}{}\r",
        color::Fg(color::Green),
        calc_wpm(inputs.len(), elapsed_timer, incorrect_chars),
        style::Reset
    );

    Ok(())
}

fn print_intro() {
    println!(
        "{}{}{}{goto}{lightblue}{bold}{italic}R-Typing - Rust Typing Practis Program{reset}",
        termion::clear::CurrentLine,
        termion::clear::AfterCursor,
        termion::clear::BeforeCursor,
        goto = termion::cursor::Goto(1, 1),
        lightblue = color::Fg(color::LightBlue),
        bold = style::Bold,
        italic = style::Italic,
        reset = style::Reset
    );
    println!("Press *ENTER* key to start.\r");

    let mut start: String = String::new();

    io::stdin()
        .read_line(&mut start)
        .expect("==> Failed to read line.");
}

fn print_timer(timer: i32) {
    print!("{}", termion::cursor::Save);
    print!("{}", termion::cursor::Goto(1, 3));
    print!("{}", termion::clear::CurrentLine);
    print!("Time: {}sec", timer);
    print!("{}", termion::cursor::Restore);

    io::stdout().flush().unwrap();
}

fn play_audio() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    let bytes = include_bytes!("../audio/BGM.mp3");
    let cursor = Cursor::new(bytes);

    sink.append(rodio::Decoder::new(BufReader::new(cursor)).unwrap());
    sink.set_volume(0.4);
    sink.sleep_until_end();
}

fn load_words(level: usize) -> String {
    let words: Vec<_> = fs::read_dir("/usr/bin")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect();
    let len = words.len();

    if len < level {
        panic!("Not enough words available!");
    }

    let start = rand::thread_rng().gen_range(0..=len - level);
    words[start..start + level].join(" ")
}

fn calc_wpm(inputs_length: usize, seconds: i32, incorrect: i32) -> f64 {
    (inputs_length as f64 - incorrect as f64) / (5.0 * seconds as f64 / 60.0)
}
