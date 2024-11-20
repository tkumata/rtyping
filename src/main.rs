use clap::{arg, Command};
use rand::Rng;
use std::fs;
use std::io;
use std::io::{stdin, stdout, BufReader, Cursor, Write};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

fn main() {
    let matches = Command::new("rtyping")
        .author("Tomokatsu Kumata")
        .about("Typing test program.")
        .arg(
            arg!(-t --timeout <TIMEOUT> "Seconds")
                .required(false)
                .default_value("60")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            arg!(-l --level <LEVEL> "Number of words")
                .required(false)
                .default_value("9")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(arg!(-s --sound "Turn BGM on"))
        .get_matches();

    let timeout: i32 = *matches.get_one::<i32>("timeout").expect("expect number");
    let level: usize = *matches.get_one::<usize>("level").expect("expect number");
    let sound: bool = matches.get_flag("sound");

    println!(
        "{}{}{}{goto}==> {lightblue}{bold}{italic}R-typing - Typing Test Program{reset}",
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

    let mut words: Vec<String> = Vec::new();

    for entry in fs::read_dir("/usr/bin").unwrap() {
        words.push(String::from(
            entry.unwrap().path().file_name().unwrap().to_str().unwrap(),
        ));
    }

    let len: usize = words.len();

    if sound {
        let _handle = thread::spawn(|| loop {
            play_audio();
        });
    }

    let mut stdout = stdout().into_raw_mode().unwrap();

    loop {
        let stdin: io::Stdin = stdin();
        let (tx, rx) = mpsc::channel();
        let mut timer: i32 = 0;

        let _handle: thread::JoinHandle<()> = thread::spawn(move || loop {
            print!("{}", termion::cursor::Save);
            print!("{}", termion::cursor::Goto(1, 1));
            print!("{}", termion::clear::CurrentLine);
            print!("Time: {}sec", timer);
            print!("{}", termion::cursor::Restore);
            io::stdout().flush().unwrap();

            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            thread::sleep(Duration::from_millis(1000));

            timer += 1;

            if timer == timeout {
                println!(
                    "==> {red}Time up{reset}\r",
                    red = color::Fg(color::Red),
                    reset = style::Reset
                );
                println!("==> Quit process.\r");
                std::process::exit(0);
            }
        });

        let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();
        let i: usize = rnd.gen_range(0..len - level);
        let j: usize = i + level;
        let sample_string: String = words[i..=j].join(" ");
        let sample_str: &str = &sample_string;

        println!("==> Type following words.\r");
        println!(
            "{color}{}{reset}\r",
            sample_string,
            color = color::Fg(color::LightCyan),
            reset = style::Reset
        );

        let mut inputs: Vec<String> = Vec::new();

        for evt in stdin.events() {
            match evt.unwrap() {
                Event::Key(Key::Ctrl('c')) => {
                    return;
                }
                Event::Key(Key::Char('\n')) => {
                    print!("\r\n");
                    break;
                }
                Event::Key(Key::Backspace) => {
                    print!("{}", termion::cursor::Left(1));
                    print!(" ");
                    print!("{}", termion::cursor::Left(1));
                    inputs.pop();
                }
                Event::Key(Key::Char(c)) => {
                    let l = inputs.len();
                    if sample_str.chars().nth(l) == Some(c) {
                        print!("{}", c);
                    } else {
                        print!("{}{}{}", color::Fg(color::Red), c, style::Reset);
                    }
                    inputs.push(String::from(c.to_string()));
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }

        let input = inputs.join("");

        if input.trim() == sample_string.trim() {
            let _ = tx.send(());
            println!(
                "==> {green}OK{reset}\r",
                green = color::Fg(color::Green),
                reset = style::Reset
            );
            println!("==> Try next words.\r");
        } else {
            println!(
                "==> {red}NG{reset}\r",
                red = color::Fg(color::Red),
                reset = style::Reset
            );
            println!("==> Quit process.\r");
            return;
        }
    }
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
