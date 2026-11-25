//! crossterm 入門 (完成版): 端末の準備・イベントループ・描画。

use std::io::{self, Stdout, Write};
use std::process::ExitCode;
use std::time::Instant;

use crossterm::event::{self, Event};
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute, queue};

use p04_crossterm_basics::{App, Msg, TICK_RATE};

/// raw モード + 代替スクリーン + カーソル非表示 の間だけ生きるガード。
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<TerminalGuard> {
        terminal::enable_raw_mode()?;
        // ここで失敗しても raw モードは戻したいので、先にガードを作っておく
        let guard = TerminalGuard;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// 端末を元に戻す。エラーは無視する (後片付け中にできることはないため)。
fn restore_terminal() {
    let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

/// パニック時、メッセージを表示する前に端末を戻す。
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
}

fn draw(out: &mut Stdout, app: &App) -> io::Result<()> {
    // queue! は溜めるだけ。最後の flush で一度に送るのでちらつきにくい
    queue!(out, terminal::Clear(ClearType::All))?;
    let rows = app.size.1 as usize;
    for (i, line) in app.view().iter().enumerate().take(rows) {
        queue!(out, cursor::MoveTo(0, i as u16))?;
        if i == 0 {
            queue!(
                out,
                SetAttribute(Attribute::Bold),
                SetAttribute(Attribute::Reverse),
                Print(format!(" {line} ")),
                SetAttribute(Attribute::Reset)
            )?;
        } else {
            queue!(out, Print(line))?;
        }
    }
    out.flush()
}

fn run() -> io::Result<()> {
    install_panic_hook();
    let _guard = TerminalGuard::enter()?;

    // 環境によってはサイズが 0 x 0 と報告されることがある (パイプ経由の擬似端末など)
    let (cols, rows) = match terminal::size()? {
        (0, _) | (_, 0) => (80, 24),
        size => size,
    };
    let mut app = App::new(cols, rows);
    let mut out = io::stdout();
    let mut last_tick = Instant::now();
    let mut dirty = true;

    while !app.quit {
        if dirty {
            draw(&mut out, &app)?;
            dirty = false;
        }

        // 次のティックまでの残り時間だけイベントを待つ
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let msg = match event::read()? {
                Event::Key(key) => Some(Msg::Key(key)),
                Event::Resize(c, r) => Some(Msg::Resize(c, r)),
                _ => None,
            };
            if let Some(msg) = msg {
                dirty |= app.update(msg);
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            dirty |= app.update(Msg::Tick);
            last_tick = Instant::now();
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("crossterm_basics: {e}");
            eprintln!("(端末 (TTY) の中で実行してください)");
            ExitCode::FAILURE
        }
    }
}
