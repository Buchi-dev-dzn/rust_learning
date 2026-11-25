//! 端末まわり: raw モード、キー入力の変換、画面への描画 (starter)。
//! 使い方: cargo run -p p05_editor_starter -- <ファイル名>

#![allow(unused_imports, dead_code, unused_variables)]

use std::io::{self, Stdout, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, ClearType},
};
use p05_editor_starter::{Editor, Key, Screen};

/// 生成時に raw モード + 代替スクリーンに入り、Drop で必ず元に戻す
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        // TODO(STEP 1): terminal::enable_raw_mode() と EnterAlternateScreen
        todo!()
    }
}

fn restore_terminal() {
    // TODO(STEP 1): cursor::Show, LeaveAlternateScreen, disable_raw_mode (エラーは無視してよい)
    todo!()
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// crossterm のキーイベントを、エディタ用の Key に変換する
fn to_key(ev: KeyEvent) -> Option<Key> {
    // TODO(STEP 1): KeyEventKind::Press 以外は None。
    //   Char + CONTROL → Key::Ctrl、Char → Key::Char、Tab → Key::Char('\t')、矢印など
    todo!()
}

fn draw(out: &mut Stdout, screen: &Screen) -> io::Result<()> {
    // TODO(STEP 1): カーソルを隠す → 各行を MoveTo(0, y) + Print + Clear(UntilNewLine)
    // TODO(STEP 3): ステータスバー (反転表示) とメッセージバー
    //               最後にカーソルを screen.cursor に移動して表示、flush
    todo!()
}

fn run(editor: &mut Editor) -> io::Result<()> {
    // TODO(STEP 1): should_quit になるまで render → draw → event::read → process_key
    //               Event::Resize なら editor.resize
    todo!()
}

fn main() -> io::Result<()> {
    // TODO(STEP 1): パニックフックで restore_terminal を呼ぶ
    let (w, h) = terminal::size()?;
    let mut editor = Editor::new(w as usize, h as usize);
    // TODO(STEP 3): 引数があれば editor.open(path)?
    let _guard = TerminalGuard::new()?;
    run(&mut editor)
}
