//! # テストを手がかりにバグを直す
//!
//! リスト表示のスクロール処理 `Viewport` には **バグが 3 つ** ある。
//! まずテストを実行して、失敗メッセージ (left / right) を読もう。
//! 実装 (`// BUG?` の付近) を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! 仕様:
//! - `len` 個の項目のうち、`offset` から `height` 行が画面に見えている
//! - `cursor` は常に `0..len` の範囲 (`len == 0` のときは 0)
//! - カーソルが画面外に出たら、カーソルが見える最小限だけスクロールする
//! - `visible()` は見えている項目の範囲 (末尾で `len` を超えない)
//! - 関連: 第13章「良いテストのコツ」(境界値)

use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
    pub len: usize,
    pub height: usize,
    pub offset: usize,
    pub cursor: usize,
}

impl Viewport {
    pub fn new(len: usize, height: usize) -> Self {
        Viewport { len, height, offset: 0, cursor: 0 }
    }

    /// カーソルが画面内に入るように offset を調整する
    fn follow_cursor(&mut self) {
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + self.height {
            // BUG?
            self.offset = self.cursor - self.height;
        }
    }

    pub fn move_by(&mut self, delta: isize) {
        if self.len == 0 {
            return;
        }
        let target = self.cursor as isize + delta;
        self.cursor = target.clamp(0, self.len as isize - 1) as usize;
        self.follow_cursor();
    }

    pub fn page_down(&mut self) {
        if self.len == 0 {
            return;
        }
        // BUG?
        self.cursor = (self.cursor + self.height).min(self.len);
        self.follow_cursor();
    }

    pub fn visible(&self) -> Range<usize> {
        // BUG?
        self.offset..self.offset + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_top() {
        let v = Viewport::new(100, 10);
        assert_eq!(v.visible(), 0..10);
        assert_eq!(v.cursor, 0);
    }

    #[test]
    fn moving_within_screen_does_not_scroll() {
        let mut v = Viewport::new(100, 10);
        v.move_by(9);
        assert_eq!((v.cursor, v.offset), (9, 0));
    }

    #[test]
    fn moving_past_bottom_scrolls_minimally() {
        let mut v = Viewport::new(100, 10);
        v.move_by(10); // 11 行目へ → 1 行だけスクロール
        assert_eq!((v.cursor, v.offset), (10, 1), "cursor, offset");
        assert_eq!(v.visible(), 1..11);
    }

    #[test]
    fn moving_up_scrolls_back() {
        let mut v = Viewport::new(100, 10);
        v.move_by(50);
        v.move_by(-45);
        assert_eq!((v.cursor, v.offset), (5, 5));
    }

    #[test]
    fn cursor_is_clamped() {
        let mut v = Viewport::new(3, 10);
        v.move_by(-1);
        assert_eq!(v.cursor, 0);
        v.move_by(100);
        assert_eq!(v.cursor, 2);
    }

    #[test]
    fn page_down_stops_at_last_item() {
        let mut v = Viewport::new(25, 10);
        v.page_down();
        v.page_down();
        v.page_down();
        assert_eq!(v.cursor, 24);
        assert_eq!(v.offset, 15);
        assert_eq!(v.visible(), 15..25);
    }

    #[test]
    fn short_list_visible_range() {
        let v = Viewport::new(3, 10);
        assert_eq!(v.visible(), 0..3);
    }

    #[test]
    fn empty_list() {
        let mut v = Viewport::new(0, 10);
        v.move_by(1);
        v.page_down();
        assert_eq!(v, Viewport::new(0, 10));
        assert_eq!(v.visible(), 0..0);
    }

    #[test]
    fn height_one() {
        let mut v = Viewport::new(5, 1);
        v.move_by(3);
        assert_eq!(v.visible(), 3..4);
    }
}
