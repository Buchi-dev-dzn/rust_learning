//! 自作 TUI フレームワークのデモ。使い方: cargo run -p p07_own_tui

use std::io;
use std::time::Duration;

use p07_own_tui::demo::App;
use p07_own_tui::terminal::{Event, Events, Terminal};

/// 入力がなくてもこの間隔で Tick が来る (スピナーが回る)
const TICK: Duration = Duration::from_millis(250);

fn main() -> io::Result<()> {
    let mut app = App::new();
    // Terminal は Drop で画面と raw モードを元に戻す (パニック時はフックが戻す)
    let mut terminal = Terminal::new()?;
    let mut events = Events::new();

    while !app.should_quit {
        terminal.draw(|buf| app.render(buf))?;
        app.last_output_len = terminal.last_output_len;
        for event in events.next(TICK)? {
            match event {
                Event::Key(key) => app.handle_key(key),
                Event::Tick => app.on_tick(),
                // サイズ変更は次の draw が自動で全体を描き直すので何もしなくてよい
                Event::Resize(_, _) => {}
            }
        }
    }
    Ok(())
}
