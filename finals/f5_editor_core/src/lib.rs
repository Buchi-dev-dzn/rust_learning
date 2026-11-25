//! F5: エディタのコア。
//!
//! 仕様は SPEC.md。`todo!()` を埋めて `cargo test -p f5_editor_core` を通そう。
//! 公開メソッドのシグネチャはテストが依存しているので変えないこと。
//! フィールドは自由に追加・変更してよい (Undo 用の履歴、目標列など)。

#[derive(Debug, Clone)]
pub struct Editor {
    lines: Vec<String>,
    row: usize,
    col: usize,
    // TODO: 目標列、undo/redo の履歴、直前の操作の種類 などを追加する
}

impl Default for Editor {
    fn default() -> Self {
        Editor::new()
    }
}

impl Editor {
    pub fn new() -> Editor {
        Editor::from_text("")
    }

    pub fn from_text(text: &str) -> Editor {
        // TODO: '\n' で分割して lines にする
        todo!()
    }

    pub fn text(&self) -> String {
        todo!()
    }

    pub fn line(&self, row: usize) -> &str {
        todo!()
    }

    pub fn line_count(&self) -> usize {
        todo!()
    }

    pub fn cursor(&self) -> (usize, usize) {
        todo!()
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        todo!()
    }

    // ---- 編集 (SPEC 3章)。col は文字単位なので、String の操作にはバイト位置への変換が要る

    pub fn insert_char(&mut self, c: char) {
        todo!()
    }

    pub fn insert_newline(&mut self) {
        todo!()
    }

    pub fn insert_str(&mut self, s: &str) {
        todo!()
    }

    pub fn backspace(&mut self) -> bool {
        todo!()
    }

    pub fn delete(&mut self) -> bool {
        todo!()
    }

    // ---- 移動 (SPEC 4章)

    pub fn move_left(&mut self) {
        todo!()
    }

    pub fn move_right(&mut self) {
        todo!()
    }

    pub fn move_up(&mut self) {
        todo!()
    }

    pub fn move_down(&mut self) {
        todo!()
    }

    pub fn move_home(&mut self) {
        todo!()
    }

    pub fn move_end(&mut self) {
        todo!()
    }

    pub fn move_word_left(&mut self) {
        todo!()
    }

    pub fn move_word_right(&mut self) {
        todo!()
    }

    // ---- Undo / Redo (SPEC 5章)

    pub fn undo(&mut self) -> bool {
        todo!()
    }

    pub fn redo(&mut self) -> bool {
        todo!()
    }

    // ---- 検索 (SPEC 6章)

    pub fn find(&self, query: &str, from: (usize, usize)) -> Option<(usize, usize)> {
        todo!()
    }

    pub fn find_all(&self, query: &str) -> Vec<(usize, usize)> {
        todo!()
    }
}
