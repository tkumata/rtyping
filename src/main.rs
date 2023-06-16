//
// Typing game
//
use rand::Rng;
use std::fs;
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
    let pickup_words: usize = 1;
    const TIMEOUT: i32 = 60;

    println!(
        "{}{}{}{goto}==> {lightgreen}{bold}{italic}Typing Game{reset}",
        termion::clear::CurrentLine,
        termion::clear::AfterCursor,
        termion::clear::BeforeCursor,
        goto = termion::cursor::Goto(1, 2),
        lightgreen = color::Fg(color::LightGreen),
        bold = style::Bold,
        italic = style::Italic,
        reset = style::Reset
    );

    println!("==> Press return/enter key to start");
    let mut start: String = String::new();
    io::stdin()
        .read_line(&mut start)
        .expect("Failed to read line.");

    let mut stdout = stdout().into_raw_mode().unwrap();

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

    loop {
        let stdin: io::Stdin = stdin();

        // thread sender and receiver
        let (tx, rx) = mpsc::channel();
        let mut timer: i32 = 0;

        // count 30 sec on background
        let _handle: thread::JoinHandle<()> = thread::spawn(move || loop {
            print!("{}", termion::cursor::Save);
            print!("{}Time: {}sec", termion::cursor::Goto(1, 1), timer);
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
        let i: usize = rnd.gen_range(0..len - pickup_words);
        let j: usize = i + pickup_words;
        let sample_string: String = words[i..=j].join(" ");

        println!(
            "==> {red}Type following words.{reset}\r",
            red = color::Fg(color::Red),
            reset = style::Reset
        );
        println!("{}\r", sample_string);

        let mut inputs: Vec<String> = Vec::new();
        for evt in stdin.events() {
            match evt.unwrap() {
                Event::Key(Key::Ctrl('c')) => {
                    return;
                }
                Event::Key(Key::Char('\n')) => {
                    write!(stdout, "\r\n").unwrap();
                    break;
                }
                Event::Key(Key::Backspace) => {
                    write!(stdout, "{}", termion::cursor::Left(1)).unwrap();
                    write!(stdout, " ").unwrap();
                    write!(stdout, "{}", termion::cursor::Left(1)).unwrap();
                    inputs.pop();
                }
                Event::Key(Key::Char(c)) => {
                    write!(stdout, "{}", c).unwrap();
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
