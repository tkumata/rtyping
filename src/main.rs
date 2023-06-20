// クレートの呼び出し。
use clap::{arg, Command};
use rand::Rng;
use rodio::{source::Source, Decoder};
use std::fs;
use std::io;
use std::io::{stdin, stdout, Cursor, Write};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

fn main() {
    // 引数オプションの定義
    let matches = Command::new("rtyping")
        .author("Tomokatsu Kumata")
        .version("v1.0.0")
        .about("typing app")
        .arg(
            arg!(-t --timeout <TIMEOUT>)
                .required(false)
                .default_value("60")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            arg!(-l --level <LEVEL>)
                .required(false)
                .default_value("4")
                .value_parser(clap::value_parser!(usize)),
        )
        .get_matches();

    // 引数から timeout を抜き出す。ここまでは借用状態なので &i32 型。
    let to: &i32 = matches.get_one::<i32>("timeout").expect("msg");
    let lv: &usize = matches.get_one::<usize>("level").expect("msg");

    // 借用中なので clone で値そのものを複製する。
    let timeout: i32 = to.clone();
    let level: usize = lv.clone();

    // 定数定義
    // const は常に普遍。覆い隠しもできない。必ず型宣言すること。必ず大文字で。
    // const PICKUP: usize = 5;

    // 標準出力マクロ println!() は改行あり。すなわち flush() も付いてくる。
    // print!() は改行なし。良きタイミングで flush() しないといけない。
    println!(
        "{}{}{}{goto}==> {lightblue}{bold}{italic}Typing Game{reset}",
        termion::clear::CurrentLine,  // Warp だと clear:All の挙動が
        termion::clear::AfterCursor,  // おかしいので現在行と前後行を
        termion::clear::BeforeCursor, // clear するようにする
        goto = termion::cursor::Goto(1, 2),
        lightblue = color::Fg(color::LightBlue),
        bold = style::Bold,
        italic = style::Italic,
        reset = style::Reset
    );

    println!("==> Press enter key to start.");

    // String 型の可変変数定義。
    // String 型とリテラル型 (&str) は違うものなので注意。
    let mut start: String = String::new();

    // Canonical mode で標準入力させる。
    io::stdin()
        .read_line(&mut start)
        .expect("==> Failed to read line.");

    // 空ベクターの定義
    let mut words: Vec<String> = Vec::new();

    // fs::read_dir でディレクトリ内を取得する。
    for entry in fs::read_dir("/usr/bin").unwrap() {
        // ベクターに String 型で push する。
        // DirEntry 型から直接 String 型にできないから一旦 to_str でリテラルにする。
        words.push(String::from(
            entry.unwrap().path().file_name().unwrap().to_str().unwrap(),
        ));
    }

    // ベクター内の要素数を出す。
    let len: usize = words.len();

    // BGM
    // stream handler 生成
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // build 時にソース以外のファイルを梱包するために include_bytes マクロを使う。
    // include_bytes は byte データで File::open と違うので注意。
    let bytes = include_bytes!("../audio/BGM.mp3");

    // bytes データなので Cursor 型に変換する。
    let cursor = Cursor::new(bytes);

    // 今回は mp3 なので Cursor データをデコードする。
    let source = Decoder::new(cursor).unwrap();

    // stream handler にデータソースを食わせる。その際、無限ループオプションを付加する。
    let _ = stream_handle.play_raw(source.repeat_infinite().convert_samples());

    // thread を用いて裏で sleep させる。
    thread::sleep(Duration::from_millis(1000));

    // 画面操作を行うので tty を raw mode にする。
    let mut stdout = stdout().into_raw_mode().unwrap();

    loop {
        // ここに必要。なぜここに記述か失念。
        let stdin: io::Stdin = stdin();

        // thread 制御のため sender と receiver を生成する。
        let (tx, rx) = mpsc::channel();

        let mut timer: i32 = 0;

        // thread handler 生成。裏で時間をカウントする。
        // move で変数の受け渡す許可。loop で処理をループする。
        let _handle: thread::JoinHandle<()> = thread::spawn(move || loop {
            // 現在のカーソル位置を保存
            print!("{}", termion::cursor::Save);
            // 表示したい場所にカーソルを移動
            print!("{}", termion::cursor::Goto(1, 1));
            // 桁が増えた時ゴミが残るので現在行をクリアする
            print!("{}", termion::clear::CurrentLine);
            // 時間表示
            print!("Time: {}sec", timer);
            // 元の場所にカーソルを移動
            print!("{}", termion::cursor::Restore);
            // flush して反映させる。
            io::stdout().flush().unwrap();

            // スレッド制御の信号受信待受。
            match rx.try_recv() {
                // 受信したら thread loop を break する。
                // 結果的に timer 変数が初期化することになる。
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            // 裏で sleep する。
            thread::sleep(Duration::from_millis(1000));

            // 変数の中身を覆い隠す。
            timer += 1;

            // 指定数値なら loop を break する。
            if timer == timeout {
                println!(
                    "==> {red}Time up{reset}\r",
                    red = color::Fg(color::Red),
                    reset = style::Reset
                );
                println!("==> Quit process.\r");
                break;
            }
        });

        // 乱数インスタンス生成
        let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();
        // 乱数生成
        // gen_range は usize 型の引数
        let i: usize = rnd.gen_range(0..len - level);
        let j: usize = i + level;
        // ベクターから乱数の添字の場所の文字列を取得する。
        let sample_string: String = words[i..=j].join(" ");
        // String 型からリテラルへ変換 (後で使うから)
        let sample_str: &str = &sample_string;

        // 標準出力。raw mode だから \r で終わらせる。
        println!("==> Type following words.\r");
        println!(
            "{color}{}{reset}\r",
            sample_string,
            color = color::Fg(color::LightCyan),
            reset = style::Reset
        );

        // 入力された char を貯めるベクター生成。
        let mut inputs: Vec<String> = Vec::new();
        // キー入力待受ループ。
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
                    // backspace の実装
                    print!("{}", termion::cursor::Left(1));
                    print!(" ");
                    print!("{}", termion::cursor::Left(1));
                    // ベクターから一文字削除
                    inputs.pop();
                }
                Event::Key(Key::Char(c)) => {
                    let l = inputs.len();
                    if sample_str.chars().nth(l) == Some(c) {
                        // あってれば LightCyan 文字にする。
                        print!("{}{}{}", color::Fg(color::LightCyan), c, style::Reset);
                    } else {
                        // typo してたら赤文字にする。
                        print!("{}{}{}", color::Fg(color::Red), c, style::Reset);
                    }
                    inputs.push(String::from(c.to_string()));
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }
        // loop を抜けたらベクターの中身を join して Stringa 型で保存する。
        let input = inputs.join("");

        // 出題文字列と入力文字列を比較する。
        if input.trim() == sample_string.trim() {
            // 時間をカウントしてる thread に停止命令を出す。
            let _ = tx.send(());
            // 標準出力
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
