//
// Typing game
//
use rand::Rng;
use rodio::{source::Source, Decoder};
use std::fs;
use std::fs::File;
use std::io;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

fn main() {
    const PICKUP: usize = 5;
    const TIMEOUT: i32 = 60;

    println!(
        "{}{}{}{goto}==> {lightblue}{bold}{italic}Typing Game{reset}",
        termion::clear::CurrentLine,
        termion::clear::AfterCursor,
        termion::clear::BeforeCursor,
        goto = termion::cursor::Goto(1, 2),
        lightblue = color::Fg(color::LightBlue),
        bold = style::Bold,
        italic = style::Italic,
        reset = style::Reset
    );

    println!("==> Press enter key to start");
    let mut start: String = String::new();
    io::stdin()
        .read_line(&mut start)
        .expect("Failed to read line.");

    // init vector which save words
    let mut words: Vec<String> = Vec::new();

    // push words as string to the vector
    for entry in fs::read_dir("/usr/bin").unwrap() {
        words.push(String::from(
            entry.unwrap().path().file_name().unwrap().to_str().unwrap(),
        ));
    }
    // vector length
    let len: usize = words.len();

    // BGM
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let file = File::open("audio/BGM.mp3").unwrap();
    let source = Decoder::new(file).unwrap();
    let _ = stream_handle.play_raw(source.repeat_infinite().convert_samples());
    thread::sleep(Duration::from_millis(1000));

    // raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();

    loop {
        let stdin: io::Stdin = stdin();

        // thread sender and receiver
        let (tx, rx) = mpsc::channel();

        // init time
        let mut timer: i32 = 0;

        // count 30 sec on background
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

            if timer == TIMEOUT {
                println!(
                    "==> {red}Time up{reset}\r",
                    red = color::Fg(color::Red),
                    reset = style::Reset
                );
                println!("==> Quit process\r");
                break;
            }
        });

        let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();
        let i: usize = rnd.gen_range(0..len - PICKUP);
        let j: usize = i + PICKUP;
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
                        print!("{}{}{}", color::Fg(color::LightCyan), c, style::Reset);
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

        // check string
        if input.trim() == sample_string.trim() {
            let _ = tx.send(());
            println!(
                "==> {green}OK{reset}\r",
                green = color::Fg(color::Green),
                reset = style::Reset
            );
            println!("==> Try next words\r");
        } else {
            println!(
                "==> {red}NG{reset}\r",
                red = color::Fg(color::Red),
                reset = style::Reset
            );
            println!("==> Quit process\r");
            return;
        }
    }
}
