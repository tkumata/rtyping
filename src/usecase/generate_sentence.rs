use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::{self};
use std::io::Write;
use termion;
use termion::terminal_size;

pub fn markov(text: &str, level: usize) -> Result<String, io::Error> {
    // 横幅を固定（例: 80）
    let fixed_width: u16 = 80;

    // 現在のターミナルサイズを取得
    let (width, _height) = terminal_size().unwrap_or((80, 24));

    // 使用する幅を固定幅と現在の横幅の大きい方にする
    let use_width = std::cmp::max(width, fixed_width);

    // n-gram を使用して生成
    let target_string = generate_markov_chain(text, 3, level);
    let line = "-".repeat(use_width as usize);

    print!("{}\r\n", line);
    print!("{}", termion::cursor::Save); // カーソル位置保存
    print!("{}\r\n", target_string);
    print!("{}\r\n", line);
    print!("{}", termion::cursor::Restore); // カーソル位置復元 (入力位置がここになる)
    print!("{}", termion::cursor::BlinkingBar); // カーソルをバーに変形
    io::stdout().flush().unwrap();

    Ok(target_string)
}

// マルコフ連鎖関数
fn generate_markov_chain(text: &str, n: usize, level: usize) -> String {
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
    for _ in 0..level {
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

