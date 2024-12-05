use clap::{arg, Command};
use rand::Rng;
use std::fs;
use std::io;
use std::io::{stdin, stdout, BufReader, Cursor, Write};
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
                // .required(false)
                .default_value("60")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            arg!(-l --level <LEVEL> "Number of words")
                // .required(false)
                .default_value("9")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(arg!(-s --sound "Enable BGM"))
        .get_matches();

    let timeout: i32 = *matches.get_one::<i32>("timeout").expect("expect number");
    let level: usize = *matches.get_one::<usize>("level").expect("expect number");
    let sound: bool = matches.get_flag("sound");

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
    let stdin: io::Stdin = stdin();
    let mut timer: i32 = 0;

    // タイマーの表示とカウントを thread で実装。
    thread::spawn(move || {
        while timer < timeout {
            print_timer(timer);
            thread::sleep(Duration::from_secs(1));
            timer += 1;
        }

        println!("\r==> {}Time up{}\r", color::Fg(color::Red), style::Reset);
        std::process::exit(0);
    });

    // String と str 型で打鍵すべき目的の単語を level に応じて抽出する。
    let target_string = load_words(level);
    let target_str = &target_string;

    // 目的の単語を表示する。
    println!("{}\r", target_string);

    // ユーザの入力をためるための Vec を用意する。
    let mut inputs: Vec<String> = Vec::new();

    // 入力位置を調整
    println!("{}", termion::cursor::Up(2));

    // ユーザ入力を監視する。
    for evt in stdin.events() {
        match evt.unwrap() {
            Event::Key(Key::Ctrl('c')) | Event::Key(Key::Esc) | Event::Key(Key::Char('\n')) => {
                println!("\r");
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
                } else {
                    print!("{}{}{}", color::Fg(color::Red), c, style::Reset);
                }
                inputs.push(String::from(c.to_string()));
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }

    // let input = inputs.join("");
    // if input.trim() == target_string.trim() {
    //     let _ = tx.send(());
    //     println!(
    //         "==> {green}OK{reset}\r",
    //         green = color::Fg(color::Green),
    //         reset = style::Reset
    //     );
    //     println!("==> Try next words.\r");
    // } else {
    //     println!(
    //         "==> {red}NG{reset}\r",
    //         red = color::Fg(color::Red),
    //         reset = style::Reset
    //     );
    //     println!("==> Quit process.\r");
    //     return;
    // }

    println!("Quit.\r");

    Ok(())
}

fn print_intro() {
    println!(
        "{}{}{}{goto}==> {lightblue}{bold}{italic}R-Typing - Rust Typing Practis Program{reset}",
        termion::clear::CurrentLine,
        termion::clear::AfterCursor,
        termion::clear::BeforeCursor,
        goto = termion::cursor::Goto(1, 2),
        lightblue = color::Fg(color::LightBlue),
        bold = style::Bold,
        italic = style::Italic,
        reset = style::Reset
    );
    println!("==> Press enter key to start.");
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
    let mut words: Vec<String> = Vec::new();

    for entry in fs::read_dir("/usr/bin").unwrap() {
        words.push(String::from(
            entry.unwrap().path().file_name().unwrap().to_str().unwrap(),
        ));
    }

    let len: usize = words.len();
    let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();
    let i: usize = rnd.gen_range(0..len - level);
    let j: usize = i + level;

    words[i..=j].join(" ")
}
