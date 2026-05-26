#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteState {
    Pending,
    Hit,
    Ok,
    Missed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RhythmNote {
    ch: char,
    lane_position: u32,
    state: NoteState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RhythmStats {
    pub typed: usize,
    pub correct: usize,
    pub hit: usize,
    pub ok: usize,
    pub miss: usize,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmJudgement {
    Hit,
    Ok,
    Miss,
}

impl RhythmJudgement {
    pub fn label(self) -> &'static str {
        match self {
            RhythmJudgement::Hit => "Hit",
            RhythmJudgement::Ok => "OK",
            RhythmJudgement::Miss => "Miss",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RhythmSession {
    notes: Vec<RhythmNote>,
    typed: usize,
    hit: usize,
    ok: usize,
    miss: usize,
    last_judgement: Option<RhythmJudgement>,
    elapsed_seconds: f64,
    speed: u8,
}

impl RhythmSession {
    pub const MARK_COLUMN: i16 = 3;
    pub const HIT_TOLERANCE_COLUMNS: f64 = 0.75;
    const INITIAL_GAP_COLUMNS: f64 = 12.0;

    pub fn new(target: &str, speed: u8) -> Self {
        let notes = build_notes(target);

        Self {
            notes,
            typed: 0,
            hit: 0,
            ok: 0,
            miss: 0,
            last_judgement: None,
            elapsed_seconds: 0.0,
            speed,
        }
    }

    pub fn set_elapsed_seconds(&mut self, elapsed_seconds: f64) {
        self.elapsed_seconds = elapsed_seconds.max(0.0);
        self.mark_passed_notes_missed();
    }

    pub fn push_char(&mut self, ch: char) -> RhythmJudgement {
        self.typed += 1;

        let Some((index, judgement)) = self.find_matching_note(ch) else {
            self.miss += 1;
            self.last_judgement = Some(RhythmJudgement::Miss);
            return RhythmJudgement::Miss;
        };

        if let Some(note) = self.notes.get_mut(index) {
            note.state = match judgement {
                RhythmJudgement::Hit => NoteState::Hit,
                RhythmJudgement::Ok => NoteState::Ok,
                RhythmJudgement::Miss => NoteState::Missed,
            };
        }
        match judgement {
            RhythmJudgement::Hit => self.hit += 1,
            RhythmJudgement::Ok => self.ok += 1,
            RhythmJudgement::Miss => self.miss += 1,
        }
        self.last_judgement = Some(judgement);
        judgement
    }

    pub fn visible_chars(&self, width: u16) -> Vec<(u16, char)> {
        let width = i16::try_from(width).unwrap_or(i16::MAX);
        self.notes
            .iter()
            .filter(|note| note.state == NoteState::Pending)
            .filter_map(|note| {
                let column = rounded_column(self.note_column(note))?;
                (column >= Self::MARK_COLUMN && column < width)
                    .then_some((u16::try_from(column).ok()?, note.ch))
            })
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.notes
            .iter()
            .all(|note| note.state != NoteState::Pending)
    }

    pub fn stats(&self) -> RhythmStats {
        let correct_count = self.correct_count();
        let accuracy = if self.typed == 0 {
            0.0
        } else {
            let correct = u32::try_from(correct_count).unwrap_or(u32::MAX);
            let typed = u32::try_from(self.typed).unwrap_or(u32::MAX);
            f64::from(correct) / f64::from(typed) * 100.0
        };
        RhythmStats {
            typed: self.typed,
            correct: correct_count,
            hit: self.hit,
            ok: self.ok,
            miss: self.miss,
            accuracy,
        }
    }

    pub fn last_judgement(&self) -> Option<RhythmJudgement> {
        self.last_judgement
    }

    fn correct_count(&self) -> usize {
        self.hit + self.ok
    }

    fn find_matching_note(&self, ch: char) -> Option<(usize, RhythmJudgement)> {
        self.notes
            .iter()
            .enumerate()
            .filter(|(_, note)| note.state == NoteState::Pending && note.ch == ch)
            .filter_map(|(index, note)| {
                let distance = (self.note_column(note) - f64::from(Self::MARK_COLUMN)).abs();
                judgement_for_distance(distance).map(|judgement| (index, distance, judgement))
            })
            .min_by(|(_, left_distance, _), (_, right_distance, _)| {
                left_distance.total_cmp(right_distance)
            })
            .map(|(index, _, judgement)| (index, judgement))
    }

    fn mark_passed_notes_missed(&mut self) {
        let passed_indices = self
            .notes
            .iter()
            .enumerate()
            .filter(|(_, note)| note.state == NoteState::Pending)
            .filter(|(_, note)| {
                self.note_column(note) < f64::from(Self::MARK_COLUMN) - Self::HIT_TOLERANCE_COLUMNS
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        for index in passed_indices {
            if let Some(note) = self.notes.get_mut(index) {
                note.state = NoteState::Missed;
                self.miss += 1;
                self.last_judgement = Some(RhythmJudgement::Miss);
            }
        }
    }

    fn note_column(&self, note: &RhythmNote) -> f64 {
        Self::INITIAL_GAP_COLUMNS + f64::from(note.lane_position)
            - self.elapsed_seconds * f64::from(self.speed)
    }
}

fn judgement_for_distance(distance: f64) -> Option<RhythmJudgement> {
    const HIT_DISTANCE_COLUMNS: f64 = 0.25;

    if distance <= HIT_DISTANCE_COLUMNS {
        Some(RhythmJudgement::Hit)
    } else if distance <= RhythmSession::HIT_TOLERANCE_COLUMNS {
        Some(RhythmJudgement::Ok)
    } else {
        None
    }
}

fn build_notes(target: &str) -> Vec<RhythmNote> {
    let mut lane_position = 0_u32;
    target
        .chars()
        .enumerate()
        .filter_map(|(source_position, ch)| {
            lane_position = lane_position.saturating_add(note_gap(source_position, ch));
            (!ch.is_whitespace()).then_some(RhythmNote {
                ch,
                lane_position,
                state: NoteState::Pending,
            })
        })
        .collect()
}

fn note_gap(source_position: usize, ch: char) -> u32 {
    if ch.is_whitespace() {
        return 4;
    }

    let source_position = u32::try_from(source_position).unwrap_or(u32::MAX);
    let char_value = u32::from(ch);
    1 + (source_position.wrapping_mul(7).wrapping_add(char_value) % 8)
}

fn rounded_column(value: f64) -> Option<i16> {
    let rounded = value.round();
    if rounded < f64::from(i16::MIN) || rounded > f64::from(i16::MAX) {
        return None;
    }

    #[expect(clippy::cast_possible_truncation)]
    Some(rounded as i16)
}

#[cfg(test)]
mod tests {
    use super::{RhythmJudgement, RhythmSession, build_notes};

    #[test]
    fn generated_note_gaps_are_not_constant() {
        let notes = build_notes("keyboard");
        let gaps = notes
            .windows(2)
            .filter_map(|pair| {
                let left = pair.first()?;
                let right = pair.get(1)?;
                Some(right.lane_position - left.lane_position)
            })
            .collect::<Vec<_>>();

        assert!(gaps.windows(2).any(|pair| pair.first() != pair.get(1)));
    }

    #[test]
    fn spaces_are_not_input_targets() {
        let mut session = RhythmSession::new("a b", 2);
        session.set_elapsed_seconds(3.0);

        assert_eq!(session.push_char(' '), RhythmJudgement::Miss);
        assert_eq!(session.stats().miss, 1);
    }

    #[test]
    fn exact_note_within_inner_tolerance_is_hit() {
        let mut session = RhythmSession::new("a", 2);
        session.set_elapsed_seconds(5.5);

        assert_eq!(session.push_char('a'), RhythmJudgement::Hit);
        let stats = session.stats();
        assert_eq!(stats.typed, 1);
        assert_eq!(stats.correct, 1);
        assert_eq!(stats.hit, 1);
        assert_eq!(stats.ok, 0);
        assert_eq!(stats.miss, 0);
        assert_eq!(session.last_judgement(), Some(RhythmJudgement::Hit));
        assert!(session.is_complete());
    }

    #[test]
    fn outer_tolerance_match_is_ok() {
        let mut session = RhythmSession::new("a", 2);
        session.set_elapsed_seconds(5.2);

        assert_eq!(session.push_char('a'), RhythmJudgement::Ok);
        let stats = session.stats();
        assert_eq!(stats.correct, 1);
        assert_eq!(stats.hit, 0);
        assert_eq!(stats.ok, 1);
        assert_eq!(stats.miss, 0);
        assert_eq!(session.last_judgement(), Some(RhythmJudgement::Ok));
    }

    #[test]
    fn passed_note_is_missed() {
        let mut session = RhythmSession::new("a", 2);
        session.set_elapsed_seconds(6.5);

        assert!(session.is_complete());
        assert_eq!(session.stats().miss, 1);
        assert_eq!(session.last_judgement(), Some(RhythmJudgement::Miss));
    }
}
