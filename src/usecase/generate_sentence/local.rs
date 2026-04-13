use rand::RngExt;
use rand::prelude::IndexedRandom;
use rand::rng;
use std::collections::HashMap;

use crate::domain::entity;

pub(super) fn generate_local_sentence(target_chars: usize) -> String {
    let sampling_contents = entity::get_sample();
    generate_markov_chain(&sampling_contents, 4, target_chars)
}

fn generate_markov_chain(text: &str, n: usize, target_chars: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut markov_chain: HashMap<Vec<&str>, Vec<&str>> = HashMap::new();

    for i in 0..(words.len() - n) {
        let key = words[i..i + n].to_vec();
        let value = words[i + n];
        markov_chain.entry(key).or_default().push(value);
    }

    let mut rng = rng();
    let start_index = rng.random_range(0..words.len() - n);
    let mut current_state = words[start_index..start_index + n].to_vec();
    let mut result = current_state.clone();
    let mut current_len = result
        .iter()
        .map(|word| word.chars().count())
        .sum::<usize>()
        + result.len().saturating_sub(1);

    while current_len < target_chars {
        if let Some(next_words) = markov_chain.get(&current_state) {
            let Some(next_word) = next_words.choose(&mut rng) else { break };
            result.push(*next_word);
            current_len += next_word.chars().count() + 1;
            current_state.push(*next_word);
            current_state.remove(0);
        } else {
            break;
        }
    }

    result.join(" ")
}
