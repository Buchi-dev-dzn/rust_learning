//! F5 受け入れテスト (編集しないこと)

use f5_editor_core::Editor;

fn typed(s: &str) -> Editor {
    let mut e = Editor::new();
    for c in s.chars() {
        e.insert_char(c);
    }
    e
}

#[test]
fn new_and_from_text() {
    let e = Editor::new();
    assert_eq!((e.line_count(), e.text(), e.cursor()), (1, String::new(), (0, 0)));
    let e = Editor::from_text("a\nbc\n");
    assert_eq!(e.line_count(), 3);
    assert_eq!((e.line(0), e.line(1), e.line(2)), ("a", "bc", ""));
    assert_eq!(e.text(), "a\nbc\n");
    assert_eq!(e.cursor(), (0, 0));
}

#[test]
fn set_cursor_clamps() {
    let mut e = Editor::from_text("abc\nあい");
    e.set_cursor(1, 99);
    assert_eq!(e.cursor(), (1, 2));
    e.set_cursor(9, 1);
    assert_eq!(e.cursor(), (1, 1));
    e.set_cursor(0, 2);
    assert_eq!(e.cursor(), (0, 2));
}

#[test]
fn insert_char_and_newline() {
    let mut e = typed("ab");
    e.move_left();
    e.insert_char('X');
    assert_eq!((e.text(), e.cursor()), ("aXb".to_string(), (0, 2)));
    e.insert_newline();
    assert_eq!((e.text(), e.cursor()), ("aX\nb".to_string(), (1, 0)));
    e.insert_char('\n');
    assert_eq!((e.text(), e.cursor()), ("aX\n\nb".to_string(), (2, 0)));
}

#[test]
fn multibyte_columns_are_chars() {
    let mut e = typed("日本語");
    assert_eq!(e.cursor(), (0, 3));
    e.move_left();
    e.insert_char('の');
    assert_eq!(e.text(), "日本の語");
    assert!(e.backspace());
    e.move_home();
    e.delete();
    assert_eq!((e.text(), e.cursor()), ("本語".to_string(), (0, 0)));
}

#[test]
fn insert_str_with_newlines() {
    let mut e = Editor::from_text("[]");
    e.set_cursor(0, 1);
    e.insert_str("one\ntwo");
    assert_eq!(e.text(), "[one\ntwo]");
    assert_eq!(e.cursor(), (1, 3));
}

#[test]
fn backspace_joins_lines_and_edges() {
    let mut e = Editor::from_text("abc\ndef");
    assert!(!e.backspace()); // (0,0)
    e.set_cursor(1, 0);
    assert!(e.backspace());
    assert_eq!((e.text(), e.cursor()), ("abcdef".to_string(), (0, 3)));
    assert!(e.backspace());
    assert_eq!((e.text(), e.cursor()), ("abdef".to_string(), (0, 2)));
}

#[test]
fn delete_joins_lines_and_edges() {
    let mut e = Editor::from_text("ab\ncd");
    e.set_cursor(0, 2);
    assert!(e.delete());
    assert_eq!((e.text(), e.cursor()), ("abcd".to_string(), (0, 2)));
    assert!(e.delete());
    assert_eq!(e.text(), "abd");
    e.move_end();
    assert!(!e.delete()); // 最終行の行末
    assert_eq!(e.text(), "abd");
}

#[test]
fn horizontal_movement_wraps_lines() {
    let mut e = Editor::from_text("ab\nc");
    e.move_left();
    assert_eq!(e.cursor(), (0, 0));
    e.set_cursor(0, 2);
    e.move_right();
    assert_eq!(e.cursor(), (1, 0));
    e.move_left();
    assert_eq!(e.cursor(), (0, 2));
    e.set_cursor(1, 1);
    e.move_right();
    assert_eq!(e.cursor(), (1, 1));
    e.move_home();
    assert_eq!(e.cursor(), (1, 0));
    e.set_cursor(0, 0);
    e.move_end();
    assert_eq!(e.cursor(), (0, 2));
}

#[test]
fn vertical_movement_keeps_goal_column() {
    let mut e = Editor::from_text("abcdef\nab\nabcdef");
    e.set_cursor(0, 5);
    e.move_down();
    assert_eq!(e.cursor(), (1, 2));
    e.move_down();
    assert_eq!(e.cursor(), (2, 5));
    e.move_down(); // 最終行: 動かない・目標列は保持
    assert_eq!(e.cursor(), (2, 5));
    e.move_up();
    e.move_up();
    assert_eq!(e.cursor(), (0, 5));
    e.move_up(); // 先頭行: 動かない
    assert_eq!(e.cursor(), (0, 5));
    // 横移動で目標列は忘れる
    e.set_cursor(0, 5);
    e.move_down();
    e.move_left(); // (1,1)
    e.move_down();
    assert_eq!(e.cursor(), (2, 1));
}

#[test]
fn word_movement() {
    let mut e = Editor::from_text("foo.bar  baz\nnext");
    let mut cols = Vec::new();
    for _ in 0..4 {
        e.move_word_right();
        cols.push(e.cursor());
    }
    assert_eq!(cols, vec![(0, 3), (0, 4), (0, 9), (0, 12)]);
    e.move_word_right();
    assert_eq!(e.cursor(), (1, 0));
    e.move_word_left();
    assert_eq!(e.cursor(), (0, 12));
    let mut cols = Vec::new();
    for _ in 0..4 {
        e.move_word_left();
        cols.push(e.cursor().1);
    }
    assert_eq!(cols, vec![9, 4, 3, 0]);
    e.move_word_left(); // 先頭: 動かない
    assert_eq!(e.cursor(), (0, 0));
}

#[test]
fn word_movement_from_whitespace_and_unicode() {
    let mut e = Editor::from_text("  hello_world, 日本語!");
    e.move_word_right(); // 先頭の空白を飛ばす
    assert_eq!(e.cursor(), (0, 2));
    e.move_word_right(); // '_' は単語の一部
    assert_eq!(e.cursor(), (0, 13));
    e.move_word_right(); // ',' と空白
    assert_eq!(e.cursor(), (0, 15));
    e.move_word_right(); // 日本語 は単語
    assert_eq!(e.cursor(), (0, 18));
    e.move_end();
    e.move_word_left();
    assert_eq!(e.cursor(), (0, 18));
}

#[test]
fn undo_groups_typing_by_words() {
    let mut e = typed("hello world");
    assert!(e.undo());
    assert_eq!((e.text(), e.cursor()), ("hello ".to_string(), (0, 6)));
    assert!(e.undo());
    assert_eq!((e.text(), e.cursor()), (String::new(), (0, 0)));
    assert!(!e.undo());
    assert!(e.redo());
    assert_eq!((e.text(), e.cursor()), ("hello ".to_string(), (0, 6)));
    assert!(e.redo());
    assert_eq!((e.text(), e.cursor()), ("hello world".to_string(), (0, 11)));
    assert!(!e.redo());
}

#[test]
fn movement_breaks_undo_group() {
    let mut e = typed("ab");
    e.move_left();
    e.move_right(); // 位置は同じでもグループは切れる
    e.insert_char('c');
    e.insert_char('d');
    e.undo();
    assert_eq!(e.text(), "ab");
    e.undo();
    assert_eq!(e.text(), "");
}

#[test]
fn backspace_and_delete_groups() {
    let mut e = Editor::from_text("abcdef");
    e.set_cursor(0, 4);
    e.backspace();
    e.backspace(); // "abef"
    e.delete(); // 種類が変わる → 別グループ "abf"
    e.delete(); // "ab"
    assert_eq!(e.text(), "ab");
    e.undo();
    assert_eq!((e.text(), e.cursor()), ("abef".to_string(), (0, 2)));
    e.undo();
    assert_eq!((e.text(), e.cursor()), ("abcdef".to_string(), (0, 4)));
}

#[test]
fn newline_and_insert_str_are_own_groups() {
    let mut e = typed("ab");
    e.insert_newline();
    e.insert_char('c');
    e.insert_str("XY");
    e.insert_char('d');
    assert_eq!(e.text(), "ab\ncXYd");
    e.undo();
    assert_eq!(e.text(), "ab\ncXY");
    e.undo();
    assert_eq!(e.text(), "ab\nc");
    e.undo();
    assert_eq!(e.text(), "ab\n");
    e.undo();
    assert_eq!((e.text(), e.cursor()), ("ab".to_string(), (0, 2)));
}

#[test]
fn noop_edits_do_not_touch_history_and_new_edit_clears_redo() {
    let mut e = typed("x");
    e.set_cursor(0, 0);
    assert!(!e.backspace());
    e.insert_str("");
    e.undo();
    assert_eq!(e.text(), "");
    assert!(!e.undo());
    e.redo();
    assert_eq!(e.text(), "x");
    e.undo();
    e.insert_char('y');
    assert!(!e.redo());
    assert_eq!(e.text(), "y");
}

#[test]
fn find_forward_and_wrap() {
    let e = Editor::from_text("foo bar\nbaz foo\nqux");
    assert_eq!(e.find("foo", (0, 0)), Some((0, 0)));
    assert_eq!(e.find("foo", (0, 1)), Some((1, 4)));
    assert_eq!(e.find("foo", (1, 5)), Some((0, 0))); // ラップ
    assert_eq!(e.find("qux", (2, 1)), Some((2, 0))); // 一周して from の手前で見つかる
    assert_eq!(e.find("ux", (2, 1)), Some((2, 1))); // from ちょうどの一致
    assert_eq!(e.find("zzz", (0, 0)), None);
    assert_eq!(e.find("", (0, 0)), None);
    assert_eq!(e.find("bar\nbaz", (0, 0)), None);
    assert_eq!(e.find("ba", (9, 99)), Some((0, 4))); // from は丸める
}

#[test]
fn find_uses_char_columns() {
    let e = Editor::from_text("日本語テキスト\nテキ");
    assert_eq!(e.find("テキ", (0, 0)), Some((0, 3)));
    assert_eq!(e.find("テキ", (0, 4)), Some((1, 0)));
    assert_eq!(e.find_all("テキ"), vec![(0, 3), (1, 0)]);
}

#[test]
fn find_all_non_overlapping() {
    let e = Editor::from_text("aaaa\nxa\n");
    assert_eq!(e.find_all("aa"), vec![(0, 0), (0, 2)]);
    assert_eq!(e.find_all("a"), vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 1)]);
    assert!(e.find_all("").is_empty());
}
