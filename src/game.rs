//! 三目並べのルールと完全読み（UI から独立した純粋なロジック）．

use std::collections::HashMap;

/// 盤上の印．先手は常に `X` を使う．
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
    X,
    O,
}

impl Mark {
    pub fn other(self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
        }
    }
}

/// 対局者．
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Human,
    Cpu,
}

impl Side {
    pub fn other(self) -> Self {
        match self {
            Self::Human => Self::Cpu,
            Self::Cpu => Self::Human,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Human => "あなた",
            Self::Cpu => "CPU",
        }
    }
}

/// 盤上の勝ち筋（横 3 本，縦 3 本，斜め 2 本）．
pub const WIN_LINES: [[usize; 3]; 8] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win { side: Side, line: [usize; 3] },
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveError {
    OutOfRange,
    Occupied,
    GameOver,
}

/// ある手を選んだ後の，現在の手番側から見た完全読みの結果．
/// 数値は終局までの着手数（選んだ手を 1 と数える）．
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Evaluation {
    Win(u8),
    Draw,
    Loss(u8),
}

/// `a1`〜`c3` または読み順の `1`〜`9` を内部番号へ変換する．
pub fn parse_cell(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    match bytes {
        [digit @ b'1'..=b'9'] => Some(usize::from(*digit - b'1')),
        [column, row @ b'1'..=b'3'] => {
            let column = column.to_ascii_lowercase();
            (b'a'..=b'c')
                .contains(&column)
                .then(|| usize::from(*row - b'1') * 3 + usize::from(column - b'a'))
        }
        _ => None,
    }
}

/// 内部番号を座標表記へ変換する．
pub fn cell_name(cell: usize) -> String {
    if cell >= 9 {
        return "?".into();
    }
    let column = char::from(b'a' + (cell % 3) as u8);
    let row = char::from(b'1' + (cell / 3) as u8);
    format!("{column}{row}")
}

/// 進行中の対局．
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub board: [Option<Mark>; 9],
    pub first: Side,
    pub to_move: Side,
    pub history: Vec<(Side, usize)>,
}

impl Game {
    pub fn new(first: Side) -> Self {
        Self {
            board: [None; 9],
            first,
            to_move: first,
            history: Vec::new(),
        }
    }

    pub fn mark_for(&self, side: Side) -> Mark {
        if side == self.first { Mark::X } else { Mark::O }
    }

    pub fn side_for(&self, mark: Mark) -> Side {
        if mark == Mark::X { self.first } else { self.first.other() }
    }

    pub fn legal_moves(&self) -> Vec<usize> {
        if self.outcome().is_some() {
            return Vec::new();
        }
        self.board
            .iter()
            .enumerate()
            .filter_map(|(cell, mark)| mark.is_none().then_some(cell))
            .collect()
    }

    pub fn apply(&mut self, cell: usize) -> Result<(), MoveError> {
        if cell >= self.board.len() {
            return Err(MoveError::OutOfRange);
        }
        if self.outcome().is_some() {
            return Err(MoveError::GameOver);
        }
        if self.board[cell].is_some() {
            return Err(MoveError::Occupied);
        }
        self.board[cell] = Some(self.mark_for(self.to_move));
        self.history.push((self.to_move, cell));
        self.to_move = self.to_move.other();
        Ok(())
    }

    pub fn outcome(&self) -> Option<Outcome> {
        for line in WIN_LINES {
            if let Some(mark) = self.board[line[0]]
                && self.board[line[1]] == Some(mark)
                && self.board[line[2]] == Some(mark)
            {
                return Some(Outcome::Win {
                    side: self.side_for(mark),
                    line,
                });
            }
        }
        self.board.iter().all(Option::is_some).then_some(Outcome::Draw)
    }

    pub fn is_over(&self) -> bool {
        self.outcome().is_some()
    }
}

fn better_for_mover(candidate: Evaluation, current: Evaluation) -> bool {
    match (candidate, current) {
        (Evaluation::Win(a), Evaluation::Win(b)) => a < b,
        (Evaluation::Win(_), _) => true,
        (_, Evaluation::Win(_)) => false,
        (Evaluation::Draw, Evaluation::Loss(_)) => true,
        (Evaluation::Loss(_), Evaluation::Draw) => false,
        (Evaluation::Draw, Evaluation::Draw) => false,
        (Evaluation::Loss(a), Evaluation::Loss(b)) => a > b,
    }
}

fn invert_after_reply(value: Evaluation) -> Evaluation {
    match value {
        Evaluation::Win(turns) => Evaluation::Loss(turns + 1),
        Evaluation::Draw => Evaluation::Draw,
        Evaluation::Loss(turns) => Evaluation::Win(turns + 1),
    }
}

type Memo = HashMap<([Option<Mark>; 9], Mark), Evaluation>;

fn value_with_memo(game: &Game, memo: &mut Memo) -> Evaluation {
    let key = (game.board, game.mark_for(game.to_move));
    if let Some(&value) = memo.get(&key) {
        return value;
    }

    let mut best = None;
    for cell in game.legal_moves() {
        let candidate = move_value_with_memo(game, cell, memo);
        if best.is_none_or(|current| better_for_mover(candidate, current)) {
            best = Some(candidate);
        }
    }
    let value = best.expect("value_with_memo is called only for non-terminal games");
    memo.insert(key, value);
    value
}

fn move_value_with_memo(game: &Game, cell: usize, memo: &mut Memo) -> Evaluation {
    let mut next = game.clone();
    next.apply(cell).expect("move_value_with_memo receives a legal move");
    match next.outcome() {
        Some(Outcome::Win { .. }) => Evaluation::Win(1),
        Some(Outcome::Draw) => Evaluation::Draw,
        None => invert_after_reply(value_with_memo(&next, memo)),
    }
}

/// 現局面を完全読みしたときの結果．終局済みなら `None`．
pub fn position_value(game: &Game) -> Option<Evaluation> {
    if game.is_over() {
        None
    } else {
        Some(value_with_memo(game, &mut HashMap::new()))
    }
}

/// 手番側から見た各マスの評価を返す．埋まったマスと終局後は `None`．
pub fn solve(game: &Game) -> [Option<Evaluation>; 9] {
    let mut result = [None; 9];
    if game.is_over() {
        return result;
    }

    let mut memo = HashMap::new();
    for cell in game.legal_moves() {
        result[cell] = Some(move_value_with_memo(game, cell, &mut memo));
    }
    result
}

/// 最良評価を持つ合法手を盤の読み順で返す．
pub fn best_moves(game: &Game) -> Vec<usize> {
    let evaluations = solve(game);
    let Some(best) = evaluations.iter().flatten().copied().reduce(
        |current, candidate| {
            if better_for_mover(candidate, current) { candidate } else { current }
        },
    ) else {
        return Vec::new();
    };
    evaluations
        .iter()
        .enumerate()
        .filter_map(|(cell, &value)| (value == Some(best)).then_some(cell))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn new_game_assigns_x_to_first_player() {
        let human_first = Game::new(Side::Human);
        assert_eq!(human_first.mark_for(Side::Human), Mark::X);
        assert_eq!(human_first.mark_for(Side::Cpu), Mark::O);
        assert_eq!(human_first.to_move, Side::Human);

        let cpu_first = Game::new(Side::Cpu);
        assert_eq!(cpu_first.mark_for(Side::Cpu), Mark::X);
        assert_eq!(cpu_first.mark_for(Side::Human), Mark::O);
        assert_eq!(cpu_first.to_move, Side::Cpu);
    }

    #[test]
    fn cell_notation_accepts_coordinates_numbers_and_case() {
        for (text, cell) in [
            ("a1", 0),
            ("C1", 2),
            ("b2", 4),
            ("a3", 6),
            ("c3", 8),
            ("1", 0),
            ("3", 2),
            ("7", 6),
            ("9", 8),
        ] {
            assert_eq!(parse_cell(text), Some(cell), "{text}");
            assert_eq!(parse_cell(&cell_name(cell)), Some(cell));
        }
        for invalid in ["", "0", "10", "d1", "a0", "a4", "aa1", " 1"] {
            assert_eq!(parse_cell(invalid), None, "{invalid:?}");
        }
        assert_eq!(cell_name(9), "?");
    }

    #[test]
    fn detects_all_eight_winning_lines() {
        for &line in &WIN_LINES {
            let mut game = Game::new(Side::Human);
            for &cell in &line {
                game.board[cell] = Some(Mark::X);
            }
            assert_eq!(game.outcome(), Some(Outcome::Win { side: Side::Human, line }));
        }
    }

    #[test]
    fn ninth_move_win_takes_precedence_over_draw() {
        let mut game = Game::new(Side::Human);
        game.board = [
            Some(Mark::X),
            Some(Mark::O),
            Some(Mark::O),
            Some(Mark::X),
            Some(Mark::X),
            Some(Mark::O),
            Some(Mark::X),
            Some(Mark::O),
            Some(Mark::X),
        ];
        assert!(matches!(game.outcome(), Some(Outcome::Win { side: Side::Human, .. })));
    }

    #[test]
    fn detects_draw() {
        let mut game = Game::new(Side::Human);
        game.board = [
            Some(Mark::X),
            Some(Mark::O),
            Some(Mark::X),
            Some(Mark::X),
            Some(Mark::O),
            Some(Mark::O),
            Some(Mark::O),
            Some(Mark::X),
            Some(Mark::X),
        ];
        assert_eq!(game.outcome(), Some(Outcome::Draw));
    }

    #[test]
    fn apply_rejects_three_kinds_of_illegal_move_without_mutation() {
        let mut game = Game::new(Side::Cpu);
        assert_eq!(game.apply(9), Err(MoveError::OutOfRange));
        assert_eq!(game, Game::new(Side::Cpu));

        game.apply(4).unwrap();
        let after_first = game.clone();
        assert_eq!(game.apply(4), Err(MoveError::Occupied));
        assert_eq!(game, after_first);

        game.board = [Some(Mark::X), Some(Mark::X), Some(Mark::X), None, None, None, None, None, None];
        let finished = game.clone();
        assert_eq!(game.apply(3), Err(MoveError::GameOver));
        assert_eq!(game, finished);
    }

    #[test]
    fn apply_records_side_and_toggles_turn() {
        let mut game = Game::new(Side::Cpu);
        game.apply(4).unwrap();
        assert_eq!(game.board[4], Some(Mark::X));
        assert_eq!(game.history, vec![(Side::Cpu, 4)]);
        assert_eq!(game.to_move, Side::Human);
        assert_eq!(game.legal_moves(), vec![0, 1, 2, 3, 5, 6, 7, 8]);
    }

    fn better_for_mover(left: Evaluation, right: Evaluation) -> bool {
        match (left, right) {
            (Evaluation::Win(a), Evaluation::Win(b)) => a < b,
            (Evaluation::Win(_), _) => true,
            (_, Evaluation::Win(_)) => false,
            (Evaluation::Draw, Evaluation::Loss(_)) => true,
            (Evaluation::Loss(_), Evaluation::Draw) => false,
            (Evaluation::Draw, Evaluation::Draw) => false,
            (Evaluation::Loss(a), Evaluation::Loss(b)) => a > b,
        }
    }

    fn naive_value(game: &Game) -> Evaluation {
        let mut best = None;
        for cell in game.legal_moves() {
            let mut next = game.clone();
            next.apply(cell).unwrap();
            let candidate = match next.outcome() {
                Some(Outcome::Win { .. }) => Evaluation::Win(1),
                Some(Outcome::Draw) => Evaluation::Draw,
                None => match naive_value(&next) {
                    Evaluation::Win(n) => Evaluation::Loss(n + 1),
                    Evaluation::Draw => Evaluation::Draw,
                    Evaluation::Loss(n) => Evaluation::Win(n + 1),
                },
            };
            if best.is_none_or(|current| better_for_mover(candidate, current)) {
                best = Some(candidate);
            }
        }
        best.expect("naive_value is called only for non-terminal games")
    }

    #[test]
    fn empty_board_is_a_draw_and_every_opening_move_draws() {
        let game = Game::new(Side::Human);
        assert_eq!(position_value(&game), Some(Evaluation::Draw));
        assert_eq!(solve(&game), [Some(Evaluation::Draw); 9]);
        assert_eq!(best_moves(&game), (0..9).collect::<Vec<_>>());
    }

    #[test]
    fn solve_reports_immediate_win_from_movers_viewpoint() {
        let mut game = Game::new(Side::Human);
        for cell in [0, 3, 1, 4] {
            game.apply(cell).unwrap();
        }
        let evaluations = solve(&game);
        assert_eq!(evaluations[2], Some(Evaluation::Win(1)));
        assert_eq!(best_moves(&game), vec![2]);
        assert_eq!(evaluations[0], None);
    }

    #[test]
    fn terminal_position_has_no_best_move() {
        let mut game = Game::new(Side::Human);
        for cell in [0, 3, 1, 4, 2] {
            game.apply(cell).unwrap();
        }
        assert!(game.is_over());
        assert!(best_moves(&game).is_empty());
    }

    fn collect_reachable(game: &Game, seen: &mut HashSet<([Option<Mark>; 9], Side, Side)>) {
        if !seen.insert((game.board, game.first, game.to_move)) || game.is_over() {
            return;
        }
        for cell in game.legal_moves() {
            let mut next = game.clone();
            next.apply(cell).unwrap();
            collect_reachable(&next, seen);
        }
    }

    #[test]
    fn memoized_solver_matches_naive_minimax_on_every_reachable_position() {
        let mut seen = HashSet::new();
        collect_reachable(&Game::new(Side::Human), &mut seen);
        collect_reachable(&Game::new(Side::Cpu), &mut seen);

        for (board, first, to_move) in seen {
            let game = Game {
                board,
                first,
                to_move,
                history: Vec::new(),
            };
            if game.is_over() {
                assert_eq!(position_value(&game), None);
                assert_eq!(solve(&game), [None; 9]);
                continue;
            }
            assert_eq!(position_value(&game), Some(naive_value(&game)), "{game:?}");
            let evaluations = solve(&game);
            for cell in game.legal_moves() {
                let mut next = game.clone();
                next.apply(cell).unwrap();
                let expected = match next.outcome() {
                    Some(Outcome::Win { .. }) => Evaluation::Win(1),
                    Some(Outcome::Draw) => Evaluation::Draw,
                    None => match naive_value(&next) {
                        Evaluation::Win(n) => Evaluation::Loss(n + 1),
                        Evaluation::Draw => Evaluation::Draw,
                        Evaluation::Loss(n) => Evaluation::Win(n + 1),
                    },
                };
                assert_eq!(evaluations[cell], Some(expected), "{game:?}, cell={cell}");
            }
        }
    }

    #[test]
    fn best_moves_prefer_fast_wins_and_slow_losses_on_every_position() {
        let mut seen = HashSet::new();
        collect_reachable(&Game::new(Side::Human), &mut seen);

        for (board, first, to_move) in seen {
            let game = Game {
                board,
                first,
                to_move,
                history: Vec::new(),
            };
            let values: Vec<_> = solve(&game).into_iter().flatten().collect();
            let expected = if let Some(fastest) = values
                .iter()
                .filter_map(|value| match value {
                    Evaluation::Win(turns) => Some(*turns),
                    _ => None,
                })
                .min()
            {
                Some(Evaluation::Win(fastest))
            } else if values.contains(&Evaluation::Draw) {
                Some(Evaluation::Draw)
            } else {
                values
                    .iter()
                    .filter_map(|value| match value {
                        Evaluation::Loss(turns) => Some(*turns),
                        _ => None,
                    })
                    .max()
                    .map(Evaluation::Loss)
            };
            let evaluations = solve(&game);
            let actual: Vec<_> = best_moves(&game).into_iter().map(|cell| evaluations[cell].unwrap()).collect();
            assert!(actual.iter().all(|value| Some(*value) == expected), "{game:?}");
            assert_eq!(actual.len(), values.iter().filter(|&&value| Some(value) == expected).count());
        }
    }
}
