//! crossterm 入門 (完成版): アプリの状態と更新ロジック。
//!
//! 端末には触れない。main.rs が crossterm のイベントを `Msg` に変換して `App::update` に渡し、
//! `App::view` が返す行を画面に書く。この「Msg → update → view」の形は Elm アーキテクチャと呼ばれ、
//! ratatui や自作フレームワークでもそのまま使う。

use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// ティック (定期処理) の間隔。
pub const TICK_RATE: Duration = Duration::from_millis(250);
/// キーログに残す件数。
pub const KEY_LOG_LEN: usize = 8;

/// アプリに届くメッセージ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

/// アプリの状態。
#[derive(Debug)]
pub struct App {
    pub counter: i64,
    /// 一時停止していない間に経過したティック数
    pub ticks: u64,
    pub paused: bool,
    pub size: (u16, u16),
    pub key_log: VecDeque<String>,
    pub quit: bool,
}

impl App {
    pub fn new(cols: u16, rows: u16) -> Self {
        App {
            counter: 0,
            ticks: 0,
            paused: false,
            size: (cols, rows),
            key_log: VecDeque::with_capacity(KEY_LOG_LEN),
            quit: false,
        }
    }

    /// 状態を更新する。再描画が必要なら true を返す。
    pub fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Key(key) => {
                // Windows などではキーを離したとき (Release) にもイベントが来るので、押下だけを扱う
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                self.log_key(&key);
                self.handle_key(key);
                true
            }
            Msg::Resize(cols, rows) => {
                self.size = (cols, rows);
                true
            }
            Msg::Tick => {
                if self.paused {
                    false
                } else {
                    self.ticks += 1;
                    true
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('c') if ctrl => self.quit = true,
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Char('+') | KeyCode::Char('k') | KeyCode::Up => self.counter += 1,
            KeyCode::Char('-') | KeyCode::Char('j') | KeyCode::Down => self.counter -= 1,
            KeyCode::Char('r') => {
                self.counter = 0;
                self.ticks = 0;
            }
            KeyCode::Char(' ') => self.paused = !self.paused,
            // パニック時の後片付けを確かめるためのキー
            KeyCode::Char('!') => panic!("'!' が押されたのでわざとパニックしました"),
            _ => {}
        }
    }

    fn log_key(&mut self, key: &KeyEvent) {
        if self.key_log.len() == KEY_LOG_LEN {
            self.key_log.pop_front();
        }
        self.key_log.push_back(format_key(key));
    }

    /// 経過時間 (秒)。
    pub fn elapsed_secs(&self) -> f64 {
        self.ticks as f64 * TICK_RATE.as_secs_f64()
    }

    /// 画面に表示する行。1 行目から順に並ぶ。
    pub fn view(&self) -> Vec<String> {
        let mut lines = vec![
            "crossterm basics".to_string(),
            String::new(),
            format!("  カウンタ   : {}", self.counter),
            format!(
                "  経過時間   : {:.2} 秒{}",
                self.elapsed_secs(),
                if self.paused { " (一時停止中)" } else { "" }
            ),
            format!("  ティック数 : {}", self.ticks),
            format!("  端末サイズ : {} x {}", self.size.0, self.size.1),
            String::new(),
            "  最近のキー:".to_string(),
        ];
        if self.key_log.is_empty() {
            lines.push("    (まだありません)".to_string());
        }
        for k in &self.key_log {
            lines.push(format!("    {k}"));
        }
        lines.push(String::new());
        lines.push(
            "  +/k/Up: 増やす  -/j/Down: 減らす  r: リセット  Space: 一時停止  !: パニック  q/Esc: 終了"
                .to_string(),
        );
        lines
    }
}

/// キーイベントを "Ctrl+Shift+a" のような文字列にする。
pub fn format_key(key: &KeyEvent) -> String {
    let mut s = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("Ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("Alt+");
    }
    // 大文字などの印字文字はそれ自体に Shift が表れるので、文字以外のときだけ表示する
    if key.modifiers.contains(KeyModifiers::SHIFT) && !matches!(key.code, KeyCode::Char(_)) {
        s.push_str("Shift+");
    }
    match key.code {
        KeyCode::Char(' ') => s.push_str("Space"),
        KeyCode::Char(c) => s.push(c),
        KeyCode::F(n) => s.push_str(&format!("F{n}")),
        KeyCode::Up => s.push_str("Up"),
        KeyCode::Down => s.push_str("Down"),
        KeyCode::Left => s.push_str("Left"),
        KeyCode::Right => s.push_str("Right"),
        KeyCode::Enter => s.push_str("Enter"),
        KeyCode::Esc => s.push_str("Esc"),
        KeyCode::Backspace => s.push_str("Backspace"),
        KeyCode::Tab => s.push_str("Tab"),
        other => s.push_str(&format!("{other:?}")),
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventState;

    fn key(code: KeyCode) -> Msg {
        Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn counter_keys() {
        let mut app = App::new(80, 24);
        app.update(key(KeyCode::Char('+')));
        app.update(key(KeyCode::Up));
        app.update(key(KeyCode::Char('k')));
        app.update(key(KeyCode::Char('-')));
        assert_eq!(app.counter, 2);
        app.update(key(KeyCode::Char('r')));
        assert_eq!(app.counter, 0);
    }

    #[test]
    fn quit_keys() {
        for m in [
            key(KeyCode::Char('q')),
            key(KeyCode::Esc),
            Msg::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        ] {
            let mut app = App::new(80, 24);
            app.update(m);
            assert!(app.quit);
        }
        let mut app = App::new(80, 24);
        app.update(key(KeyCode::Char('c')));
        assert!(!app.quit, "Ctrl なしの c では終了しない");
    }

    #[test]
    fn ignores_key_release() {
        let mut app = App::new(80, 24);
        let release = KeyEvent {
            code: KeyCode::Char('+'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        };
        assert!(!app.update(Msg::Key(release)));
        assert_eq!(app.counter, 0);
        assert!(app.key_log.is_empty());
    }

    #[test]
    fn tick_and_pause() {
        let mut app = App::new(80, 24);
        assert!(app.update(Msg::Tick));
        assert!(app.update(Msg::Tick));
        assert_eq!(app.ticks, 2);
        assert_eq!(app.elapsed_secs(), 0.5);
        app.update(key(KeyCode::Char(' ')));
        assert!(!app.update(Msg::Tick), "一時停止中は再描画不要");
        assert_eq!(app.ticks, 2);
        app.update(key(KeyCode::Char(' ')));
        app.update(Msg::Tick);
        assert_eq!(app.ticks, 3);
    }

    #[test]
    fn resize_updates_size() {
        let mut app = App::new(80, 24);
        assert!(app.update(Msg::Resize(100, 30)));
        assert_eq!(app.size, (100, 30));
        assert!(app.view().iter().any(|l| l.contains("100 x 30")));
    }

    #[test]
    fn key_log_keeps_last_n() {
        let mut app = App::new(80, 24);
        for c in "abcdefghij".chars() {
            app.update(key(KeyCode::Char(c)));
        }
        assert_eq!(app.key_log.len(), KEY_LOG_LEN);
        assert_eq!(app.key_log.front().unwrap(), "c");
        assert_eq!(app.key_log.back().unwrap(), "j");
    }

    #[test]
    fn format_keys() {
        let k = |c, m| format_key(&KeyEvent::new(c, m));
        assert_eq!(k(KeyCode::Char('a'), KeyModifiers::NONE), "a");
        assert_eq!(k(KeyCode::Char('c'), KeyModifiers::CONTROL), "Ctrl+c");
        assert_eq!(k(KeyCode::Char('A'), KeyModifiers::SHIFT), "A");
        assert_eq!(k(KeyCode::Up, KeyModifiers::SHIFT), "Shift+Up");
        assert_eq!(k(KeyCode::Char(' '), KeyModifiers::NONE), "Space");
        assert_eq!(k(KeyCode::F(5), KeyModifiers::ALT), "Alt+F5");
    }

    #[test]
    #[should_panic(expected = "わざとパニック")]
    fn bang_panics() {
        let mut app = App::new(80, 24);
        app.update(key(KeyCode::Char('!')));
    }

    #[test]
    fn view_shows_state() {
        let mut app = App::new(80, 24);
        let v = app.view();
        assert!(v.iter().any(|l| l.contains("(まだありません)")));
        app.update(key(KeyCode::Char('+')));
        let v = app.view();
        assert!(v.iter().any(|l| l.contains("カウンタ   : 1")));
        assert!(v.iter().any(|l| l.trim() == "+"));
    }
}
