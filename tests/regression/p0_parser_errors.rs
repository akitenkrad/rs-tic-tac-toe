use rs_tic_tac_toe::game::Side;
use rs_tic_tac_toe::record::{RecordError, parse, to_text};

#[test]
fn empty_record_uses_all_defaults() {
    let record = parse("").unwrap();
    assert_eq!(record.first, Side::Human);
    assert_eq!(record.names, ["あなた", "CPU"]);
    assert!(record.moves.is_empty());
}

#[test]
fn crlf_comments_and_unicode_names_round_trip() {
    let record = parse("first = cpu\r\nhuman = 山田 Alice\r\ncpu = ボブ\r\n\r\nA1 # opening\r\n5\r\n").unwrap();
    assert_eq!(record.names, ["山田 Alice", "ボブ"]);
    assert_eq!(parse(&to_text(&record)).unwrap(), record);
}

#[test]
fn parser_errors_keep_the_exact_source_line() {
    let late = parse("# one\n\na1\nfirst = cpu\n").unwrap_err();
    assert_eq!(late, RecordError::HeaderAfterMove { line: 4 });
    assert_eq!(late.to_string(), "line 4: header after first move");

    let invalid = parse("human = Alice\n# ignored\nd4\n").unwrap_err();
    assert_eq!(invalid.line(), 3);
    assert!(invalid.to_string().starts_with("line 3: "));
}
