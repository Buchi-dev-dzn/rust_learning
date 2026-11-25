//! デモアプリの状態・更新・描画。端末には一切触れないのでテストできる。

use crate::ansi;
use crate::key::Key;

pub const BOX_W: u16 = 12;
pub const BOX_H: u16 = 5;

/// アプリの状態。
#[derive(Debug)]
pub struct App {
    /// 端末の大きさ (列数, 行数)
    pub cols: u16,
    pub rows: u16,
    /// 箱の左上 (0 始まり)
    pub x: u16,
    pub y: u16,
    /// 最後に押されたキーと、その生のバイト列
    pub last_key: Option<Key>,
    pub last_bytes: Vec<u8>,
    pub quit: bool,
}

impl App {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut app = App {
            cols,
            rows,
            x: 0,
            y: 0,
            last_key: None,
            last_bytes: Vec::new(),
            quit: false,
        };
        app.x = app.max_x() / 2;
        app.y = app.max_y() / 2;
        app
    }

    /// 箱が動ける範囲。1 行目はタイトル、最終行はステータス行に使う。
    fn min_y(&self) -> u16 {
        1
    }
    fn max_x(&self) -> u16 {
        self.cols.saturating_sub(BOX_W)
    }
    fn max_y(&self) -> u16 {
        // 最終行 (rows - 1) はステータス行
        self.rows.saturating_sub(BOX_H + 1).max(self.min_y())
    }

    fn clamp(&mut self) {
        self.x = self.x.min(self.max_x());
        self.y = self.y.clamp(self.min_y(), self.max_y());
    }

    /// キー入力で状態を更新する。
    pub fn handle_key(&mut self, key: Key) {
        // TODO(STEP 7): match key { ... } で状態を更新する
        //   q / Ctrl-C / Ctrl-Q        -> self.quit = true
        //   Up/k, Down/j               -> y を 1 ずつ (saturating_sub / saturating_add)
        //   Left/h, Right/l            -> x を 2 ずつ (端末の文字は縦長なので横は 2)
        //   Home/End/PageUp/PageDown   -> 端まで移動
        // 最後に self.last_key = Some(key) と self.clamp()
        let _ = key;
        todo!("handle_key")
    }

    /// 端末サイズの変更を反映する。
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.clamp();
    }

    /// 画面全体を描画する ANSI 文字列を作る。
    ///
    /// raw モードでは出力の改行変換 (OPOST) も切っているので、`\n` だけでは行頭に戻らない。
    /// そのため常に `move_to` で位置を指定して書く。
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(ansi::RESET);
        out.push_str(ansi::CLEAR_SCREEN);

        // タイトル行
        out.push_str(&ansi::move_to(0, 0));
        out.push_str(ansi::BOLD);
        out.push_str(&truncate(
            " ansi_raw demo - arrows/hjkl: move  Home/End/PgUp/PgDn  q: quit",
            self.cols,
        ));
        out.push_str(ansi::RESET);

        // 箱
        let color = ansi::fg(45);
        let inner = (BOX_W - 2) as usize;
        for dy in 0..BOX_H {
            let row = self.y + dy;
            if row + 1 >= self.rows || self.x + BOX_W > self.cols {
                break; // 端末が小さすぎるときは描けるところまで
            }
            let line = if dy == 0 {
                format!("┌{}┐", "─".repeat(inner))
            } else if dy == BOX_H - 1 {
                format!("└{}┘", "─".repeat(inner))
            } else if dy == BOX_H / 2 {
                format!("│{:^inner$}│", "Rust!")
            } else {
                format!("│{}│", " ".repeat(inner))
            };
            out.push_str(&ansi::move_to(row, self.x));
            out.push_str(&color);
            out.push_str(&line);
            out.push_str(ansi::RESET);
        }

        // ステータス行 (反転表示)
        if self.rows > 0 {
            out.push_str(&ansi::move_to(self.rows - 1, 0));
            out.push_str(ansi::REVERSE);
            let status = format!("{:<w$}", self.status_line(), w = self.cols as usize);
            out.push_str(&truncate(&status, self.cols));
            out.push_str(ansi::RESET);
        }
        out
    }

    /// ステータス行の文字列 (ASCII のみなので 1 文字 = 1 列)。
    pub fn status_line(&self) -> String {
        let key = match &self.last_key {
            Some(k) => format!("{k:?}"),
            None => "-".to_string(),
        };
        let bytes: Vec<String> = self.last_bytes.iter().map(|b| format!("{b:02x}")).collect();
        format!(
            " key: {key}  bytes: [{}]  pos: {},{}  size: {}x{}",
            bytes.join(" "),
            self.x,
            self.y,
            self.cols,
            self.rows
        )
    }
}

/// 文字数で切り詰める (この関数は 1 文字 = 1 列の文字列を前提にしている)。
fn truncate(s: &str, width: u16) -> String {
    s.chars().take(width as usize).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_centered() {
        let app = App::new(80, 24);
        assert_eq!(app.x, (80 - BOX_W) / 2);
        assert_eq!(app.y, (24 - BOX_H - 1) / 2);
    }

    #[test]
    fn moves_and_clamps() {
        let mut app = App::new(40, 12);
        for _ in 0..100 {
            app.handle_key(Key::Up);
        }
        assert_eq!(app.y, 1, "タイトル行には入らない");
        for _ in 0..100 {
            app.handle_key(Key::Right);
        }
        assert_eq!(app.x, 40 - BOX_W);
        for _ in 0..100 {
            app.handle_key(Key::Char('j'));
        }
        assert_eq!(app.y, 12 - BOX_H - 1, "ステータス行には入らない");
        app.handle_key(Key::Home);
        assert_eq!(app.x, 0);
        assert_eq!(app.last_key, Some(Key::Home));
    }

    #[test]
    fn quit_keys() {
        for k in [Key::Char('q'), Key::Ctrl('c'), Key::Ctrl('q')] {
            let mut app = App::new(80, 24);
            app.handle_key(k);
            assert!(app.quit);
        }
        let mut app = App::new(80, 24);
        app.handle_key(Key::Char('x'));
        assert!(!app.quit);
    }

    #[test]
    fn resize_clamps_position() {
        let mut app = App::new(80, 24);
        app.handle_key(Key::End);
        app.handle_key(Key::PageDown);
        app.resize(30, 10);
        assert_eq!(app.x, 30 - BOX_W);
        assert_eq!(app.y, 10 - BOX_H - 1);
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        let mut app = App::new(3, 2);
        app.handle_key(Key::Down);
        app.handle_key(Key::Right);
        let _ = app.render();
        app.resize(0, 0);
        let _ = app.render();
    }

    #[test]
    fn render_contains_box_and_status() {
        let mut app = App::new(80, 24);
        app.last_bytes = vec![0x1b, 0x5b, 0x41];
        app.handle_key(Key::Up);
        let s = app.render();
        assert!(s.starts_with("\x1b[0m\x1b[2J"));
        assert!(s.contains(&ansi::move_to(app.y, app.x)));
        assert!(s.contains("┌──────────┐"));
        assert!(s.contains("key: Up"));
        assert!(s.contains("bytes: [1b 5b 41]"));
        assert!(s.contains(&ansi::move_to(23, 0)));
    }
}
