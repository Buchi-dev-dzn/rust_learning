//! Editor の動きを目で確かめる小さなデモ (端末 UI は無し)。

use f5_editor_core::Editor;

/// カーソル位置に '|' を入れてテキストを表示する
fn show(label: &str, ed: &Editor) {
    let (row, col) = ed.cursor();
    println!("--- {label} (cursor {row},{col})");
    for r in 0..ed.line_count() {
        let line = ed.line(r);
        if r == row {
            let mut s: String = line.chars().take(col).collect();
            s.push('|');
            s.extend(line.chars().skip(col));
            println!("{s}");
        } else {
            println!("{line}");
        }
    }
}

fn main() {
    let mut ed = Editor::new();
    for c in "hello world".chars() {
        ed.insert_char(c);
    }
    show("type 'hello world'", &ed);
    ed.undo();
    show("undo (removes 'world')", &ed);
    ed.redo();
    ed.insert_newline();
    ed.insert_str("日本語のテキスト");
    show("newline + insert_str", &ed);
    ed.move_up();
    ed.move_word_left();
    show("move_up + move_word_left", &ed);
    ed.set_cursor(1, 0);
    ed.backspace();
    show("backspace at line start (join)", &ed);
    println!("find 'テキスト' from (0,0): {:?}", ed.find("テキスト", (0, 0)));
    println!("find_all 'o': {:?}", ed.find_all("o"));
}
