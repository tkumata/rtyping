pub(super) fn normalize_sentence(sentence: &str, target_chars: usize) -> String {
    let cleaned = sentence.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut normalized = String::with_capacity(cleaned.len());

    for ch in cleaned.chars() {
        if ch.is_ascii() && !ch.is_control() {
            normalized.push(ch);
        } else if ch.is_whitespace() {
            normalized.push(' ');
        }
    }

    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return "typing practice fallback text"
            .chars()
            .take(target_chars)
            .collect();
    }

    trimmed.chars().take(target_chars).collect()
}
