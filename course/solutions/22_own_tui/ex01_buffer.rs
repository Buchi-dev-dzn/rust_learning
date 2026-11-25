//! # セルバッファへ安全に描く
//!
//! 端末の各マスを `Cell` で表す。`Buffer::new` は空白で埋める。
//! `put(x, y, ch)` は範囲内だけを書き換え、範囲外なら何もしない。
//! `draw_text(x, y, text)` は右端で切り、次の行に折り返さない。
//! この演習では ASCII の 1 文字 = 1 セルを想定する。全角文字は第16章の表示幅処理が別途必要。
//!
//! - 関連: 第22章「セルバッファ」
//! - テストは編集しないこと。

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cell { ch: char }
#[derive(Debug, Clone, PartialEq, Eq)]
struct Buffer { width: usize, height: usize, cells: Vec<Cell> }

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        let count = width.checked_mul(height).expect("buffer dimensions overflow");
        Self { width, height, cells: vec![Cell { ch: ' ' }; count] }
    }
    fn put(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x].ch = ch;
        }
    }
    fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        if x >= self.width || y >= self.height { return; }
        for (offset, ch) in text.chars().take(self.width - x).enumerate() {
            self.put(x + offset, y, ch);
        }
    }
    fn get(&self, x: usize, y: usize) -> Option<char> {
        if x < self.width && y < self.height { Some(self.cells[y * self.width + x].ch) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initializes_and_clips() {
        let mut b = Buffer::new(4, 2);
        assert_eq!(b.cells.len(), 8);
        assert_eq!(b.get(3, 1), Some(' '));
        b.put(3, 1, '#');
        b.put(4, 0, '!');
        b.put(0, 2, '!');
        b.put(usize::MAX, usize::MAX, '!');
        assert_eq!(b.get(3, 1), Some('#'));
        assert_eq!(b.cells.iter().filter(|c| c.ch == '!').count(), 0);
    }
    #[test]
    fn text_no_wrap() {
        let mut b = Buffer::new(4, 2);
        b.draw_text(2, 0, "ABCDE");
        b.draw_text(0, 1, "xy");
        assert_eq!(b.cells.iter().map(|c| c.ch).collect::<String>(), "  ABxy  ");
        b.draw_text(usize::MAX, 0, "ignored");
        assert_eq!(b.get(0, 1), Some('x'));
    }
    #[test]
    fn zero_sized() {
        let mut b = Buffer::new(0, 0);
        b.draw_text(0, 0, "a");
        assert!(b.cells.is_empty());
    }
}
