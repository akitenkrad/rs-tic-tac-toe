use rs_tic_tac_toe::game::{Game, MoveError, Outcome, Side};
use rs_tic_tac_toe::record::parse;

#[test]
fn board_index_boundaries_preserve_state_on_error() {
    let mut game = Game::new(Side::Human);
    game.apply(0).unwrap();
    game.apply(8).unwrap();
    let before = game.clone();

    assert_eq!(game.apply(9), Err(MoveError::OutOfRange));
    assert_eq!(game, before);
}

#[test]
fn record_position_boundaries_are_exact() {
    let record = parse("a1\nb2\nc3\n").unwrap();
    assert_eq!(record.game_at(0).unwrap().history.len(), 0);
    assert_eq!(record.game_at(3).unwrap().history.len(), 3);
    assert!(record.game_at(4).is_none());
}

#[test]
fn terminal_positions_have_no_legal_moves() {
    let record = parse("a1\na2\nb1\nb2\nc1\n").unwrap();
    let game = record.game_at(5).unwrap();
    assert!(matches!(game.outcome(), Some(Outcome::Win { .. })));
    assert!(game.legal_moves().is_empty());
}
