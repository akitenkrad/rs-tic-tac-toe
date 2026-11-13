//! CPU の思考と乱数．

use crate::game::{Game, best_moves};
use crate::i18n::{Lang, Msg};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Perfect,
}

impl Difficulty {
    pub fn next(self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Perfect,
            Self::Perfect => Self::Easy,
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Easy => Msg::Easy.text(lang),
            Self::Normal => Msg::Normal.text(lang),
            Self::Perfect => Msg::Perfect.text(lang),
        }
    }

    /// 最善手を選ぶ確率（%）．
    pub fn accuracy(self) -> u32 {
        match self {
            Self::Easy => 20,
            Self::Normal => 65,
            Self::Perfect => 100,
        }
    }
}

/// 依存を増やさないための小さな xorshift 乱数．
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn seeded() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self::from_seed(seed)
    }

    pub fn from_seed(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    /// `low..=high` の一様乱数．
    pub fn range(&mut self, low: usize, high: usize) -> usize {
        debug_assert!(low <= high);
        low + self.next_u64() as usize % (high - low + 1)
    }

    /// `[0, 1)` の一様乱数．
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1_u64 << 24) as f32
    }

    pub fn chance(&mut self, percent: u32) -> bool {
        self.range(0, 99) < percent as usize
    }
}

/// CPU の着手を選ぶ．終局局面では `None` を返す．
pub fn choose(game: &Game, difficulty: Difficulty, rng: &mut Rng) -> Option<usize> {
    let legal = game.legal_moves();
    if legal.is_empty() {
        return None;
    }

    let candidates = if difficulty == Difficulty::Perfect || rng.chance(difficulty.accuracy()) {
        best_moves(game)
    } else {
        legal
    };
    Some(candidates[rng.range(0, candidates.len() - 1)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Outcome, Side, best_moves};

    #[test]
    fn difficulty_metadata_matches_specification() {
        assert_eq!(Difficulty::Easy.label(Lang::En), "Easy");
        assert_eq!(Difficulty::Normal.label(Lang::Ja), "ふつう");
        assert_eq!(Difficulty::Perfect.label(Lang::En), "Perfect (full search)");
        assert_eq!(Difficulty::Easy.accuracy(), 20);
        assert_eq!(Difficulty::Normal.accuracy(), 65);
        assert_eq!(Difficulty::Perfect.accuracy(), 100);
        assert_eq!(Difficulty::Easy.next(), Difficulty::Normal);
        assert_eq!(Difficulty::Normal.next(), Difficulty::Perfect);
        assert_eq!(Difficulty::Perfect.next(), Difficulty::Easy);
    }

    #[test]
    fn rng_boundary_operations_stay_in_range() {
        let mut rng = Rng::from_seed(0);
        for _ in 0..100 {
            assert_eq!(rng.range(4, 4), 4);
            assert!((3..=7).contains(&rng.range(3, 7)));
            assert!((0.0..1.0).contains(&rng.unit()));
            assert!(!rng.chance(0));
            assert!(rng.chance(100));
        }
    }

    #[test]
    fn perfect_always_selects_a_best_move() {
        let mut game = Game::new(Side::Human);
        for cell in [0, 4, 1] {
            game.apply(cell).unwrap();
        }
        for seed in 1..=100 {
            let mut rng = Rng::from_seed(seed);
            assert!(best_moves(&game).contains(&choose(&game, Difficulty::Perfect, &mut rng).unwrap()));
        }
    }

    #[test]
    fn every_difficulty_returns_only_legal_moves() {
        let mut game = Game::new(Side::Human);
        for cell in [0, 4, 8] {
            game.apply(cell).unwrap();
        }
        for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Perfect] {
            let mut rng = Rng::from_seed(7);
            for _ in 0..100 {
                let cell = choose(&game, difficulty, &mut rng).unwrap();
                assert!(game.legal_moves().contains(&cell));
            }
        }

        let mut terminal = Game::new(Side::Human);
        for cell in [0, 3, 1, 4, 2] {
            terminal.apply(cell).unwrap();
        }
        assert_eq!(choose(&terminal, Difficulty::Perfect, &mut Rng::from_seed(1)), None);
    }

    fn perfect_never_loses_from(game: &Game, cpu: Side) {
        if let Some(outcome) = game.outcome() {
            assert_ne!(
                outcome,
                Outcome::Win {
                    side: cpu.other(),
                    line: match outcome {
                        Outcome::Win { line, .. } => line,
                        Outcome::Draw => [0, 0, 0],
                    }
                }
            );
            return;
        }

        let moves = if game.to_move == cpu { best_moves(game) } else { game.legal_moves() };
        for cell in moves {
            let mut next = game.clone();
            next.apply(cell).unwrap();
            perfect_never_loses_from(&next, cpu);
        }
    }

    #[test]
    fn perfect_cpu_never_loses_as_first_or_second_against_any_play() {
        perfect_never_loses_from(&Game::new(Side::Cpu), Side::Cpu);
        perfect_never_loses_from(&Game::new(Side::Human), Side::Cpu);
    }

    #[test]
    fn perfect_play_against_itself_always_draws() {
        for seed in 1..=64 {
            let mut game = Game::new(Side::Human);
            let mut rng = Rng::from_seed(seed);
            while !game.is_over() {
                let cell = choose(&game, Difficulty::Perfect, &mut rng).unwrap();
                game.apply(cell).unwrap();
            }
            assert_eq!(game.outcome(), Some(Outcome::Draw), "seed={seed}");
        }
    }
}
