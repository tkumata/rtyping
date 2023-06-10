//
// Typing game
//
use std::fs;
use std::io;
use std::io::Write;
use rand::Rng;
use std::thread;
use std::time::Duration;
use std::process;
use termion;
use termion::{color, style};

fn main() {
    let pickup_words : usize = 4;

    println!(
        "{}==> {lightgreen}{bold}{italic}Typing Game{reset}",
        termion::clear::All,
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
        // count 30 sec on background
        // todo 正解して戻ってきたら初期化する
        let _handle = thread::spawn(|| {
            for _sec in 0..30 {
                print!("{}", termion::cursor::Save);
                print!(
                    "{}Time: {} sec",
                    termion::cursor::Goto(1, 1),
                    _sec
                );
                print!("{}", termion::cursor::Restore);
                io::stdout().flush().unwrap();

                // thread.sleep
                thread::sleep(Duration::from_secs(1));
            }

            println!(
                "==> 🔴{red}Time up{reset} (30 sec)",
                red = color::Fg(color::Red),
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
