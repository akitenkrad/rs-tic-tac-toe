use rs_tic_tac_toe::i18n::{Lang, Msg};

#[test]
fn language_defaults_toggles_and_parses_case_insensitively() {
    assert_eq!(Lang::default(), Lang::En);
    assert_eq!(Lang::En.toggle(), Lang::Ja);
    assert_eq!(Lang::Ja.toggle(), Lang::En);
    assert_eq!(Lang::from_str("en"), Some(Lang::En));
    assert_eq!(Lang::from_str("EN"), Some(Lang::En));
    assert_eq!(Lang::from_str("Ja"), Some(Lang::Ja));
    assert_eq!(Lang::from_str("fr"), None);
    assert_eq!(Lang::from_str(""), None);
}

#[test]
fn every_message_is_non_empty_and_translated() {
    for message in Msg::ALL {
        let en = message.text(Lang::En);
        let ja = message.text(Lang::Ja);
        assert!(!en.is_empty(), "missing English text for {message:?}");
        assert!(!ja.is_empty(), "missing Japanese text for {message:?}");
        assert_ne!(en, ja, "untranslated message: {message:?}");
    }
}

#[test]
fn language_supplies_localized_default_names_and_formatted_messages() {
    assert_eq!(Lang::En.default_human_name(), "You");
    assert_eq!(Lang::Ja.default_human_name(), "あなた");
    assert_eq!(Lang::En.default_cpu_name(), "CPU");
    assert_eq!(Lang::Ja.default_cpu_name(), "CPU");
    assert_eq!(Lang::En.move_line(3, "Alice", "X", "a1"), "Move 3: Alice X a1");
    assert_eq!(Lang::Ja.move_line(3, "Alice", "X", "a1"), "手数 3: Alice X a1");
    assert_eq!(Lang::En.turn("You", "X"), "Your turn X");
    assert_eq!(Lang::En.turn("CPU", "O"), "CPU's turn O");
    assert_eq!(Lang::Ja.turn("あなた", "X"), "あなたの番 X");
}
