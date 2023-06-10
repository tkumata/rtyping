//
// Typing game
//
use std::fs;
use std::io;
use std::io::{Write};
use rand::Rng;
use std::thread;
use std::time::Duration;
use std::process;
use termion;
use termion::{color, style};
use std::sync::mpsc::{self, TryRecvError};

fn main() {
    let pickup_words : usize = 4;

    println!(
        "{}{}==> {lightgreen}{bold}{italic}Typing Game{reset}",
        termion::clear::All,
        termion::cursor::Goto(1, 2),
        lightgreen = color::Fg(color::LightGreen),
        bold = style::Bold,
        italic = style::Italic,
        reset = style::Reset
    );

    // init vector which save words
    let mut words = Vec::new();

    // push words as string to the vector
    for entry in fs::read_dir("/usr/bin").unwrap() {
        words.push(
            String::from(entry.unwrap().path().file_name().unwrap().to_str().unwrap())
        );
    }

    // vector length
    let len = words.len();

    loop {
        // thread sender and receiver
        let (tx, rx) = mpsc::channel();
        let mut timer = 0;

        // count 30 sec on background
        let _handle = thread::spawn(move || loop {
            print!("{}", termion::cursor::Save);
            print!(
                "{}Time: {} ms",
                termion::cursor::Goto(1, 1),
                timer
            );
            print!("{}", termion::cursor::Restore);
            io::stdout().flush().unwrap();

            // thread.sleep
            thread::sleep(Duration::from_millis(1000));

            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    // println!("Terminating.");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            if timer == 30 {
                println!(
                    "==> 🔴{red}Time up{reset} (30 sec)",
                    red = color::Fg(color::Red),
                    reset = style::Reset
                );
                println!("==> Quit process");
                process::exit(0);
            }

            timer += 1;
        });

        let mut rnd = rand::thread_rng();
        let i = rnd.gen_range(0..len - pickup_words);
        let j = i + pickup_words;
        let sample_string:String = words[i..=j].join(" ");

        println!(
            "==> {red}Type following words.{reset}",
            red = color::Fg(color::Red),
            reset = style::Reset
        );
        println!("{}", sample_string);

        let mut input = String::new();
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
