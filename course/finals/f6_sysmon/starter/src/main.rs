//! htop 風システムモニタ (ratatui)。q で終了。
//!
//! イベントループと ps の取得スレッドは用意してある。
//! TODO: `ui` 関数で CPU グラフ (Sparkline) とプロセス表 (Table) とステータス行を描こう (SPEC 4章)。

use f6_sysmon::{App, Input, Snapshot, parse_ps};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::{Block, Paragraph};
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

/// 描画。テーブルの表示行数を返す (PageUp/Down の幅に使う)
fn ui(frame: &mut Frame, app: &mut App) -> usize {
    // TODO: Layout::vertical([Length(5), Min(3), Length(1)]) で 3 分割し、
    //       - 上: Sparkline (app.history().sparkline_data())
    //       - 中: Table (app.visible() を app.scroll_offset(高さ) から表示し、選択行を反転)
    //       - 下: ステータス行 (モード・フィルタ・一時停止)
    let text = format!(
        "TODO: ここに CPU グラフとプロセス表を描く\n\nプロセス数: {}  (q で終了)",
        app.visible().len()
    );
    frame.render_widget(
        Paragraph::new(text).block(Block::bordered().title(" f6_sysmon ")),
        frame.area(),
    );
    10
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
