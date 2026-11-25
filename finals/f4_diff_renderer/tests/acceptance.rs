//! F4 受け入れテスト (編集しないこと)

use f4_diff_renderer::*;

const CONT: char = Cell::CONTINUATION_CHAR;

#[test]
fn char_widths() {
    assert_eq!(char_width('a'), 1);
    assert_eq!(char_width('あ'), 2);
    assert_eq!(char_width('🎉'), 2);
    assert_eq!(char_width('\u{0301}'), 0);
    assert_eq!(char_width('\n'), 0);
}

#[test]
fn buffer_new_get_set() {
    let mut b = Buffer::new(4, 2);
    assert_eq!((b.width(), b.height()), (4, 2));
    assert_eq!(b.get(3, 1), Cell::blank());
    let red = Style::new().fg(Color::Red);
    b.set(1, 1, 'x', red);
    assert_eq!(b.get(1, 1), Cell { ch: 'x', style: red });
    // 範囲外・幅 0 は無視
    b.set(4, 0, 'y', red);
    b.set(0, 2, 'y', red);
    b.set(0, 0, '\u{0301}', red);
    assert_eq!(b.lines(), vec!["    ", " x  "]);
}

#[test]
fn set_str_clips_and_returns_next_x() {
    let mut b = Buffer::new(5, 1);
    assert_eq!(b.set_str(1, 0, "abc", Style::new()), 4);
    assert_eq!(b.set_str(3, 0, "XYZ", Style::new()), 5);
    assert_eq!(b.lines(), vec![" abXY"]);
    b.clear();
    assert_eq!(b.lines(), vec!["     "]);
}

#[test]
fn wide_chars_occupy_two_cells() {
    let mut b = Buffer::new(5, 1);
    let s = Style::new().bold();
    assert_eq!(b.set_str(0, 0, "日本a", s), 5);
    assert_eq!(b.get(0, 0).ch, '日');
    assert_eq!(b.get(1, 0), Cell { ch: CONT, style: s });
    assert_eq!(b.get(2, 0).ch, '本');
    assert!(b.get(3, 0).is_continuation());
    assert_eq!(b.lines(), vec!["日本a"]);
}

#[test]
fn wide_char_at_right_edge_becomes_space() {
    let mut b = Buffer::new(3, 1);
    let s = Style::new().fg(Color::Green);
    assert_eq!(b.set_str(1, 0, "a漢", s), 3);
    assert_eq!(b.get(2, 0), Cell { ch: ' ', style: s });
    assert_eq!(b.lines(), vec![" a "]);
}

#[test]
fn overwriting_half_of_wide_char_clears_other_half() {
    let s = Style::new().fg(Color::Blue);
    // 右半分を上書き → 左半分が空白に (スタイルは残る)
    let mut b = Buffer::new(4, 1);
    b.set(0, 0, '漢', s);
    b.set(1, 0, 'x', Style::new());
    assert_eq!(b.get(0, 0), Cell { ch: ' ', style: s });
    assert_eq!(b.get(1, 0).ch, 'x');
    // 左半分を上書き → 右半分が空白に
    let mut b = Buffer::new(4, 1);
    b.set(1, 0, '漢', s);
    b.set(1, 0, 'y', Style::new());
    assert_eq!(b.get(2, 0), Cell { ch: ' ', style: s });
    assert_eq!(b.lines(), vec![" y  "]);
    // 全角を 1 セルずらして上書き → 古い全角の左半分が空白に
    let mut b = Buffer::new(4, 1);
    b.set(0, 0, '漢', s);
    b.set(1, 0, '字', Style::new());
    assert_eq!(b.lines(), vec![" 字 "]);
    assert_eq!(b.get(0, 0), Cell { ch: ' ', style: s });
    // 全角の右に全角を重ねる: x+1 側の古い全角の右半分も空白に
    let mut b = Buffer::new(5, 1);
    b.set(2, 0, '漢', s);
    b.set(1, 0, '字', Style::new());
    assert_eq!(b.lines(), vec![" 字  "]);
    assert_eq!(b.get(3, 0), Cell { ch: ' ', style: s });
}

#[test]
fn sgr_format() {
    assert_eq!(sgr(&Style::default()), "\x1b[0m");
    assert_eq!(sgr(&Style::new().bold()), "\x1b[0;1m");
    assert_eq!(sgr(&Style::new().fg(Color::Red)), "\x1b[0;31m");
    assert_eq!(sgr(&Style::new().fg(Color::White).bg(Color::Black)), "\x1b[0;37;40m");
    assert_eq!(
        sgr(&Style::new().bold().fg(Color::Red).bg(Color::Indexed(236))),
        "\x1b[0;1;31;48;5;236m"
    );
    assert_eq!(sgr(&Style::new().fg(Color::Indexed(7)).bg(Color::Cyan)), "\x1b[0;38;5;7;46m");
}

#[test]
fn diff_identical_is_empty() {
    let mut a = Buffer::new(6, 3);
    a.set_str(0, 1, "hello", Style::new().fg(Color::Red));
    let b = a.clone();
    assert_eq!(diff(&a, &b), "");
}

#[test]
fn diff_spec_example() {
    let prev = Buffer::new(5, 2);
    let mut next = prev.clone();
    next.set_str(1, 0, "hi", Style::new().bold().fg(Color::Red));
    assert_eq!(diff(&prev, &next), "\x1b[1;2H\x1b[0;1;31mhi\x1b[0m");
}

#[test]
fn diff_default_style_needs_no_sgr() {
    let prev = Buffer::new(5, 2);
    let mut next = prev.clone();
    next.set_str(0, 1, "ab", Style::new());
    assert_eq!(diff(&prev, &next), "\x1b[2;1Hab");
}

#[test]
fn diff_moves_cursor_only_on_gaps() {
    let prev = Buffer::new(8, 2);
    let mut next = prev.clone();
    next.set(0, 0, 'a', Style::new());
    next.set(1, 0, 'b', Style::new());
    next.set(4, 0, 'c', Style::new());
    next.set(0, 1, 'd', Style::new());
    assert_eq!(diff(&prev, &next), "\x1b[1;1Hab\x1b[1;5Hc\x1b[2;1Hd");
}

#[test]
fn diff_sgr_only_on_style_change() {
    let prev = Buffer::new(6, 1);
    let mut next = prev.clone();
    let r = Style::new().fg(Color::Red);
    let g = Style::new().fg(Color::Green);
    next.set_str(0, 0, "aa", r);
    next.set_str(2, 0, "b", g);
    next.set_str(3, 0, "c", r);
    next.set_str(4, 0, "d", Style::new());
    assert_eq!(
        diff(&prev, &next),
        "\x1b[1;1H\x1b[0;31maa\x1b[0;32mb\x1b[0;31mc\x1b[0md"
    );
}

#[test]
fn diff_style_only_change_redraws_cell() {
    let mut prev = Buffer::new(3, 1);
    prev.set_str(0, 0, "abc", Style::new());
    let mut next = prev.clone();
    next.set(1, 0, 'b', Style::new().bold());
    assert_eq!(diff(&prev, &next), "\x1b[1;2H\x1b[0;1mb\x1b[0m");
}

#[test]
fn diff_wide_chars_advance_cursor_by_two() {
    let prev = Buffer::new(6, 1);
    let mut next = prev.clone();
    next.set_str(0, 0, "漢字x", Style::new());
    assert_eq!(diff(&prev, &next), "\x1b[1;1H漢字x");
    // 全角 → 半角 2 文字に置き換え
    let mut next2 = next.clone();
    next2.set_str(0, 0, "ab", Style::new());
    assert_eq!(diff(&next, &next2), "\x1b[1;1Hab");
}

#[test]
fn diff_cursor_unknown_after_right_edge() {
    let prev = Buffer::new(3, 2);
    let mut next = prev.clone();
    next.set_str(1, 0, "ab", Style::new());
    next.set(0, 1, 'c', Style::new());
    // 右端まで書いた後は、次の行頭でも必ずカーソル移動を出す
    assert_eq!(diff(&prev, &next), "\x1b[1;2Hab\x1b[2;1Hc");
}

#[test]
fn diff_size_change_clears_screen() {
    let prev = Buffer::new(2, 2);
    let mut next = Buffer::new(3, 1);
    next.set(2, 0, 'z', Style::new());
    assert_eq!(diff(&prev, &next), "\x1b[2J\x1b[1;3Hz");
    let empty = Buffer::new(3, 1);
    assert_eq!(diff(&prev, &empty), "\x1b[2J");
}

#[test]
fn renderer_tracks_previous_frame() {
    let mut r = Renderer::new();
    let mut f = Buffer::new(4, 1);
    f.set_str(0, 0, "ab", Style::new());
    assert_eq!(r.render(&f), "\x1b[2J\x1b[1;1Hab");
    assert_eq!(r.render(&f), "");
    f.set(1, 0, 'c', Style::new().fg(Color::Yellow));
    assert_eq!(r.render(&f), "\x1b[1;2H\x1b[0;33mc\x1b[0m");
    r.invalidate();
    assert_eq!(r.render(&f), "\x1b[2J\x1b[1;1Ha\x1b[0;33mc\x1b[0m");
}
