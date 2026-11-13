//! `.tttrec` 棋譜の読み書きと局面再構成．

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use crate::game::{Game, MoveError, Side, cell_name, parse_cell};

/// 先手，表示名，着手列を保持する棋譜．
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub first: Side,
    /// `[人間側, CPU側]` の表示名．
    pub names: [String; 2],
    pub moves: Vec<usize>,
}

impl Record {
    /// `ply` 手を適用した局面を返す．棋譜の範囲外なら `None`．
    pub fn game_at(&self, ply: usize) -> Option<Game> {
        game_at(self, ply)
    }

    /// 対局者の表示名を返す．
    pub fn name(&self, side: Side) -> &str {
        match side {
            Side::Human => &self.names[0],
            Side::Cpu => &self.names[1],
        }
    }
}

/// 棋譜の構文・合法性エラー．すべて入力上の行番号を保持する．
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordError {
    UnknownKey { line: usize, key: String },
    DuplicateKey { line: usize, key: String },
    HeaderAfterMove { line: usize },
    InvalidFirst { line: usize, value: String },
    EmptyName { line: usize, key: String },
    InvalidMove { line: usize, value: String },
    CellOccupied { line: usize, cell: usize },
    MoveAfterGameOver { line: usize },
}

impl RecordError {
    pub fn line(&self) -> usize {
        match self {
            Self::UnknownKey { line, .. }
            | Self::DuplicateKey { line, .. }
            | Self::HeaderAfterMove { line }
            | Self::InvalidFirst { line, .. }
            | Self::EmptyName { line, .. }
            | Self::InvalidMove { line, .. }
            | Self::CellOccupied { line, .. }
            | Self::MoveAfterGameOver { line } => *line,
        }
    }
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: ", self.line())?;
        match self {
            Self::UnknownKey { key, .. } => write!(f, "unknown key '{key}'"),
            Self::DuplicateKey { key, .. } => write!(f, "duplicate key '{key}'"),
            Self::HeaderAfterMove { .. } => write!(f, "header after first move"),
            Self::InvalidFirst { value, .. } => write!(f, "invalid first side '{value}'"),
            Self::EmptyName { key, .. } => write!(f, "{key} name must not be empty"),
            Self::InvalidMove { value, .. } => write!(f, "invalid move '{value}'"),
            Self::CellOccupied { cell, .. } => {
                write!(f, "cell {} is already occupied", cell_name(*cell))
            }
            Self::MoveAfterGameOver { .. } => write!(f, "move after game over"),
        }
    }
}

impl Error for RecordError {}

/// UTF-8 の `.tttrec` テキストを解析する．
pub fn parse(text: &str) -> Result<Record, RecordError> {
    let mut first = Side::Human;
    let mut names = ["あなた".to_string(), "CPU".to_string()];
    let mut moves = Vec::new();
    let mut game = Game::new(first);
    let mut seen_keys = HashSet::new();
    let mut saw_move = false;

    for (index, raw_line) in text.lines().enumerate() {
        let line = index + 1;
        let content = raw_line.split_once('#').map_or(raw_line, |(before, _)| before).trim();
        if content.is_empty() {
            continue;
        }

        if let Some((raw_key, raw_value)) = content.split_once('=') {
            if saw_move {
                return Err(RecordError::HeaderAfterMove { line });
            }
            let key = raw_key.trim();
            let value = raw_value.trim();
            if !matches!(key, "first" | "human" | "cpu") {
                return Err(RecordError::UnknownKey { line, key: key.to_string() });
            }
            if !seen_keys.insert(key.to_string()) {
                return Err(RecordError::DuplicateKey { line, key: key.to_string() });
            }
            match key {
                "first" => {
                    first = match value {
                        "human" => Side::Human,
                        "cpu" => Side::Cpu,
                        _ => {
                            return Err(RecordError::InvalidFirst {
                                line,
                                value: value.to_string(),
                            });
                        }
                    };
                    game = Game::new(first);
                }
                "human" | "cpu" => {
                    if value.is_empty() {
                        return Err(RecordError::EmptyName { line, key: key.to_string() });
                    }
                    names[usize::from(key == "cpu")] = value.to_string();
                }
                _ => unreachable!(),
            }
            continue;
        }

        saw_move = true;
        if game.is_over() {
            return Err(RecordError::MoveAfterGameOver { line });
        }
        let cell = parse_cell(content).ok_or_else(|| RecordError::InvalidMove {
            line,
            value: content.to_string(),
        })?;
        match game.apply(cell) {
            Ok(()) => moves.push(cell),
            Err(MoveError::Occupied) => {
                return Err(RecordError::CellOccupied { line, cell });
            }
            Err(MoveError::GameOver) => {
                return Err(RecordError::MoveAfterGameOver { line });
            }
            Err(MoveError::OutOfRange) => unreachable!("parse_cell returned an invalid cell"),
        }
    }

    Ok(Record { first, names, moves })
}

/// 棋譜を座標表記の標準テキストへ変換する．
pub fn to_text(record: &Record) -> String {
    let first = match record.first {
        Side::Human => "human",
        Side::Cpu => "cpu",
    };
    let mut text = format!(
        "# rs-tic-tac-toe record v1\nfirst = {first}\nhuman = {}\ncpu = {}\n\n",
        record.names[0], record.names[1]
    );
    for &cell in &record.moves {
        text.push_str(&cell_name(cell));
        text.push('\n');
    }
    text
}

/// `ply` 手を適用した局面を，履歴を含めて再構成する．
pub fn game_at(record: &Record, ply: usize) -> Option<Game> {
    if ply > record.moves.len() {
        return None;
    }
    let mut game = Game::new(record.first);
    for &cell in record.moves.iter().take(ply) {
        game.apply(cell).ok()?;
    }
    Some(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Evaluation, Outcome, Side, best_moves, position_value};

    fn assert_error(input: &str, expected: RecordError, message: &str) {
        let error = parse(input).unwrap_err();
        assert_eq!(error, expected);
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn parses_both_move_notations_case_insensitively() {
        let record = parse(
            "# rs-tic-tac-toe record v1\n\
             first = cpu # X is the CPU\n\
             human = Alice\n\
             cpu = Bob\n\
             \n\
             A1\n\
             5\n\
             c3\n",
        )
        .unwrap();

        assert_eq!(record.first, Side::Cpu);
        assert_eq!(record.names, ["Alice", "Bob"]);
        assert_eq!(record.moves, vec![0, 4, 8]);
    }

    #[test]
    fn applies_header_defaults_and_ignores_comments_and_blank_lines() {
        let record = parse("# comment\n\na1 # opening\nB2\n").unwrap();
        assert_eq!(record.first, Side::Human);
        assert_eq!(record.names, ["あなた", "CPU"]);
        assert_eq!(record.moves, vec![0, 4]);
    }

    #[test]
    fn rejects_unknown_duplicate_and_late_headers_with_line_numbers() {
        assert_error(
            "mystery = value\n",
            RecordError::UnknownKey {
                line: 1,
                key: "mystery".into(),
            },
            "line 1: unknown key 'mystery'",
        );
        assert_error(
            "first = human\nfirst = cpu\n",
            RecordError::DuplicateKey {
                line: 2,
                key: "first".into(),
            },
            "line 2: duplicate key 'first'",
        );
        assert_error(
            "a1\nhuman = Alice\n",
            RecordError::HeaderAfterMove { line: 2 },
            "line 2: header after first move",
        );
    }

    #[test]
    fn rejects_invalid_header_values_and_empty_names() {
        assert_error(
            "first = robot\n",
            RecordError::InvalidFirst {
                line: 1,
                value: "robot".into(),
            },
            "line 1: invalid first side 'robot'",
        );
        assert_error(
            "human = # absent\n",
            RecordError::EmptyName {
                line: 1,
                key: "human".into(),
            },
            "line 1: human name must not be empty",
        );
        assert_error(
            "cpu =\n",
            RecordError::EmptyName { line: 1, key: "cpu".into() },
            "line 1: cpu name must not be empty",
        );
    }

    #[test]
    fn rejects_malformed_out_of_range_and_occupied_moves() {
        let invalid = ["a0", "d1", "0", "10", "a1 extra", "aa1", "１"];
        for text in invalid {
            assert_error(
                &format!("# heading\n{text}\n"),
                RecordError::InvalidMove { line: 2, value: text.into() },
                &format!("line 2: invalid move '{text}'"),
            );
        }
        assert_error(
            "a1\n1\n",
            RecordError::CellOccupied { line: 2, cell: 0 },
            "line 2: cell a1 is already occupied",
        );
    }

    #[test]
    fn rejects_a_move_after_game_over() {
        assert_error(
            "a1\nb1\na2\nb2\na3\nc3\n",
            RecordError::MoveAfterGameOver { line: 6 },
            "line 6: move after game over",
        );
    }

    #[test]
    fn to_text_uses_coordinates_and_round_trips() {
        let original = Record {
            first: Side::Cpu,
            names: ["Alice Smith".into(), "Bob CPU".into()],
            moves: vec![0, 4, 8],
        };
        let text = to_text(&original);
        assert!(text.starts_with("# rs-tic-tac-toe record v1\n"));
        assert!(text.contains("first = cpu\n"));
        assert!(text.ends_with("a1\nb2\nc3\n"));
        assert_eq!(parse(&text).unwrap(), original);
    }

    #[test]
    fn reconstructs_every_ply_with_history() {
        let record = parse("first = cpu\na1\nb2\nc3\n").unwrap();
        for ply in 0..=record.moves.len() {
            let game = record.game_at(ply).unwrap();
            assert_eq!(game.history.len(), ply);
            assert_eq!(game.to_move, if ply % 2 == 0 { Side::Cpu } else { Side::Human });
        }
        assert!(record.game_at(record.moves.len() + 1).is_none());
    }

    #[test]
    fn parses_every_example_record() {
        let examples = [
            include_str!("../examples/records/perfect-draw.tttrec"),
            include_str!("../examples/records/x-wins.tttrec"),
            include_str!("../examples/records/unfinished.tttrec"),
        ];
        let records = examples.map(|text| parse(text).unwrap());
        assert_eq!(records[0].game_at(records[0].moves.len()).unwrap().outcome(), Some(Outcome::Draw));
        assert!(matches!(
            records[1].game_at(records[1].moves.len()).unwrap().outcome(),
            Some(Outcome::Win { side: Side::Human, .. })
        ));
        assert_eq!(records[2].game_at(records[2].moves.len()).unwrap().outcome(), None);
    }

    #[test]
    fn perfect_draw_uses_only_best_moves_and_x_wins_through_a_fork() {
        let draw = parse(include_str!("../examples/records/perfect-draw.tttrec")).unwrap();
        let mut game = Game::new(draw.first);
        for &cell in &draw.moves {
            assert!(best_moves(&game).contains(&cell), "{cell} is not a best move in {game:?}");
            game.apply(cell).unwrap();
        }
        assert_eq!(game.outcome(), Some(Outcome::Draw));

        let x_wins = parse(include_str!("../examples/records/x-wins.tttrec")).unwrap();
        let fork = x_wins.game_at(5).unwrap();
        assert!(matches!(position_value(&fork), Some(Evaluation::Loss(_))));
    }
}
