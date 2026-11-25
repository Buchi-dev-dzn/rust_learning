//! エディタ本体の状態とロジック (starter)。
//!
//! ここには端末 I/O を一切書かない。キー入力は自前の `Key` 型で受け取り、
//! 描画結果は `Screen` (文字列の集まり) として返す。

#![allow(unused_imports, dead_code, unused_variables)]

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::row::Row;
use crate::width::{str_width, truncate_to_width};

pub const VERSION: &str = "0.1.0";
const MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);

/// 端末ライブラリに依存しないキー表現
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    /// Ctrl + 英字 (小文字で保持)
    Ctrl(char),
    Enter,
    Backspace,
    Delete,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptKind {
    SaveAs,
    Find,
}

/// 画面下のメッセージバーで文字列を入力している状態 (STEP 6, 7 で使う)
#[derive(Debug, Clone)]
struct Prompt {
    kind: PromptKind,
    input: String,
    /// 検索を ESC で中止したときに戻すカーソル位置 (cx, cy, rowoff, coloff)
    saved: (usize, usize, usize, usize),
    last_match: Option<usize>,
    forward: bool,
}

/// 1 フレーム分の描画内容
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    pub rows: Vec<String>,
    pub status: String,
    pub message: String,
    /// カーソルの画面上の位置 (列, 行)
    pub cursor: (usize, usize),
}

#[derive(Debug)]
pub struct Editor {
    pub rows: Vec<Row>,
    pub cx: usize,
    pub cy: usize,
    pub rowoff: usize,
    pub coloff: usize,
    pub screen_rows: usize,
    pub screen_cols: usize,
    pub filename: Option<PathBuf>,
    pub dirty: bool,
    pub should_quit: bool,
    status_msg: String,
    status_time: Option<Instant>,
    quit_confirm: bool,
    prompt: Option<Prompt>,
}

impl Editor {
    /// `cols` x `rows` の端末用エディタを作る (下 2 行はステータスバーとメッセージバー)
    pub fn new(cols: usize, rows: usize) -> Self {
        Editor {
            rows: Vec::new(),
            cx: 0,
            cy: 0,
            rowoff: 0,
            coloff: 0,
            screen_rows: rows.saturating_sub(2).max(1),
            screen_cols: cols.max(1),
            filename: None,
            dirty: false,
            should_quit: false,
            status_msg: String::new(),
            status_time: None,
            quit_confirm: false,
            prompt: None,
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.screen_rows = rows.saturating_sub(2).max(1);
        self.screen_cols = cols.max(1);
    }

    pub fn load_text(&mut self, text: &str) {
        // TODO(STEP 3): text.lines() を Row にして self.rows へ。カーソルと dirty をリセット
        todo!()
    }

    pub fn open(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        // TODO(STEP 3): fs::read_to_string。ErrorKind::NotFound なら空バッファ + メッセージ。
        //               最後に self.filename = Some(path)
        todo!()
    }

    pub fn text(&self) -> String {
        // TODO(STEP 6): 各行の末尾に '\n' を付けて連結
        todo!()
    }

    pub fn save(&mut self) -> io::Result<usize> {
        // TODO(STEP 6): filename が None ならエラー。fs::write して dirty = false、バイト数を返す
        todo!()
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.status_msg = msg.into();
        self.status_time = Some(Instant::now());
    }

    pub fn message(&self) -> &str {
        &self.status_msg
    }

    fn row_len(&self, y: usize) -> usize {
        self.rows.get(y).map_or(0, Row::len)
    }

    /// カーソルの画面上の列 (rx)
    pub fn rx(&self) -> usize {
        // TODO(STEP 4): 現在行の cx_to_rx(self.cx)。行がなければ 0
        todo!()
    }

    pub fn insert_char(&mut self, c: char) {
        // TODO(STEP 5): 最終行の次にいるなら空行を追加してから挿入。cx += 1, dirty = true
        todo!()
    }

    pub fn insert_newline(&mut self) {
        // TODO(STEP 5): 現在行を cx で split_off し、残りを次の行として挿入
        todo!()
    }

    pub fn delete_back(&mut self) {
        // TODO(STEP 5): cx > 0 なら 1 文字削除、行頭なら前の行と連結
        todo!()
    }

    pub fn delete_forward(&mut self) {
        // TODO(STEP 5): カーソル位置の文字を削除、行末なら次の行と連結
        todo!()
    }

    pub fn move_cursor(&mut self, key: Key) {
        // TODO(STEP 4): Left/Right (行をまたぐ), Up/Down (rx を保つ),
        //               Home/End, PageUp/PageDown。最後に cx を行の長さに収める
        todo!()
    }

    pub fn process_key(&mut self, key: Key) {
        match key {
            // TODO(STEP 1): Ctrl-Q で self.should_quit = true
            // TODO(STEP 4): 矢印キーなどは move_cursor へ
            // TODO(STEP 5): Enter / Backspace / Delete / 文字入力
            // TODO(STEP 6): Ctrl-S で保存、未保存のときの Ctrl-Q 確認、プロンプト中の処理
            // TODO(STEP 7): Ctrl-F で検索
            _ => {}
        }
    }

    fn find_step(&mut self, p: &mut Prompt) {
        // TODO(STEP 7): last_match の次 (前) の行から順に探し、見つかったらカーソルを移動
        todo!()
    }

    pub fn scroll(&mut self) {
        // TODO(STEP 4): カーソルが画面内に入るように rowoff / coloff を調整
    }

    pub fn render(&mut self) -> Screen {
        self.scroll();
        // TODO(STEP 3): ファイルの行を row.render(coloff, screen_cols) で描画し、
        //               空バッファならようこそメッセージ、ステータスバー・メッセージバーも作る
        // いまは全行 "~" を返すだけの仮実装
        Screen {
            rows: vec!["~".to_string(); self.screen_rows],
            status: String::new(),
            message: String::new(),
            cursor: (0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_with(text: &str) -> Editor {
        let mut e = Editor::new(20, 7); // テキスト領域は 5 行
        e.load_text(text);
        e
    }

    fn type_str(e: &mut Editor, s: &str) {
        for c in s.chars() {
            e.process_key(Key::Char(c));
        }
    }

    #[test]
    fn welcome_message_on_empty_buffer() {
        let mut e = Editor::new(40, 12);
        let s = e.render();
        assert!(s.rows.iter().any(|r| r.contains("Kilo")));
        assert_eq!(s.rows[0], "~");
    }

    #[test]
    fn status_bar_fills_width() {
        let mut e = Editor::new(30, 7);
        e.load_text("a\nb");
        e.filename = Some("日本語.txt".into());
        let s = e.render();
        assert_eq!(str_width(&s.status), 30);
        assert!(s.status.ends_with("1/2"));
    }

    #[test]
    fn cursor_wraps_at_line_ends() {
        let mut e = editor_with("ab\ncd");
        e.move_cursor(Key::End);
        e.move_cursor(Key::Right);
        assert_eq!((e.cx, e.cy), (0, 1));
        e.move_cursor(Key::Left);
        assert_eq!((e.cx, e.cy), (2, 0));
    }

    #[test]
    fn vertical_move_keeps_display_column() {
        let mut e = editor_with("abcd\nあい");
        e.cx = 2;
        e.move_cursor(Key::Down);
        assert_eq!((e.cx, e.cy), (1, 1));
        e.move_cursor(Key::Up);
        assert_eq!(e.cx, 2);
    }

    #[test]
    fn vertical_scroll() {
        let text: String = (0..20).map(|i| format!("line {i}\n")).collect();
        let mut e = editor_with(&text);
        for _ in 0..10 {
            e.move_cursor(Key::Down);
        }
        let s = e.render();
        assert_eq!(e.rowoff, 6);
        assert_eq!(s.rows[4], "line 10");
        assert_eq!(s.cursor, (0, 4));
    }

    #[test]
    fn horizontal_scroll_with_wide_chars() {
        let mut e = editor_with(&"あ".repeat(30));
        e.move_cursor(Key::End);
        let s = e.render();
        assert_eq!(e.coloff, 41);
        assert_eq!(s.cursor.0, 19);
        assert!(str_width(&s.rows[0]) <= 20);
    }

    #[test]
    fn insert_into_empty_buffer() {
        let mut e = editor_with("");
        type_str(&mut e, "abc");
        assert_eq!(e.rows[0].chars, "abc");
        assert_eq!(e.cx, 3);
        assert!(e.dirty);
    }

    #[test]
    fn newline_splits_row() {
        let mut e = editor_with("hello");
        e.cx = 2;
        e.process_key(Key::Enter);
        assert_eq!(e.rows[0].chars, "he");
        assert_eq!(e.rows[1].chars, "llo");
        assert_eq!((e.cx, e.cy), (0, 1));
    }

    #[test]
    fn backspace_joins_rows() {
        let mut e = editor_with("ab\ncd");
        e.cy = 1;
        e.process_key(Key::Backspace);
        assert_eq!(e.rows[0].chars, "abcd");
        assert_eq!((e.cx, e.cy), (2, 0));
    }

    #[test]
    fn edit_japanese() {
        let mut e = editor_with("あいう");
        e.move_cursor(Key::Right);
        type_str(&mut e, "X");
        assert_eq!(e.rows[0].chars, "あXいう");
        assert_eq!(e.rx(), 3);
    }

    #[test]
    fn quit_requires_confirmation_when_dirty() {
        let mut e = editor_with("");
        type_str(&mut e, "x");
        e.process_key(Key::Ctrl('q'));
        assert!(!e.should_quit);
        e.process_key(Key::Ctrl('q'));
        assert!(e.should_quit);
    }

    #[test]
    fn save_and_open_roundtrip() {
        let path = std::env::temp_dir().join(format!("p05_starter_test_{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut e = Editor::new(80, 24);
        e.open(&path).unwrap();
        type_str(&mut e, "こんにちは");
        e.process_key(Key::Ctrl('s'));
        assert!(!e.dirty);
        assert_eq!(fs::read_to_string(&path).unwrap(), "こんにちは\n");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn incremental_search() {
        let mut e = editor_with("apple\nbanana\ncherry\nbanana2");
        e.process_key(Key::Ctrl('f'));
        type_str(&mut e, "nan");
        assert_eq!((e.cx, e.cy), (2, 1));
        e.process_key(Key::Down);
        assert_eq!((e.cx, e.cy), (2, 3));
        e.process_key(Key::Esc);
        assert_eq!((e.cx, e.cy), (0, 0));
    }
}
