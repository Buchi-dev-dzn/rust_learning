//! エディタ本体の状態とロジック。
//!
//! ここには端末 I/O を一切書かない。キー入力は自前の `Key` 型で受け取り、
//! 描画結果は `Screen` (文字列の集まり) として返す。
//! こうしておくと、端末なしで単体テストができる。

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

/// 画面下のメッセージバーで文字列を入力している状態
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
    /// テキスト領域 (screen_rows 行)
    pub rows: Vec<String>,
    /// ステータスバー (反転表示する。幅ちょうどに空白で埋めてある)
    pub status: String,
    /// メッセージバー
    pub message: String,
    /// カーソルの画面上の位置 (列, 行)
    pub cursor: (usize, usize),
}

#[derive(Debug)]
pub struct Editor {
    pub rows: Vec<Row>,
    /// カーソル位置: cx は文字単位、cy は行番号 (rows.len() は「最終行の次」)
    pub cx: usize,
    pub cy: usize,
    /// スクロール量: rowoff は行、coloff は表示列
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

    /// テキストを行に分割して読み込む
    pub fn load_text(&mut self, text: &str) {
        self.rows = text
            .lines()
            .map(|l| Row::new(l.strip_suffix('\r').unwrap_or(l)))
            .collect();
        self.cx = 0;
        self.cy = 0;
        self.dirty = false;
    }

    /// ファイルを開く。存在しなければ空の新規バッファにする。
    pub fn open(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        match fs::read_to_string(&path) {
            Ok(text) => self.load_text(&text),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.load_text("");
                self.set_message(format!("新規ファイル: {}", path.display()));
            }
            Err(e) => return Err(e),
        }
        self.filename = Some(path);
        Ok(())
    }

    /// バッファ全体を 1 つの文字列にする (各行末に改行)
    pub fn text(&self) -> String {
        let mut s = String::new();
        for row in &self.rows {
            s.push_str(&row.chars);
            s.push('\n');
        }
        s
    }

    /// ファイルに保存して書き込んだバイト数を返す
    pub fn save(&mut self) -> io::Result<usize> {
        let Some(path) = &self.filename else {
            return Err(io::Error::other("ファイル名がありません"));
        };
        let text = self.text();
        fs::write(path, &text)?;
        self.dirty = false;
        Ok(text.len())
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.status_msg = msg.into();
        self.status_time = Some(Instant::now());
    }

    pub fn message(&self) -> &str {
        &self.status_msg
    }

    // ------------------------------------------------------------ 編集操作

    fn row_len(&self, y: usize) -> usize {
        self.rows.get(y).map_or(0, Row::len)
    }

    /// カーソルの画面上の列 (rx)
    pub fn rx(&self) -> usize {
        self.rows.get(self.cy).map_or(0, |r| r.cx_to_rx(self.cx))
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cy == self.rows.len() {
            self.rows.push(Row::default());
        }
        self.rows[self.cy].insert_char(self.cx, c);
        self.cx += 1;
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        if self.cy == self.rows.len() {
            self.rows.push(Row::default());
        } else {
            let rest = self.rows[self.cy].split_off(self.cx);
            self.rows.insert(self.cy + 1, Row::new(rest));
        }
        self.cy += 1;
        self.cx = 0;
        self.dirty = true;
    }

    /// Backspace: カーソルの左の文字を消す。行頭なら前の行と連結する。
    pub fn delete_back(&mut self) {
        if self.cy == self.rows.len() {
            self.move_cursor(Key::Left);
            return;
        }
        if self.cx > 0 {
            self.rows[self.cy].delete_char(self.cx - 1);
            self.cx -= 1;
        } else if self.cy > 0 {
            let row = self.rows.remove(self.cy);
            self.cy -= 1;
            self.cx = self.rows[self.cy].len();
            self.rows[self.cy].append(&row.chars);
        } else {
            return;
        }
        self.dirty = true;
    }

    /// Delete: カーソル位置の文字を消す。行末なら次の行と連結する。
    pub fn delete_forward(&mut self) {
        if self.cy >= self.rows.len() {
            return;
        }
        if self.cx < self.row_len(self.cy) {
            self.rows[self.cy].delete_char(self.cx);
        } else if self.cy + 1 < self.rows.len() {
            let next = self.rows.remove(self.cy + 1);
            self.rows[self.cy].append(&next.chars);
        } else {
            return;
        }
        self.dirty = true;
    }

    pub fn move_cursor(&mut self, key: Key) {
        match key {
            Key::Left => {
                if self.cx > 0 {
                    self.cx -= 1;
                } else if self.cy > 0 {
                    self.cy -= 1;
                    self.cx = self.row_len(self.cy);
                }
            }
            Key::Right => {
                if self.cy < self.rows.len() {
                    if self.cx < self.row_len(self.cy) {
                        self.cx += 1;
                    } else {
                        self.cy += 1;
                        self.cx = 0;
                    }
                }
            }
            Key::Up | Key::Down => {
                // 縦移動では「画面上の列」を保つ (全角文字があってもずれない)
                let rx = self.rx();
                if key == Key::Up && self.cy > 0 {
                    self.cy -= 1;
                } else if key == Key::Down && self.cy < self.rows.len() {
                    self.cy += 1;
                }
                self.cx = self.rows.get(self.cy).map_or(0, |r| r.rx_to_cx(rx));
            }
            Key::Home => self.cx = 0,
            Key::End => self.cx = self.row_len(self.cy),
            Key::PageUp => {
                self.cy = self.rowoff;
                for _ in 0..self.screen_rows {
                    self.move_cursor(Key::Up);
                }
            }
            Key::PageDown => {
                self.cy = (self.rowoff + self.screen_rows - 1).min(self.rows.len());
                for _ in 0..self.screen_rows {
                    self.move_cursor(Key::Down);
                }
            }
            _ => {}
        }
        self.cx = self.cx.min(self.row_len(self.cy));
    }

    // ------------------------------------------------------------ キー処理

    pub fn process_key(&mut self, key: Key) {
        if self.prompt.is_some() {
            self.process_prompt_key(key);
            return;
        }
        if key != Key::Ctrl('q') {
            self.quit_confirm = false;
        }
        match key {
            Key::Ctrl('q') => {
                if self.dirty && !self.quit_confirm {
                    self.set_message("警告: 未保存の変更があります。もう一度 Ctrl-Q で破棄して終了します");
                    self.quit_confirm = true;
                } else {
                    self.should_quit = true;
                }
            }
            Key::Ctrl('s') => {
                if self.filename.is_none() {
                    self.start_prompt(PromptKind::SaveAs);
                } else {
                    self.save_with_message();
                }
            }
            Key::Ctrl('f') => self.start_prompt(PromptKind::Find),
            Key::Enter => self.insert_newline(),
            Key::Backspace | Key::Ctrl('h') => self.delete_back(),
            Key::Delete => self.delete_forward(),
            Key::Char(c) if c == '\t' || !c.is_control() => self.insert_char(c),
            Key::Left
            | Key::Right
            | Key::Up
            | Key::Down
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown => self.move_cursor(key),
            _ => {}
        }
    }

    fn save_with_message(&mut self) {
        match self.save() {
            Ok(n) => self.set_message(format!("{n} バイト書き込みました")),
            Err(e) => self.set_message(format!("保存できません: {e}")),
        }
    }

    fn start_prompt(&mut self, kind: PromptKind) {
        self.prompt = Some(Prompt {
            kind,
            input: String::new(),
            saved: (self.cx, self.cy, self.rowoff, self.coloff),
            last_match: None,
            forward: true,
        });
    }

    fn process_prompt_key(&mut self, key: Key) {
        let Some(mut p) = self.prompt.take() else {
            return;
        };
        match key {
            Key::Esc => {
                if p.kind == PromptKind::Find {
                    (self.cx, self.cy, self.rowoff, self.coloff) = p.saved;
                }
                self.set_message("");
                return; // prompt は None のまま = 終了
            }
            Key::Enter => {
                if p.kind == PromptKind::SaveAs {
                    if p.input.is_empty() {
                        self.prompt = Some(p);
                        return;
                    }
                    self.filename = Some(PathBuf::from(&p.input));
                    self.save_with_message();
                } else {
                    self.set_message("");
                }
                return;
            }
            Key::Backspace | Key::Ctrl('h') => {
                p.input.pop();
                p.last_match = None;
                p.forward = true;
            }
            Key::Char(c) if !c.is_control() => {
                p.input.push(c);
                p.last_match = None;
                p.forward = true;
            }
            Key::Right | Key::Down if p.kind == PromptKind::Find => p.forward = true,
            Key::Left | Key::Up if p.kind == PromptKind::Find => p.forward = false,
            _ => {}
        }
        if p.kind == PromptKind::Find {
            self.find_step(&mut p);
        }
        self.prompt = Some(p);
    }

    /// インクリメンタル検索の 1 ステップ。last_match の次 (前) の行から探す。
    fn find_step(&mut self, p: &mut Prompt) {
        if p.input.is_empty() || self.rows.is_empty() {
            return;
        }
        let n = self.rows.len();
        let mut current = p.last_match;
        for _ in 0..n {
            let next = match (current, p.forward) {
                (None, true) => 0,
                (None, false) => n - 1,
                (Some(i), true) => (i + 1) % n,
                (Some(i), false) => (i + n - 1) % n,
            };
            current = Some(next);
            let row = &self.rows[next];
            if let Some(byte) = row.chars.find(&p.input) {
                p.last_match = Some(next);
                self.cy = next;
                self.cx = row.chars[..byte].chars().count();
                // 次の scroll() で一致行が画面の一番上に来るようにする (kilo の小技)
                self.rowoff = n;
                return;
            }
        }
    }

    // ------------------------------------------------------------ 描画

    /// カーソルが画面内に入るように rowoff / coloff を調整する
    pub fn scroll(&mut self) {
        let rx = self.rx();
        if self.cy < self.rowoff {
            self.rowoff = self.cy;
        }
        if self.cy >= self.rowoff + self.screen_rows {
            self.rowoff = self.cy + 1 - self.screen_rows;
        }
        let w = self.rows.get(self.cy).map_or(1, |r| r.width_of(self.cx));
        if rx < self.coloff {
            self.coloff = rx;
        }
        if w > self.screen_cols {
            // 1 列しかない端末では全角文字は表示できないが、カーソルは画面内に保つ。
            self.coloff = rx;
        } else if rx + w > self.coloff + self.screen_cols {
            self.coloff = (rx + w).saturating_sub(self.screen_cols);
        }
    }

    /// 現在の状態から 1 フレーム分の描画内容を作る
    pub fn render(&mut self) -> Screen {
        self.scroll();
        let cols = self.screen_cols;
        let mut rows = Vec::with_capacity(self.screen_rows);
        for y in 0..self.screen_rows {
            let file_row = y + self.rowoff;
            if let Some(row) = self.rows.get(file_row) {
                rows.push(row.render(self.coloff, cols));
            } else if self.rows.is_empty() && y == self.screen_rows / 3 {
                let welcome = truncate_to_width(&format!("Kilo 風エディタ -- version {VERSION}"), cols);
                let pad = (cols - str_width(&welcome)) / 2;
                let mut line = String::new();
                if pad > 0 {
                    line.push('~');
                    line.extend(std::iter::repeat_n(' ', pad - 1));
                }
                line.push_str(&welcome);
                rows.push(line);
            } else {
                rows.push("~".to_string());
            }
        }

        let status = self.status_line();
        let (message, prompt_cursor) = self.message_line();
        let cursor = match prompt_cursor {
            Some(x) => (x, self.screen_rows + 1),
            None => (self.rx() - self.coloff, self.cy - self.rowoff),
        };
        Screen { rows, status, message, cursor }
    }

    fn status_line(&self) -> String {
        let cols = self.screen_cols;
        let name = self
            .filename
            .as_ref()
            .map_or("[無名]".to_string(), |p| p.display().to_string());
        let left = format!(
            "{} - {} 行{}",
            truncate_to_width(&name, 20),
            self.rows.len(),
            if self.dirty { " (変更あり)" } else { "" }
        );
        let right = format!("{}/{}", self.cy + 1, self.rows.len());
        let mut line = truncate_to_width(&left, cols);
        let used = str_width(&line);
        let rw = str_width(&right);
        if used + rw < cols {
            line.extend(std::iter::repeat_n(' ', cols - used - rw));
            line.push_str(&right);
        } else {
            line.extend(std::iter::repeat_n(' ', cols - used));
        }
        line
    }

    /// メッセージバーの内容と、入力中ならカーソル列
    fn message_line(&self) -> (String, Option<usize>) {
        let cols = self.screen_cols;
        if let Some(p) = &self.prompt {
            let (label, help) = match p.kind {
                PromptKind::SaveAs => ("保存先: ", " (Enter で保存 / ESC で中止)"),
                PromptKind::Find => ("検索: ", " (矢印で次/前 / Enter 確定 / ESC 中止)"),
            };
            let head = format!("{label}{}", p.input);
            let full = truncate_to_width(&format!("{head}{help}"), cols);
            let cursor = (p.kind == PromptKind::SaveAs).then(|| str_width(&head).min(cols - 1));
            return (full, cursor);
        }
        let fresh = self.status_time.is_some_and(|t| t.elapsed() < MESSAGE_TIMEOUT);
        if fresh {
            (truncate_to_width(&self.status_msg, cols), None)
        } else {
            (String::new(), None)
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
    fn insert_into_empty_buffer() {
        let mut e = editor_with("");
        type_str(&mut e, "abc");
        assert_eq!(e.text(), "abc\n");
        assert_eq!(e.cx, 3);
        assert!(e.dirty);
    }

    #[test]
    fn newline_splits_row() {
        let mut e = editor_with("hello");
        e.cx = 2;
        e.process_key(Key::Enter);
        assert_eq!(e.text(), "he\nllo\n");
        assert_eq!((e.cx, e.cy), (0, 1));
    }

    #[test]
    fn backspace_joins_rows() {
        let mut e = editor_with("ab\ncd");
        e.cy = 1;
        e.process_key(Key::Backspace);
        assert_eq!(e.text(), "abcd\n");
        assert_eq!((e.cx, e.cy), (2, 0));
        e.process_key(Key::Backspace);
        assert_eq!(e.text(), "acd\n");
    }

    #[test]
    fn delete_forward_joins_rows() {
        let mut e = editor_with("ab\ncd");
        e.cx = 2;
        e.process_key(Key::Delete);
        assert_eq!(e.text(), "abcd\n");
    }

    #[test]
    fn edit_japanese() {
        let mut e = editor_with("あいう");
        e.move_cursor(Key::Right);
        type_str(&mut e, "X");
        assert_eq!(e.text(), "あXいう\n");
        assert_eq!(e.rx(), 3);
        e.process_key(Key::Backspace);
        e.process_key(Key::Backspace);
        assert_eq!(e.text(), "いう\n");
    }

    #[test]
    fn vertical_move_keeps_display_column() {
        let mut e = editor_with("abcd\nあい");
        e.cx = 2; // rx = 2
        e.move_cursor(Key::Down);
        assert_eq!((e.cx, e.cy), (1, 1)); // 「い」の位置 (rx=2)
        e.move_cursor(Key::Up);
        assert_eq!(e.cx, 2);
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
        // 60 列目にカーソル。画面幅 20 なので coloff = 41
        assert_eq!(e.coloff, 41);
        assert_eq!(s.cursor.0, 19);
        assert!(str_width(&s.rows[0]) <= 20);
    }

    #[test]
    fn wide_char_on_one_column_terminal_does_not_underflow() {
        let mut e = Editor::new(1, 3);
        e.load_text("あ");
        let screen = e.render();
        assert_eq!(screen.cursor, (0, 0));
        assert_eq!(str_width(&screen.rows[0]), 1);
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
    fn quit_requires_confirmation_when_dirty() {
        let mut e = editor_with("");
        type_str(&mut e, "x");
        e.process_key(Key::Ctrl('q'));
        assert!(!e.should_quit);
        e.process_key(Key::Ctrl('q'));
        assert!(e.should_quit);
    }

    #[test]
    fn quit_immediately_when_clean() {
        let mut e = editor_with("abc");
        e.process_key(Key::Ctrl('q'));
        assert!(e.should_quit);
    }

    #[test]
    fn incremental_search() {
        let mut e = editor_with("apple\nbanana\ncherry\nbanana2");
        e.process_key(Key::Ctrl('f'));
        type_str(&mut e, "nan");
        assert_eq!((e.cx, e.cy), (2, 1));
        e.process_key(Key::Down); // 次を検索
        assert_eq!((e.cx, e.cy), (2, 3));
        e.process_key(Key::Down); // 先頭に戻る
        assert_eq!(e.cy, 1);
        e.process_key(Key::Enter);
        assert_eq!(e.cy, 1);
        // 検索終了後は普通に編集できる
        type_str(&mut e, "Z");
        assert_eq!(e.rows[1].chars, "baZnana");
    }

    #[test]
    fn search_escape_restores_cursor() {
        let mut e = editor_with("abc\nxyz");
        e.process_key(Key::Ctrl('f'));
        type_str(&mut e, "y");
        assert_eq!(e.cy, 1);
        e.process_key(Key::Esc);
        assert_eq!((e.cx, e.cy), (0, 0));
    }

    #[test]
    fn search_japanese_sets_char_index() {
        let mut e = editor_with("今日は良い天気");
        e.process_key(Key::Ctrl('f'));
        type_str(&mut e, "天気");
        assert_eq!(e.cx, 5);
        assert_eq!(e.rx(), 10);
    }

    #[test]
    fn save_and_open_roundtrip() {
        let path = std::env::temp_dir().join(format!("p05_editor_test_{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut e = Editor::new(80, 24);
        e.open(&path).unwrap(); // 存在しない → 新規バッファ
        assert!(e.rows.is_empty());
        type_str(&mut e, "こんにちは");
        e.process_key(Key::Enter);
        type_str(&mut e, "world");
        e.process_key(Key::Ctrl('s'));
        assert!(!e.dirty);
        assert_eq!(fs::read_to_string(&path).unwrap(), "こんにちは\nworld\n");

        let mut e2 = Editor::new(80, 24);
        e2.open(&path).unwrap();
        assert_eq!(e2.rows.len(), 2);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn save_as_prompt_when_no_filename() {
        let path = std::env::temp_dir().join(format!("p05_editor_saveas_{}.txt", std::process::id()));
        let mut e = editor_with("");
        type_str(&mut e, "hi");
        e.process_key(Key::Ctrl('s'));
        let s = e.render();
        assert!(s.message.starts_with("保存先: "));
        assert_eq!(s.cursor.1, e.screen_rows + 1);
        type_str(&mut e, path.to_str().unwrap());
        e.process_key(Key::Enter);
        assert_eq!(fs::read_to_string(&path).unwrap(), "hi\n");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn welcome_message_on_empty_buffer() {
        let mut e = Editor::new(40, 12);
        let s = e.render();
        assert!(s.rows.iter().any(|r| r.contains("Kilo")));
        assert_eq!(s.rows[0], "~");
    }
}
