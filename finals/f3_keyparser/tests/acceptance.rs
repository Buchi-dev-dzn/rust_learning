//! F3 受け入れテスト (編集しないこと)

use f3_keyparser::Key::*;
use f3_keyparser::*;

#[test]
fn plain_ascii_and_control() {
    assert_eq!(parse(b"aZ 9~"), vec![Char('a'), Char('Z'), Char(' '), Char('9'), Char('~')]);
    assert_eq!(parse(b"\r\n\t\x7f\x08"), vec![Enter, Enter, Tab, Backspace, Backspace]);
    assert_eq!(
        parse(b"\x00\x01\x03\x11\x1a"),
        vec![Ctrl(' '), Ctrl('a'), Ctrl('c'), Ctrl('q'), Ctrl('z')]
    );
    assert_eq!(
        parse(b"\x1c\x1d\x1e\x1f"),
        vec![Ctrl('\\'), Ctrl(']'), Ctrl('^'), Ctrl('_')]
    );
    assert_eq!(parse(b""), vec![]);
}

#[test]
fn arrows_csi_and_ss3() {
    assert_eq!(parse(b"\x1b[A\x1b[B\x1b[C\x1b[D"), vec![Up, Down, Right, Left]);
    assert_eq!(parse(b"\x1bOA\x1bOB\x1bOC\x1bOD"), vec![Up, Down, Right, Left]);
    assert_eq!(parse(b"\x1b[H\x1b[F\x1bOH\x1bOF"), vec![Home, End, Home, End]);
    assert_eq!(parse(b"\x1b[Z"), vec![BackTab]);
}

#[test]
fn modifiers_are_ignored() {
    assert_eq!(parse(b"\x1b[1;5A\x1b[1;2D\x1b[3;5~"), vec![Up, Left, Delete]);
}

#[test]
fn tilde_sequences() {
    assert_eq!(
        parse(b"\x1b[1~\x1b[2~\x1b[3~\x1b[4~\x1b[5~\x1b[6~\x1b[7~\x1b[8~"),
        vec![Home, Insert, Delete, End, PageUp, PageDown, Home, End]
    );
}

#[test]
fn function_keys() {
    assert_eq!(parse(b"\x1bOP\x1bOQ\x1bOR\x1bOS"), vec![F(1), F(2), F(3), F(4)]);
    assert_eq!(parse(b"\x1b[11~\x1b[15~\x1b[17~"), vec![F(1), F(5), F(6)]);
    assert_eq!(parse(b"\x1b[21~\x1b[23~\x1b[24~"), vec![F(10), F(11), F(12)]);
    assert_eq!(parse(b"\x1b[1;2P"), vec![F(1)]);
}

#[test]
fn alt_and_lone_esc() {
    assert_eq!(parse(b"\x1bx\x1bA\x1b1"), vec![Alt('x'), Alt('A'), Alt('1')]);
    assert_eq!(parse(b"\x1b"), vec![Esc]);
    // ESC ESC → Esc を 1 つ確定し、2 つ目は次のキーの先頭
    assert_eq!(parse(b"\x1b\x1b[A"), vec![Esc, Up]);
    assert_eq!(parse(b"\x1b\x1b"), vec![Esc, Esc]);
    // ESC + 制御文字
    assert_eq!(parse(b"\x1b\x01"), vec![Esc, Ctrl('a')]);
}

#[test]
fn utf8_multibyte() {
    assert_eq!(parse("あ漢é🎉".as_bytes()), vec![Char('あ'), Char('漢'), Char('é'), Char('🎉')]);
    assert_eq!(parse("aあb".as_bytes()), vec![Char('a'), Char('あ'), Char('b')]);
}

#[test]
fn invalid_utf8_is_unknown() {
    assert_eq!(parse(&[0xff, b'a']), vec![Unknown(vec![0xff]), Char('a')]);
    assert_eq!(parse(&[0x80]), vec![Unknown(vec![0x80])]);
    // 継続バイトが来るべき所に 'a'
    assert_eq!(parse(&[0xe3, b'a']), vec![Unknown(vec![0xe3]), Char('a')]);
}

#[test]
fn unknown_sequences() {
    assert_eq!(parse(b"\x1b[99~"), vec![Unknown(b"\x1b[99~".to_vec())]);
    assert_eq!(parse(b"\x1b[5n"), vec![Unknown(b"\x1b[5n".to_vec())]);
    assert_eq!(parse(b"\x1bOx"), vec![Unknown(b"\x1bOx".to_vec())]);
    // CSI の途中に制御文字 → そこまでが Unknown、制御文字は別のキー
    assert_eq!(parse(b"\x1b[1\x03"), vec![Unknown(b"\x1b[1".to_vec()), Ctrl('c')]);
}

#[test]
fn incremental_csi_across_chunks() {
    let mut p = Parser::new();
    assert_eq!(p.feed(b"a\x1b"), vec![Char('a')]);
    assert!(p.has_pending());
    assert_eq!(p.feed(b"["), vec![]);
    assert_eq!(p.feed(b"5"), vec![]);
    assert_eq!(p.feed(b"~b"), vec![PageUp, Char('b')]);
    assert!(!p.has_pending());
    assert_eq!(p.flush(), vec![]);
}

#[test]
fn incremental_utf8_across_chunks() {
    let bytes = "漢".as_bytes();
    let mut p = Parser::new();
    assert_eq!(p.feed(&bytes[..1]), vec![]);
    assert_eq!(p.feed(&bytes[1..2]), vec![]);
    assert!(p.has_pending());
    assert_eq!(p.feed(&bytes[2..]), vec![Char('漢')]);
}

#[test]
fn every_split_point_gives_same_result() {
    let input: &[u8] = "x\x1b[1;5C\x1bOQあ\x1b[3~\x1bq\r".as_bytes();
    let expected = vec![Char('x'), Right, F(2), Char('あ'), Delete, Alt('q'), Enter];
    assert_eq!(parse(input), expected);
    for split in 0..=input.len() {
        let mut p = Parser::new();
        let mut keys = p.feed(&input[..split]);
        keys.extend(p.feed(&input[split..]));
        keys.extend(p.flush());
        assert_eq!(keys, expected, "split at {split}");
    }
}

#[test]
fn flush_rules() {
    let mut p = Parser::new();
    assert_eq!(p.feed(b"\x1b"), vec![]);
    assert_eq!(p.flush(), vec![Esc]);
    assert!(!p.has_pending());

    assert_eq!(p.feed(b"\x1b["), vec![]);
    assert_eq!(p.flush(), vec![Alt('[')]);

    assert_eq!(p.feed(b"\x1bO"), vec![]);
    assert_eq!(p.flush(), vec![Alt('O')]);

    assert_eq!(p.feed(b"\x1b[12"), vec![]);
    assert_eq!(p.flush(), vec![Unknown(b"\x1b[12".to_vec())]);

    assert_eq!(p.feed(&[0xe3, 0x81]), vec![]);
    assert_eq!(p.flush(), vec![Unknown(vec![0xe3, 0x81])]);
    assert!(!p.has_pending());
}

#[test]
fn bracketed_paste() {
    assert_eq!(
        parse(b"\x1b[200~hello\nworld\x1b[201~q"),
        vec![Paste("hello\nworld".into()), Char('q')]
    );
    // 中身のエスケープシーケンスはキーとして解釈しない
    assert_eq!(parse(b"\x1b[200~hi\x1b[A\x1b[201~"), vec![Paste("hi\x1b[A".into())]);
    assert_eq!(parse("\x1b[200~日本\x1b[201~".as_bytes()), vec![Paste("日本".into())]);
}

#[test]
fn bracketed_paste_incremental_and_flush_keeps_it() {
    let mut p = Parser::new();
    assert_eq!(p.feed(b"\x1b[200~abc"), vec![]);
    assert_eq!(p.flush(), vec![]); // ペースト途中は確定しない
    assert!(p.has_pending());
    assert_eq!(p.feed(b"def\x1b[20"), vec![]);
    assert_eq!(p.feed(b"1~\x1b[B"), vec![Paste("abcdef".into()), Down]);
}
