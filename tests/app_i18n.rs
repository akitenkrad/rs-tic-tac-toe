use std::time::Duration;

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use rs_tic_tac_toe::app::App;
use rs_tic_tac_toe::game::Side;
use rs_tic_tac_toe::i18n::Lang;
use rs_tic_tac_toe::record::Record;

#[test]
fn title_l_toggles_language_and_updates_default_player_name() {
    let mut app = App::new();
    assert_eq!(app.lang, Lang::En);
    assert_eq!(app.side_name(Side::Human), "You");

    app.on_key(KeyEvent::from(KeyCode::Char('l')));
    assert_eq!(app.lang, Lang::Ja);
    assert_eq!(app.side_name(Side::Human), "あなた");

    app.on_key(KeyEvent::from(KeyCode::Char('l')));
    assert_eq!(app.lang, Lang::En);
}

#[test]
fn replay_l_advances_without_toggling_language() {
    let record = Record {
        first: Side::Human,
        names: [None, None],
        moves: vec![0],
    };
    let mut app = App::new_replay(record, Duration::from_millis(1_500), true, false, Lang::Ja);
    app.on_key(KeyEvent::from(KeyCode::Char('l')));
    assert_eq!(app.lang, Lang::Ja);
    assert_eq!(app.replay().unwrap().position, 1);
}
