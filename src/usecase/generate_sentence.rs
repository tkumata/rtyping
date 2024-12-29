use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::{self};

use crate::domain::entity;

pub fn markov(level: usize) -> Result<String, io::Error> {
    // サンプルテキスト
    let mut sample_contents = String::new();
    match entity::get_sample() {
        Ok(contents) => {
            sample_contents = contents;
        }
        Err(err) => {
            eprintln!("Failed to read file: {}", err);
        }
    }
    let text = sample_contents.as_str();

    // n-gram を使用して生成
    let target_string = generate_markov_chain(text, 4, level);

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
