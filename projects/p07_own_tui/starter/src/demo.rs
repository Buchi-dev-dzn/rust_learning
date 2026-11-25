//! 自作フレームワークの上に作るデモアプリ: タブ付きのタスクリスト。
//!
//! 状態の更新 (`handle_key`) と描画 (`render`) は端末に依存しないので、単体テストできる。

use crate::buffer::Buffer;
use crate::input::Key;
use crate::layout::{Constraint, Layout};
use crate::style::{Color, Style};
use crate::widgets::{Block, Clear, Gauge, List, ListItem, Paragraph, Tabs, Widget};

pub const TAB_TITLES: [&str; 3] = ["タスク", "ヘルプ", "情報"];
const SPINNER: [char; 4] = ['|', '/', '-', '\\'];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// 新しいタスクの名前を入力中
    Adding(String),
}

#[derive(Debug)]
pub struct App {
    pub tab: usize,
    pub tasks: Vec<Task>,
    pub selected: usize,
    pub mode: Mode,
    pub should_quit: bool,
    pub ticks: u64,
    /// 直前のフレームで端末に送ったバイト数 (main が設定する)
    pub last_output_len: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let tasks = ["Rect と Buffer を作る", "差分描画を実装する", "Layout で画面を分割する", "Widget を増やす"]
            .iter()
            .enumerate()
            .map(|(i, t)| Task { title: t.to_string(), done: i < 2 })
            .collect();
        App {
            tab: 0,
            tasks,
            selected: 0,
            mode: Mode::Normal,
            should_quit: false,
            ticks: 0,
            last_output_len: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.ticks += 1;
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    pub fn handle_key(&mut self, key: Key) {
        if let Mode::Adding(input) = &mut self.mode {
            match key {
                Key::Esc => self.mode = Mode::Normal,
                Key::Enter => {
                    let title = input.trim().to_string();
                    if !title.is_empty() {
                        self.tasks.push(Task { title, done: false });
                        self.selected = self.tasks.len() - 1;
                    }
                    self.mode = Mode::Normal;
                }
                Key::Backspace => {
                    input.pop();
                }
                Key::Char(c) => input.push(c),
                _ => {}
            }
            return;
        }
        match key {
            Key::Char('q') | Key::Ctrl('c') | Key::Ctrl('d') => self.should_quit = true,
            Key::Tab | Key::Right | Key::Char('l') => self.tab = (self.tab + 1) % TAB_TITLES.len(),
            Key::BackTab | Key::Left | Key::Char('h') => {
                self.tab = (self.tab + TAB_TITLES.len() - 1) % TAB_TITLES.len()
            }
            Key::Char(c @ '1'..='3') => self.tab = c as usize - '1' as usize,
            _ if self.tab != 0 => {}
            Key::Char('j') | Key::Down => {
                if self.selected + 1 < self.tasks.len() {
                    self.selected += 1;
                }
            }
            Key::Char('k') | Key::Up => self.selected = self.selected.saturating_sub(1),
            Key::Char(' ') | Key::Enter => {
                if let Some(t) = self.tasks.get_mut(self.selected) {
                    t.done = !t.done;
                }
            }
            Key::Char('a') => self.mode = Mode::Adding(String::new()),
            Key::Char('d') | Key::Delete => {
                if self.selected < self.tasks.len() {
                    self.tasks.remove(self.selected);
                    self.selected = self.selected.min(self.tasks.len().saturating_sub(1));
                }
            }
            _ => {}
        }
    }

    /// 画面全体を描く
    pub fn render(&self, buf: &mut Buffer) {
        let area = buf.area;
        let rows = Layout::vertical([Constraint::Length(3), Constraint::Fill(1), Constraint::Length(1)])
            .split(area);

        // タブ見出し
        let block = Block::bordered().title(" 自作 TUI デモ ").border_style(Style::new().fg(Color::Cyan));
        block.render(rows[0], buf);
        Tabs::new(TAB_TITLES.iter().map(|s| s.to_string()).collect())
            .selected(self.tab)
            .highlight_style(Style::new().fg(Color::Black).bg(Color::Yellow).bold())
            .render(block.inner(rows[0]), buf);

        match self.tab {
            0 => self.render_tasks(rows[1], buf),
            1 => self.render_help(rows[1], buf),
            _ => self.render_info(rows[1], buf),
        }

        // フッタ: 操作説明 + poll のタイムアウトで回るスピナー
        let spinner = SPINNER[(self.ticks % 4) as usize];
        let footer = format!(" {spinner} q:終了  Tab/1-3:タブ  j/k:移動  Space:完了  a:追加  d:削除");
        buf.set_style(rows[2], Style::new().reversed());
        buf.set_stringn(rows[2].x, rows[2].y, &footer, rows[2].width as usize, Style::new().reversed());

        if let Mode::Adding(input) = &self.mode {
            let popup = area.centered(40, 3);
            Clear.render(popup, buf);
            let block = Block::bordered()
                .title(" 新しいタスク (Enter:追加 Esc:中止) ")
                .border_style(Style::new().fg(Color::Yellow));
            block.render(popup, buf);
            let inner = block.inner(popup);
            let x = buf.set_stringn(inner.x, inner.y, input, inner.width as usize, Style::new());
            // 入力位置を示す疑似カーソル
            if x < inner.right() {
                buf.set_string(x, inner.y, " ", Style::new().reversed());
            }
        }
    }

    fn render_tasks(&self, area: crate::geometry::Rect, buf: &mut Buffer) {
        let cols = Layout::horizontal([Constraint::Percentage(60), Constraint::Fill(1)]).split(area);
        let items = self
            .tasks
            .iter()
            .map(|t| {
                if t.done {
                    ListItem::new(format!("[x] {}", t.title)).style(Style::new().fg(Color::DarkGray))
                } else {
                    ListItem::new(format!("[ ] {}", t.title))
                }
            })
            .collect();
        List::new(items)
            .selected((!self.tasks.is_empty()).then_some(self.selected))
            .highlight_symbol("> ")
            .highlight_style(Style::new().bg(Color::Blue).fg(Color::White).bold())
            .block(Block::bordered().title(format!(" タスク ({}) ", self.tasks.len())))
            .render(cols[0], buf);

        let right = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(cols[1]);
        let total = self.tasks.len();
        let done = self.done_count();
        let ratio = if total == 0 { 0.0 } else { done as f64 / total as f64 };
        let block = Block::bordered().title(" 進捗 ");
        block.render(right[0], buf);
        Gauge::new(ratio)
            .label(format!("{done}/{total}"))
            .filled_style(Style::new().bg(Color::Green).fg(Color::Black))
            .render(block.inner(right[0]), buf);

        let detail = match self.tasks.get(self.selected) {
            Some(t) => format!(
                "{}\n\n状態: {}",
                t.title,
                if t.done { "完了" } else { "未完了" }
            ),
            None => "タスクがありません。a で追加しよう。".to_string(),
        };
        Paragraph::new(&detail)
            .wrap(true)
            .block(Block::bordered().title(" 詳細 "))
            .render(right[1], buf);
    }

    fn render_help(&self, area: crate::geometry::Rect, buf: &mut Buffer) {
        let text = "\
q / Ctrl-C      終了
Tab / → / l     次のタブ
Shift-Tab / ←   前のタブ
1 2 3           タブを直接選ぶ
j / k / ↓ / ↑   タスクの選択
Space / Enter   完了 / 未完了の切り替え
a               タスクを追加 (日本語も入力できる)
d / Delete      タスクを削除";
        Paragraph::new(text).block(Block::bordered().title(" キー操作 ")).render(area, buf);
    }

    fn render_info(&self, area: crate::geometry::Rect, buf: &mut Buffer) {
        let lines = vec![
            ("このアプリは crossterm も ratatui も使わず、libc だけで動いています。".to_string(), Style::new().bold()),
            (String::new(), Style::new()),
            ("- termios で raw モード、ioctl で端末サイズ、poll で入力待ち".to_string(), Style::new()),
            ("- Buffer に描いて、前フレームとの差分だけを ANSI で出力".to_string(), Style::new()),
            ("- Layout で領域を分割し、Widget がそれぞれの領域に描く".to_string(), Style::new()),
            (String::new(), Style::new()),
            (format!("経過 tick 数: {}", self.ticks), Style::new().fg(Color::Cyan)),
            (
                format!("直前のフレームで送ったバイト数: {}", self.last_output_len),
                Style::new().fg(Color::Cyan),
            ),
            ("(何も操作しないとスピナー 1 文字ぶんだけになる = 差分描画の効果)".to_string(), Style::new().fg(Color::DarkGray)),
        ];
        Paragraph::styled_lines(lines)
            .wrap(true)
            .block(Block::bordered().title(" 情報 "))
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    fn screen(app: &App) -> Vec<String> {
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 16));
        app.render(&mut buf);
        (0..16).map(|y| buf.line(y)).collect()
    }

    #[test]
    fn toggle_move_and_delete() {
        let mut app = App::new();
        assert_eq!(app.done_count(), 2);
        app.handle_key(Key::Char('j'));
        app.handle_key(Key::Char('j'));
        app.handle_key(Key::Char(' '));
        assert!(app.tasks[2].done);
        app.handle_key(Key::Char('d'));
        assert_eq!(app.tasks.len(), 3);
        assert_eq!(app.selected, 2);
        app.handle_key(Key::Char('j')); // 末尾より先には行かない
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn add_task_with_japanese_input() {
        let mut app = App::new();
        app.handle_key(Key::Char('a'));
        for c in "牛乳を買うx".chars() {
            app.handle_key(Key::Char(c));
        }
        app.handle_key(Key::Backspace);
        // 入力中は q で終了しない
        assert!(!app.should_quit);
        app.handle_key(Key::Enter);
        assert_eq!(app.tasks.last().unwrap().title, "牛乳を買う");
        assert_eq!(app.selected, app.tasks.len() - 1);
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn tabs_switch_and_wrap() {
        let mut app = App::new();
        app.handle_key(Key::Tab);
        assert_eq!(app.tab, 1);
        app.handle_key(Key::BackTab);
        app.handle_key(Key::BackTab);
        assert_eq!(app.tab, 2);
        app.handle_key(Key::Char('1'));
        assert_eq!(app.tab, 0);
        app.handle_key(Key::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn render_tasks_tab() {
        let app = App::new();
        let s = screen(&app);
        assert!(s[0].contains("自作 TUI デモ"));
        assert!(s[1].contains("タスク │ ヘルプ"));
        assert!(s.iter().any(|l| l.contains("> [x] Rect と Buffer を作る")));
        assert!(s.iter().any(|l| l.contains("2/4")));
        assert!(s[15].contains("q:終了"));
    }

    #[test]
    fn render_popup_while_adding() {
        let mut app = App::new();
        app.handle_key(Key::Char('a'));
        app.handle_key(Key::Char('あ'));
        let s = screen(&app);
        assert!(s.iter().any(|l| l.contains("新しいタスク")));
        assert!(s.iter().any(|l| l.contains("│あ")));
    }

    #[test]
    fn render_other_tabs() {
        let mut app = App::new();
        app.tab = 1;
        assert!(screen(&app).iter().any(|l| l.contains("キー操作")));
        app.tab = 2;
        assert!(screen(&app).iter().any(|l| l.contains("libc")));
    }
}
