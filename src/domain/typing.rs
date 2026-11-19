#[derive(Default)]
pub struct TypingSession {
    target: String,
    inputs: Vec<char>,
    typed_count: usize,
    incorrects: usize,
    missed_chars: Vec<char>,
    target_len: usize,
}

impl TypingSession {
    pub fn new(target: String) -> Self {
        Self {
            target_len: target.chars().count(),
            target,
            ..Self::default()
        }
    }

    pub fn push_char(&mut self, ch: char, practice_mode: bool) -> bool {
        let expected = self.target.chars().nth(self.inputs.len());
        let correct = expected == Some(ch);
        self.typed_count += 1;
        if correct || !practice_mode {
            self.inputs.push(ch);
        }
        if !correct {
            self.incorrects += 1;
            if let Some(expected) = expected {
                self.missed_chars.push(expected);
            }
        }
        correct
    }

    pub fn pop_char(&mut self) -> Option<char> {
        self.inputs.pop()
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn inputs(&self) -> &[char] {
        &self.inputs
    }

    pub fn typed_count(&self) -> usize {
        self.typed_count
    }

    pub fn incorrects(&self) -> usize {
        self.incorrects
    }

    pub fn missed_chars(&self) -> &[char] {
        &self.missed_chars
    }

    pub fn is_complete(&self) -> bool {
        self.inputs.len() >= self.target_len
    }
}
