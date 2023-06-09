//
// Typing game
//
use std::io;
use std::fs;
use rand::Rng;
use std::thread;
use std::time::Duration;
use std::process;

fn main() {
    println!("==> Typing game");

    let mut words = Vec::new();

    for entry in fs::read_dir("/usr/bin").unwrap() {
        words.push(
            String::from(entry.unwrap().path().file_name().unwrap().to_str().unwrap())
        );
    }

    let len = words.len();

    loop {
        // count 30 sec on background
        let _handle = thread::spawn(|| {
            for _sec in 1..30 {
                thread::sleep(Duration::from_millis(1000));
            }

            println!("==> Time up (30 sec)");
            println!("==> Quit process");

            process::exit(0);
        });

        let mut rnd = rand::thread_rng();
        let i = rnd.gen_range(0..len - 5);
        let j = i + 5;
        let sample_string:String = words[i..=j].join(" ");

        println!("==> Type following words.");
        println!("{}", sample_string);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");

        // check string
        if input.trim() == sample_string.trim() {
            println!("==> OK");
        } else {
            println!("==> NG");
            println!("==> Quit process");
            process::exit(0);
        }
    }
}
