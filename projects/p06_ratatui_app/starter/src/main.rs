//! 使い方: cargo run -p p06_ratatui_app_starter [-- <ディレクトリ>]

#![allow(unused_imports, unused_variables, unused_mut, dead_code)]

use std::io;

use p06_ratatui_app_starter::{App, ui};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event};

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    // TODO(STEP 1): should_quit になるまで
    //   terminal.draw(|frame| ui::draw(frame, app))?;
    //   if let Event::Key(key) = event::read()? { app.handle_key(key); }
    todo!()
}

fn main() -> io::Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let mut app = App::new(dir)?;
    // TODO(STEP 1): let mut terminal = ratatui::init();
    //               let result = run(&mut terminal, &mut app);
    //               ratatui::restore();
    //               result
    todo!()
}
