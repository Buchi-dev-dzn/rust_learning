//! F5: エディタのコアの模範解答。
//!
//! テキストは「行の Vec<String>」で持ち、カーソルの列は**文字単位**で数える。
//! Undo はスナップショット方式 (グループ開始前の状態を丸ごと保存) で実装している。
//! 大きなファイルでは非効率だが、正しさを確かめやすく、規則に集中できる。

#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    lines: Vec<String>,
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastEdit {
    /// 文字挿入。after_space = 直前に挿入した文字が空白か
    Insert { after_space: bool },
    Backspace,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Space,
    Word,
    Punct,
}

fn class(c: char) -> Class {
    if c.is_whitespace() {
        Class::Space
    } else if c.is_alphanumeric() || c == '_' {
        Class::Word
    } else {
        Class::Punct
    }
}

/// 文字位置 → バイト位置
fn byte_idx(s: &str, col: usize) -> usize {
    s.char_indices().nth(col).map(|(i, _)| i).unwrap_or(s.len())
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

#[derive(Debug, Clone)]
pub struct Editor {
    lines: Vec<String>,
    row: usize,
    col: usize,
    goal_col: Option<usize>,
    undo_stack: Vec<State>,
    redo_stack: Vec<State>,
    last: Option<LastEdit>,
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
        Editor {
            lines: text.split('\n').map(String::from).collect(),
            row: 0,
            col: 0,
            goal_col: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last: None,
        }
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line(&self, row: usize) -> &str {
        &self.lines[row]
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    fn line_len(&self, row: usize) -> usize {
        char_len(&self.lines[row])
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.row = row.min(self.lines.len() - 1);
        self.col = col.min(self.line_len(self.row));
        self.goal_col = None;
        self.after_move();
    }

    // ---------------------------------------------------------------- undo helpers

    fn snapshot(&self) -> State {
        State {
            lines: self.lines.clone(),
            row: self.row,
            col: self.col,
        }
    }

    fn restore(&mut self, s: State) {
        self.lines = s.lines;
        self.row = s.row;
        self.col = s.col;
    }

    /// 編集の直前に呼ぶ。新しいグループを始めるなら現在の状態を保存する。
    fn begin_edit(&mut self, merge: bool) {
        if !merge {
            let s = self.snapshot();
            self.undo_stack.push(s);
        }
        self.redo_stack.clear();
        self.goal_col = None;
    }

    fn after_move(&mut self) {
        self.last = None;
    }

    // ---------------------------------------------------------------- edits

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
            return;
        }
        let merge = matches!(self.last, Some(LastEdit::Insert { after_space: false }));
        self.begin_edit(merge);
        let line = &mut self.lines[self.row];
        let i = byte_idx(line, self.col);
        line.insert(i, c);
        self.col += 1;
        self.last = Some(LastEdit::Insert {
            after_space: c.is_whitespace(),
        });
    }

    fn raw_newline(&mut self) {
        let line = &mut self.lines[self.row];
        let i = byte_idx(line, self.col);
        let rest = line.split_off(i);
        self.lines.insert(self.row + 1, rest);
        self.row += 1;
        self.col = 0;
    }

    pub fn insert_newline(&mut self) {
        self.begin_edit(false);
        self.raw_newline();
        self.last = None;
    }

    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.begin_edit(false);
        for c in s.chars() {
            if c == '\n' {
                self.raw_newline();
            } else {
                let line = &mut self.lines[self.row];
                let i = byte_idx(line, self.col);
                line.insert(i, c);
                self.col += 1;
            }
        }
        self.last = None;
    }

    pub fn backspace(&mut self) -> bool {
        if self.row == 0 && self.col == 0 {
            return false;
        }
        let merge = self.last == Some(LastEdit::Backspace);
        self.begin_edit(merge);
        if self.col > 0 {
            let line = &mut self.lines[self.row];
            let i = byte_idx(line, self.col - 1);
            line.remove(i);
            self.col -= 1;
        } else {
            let cur = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.line_len(self.row);
            self.lines[self.row].push_str(&cur);
        }
        self.last = Some(LastEdit::Backspace);
        true
    }

    pub fn delete(&mut self) -> bool {
        let len = self.line_len(self.row);
        if self.col == len && self.row + 1 == self.lines.len() {
            return false;
        }
        let merge = self.last == Some(LastEdit::Delete);
        self.begin_edit(merge);
        if self.col < len {
            let line = &mut self.lines[self.row];
            let i = byte_idx(line, self.col);
            line.remove(i);
        } else {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        }
        self.last = Some(LastEdit::Delete);
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some(prev) = self.undo_stack.pop() else {
            return false;
        };
        let cur = self.snapshot();
        self.redo_stack.push(cur);
        self.restore(prev);
        self.last = None;
        self.goal_col = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        let cur = self.snapshot();
        self.undo_stack.push(cur);
        self.restore(next);
        self.last = None;
        self.goal_col = None;
        true
    }

    // ---------------------------------------------------------------- movement

    fn horizontal(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
        self.goal_col = None;
        self.after_move();
    }

    pub fn move_left(&mut self) {
        if self.col > 0 {
            self.horizontal(self.row, self.col - 1);
        } else if self.row > 0 {
            let r = self.row - 1;
            self.horizontal(r, self.line_len(r));
        } else {
            self.horizontal(0, 0);
        }
    }

    pub fn move_right(&mut self) {
        if self.col < self.line_len(self.row) {
            self.horizontal(self.row, self.col + 1);
        } else if self.row + 1 < self.lines.len() {
            self.horizontal(self.row + 1, 0);
        } else {
            self.horizontal(self.row, self.col);
        }
    }

    fn vertical(&mut self, row: usize) {
        let goal = *self.goal_col.get_or_insert(self.col);
        self.row = row;
        self.col = goal.min(self.line_len(row));
        self.after_move();
    }

    pub fn move_up(&mut self) {
        if self.row > 0 {
            self.vertical(self.row - 1);
        } else {
            self.after_move();
        }
    }

    pub fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.vertical(self.row + 1);
        } else {
            self.after_move();
        }
    }

    pub fn move_home(&mut self) {
        self.horizontal(self.row, 0);
    }

    pub fn move_end(&mut self) {
        self.horizontal(self.row, self.line_len(self.row));
    }

    pub fn move_word_right(&mut self) {
        let chars: Vec<char> = self.lines[self.row].chars().collect();
        let len = chars.len();
        let mut col = self.col;
        if col == len {
            if self.row + 1 < self.lines.len() {
                self.horizontal(self.row + 1, 0);
            } else {
                self.horizontal(self.row, col);
            }
            return;
        }
        let cls = class(chars[col]);
        if cls != Class::Space {
            while col < len && class(chars[col]) == cls {
                col += 1;
            }
        }
        while col < len && class(chars[col]) == Class::Space {
            col += 1;
        }
        self.horizontal(self.row, col);
    }

    pub fn move_word_left(&mut self) {
        if self.col == 0 {
            if self.row > 0 {
                let r = self.row - 1;
                self.horizontal(r, self.line_len(r));
            } else {
                self.horizontal(0, 0);
            }
            return;
        }
        let chars: Vec<char> = self.lines[self.row].chars().collect();
        let mut col = self.col;
        while col > 0 && class(chars[col - 1]) == Class::Space {
            col -= 1;
        }
        if col > 0 {
            let cls = class(chars[col - 1]);
            while col > 0 && class(chars[col - 1]) == cls {
                col -= 1;
            }
        }
        self.horizontal(self.row, col);
    }

    // ---------------------------------------------------------------- search

    /// 行 `row` の中で、文字位置 `start` 以降に最初に現れる query の文字位置
    fn find_in_line(&self, row: usize, query: &str, start: usize) -> Option<usize> {
        let line = &self.lines[row];
        let b = byte_idx(line, start);
        line[b..].find(query).map(|i| start + char_len(&line[b..b + i]))
    }

    pub fn find(&self, query: &str, from: (usize, usize)) -> Option<(usize, usize)> {
        if query.is_empty() || query.contains('\n') {
            return None;
        }
        let n = self.lines.len();
        let (frow, fcol) = (from.0.min(n - 1), from.1);
        if let Some(c) = self.find_in_line(frow, query, fcol.min(self.line_len(frow))) {
            return Some((frow, c));
        }
        for k in 1..=n {
            let r = (frow + k) % n;
            if let Some(c) = self.find_in_line(r, query, 0) {
                // 一周して元の行に戻ったら、from より前の一致だけが対象
                if r == frow && c >= fcol {
                    return None;
                }
                return Some((r, c));
            }
        }
        None
    }

    pub fn find_all(&self, query: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        if query.is_empty() || query.contains('\n') {
            return out;
        }
        let qlen = char_len(query);
        for r in 0..self.lines.len() {
            let mut start = 0;
            while let Some(c) = self.find_in_line(r, query, start) {
                out.push((r, c));
                start = c + qlen;
            }
        }
        out
    }
}
