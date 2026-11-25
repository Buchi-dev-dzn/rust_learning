//! 差分レンダラのデモ。文字が動くアニメーションを描き、1 フレームあたりの出力バイト数を表示する。
//! q / Esc / Ctrl-C で終了。

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, terminal};
use f4_diff_renderer::{Buffer, Color, Renderer, Style};
use std::io::Write;
use std::time::Duration;

fn restore() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(std::io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
}

fn draw(buf: &mut Buffer, frame: usize, last_bytes: usize) {
    buf.clear();
    let (w, h) = (buf.width(), buf.height());
    let title = Style::new().fg(Color::Black).bg(Color::Cyan).bold();
    for x in 0..w {
        buf.set(x, 0, ' ', title);
    }
    buf.set_str(1, 0, "F4 差分レンダラ (q で終了)", title);
    if h > 2 && w > 0 {
        let text = "★ 差分だけ描く ★";
        let span = w.saturating_sub(18).max(1);
        let x = frame % span;
        let y = 2 + (frame / span) % (h - 2).max(1);
        buf.set_str(x, y.min(h - 1), text, Style::new().fg(Color::Yellow).bold());
    }
    let status = format!(" frame {frame:>6} | last frame: {last_bytes:>5} bytes ");
    if h > 0 {
        buf.set_str(0, h - 1, &status, Style::new().fg(Color::Indexed(245)));
    }
}

fn main() -> std::io::Result<()> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        default_hook(info);
    }));

    terminal::enable_raw_mode()?;
    let mut out = std::io::stdout();
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

    let (w, h) = terminal::size()?;
    let mut buf = Buffer::new(w as usize, h as usize);
    let mut renderer = Renderer::new();
    let mut frame = 0usize;
    let mut last_bytes = 0usize;
    loop {
        draw(&mut buf, frame, last_bytes);
        let s = renderer.render(&buf);
        last_bytes = s.len();
        out.write_all(s.as_bytes())?;
        out.flush()?;

        if event::poll(Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(k) => {
                    let ctrl_c = k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL);
                    if k.code == KeyCode::Char('q') || k.code == KeyCode::Esc || ctrl_c {
                        break;
                    }
                }
                Event::Resize(w, h) => buf = Buffer::new(w as usize, h as usize),
                _ => {}
            }
        }
        frame += 1;
    }
    restore();
    Ok(())
}
