use clap::{arg, Command};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rodio::Source;
use rodio::{source::SineWave, OutputStream};
use std::collections::HashMap;
use std::fs;
use std::io::{self};
use std::io::{stdin, stdout, BufReader, Cursor, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style, terminal_size};

fn main() -> io::Result<()> {
    let matches = Command::new("rtyping")
        .author("Tomokatsu Kumata")
        .about("R-Typing: A terminal-based typing practice app.")
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

    // 引数処理
    let timeout: i32 = *matches.get_one::<i32>("timeout").expect("expect number");
    let level: usize = *matches.get_one::<usize>("level").expect("expect number");
    let sound: bool = matches.get_flag("sound");
    let freq: f32 = *matches.get_one::<f32>("freq").expect("expect frequency");

    // sine 波生成ストリーミング
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // スレッド間通信チャンネル
    let (mt_tx, mt_rx) = mpsc::channel(); // メイン -> タイマー
    let (tt_tx, tt_rx) = mpsc::channel(); // タイマー -> メイン
    let (bgm_tx, bgm_rx) = mpsc::channel();

    // 横幅を固定（例: 80）
    let fixed_width: u16 = 80;
    // 現在のターミナルサイズを取得
    let (width, _height) = terminal_size().unwrap_or((80, 24));
    // 使用する幅を固定幅と現在の横幅の大きい方にする
    let use_width = std::cmp::max(width, fixed_width);

    // サンプルテキスト定数
    let text = "The quick brown fox jumps over the lazy dog. The sun was shining brightly over the hill, and the wind blew gently through the trees. The birds chirped merrily as they fluttered from branch to branch, and the scent of fresh flowers filled the air. It was a peaceful morning in the village, with everyone going about their daily routines. The children played in the field, their laughter echoing through the valley. A few farmers were tending to their crops, while the cows grazed lazily in the meadow. Suddenly, a loud noise echoed from the nearby forest, and the animals in the field froze in place. The birds stopped singing, and even the wind seemed to hold its breath. The noise grew louder, and the ground began to shake slightly. From the trees emerged a figure, tall and mysterious, cloaked in a dark robe. The villagers watched in awe as the figure approached the center of the village, moving with a grace that was almost unnatural. The figure raised a hand, and the air around them seemed to shimmer. With a voice that was soft yet commanding, the figure spoke. I have come to bring a message, they said. The villagers gathered around, their curiosity piqued. The figure paused for a moment, as if gathering their thoughts, before continuing. The time has come for change, they said. The winds of destiny are shifting, and a new chapter is about to begin. The villagers exchanged confused glances, unsure of what the figure meant. Some whispered among themselves, wondering if this was some kind of omen or prophecy. Others felt a sense of unease, as if the figure’s presence brought a chill to the air. The figure lowered their hand, and the shimmering aura around them faded. Do not be afraid, the figure said, sensing the fear in the crowd. This is not a warning, but an invitation. An invitation to join me on a journey that will change everything. The villagers were silent, unsure of how to respond. They had never seen anyone like this before, and the idea of leaving their peaceful village was unsettling. But the figure was undeterred. Come with me, they urged. There is much you do not know, much that is hidden from your eyes. But together, we can uncover the truth and shape the future. Slowly, one by one, the villagers began to approach the figure. Some were hesitant, while others were eager to know more. The children, sensing something extraordinary, crowded around, their eyes wide with wonder. The figure smiled, a faint and mysterious smile, as they led the group toward the edge of the village. As they walked, the air seemed to grow heavier, and the atmosphere became charged with a strange energy. The villagers had no idea what lay ahead, but they knew that their lives were about to change forever. And so, with a sense of trepidation and excitement, they followed the figure into the unknown.";

    // 音の処理
    if sound {
        thread::spawn(move || loop {
            if bgm_rx.try_recv().is_ok() {
                break;
            }
            play_audio();
        });
    }

    // イントロを表示
    print_intro();

    // 目標単語列表示
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut inputs: Vec<String> = Vec::new(); // ユーザ入力保持 Vec 用意
    let mut incorrect_chars = 0; // 入力間違い文字数
    // 目標単語列取得
    // let target_string = load_words(level);
    // n-gram を使用して生成
    let target_string = generate_markov_chain(text, 3, level);
    let target_str = &target_string;
    let line = "-".repeat(use_width as usize);
    print!("{}\r\n", line);
    print!("{}", termion::cursor::Save); // カーソル位置保存
    print!("{}\r\n", target_string);
    print!("{}\r\n", line);
    print!("{}", termion::cursor::Restore); // カーソル位置復元 (入力位置がここになる)
    print!("{}", termion::cursor::BlinkingBar); // カーソルをバーに変形
    io::stdout().flush().unwrap();

    // タイマーの表示とカウント
    let timer = Arc::new(Mutex::new(0));
    let timer_clone = Arc::clone(&timer);
    let timer_thread = thread::spawn(move || {
        loop {
            if tt_rx.try_recv().is_ok() {
                return;
            }

            if let Ok(mut timer_value) = timer_clone.try_lock() {
                if *timer_value > timeout {
                    break;
                }
                print_timer(*timer_value);
                *timer_value += 1;
            }

            thread::sleep(Duration::from_secs(1));
        }

        print!(
            "\r\n\r\n{}{}⏰Time up. Press any key.{}\r\n",
            termion::cursor::Down(1),
            color::Fg(color::Red),
            style::Reset
        );
        mt_tx.send(()).unwrap();
    });

    // ユーザ入力を監視する
    for evt in stdin.events() {
        // Todo: Change stdin.events
        if mt_rx.try_recv().is_ok() {
            break;
        }

        if let Ok(Event::Key(key)) = evt {
            match key {
                Key::Ctrl('c') | Key::Esc | Key::Char('\n') => {
                    print!("\r\n");
                    tt_tx.send(()).unwrap();
                    break;
                }
                Key::Backspace => {
                    if !inputs.is_empty() {
                        let l = inputs.len();
                        print!("{}", termion::cursor::Left(1));
                        print!("{}", target_str.chars().nth(l - 1).unwrap().to_string());
                        print!("{}", termion::cursor::Left(1));
                        inputs.pop();
                    }
                }
                Key::Char(c) => {
                    let l = inputs.len();

                    if target_str.chars().nth(l) == Some(c) {
                        print!("{}{}{}", color::Fg(color::Green), c, style::Reset);

                        // Produce a <FREQ> beep sound
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
    }

    timer_thread.join().unwrap();

    print!("\r\n\r\nQuit.\r\n");

    // WPM 計算と表示
    let elapsed_timer = *timer.lock().unwrap() - 1;
    print!("{:<13}: {} sec\r\n", "⌚Total Time", elapsed_timer);
    print!("{:<13}: {} chars\r\n", "🔢Total Typing", inputs.len());
    print!("{:<13}: {} chars\r\n", "❌Misses", incorrect_chars);
    print!(
        "{:<13}: {}{:.2}{}\r\n",
        "🎯WPM",
        color::Fg(color::Green),
        calc_wpm(inputs.len(), elapsed_timer, incorrect_chars),
        style::Reset
    );
    print!("{}", termion::cursor::BlinkingBlock); // カーソルをブロックに変形

    bgm_tx.send(()).unwrap();
    Ok(())
}

fn print_intro() {
    print!(
        "{}{}{}🦀 R-Typing - Typing Practice Program ⌨️{}\r\n",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        color::Fg(color::LightBlue),
        style::Reset
    );
    print!("🚀Press *ENTER* key to start.\r\n");

    let mut start: String = String::new();

    io::stdin()
        .read_line(&mut start)
        .expect("Failed to read line.");
}

fn print_timer(timer: i32) {
    print!("{}", termion::cursor::Save);
    print!("{}", termion::cursor::Goto(1, 3));
    print!("{}", termion::clear::CurrentLine);
    print!("Time: {} sec", timer);
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

fn _load_words(level: usize) -> String {
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

// マルコフ連鎖関数
fn generate_markov_chain(text: &str, n: usize, level: usize) -> String {
    //
    let limit_sentence = level + 9;

    // サンプルテキストを単語に分割
    let words: Vec<&str> = text.split_whitespace().collect();

    // n-gram モデルを作成
    let mut markov_chain: HashMap<Vec<&str>, Vec<&str>> = HashMap::new();

    for i in 0..(words.len() - n) {
        let key = words[i..i + n].to_vec();
        let value = words[i + n];
        markov_chain.entry(key).or_insert_with(Vec::new).push(value);
    }

    // 初期状態としてランダムな開始単語を選ぶ
    let mut rng = thread_rng();
    let start_index = rand::Rng::gen_range(&mut rng, 0..words.len() - n);
    let mut current_state = words[start_index..start_index + n].to_vec();

    // 次の単語をランダムに選びながら生成
    let mut result = current_state.clone();
    for _ in 0..limit_sentence {
        if let Some(next_words) = markov_chain.get(&current_state) {
            let next_word = next_words.choose(&mut rng).unwrap();
            result.push(*next_word);
            current_state.push(*next_word);
            current_state.remove(0); // 最初の単語を削除して次の状態に移動
        } else {
            break; // マッチするパターンが見つからない場合、終了
        }
    }

    // 結果を結合して文を返す
    result.join(" ")
}
