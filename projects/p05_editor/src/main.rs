//! 端末まわり: raw モード、キー入力の変換、画面への描画。
//! 使い方: cargo run -p p05_editor -- <ファイル名>

use std::io::{self, Stdout, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, ClearType},
};
use p05_editor::{Editor, Key, Screen};

/// 生成時に raw モード + 代替スクリーンに入り、Drop で必ず元に戻す
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        if let Err(err) = execute!(io::stdout(), terminal::EnterAlternateScreen) {
            let _ = terminal::disable_raw_mode();
            return Err(err);
        }
        Ok(TerminalGuard)
    }
}

fn restore_terminal() {
    let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// crossterm のキーイベントを、エディタ用の Key に変換する
fn to_key(ev: KeyEvent) -> Option<Key> {
    if ev.kind != KeyEventKind::Press {
        return None;
    }
    let key = match ev.code {
        KeyCode::Char(c) if ev.modifiers.contains(KeyModifiers::CONTROL) => {
            Key::Ctrl(c.to_ascii_lowercase())
        }
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Tab => Key::Char('\t'),
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Esc => Key::Esc,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        _ => return None,
    };
    Some(key)
}

fn draw(out: &mut Stdout, screen: &Screen) -> io::Result<()> {
    queue!(out, cursor::Hide)?;
    for (y, line) in screen.rows.iter().enumerate() {
        queue!(
            out,
            cursor::MoveTo(0, y as u16),
            Print(line),
            terminal::Clear(ClearType::UntilNewLine)
        )?;
    }
    let status_y = screen.rows.len() as u16;
    queue!(
        out,
        cursor::MoveTo(0, status_y),
        SetAttribute(Attribute::Reverse),
        Print(&screen.status),
        SetAttribute(Attribute::Reset),
        cursor::MoveTo(0, status_y + 1),
        Print(&screen.message),
        terminal::Clear(ClearType::UntilNewLine),
        cursor::MoveTo(screen.cursor.0 as u16, screen.cursor.1 as u16),
        cursor::Show
    )?;
    out.flush()
}

fn run(editor: &mut Editor) -> io::Result<()> {
    let mut out = io::stdout();
    while !editor.should_quit {
        let screen = editor.render();
        draw(&mut out, &screen)?;
        match event::read()? {
            Event::Key(ev) => {
                if let Some(key) = to_key(ev) {
                    editor.process_key(key);
                }
            }
            Event::Resize(w, h) => editor.resize(w as usize, h as usize),
            _ => {}
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // パニックしても端末を元に戻してからメッセージを出す
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let (w, h) = terminal::size()?;
    let mut editor = Editor::new(w as usize, h as usize);
    if let Some(path) = std::env::args().nth(1) {
        editor.open(path)?;
    }
    if editor.message().is_empty() {
        editor.set_message("ヘルプ: Ctrl-S = 保存 | Ctrl-Q = 終了 | Ctrl-F = 検索");
    }

    let _guard = TerminalGuard::new()?;
    run(&mut editor)
}
