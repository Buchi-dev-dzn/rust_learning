//! 使い方: cargo run -p p06_ratatui_app [-- <ディレクトリ>]

use std::io;

use p06_ratatui_app::{App, ui};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event};

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        // Resize などキー以外のイベントは、次のループで再描画されるだけで十分
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    // 端末を初期化する前にディレクトリを読む (失敗しても端末が汚れない)
    let mut app = App::new(dir)?;

    // init() は raw モード + 代替スクリーンに入り、パニック時に restore するフックも設定する
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}
