//
// Typing game
//
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::process;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use termion;
use termion::{color, style};

fn main() {
    let pickup_words: usize = 4;

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

    print!("==> Press return/enter key to start");
    io::stdout().flush().unwrap();
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

    loop {
        // thread sender and receiver
        let (tx, rx) = mpsc::channel();
        let mut timer: i32 = 0;

        // count 30 sec on background
        let _handle: thread::JoinHandle<()> = thread::spawn(move || loop {
            // print!("{}", termion::cursor::Save);
            // print!("{}Time: {}sec", termion::cursor::Goto(1, 1), timer);
            // print!("{}", termion::cursor::Restore);
            // io::stdout().flush().unwrap();

            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            thread::sleep(Duration::from_millis(1000));
            timer += 1;

            if timer == 30 {
                println!(
                    "==> 🔴{red}Time up{reset} (30 sec)",
                    red = color::Fg(color::Red),
                    reset = style::Reset
                );
                println!("==> Quit process");
                process::exit(0);
            }
        });

        let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();
        let i: usize = rnd.gen_range(0..len - pickup_words);
        let j: usize = i + pickup_words;
        let sample_string: String = words[i..=j].join(" ");

        println!(
            "==> {red}Type following words.{reset}",
            red = color::Fg(color::Red),
            reset = style::Reset
        );
        println!("{}", sample_string);

        // todo: change stdin input method
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");

        // check string
        if input.trim() == sample_string.trim() {
            let _ = tx.send(());
            println!(
                "==> 🟢{green}OK{reset}💮",
                green = color::Fg(color::Green),
                reset = style::Reset
            );
            println!("==> Try next words");
        } else {
            println!(
                "==> ❌{red}NG{reset}",
                red = color::Fg(color::Red),
                reset = style::Reset
            );
            println!("==> Quit process");
            process::exit(0);
        }
    }
}
