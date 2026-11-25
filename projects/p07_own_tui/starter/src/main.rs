//! 自作フレームワークのデモ (骨組みを完成させてから実行する)。

use std::io;
use std::time::Duration;

use p07_own_tui_starter::demo::App;
use p07_own_tui_starter::terminal::{Event, Events, Terminal};

fn main() -> io::Result<()> {
    let mut app = App::new();
    // Terminal の Drop とパニックフックが raw モード・代替スクリーンを戻す。
    let mut terminal = Terminal::new()?;
    let mut events = Events::new();
    while !app.should_quit {
        terminal.draw(|buf| app.render(buf))?;
        app.last_output_len = terminal.last_output_len;
        for event in events.next(Duration::from_millis(250))? {
            match event {
                Event::Key(key) => app.handle_key(key),
                Event::Tick => app.on_tick(),
                Event::Resize(_, _) => {} // 次の draw でサイズ変更を反映
            }
        }
    }
    Ok(())
}
