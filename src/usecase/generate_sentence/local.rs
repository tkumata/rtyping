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
    if words.len() <= n {
        return words.join(" ");
    }

    let mut markov_chain: HashMap<Vec<&str>, Vec<&str>> = HashMap::new();

    for i in 0..(words.len() - n) {
        let Some(key_slice) = words.get(i..i + n) else {
            continue;
        };
        let Some(&value) = words.get(i + n) else {
            continue;
        };
        markov_chain
            .entry(key_slice.to_vec())
            .or_default()
            .push(value);
    }

    let mut rng = rng();
    let start_index = rng.random_range(0..words.len() - n);
    let Some(current_state_slice) = words.get(start_index..start_index + n) else {
        return words.join(" ");
    };
    let mut current_state = current_state_slice.to_vec();
    let mut result = current_state.clone();
    let mut current_len = result
        .iter()
        .map(|word| word.chars().count())
        .sum::<usize>()
        + result.len().saturating_sub(1);

    while current_len < target_chars {
        if let Some(next_words) = markov_chain.get(&current_state) {
            let Some(next_word) = next_words.choose(&mut rng) else {
                break;
            };
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
