//! htop 風システムモニタ (ratatui)。q で終了。
//!
//! - 取得スレッド: 1 秒ごとに `ps` を実行し、Snapshot を mpsc で送る
//! - メインスレッド: キー入力とスナップショットを受けて App を更新し、描画する

use f6_sysmon::{App, Input, Mode, Snapshot, SortKey, parse_ps};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Paragraph, Row, Sparkline, Table};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

fn collect_snapshot() -> Option<Snapshot> {
    let out = Command::new("ps")
        .args(["-axo", "pid,pcpu,pmem,comm"])
        .env("LC_ALL", "C") // 小数点を '.' にする
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    Some(Snapshot {
        processes: parse_ps(&text),
    })
}

fn to_input(code: KeyCode) -> Option<Input> {
    Some(match code {
        KeyCode::Char(c) => Input::Char(c),
        KeyCode::Up => Input::Up,
        KeyCode::Down => Input::Down,
        KeyCode::PageUp => Input::PageUp,
        KeyCode::PageDown => Input::PageDown,
        KeyCode::Home => Input::Home,
        KeyCode::End => Input::End,
        KeyCode::Enter => Input::Enter,
        KeyCode::Esc => Input::Esc,
        KeyCode::Backspace => Input::Backspace,
        _ => return None,
    })
}

fn sort_label(key: SortKey) -> &'static str {
    match key {
        SortKey::Pid => "PID",
        SortKey::Cpu => "CPU%",
        SortKey::Mem => "MEM%",
        SortKey::Command => "COMMAND",
    }
}

/// 描画。テーブルの表示行数を返す (PageUp/Down の幅に使う)
fn ui(frame: &mut Frame, app: &mut App) -> usize {
    let [spark_area, table_area, status_area] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let latest = app.history().latest().unwrap_or(0.0);
    let data = app.history().sparkline_data();
    let spark = Sparkline::default()
        .block(Block::bordered().title(format!(" CPU 合計 {latest:.1}% ")))
        .data(&data)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(spark, spark_area);

    // 枠 2 行 + ヘッダ 1 行を引いた残りがデータ行
    let height = table_area.height.saturating_sub(3) as usize;
    let offset = app.scroll_offset(height);
    let selected = app.selected();
    let visible = app.visible();
    let total = visible.len();
    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, p)| {
            let row = Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(format!("{:5.1}", p.cpu)),
                Cell::from(format!("{:5.1}", p.mem)),
                Cell::from(p.command.clone()),
            ]);
            if i == selected {
                row.style(Style::default().add_modifier(Modifier::REVERSED))
            } else {
                row
            }
        })
        .collect();
    let arrow = if app.sort_desc() { "▼" } else { "▲" };
    let header = ["PID", "CPU%", "MEM%", "COMMAND"].map(|h| {
        if h == sort_label(app.sort_key()) {
            Cell::from(format!("{h}{arrow}"))
        } else {
            Cell::from(h)
        }
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Fill(1),
        ],
    )
    .header(Row::new(header).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::bordered().title(format!(" プロセス {}/{} ", total, app.process_count())));
    frame.render_widget(table, table_area);

    let status = match app.mode() {
        Mode::Filter => format!(" /{}▏ (Enter: 確定  Esc: 解除)", app.filter()),
        Mode::Normal => format!(
            " q:終了 j/k:移動 c/m/p/n:ソート /:絞り込み Space:{}  filter=\"{}\"",
            if app.paused() { "再開 [一時停止中]" } else { "一時停止" },
            app.filter()
        ),
    };
    frame.render_widget(
        Paragraph::new(Line::from(status)).style(Style::default().fg(Color::Black).bg(Color::Cyan)),
        status_area,
    );
    height
}

fn main() -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel::<Snapshot>();
    std::thread::spawn(move || {
        loop {
            if let Some(s) = collect_snapshot() {
                if tx.send(s).is_err() {
                    break; // メインスレッドが終了した
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });

    // ratatui::init は raw モード + 代替画面 + パニック時の復元フックを設定する
    let mut terminal = ratatui::init();
    let mut app = App::new(120);
    let mut page = 10;
    let result = (|| -> std::io::Result<()> {
        while !app.should_quit() {
            while let Ok(s) = rx.try_recv() {
                app.update(s);
            }
            terminal.draw(|f| page = ui(f, &mut app).max(1))?;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(k) = event::read()? {
                    if k.kind != KeyEventKind::Press {
                        continue;
                    }
                    if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }
                    if let Some(input) = to_input(k.code) {
                        app.handle_input(input, page);
                    }
                }
            }
        }
        Ok(())
    })();
    ratatui::restore();
    result
}
