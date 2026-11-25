//! crossterm 入門 (starter): 端末の準備・イベントループ・描画。

#![allow(unused_imports, unused_mut)]

use std::io::{self, Stdout, Write};
use std::process::ExitCode;
use std::time::Instant;

use crossterm::event::{self, Event};
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute, queue};

use p04_crossterm_basics_starter::{App, Msg, TICK_RATE};

/// raw モード + 代替スクリーン + カーソル非表示 の間だけ生きるガード。
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<TerminalGuard> {
        // TODO(STEP 1): terminal::enable_raw_mode() し、
        //               execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)
        todo!("TerminalGuard::enter")
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// 端末を元に戻す。エラーは無視する。
fn restore_terminal() {
    // TODO(STEP 1): cursor::Show, terminal::LeaveAlternateScreen, disable_raw_mode
}

/// パニック時、メッセージを表示する前に端末を戻す。
fn install_panic_hook() {
    // TODO(STEP 2): take_hook / set_hook
}

fn draw(out: &mut Stdout, app: &App) -> io::Result<()> {
    // TODO(STEP 1): まずは Clear(All) → MoveTo(0, 0) → Print("Hello") だけで OK
    // TODO(STEP 7): app.view() の各行を MoveTo(0, i) して Print。1 行目は太字 + 反転
    let _ = app;
    queue!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    out.flush()
}

fn run() -> io::Result<()> {
    install_panic_hook();
    let _guard = TerminalGuard::enter()?;

    let (cols, rows) = match terminal::size()? {
        (0, _) | (_, 0) => (80, 24),
        size => size,
    };
    let mut app = App::new(cols, rows);
    let mut out = io::stdout();

    while !app.quit {
        draw(&mut out, &app)?;

        // TODO(STEP 3): event::read() でイベントを待ち、Event::Key を Msg::Key にして app.update に渡す
        // TODO(STEP 4): event::poll(timeout) で「次のティックまで」だけ待ち、時間が来たら Msg::Tick
        // TODO(STEP 5): Event::Resize(c, r) を Msg::Resize に
        let _ = (TICK_RATE, Instant::now());
        todo!("event loop")
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
