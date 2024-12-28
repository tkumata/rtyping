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

    // サンプルテキスト定数 1000 単語
    let text = "The world we live in is a complex and dynamic place, shaped by countless factors that influence our daily lives. From the natural environment to the intricacies of human society, every aspect of existence is interwoven with challenges and opportunities. Understanding this interconnectedness is crucial for individuals who seek to make a positive impact on their communities and the planet as a whole. One of the most pressing issues facing humanity today is climate change. The warming of the planet due to human activities has far-reaching consequences, affecting weather patterns, ecosystems, and economies. Scientists have warned for decades about the dangers of excessive carbon emissions, yet meaningful action has often been delayed by political inertia and conflicting interests. Despite these challenges, there is hope. Renewable energy technologies, such as solar and wind power, have advanced significantly, offering viable alternatives to fossil fuels. Additionally, grassroots movements advocating for environmental justice have gained momentum, reminding governments and corporations of their responsibility to protect the Earth. Education plays a pivotal role in addressing global challenges. It is not merely about acquiring knowledge but about fostering critical thinking and empathy. In an increasingly interconnected world, understanding diverse perspectives is essential. Access to quality education remains unequal, with millions of children unable to attend school due to poverty, conflict, or systemic barriers. Bridging this gap requires not only funding but also innovative approaches that leverage technology. Online learning platforms, for example, have the potential to reach underserved populations, offering them opportunities to learn and grow. Technology itself is a double-edged sword. While it has revolutionized communication, transportation, and healthcare, it has also introduced new dilemmas. The rise of artificial intelligence, for instance, poses ethical questions about autonomy and accountability. Who should be held responsible when an algorithm makes a life-altering decision? Furthermore, the proliferation of social media has transformed how people interact, often amplifying division and misinformation. Balancing the benefits of technological progress with its potential drawbacks is a challenge that requires careful deliberation and collaborative effort. Human relationships form the backbone of society. From family bonds to friendships and professional networks, these connections shape our identities and experiences. In modern times, however, the nature of relationships is evolving. Virtual interactions often supplement or replace face-to-face communication, leading to both benefits and drawbacks. On one hand, technology enables people to maintain relationships across great distances. On the other hand, it can create a sense of isolation, as digital interactions may lack the depth and nuance of physical presence. Cultivating meaningful connections requires intentionality and effort, regardless of the medium. Health and well-being are fundamental to human flourishing. Yet, access to healthcare is unevenly distributed, both within and between nations. Preventable diseases continue to claim lives, even as medical advancements offer solutions. Public health initiatives must address both immediate concerns and systemic inequalities. Mental health, often overlooked, is another critical component of overall well-being. The stigma surrounding mental illness prevents many from seeking help, underscoring the need for awareness and support. Promoting a holistic approach to health involves not just treating illness but also fostering environments that enable individuals to thrive. Cultural heritage is another dimension of human experience that deserves attention. Art, music, literature, and traditions reflect the richness of human creativity and the diversity of perspectives. Preserving this heritage is essential for maintaining a sense of identity and continuity. However, globalization and modernization often threaten cultural uniqueness. Striking a balance between embracing progress and honoring traditions is a delicate task. Encouraging intercultural dialogue and exchange can help bridge divides while celebrating the uniqueness of different cultures. Economics plays a central role in shaping societal outcomes. The distribution of resources influences everything from education to healthcare to infrastructure. Inequality remains a persistent issue, with wealth concentrated in the hands of a few. Addressing this disparity requires systemic reforms that promote fairness and opportunity. Social entrepreneurship, which combines profit-making with social impact, is an example of how business can contribute to positive change. Similarly, community-driven initiatives demonstrate the power of collective action in addressing local challenges. Leadership is another critical factor in driving progress. Effective leaders inspire trust and cooperation, guiding societies toward shared goals. However, leadership is not confined to positions of authority. Every individual has the potential to lead by example, whether in their personal lives or within their communities. Integrity, empathy, and resilience are qualities that define impactful leaders. Cultivating these traits requires self-reflection and a commitment to continuous growth. The arts and sciences, though often viewed as distinct domains, share a common goal of exploring and understanding the world. Scientific inquiry seeks to uncover the laws of nature, while the arts express the human condition. Together, they enrich our understanding of existence and provide tools for addressing complex challenges. Collaboration between these disciplines can yield innovative solutions, such as using storytelling to communicate scientific findings or employing design principles to create sustainable technologies. The concept of happiness is one that has intrigued philosophers and psychologists alike. What does it mean to live a good life? While material possessions and achievements often bring temporary satisfaction, enduring happiness is usually found in relationships, purpose, and self-acceptance. Practices such as mindfulness and gratitude can enhance well-being, reminding individuals to focus on what truly matters. At the same time, systemic factors like social support and economic stability play a significant role in enabling individuals to pursue happiness. Ultimately, the future is shaped by the choices we make today. Each decision, no matter how small, contributes to a larger tapestry of outcomes. While the challenges facing humanity are daunting, they are not insurmountable. Collaboration, innovation, and compassion are key to building a world where everyone can thrive. As individuals, we have the power to make a difference, whether by advocating for change, supporting others, or simply striving to live authentically. The journey toward a better future begins with a single step, taken with courage and hope.";

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
