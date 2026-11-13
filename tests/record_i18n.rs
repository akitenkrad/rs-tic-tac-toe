use rs_tic_tac_toe::game::Side;
use rs_tic_tac_toe::i18n::Lang;
use rs_tic_tac_toe::record::{parse, to_text};

#[test]
fn omitted_names_are_resolved_in_the_display_language() {
    let record = parse("first = human\na1\n").unwrap();
    assert_eq!(record.name(Side::Human, Lang::En), "You");
    assert_eq!(record.name(Side::Human, Lang::Ja), "あなた");
    assert_eq!(record.name(Side::Cpu, Lang::En), "CPU");
    assert_eq!(record.name(Side::Cpu, Lang::Ja), "CPU");
}

#[test]
fn serialization_omits_default_names_in_either_language() {
    for human in ["You", "あなた"] {
        let record = parse(&format!("human = {human}\ncpu = CPU\na1\n")).unwrap();
        let text = to_text(&record);
        assert!(!text.contains("human ="), "{text}");
        assert!(!text.contains("cpu ="), "{text}");
        assert_eq!(parse(&text).unwrap().moves, vec![0]);
    }
}

#[test]
fn serialization_preserves_custom_names() {
    let record = parse("human = Alice\ncpu = Bob\na1\n").unwrap();
    let text = to_text(&record);
    assert!(text.contains("human = Alice\n"), "{text}");
    assert!(text.contains("cpu = Bob\n"), "{text}");
}
