use ratatui::crossterm::event::{KeyCode, KeyEvent};
use rs_tic_tac_toe::app::App;
use rs_tic_tac_toe::game::Side;
use rs_tic_tac_toe::i18n::Lang;
use rs_tic_tac_toe::record::Record;
use std::time::Duration;

fn press(app: &mut App, code: KeyCode) {
    app.on_key(KeyEvent::from(code));
}

#[test]
fn arrow_navigation_wraps_within_the_three_by_three_board() {
    let mut app = App::new();
    app.start();
    app.cursor = 0;
    press(&mut app, KeyCode::Left);
    assert_eq!(app.cursor, 2);
    press(&mut app, KeyCode::Up);
    assert_eq!(app.cursor, 8);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.cursor, 6);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.cursor, 0);
}

#[test]
fn changing_number_selection_does_not_place_until_confirmation() {
    let mut app = App::new();
    app.start();
    press(&mut app, KeyCode::Char('1'));
    press(&mut app, KeyCode::Char('9'));
    assert!(app.game.history.is_empty());
    assert_eq!(app.cursor, 8);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.game.history, vec![(Side::Human, 8)]);
}

#[test]
fn replay_backward_cancels_the_active_placement() {
    let record = Record {
        first: Side::Human,
        names: [Some("Alice".into()), Some("Bob".into())],
        moves: vec![0, 4, 1],
    };
    let mut app = App::new_replay(record, Duration::from_millis(1_500), true, false, Lang::En);
    press(&mut app, KeyCode::Right);
    assert!(app.placement.is_some());
    press(&mut app, KeyCode::Left);
    assert!(app.placement.is_none());
    assert!(app.cpu.is_none());
    assert_eq!(app.replay().unwrap().position, 0);
    assert!(app.game.history.is_empty());
}
