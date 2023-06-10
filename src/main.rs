//
// Typing game
//
use std::io;
use std::fs;
use rand::Rng;
use std::thread;
use std::time::Duration;
use std::process;
use termion;
use termion::{color, style};

fn main() {
    let pickup_words : usize = 5;

    println!("==> Typing game");

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
        // count 30 sec on background
        let _handle = thread::spawn(|| {
            for _sec in 1..30 {
                // todo: through the variable from thread to the any function

                // thread.sleep
                thread::sleep(Duration::from_millis(1000));
            }

            println!(
                "==> {color_red}Time up{reset} (30 sec)",
                color_red = color::Fg(color::Red),
                reset = style::Reset
            );
            println!("==> Quit process");

            process::exit(0);
        });

        let mut rnd = rand::thread_rng();
        let i = rnd.gen_range(0..len - pickup_words);
        let j = i + pickup_words;
        let sample_string:String = words[i..=j].join(" ");

        println!(
            "==> {color_red}Type following words.{reset}",
            color_red = color::Fg(color::Red),
            reset = style::Reset
        );
        println!("{}", sample_string);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");

        // check string
        if input.trim() == sample_string.trim() {
            println!(
                "==> 🟢{color_green}OK{reset}💮",
                color_green = color::Fg(color::Green),
                reset = style::Reset
            );
            println!("==> Try next words");
        } else {
            println!(
                "==> ❌{color_red}NG{reset}",
                color_red = color::Fg(color::Red),
                reset = style::Reset
            );
            println!("==> Quit process");
            process::exit(0);
        }
    }
}
