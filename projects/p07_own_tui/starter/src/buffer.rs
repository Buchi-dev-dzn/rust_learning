//! セルバッファ: 画面 1 枚ぶんの「どの位置にどの文字をどの色で描くか」。
//!
//! ウィジェットは端末に直接書かず、このバッファに書く。
//! 前フレームのバッファと比べて (diff)、変わったセルだけを端末に送るのが「ダブルバッファリング」。

use crate::geometry::Rect;
use crate::style::Style;
use crate::width::char_width;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
    /// 全角文字の右半分。このセル自体は描画しない (左隣の文字が 2 セル使う)
    pub skip: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', style: Style::new(), skip: false }
    }
}

impl Cell {
    pub fn width(&self) -> usize {
        if self.skip { 0 } else { char_width(self.ch).max(1) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub area: Rect,
    pub cells: Vec<Cell>,
}

impl Buffer {
    /// 空白で埋めたバッファ
    pub fn empty(area: Rect) -> Self {
        Buffer { area, cells: vec![Cell::default(); area.area()] }
    }

    fn index_of(&self, x: u16, y: u16) -> usize {
        debug_assert!(self.area.contains(x, y), "({x}, {y}) は {:?} の外", self.area);
        (y - self.area.y) as usize * self.area.width as usize + (x - self.area.x) as usize
    }

    pub fn get(&self, x: u16, y: u16) -> &Cell {
        &self.cells[self.index_of(x, y)]
    }

    /// 1 文字を置く。全角文字が半分だけ上書きされて壊れないように、相方のセルも直す。
    fn put(&mut self, x: u16, y: u16, ch: char, style: Style) {
        let w = char_width(ch);
        let i = self.index_of(x, y);
        // 全角文字の右半分を上書きするなら、左半分 (本体) を空白にする
        if self.cells[i].skip && x > self.area.x {
            self.cells[i - 1].ch = ' ';
        }
        // 全角文字の本体を上書きするなら、右半分を普通の空白に戻す
        if char_width(self.cells[i].ch) == 2 && !self.cells[i].skip && x + 1 < self.area.right() {
            self.cells[i + 1] = Cell { ch: ' ', style: self.cells[i + 1].style, skip: false };
        }
        self.cells[i] = Cell { ch, style, skip: false };
        if w == 2 {
            let j = i + 1;
            // 右半分にする位置に別の全角文字の本体があれば、その右半分も空白に
            if char_width(self.cells[j].ch) == 2 && !self.cells[j].skip && x + 2 < self.area.right() {
                self.cells[j + 1] = Cell { ch: ' ', style: self.cells[j + 1].style, skip: false };
            }
            self.cells[j] = Cell { ch: ' ', style, skip: true };
        }
    }

    /// (x, y) から文字列を書く。右端または `max_width` で打ち切り、書き終えた次の x を返す。
    pub fn set_stringn(&mut self, x: u16, y: u16, s: &str, max_width: usize, style: Style) -> u16 {
        // TODO(STEP 3): char_width で幅を数え、右端と max_width を超えない文字だけ put する。
        // 全角文字を半分に割らず、次に書く列を返す。
        let _ = (x, y, s, max_width, style);
        todo!()
    }

    pub fn set_string(&mut self, x: u16, y: u16, s: &str, style: Style) -> u16 {
        self.set_stringn(x, y, s, usize::MAX / 2, style)
    }

    /// 領域内のセルのスタイルを重ね塗りする (背景色を付けるなど)
    pub fn set_style(&mut self, area: Rect, style: Style) {
        let area = self.area.intersection(area);
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let i = self.index_of(x, y);
                self.cells[i].style = self.cells[i].style.patch(style);
            }
        }
    }

    /// 前のフレーム `prev` と比べて、変わったセルを (x, y, セル) の列で返す。
    /// 全角文字の右半分 (skip) は出力しない (左のセルを描けば自動で埋まる)。
    pub fn diff<'a>(&'a self, prev: &Buffer) -> Vec<(u16, u16, &'a Cell)> {
        // TODO(STEP 3): prev と異なるセルだけを返す。全角の右半分 (skip) は出力しない。
        let _ = prev;
        todo!()
    }

    /// テスト用: 1 行を文字列にする (全角の右半分は飛ばす)
    pub fn line(&self, y: u16) -> String {
        (self.area.x..self.area.right())
            .map(|x| self.get(x, y))
            .filter(|c| !c.skip)
            .map(|c| c.ch)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    #[test]
    fn set_string_ascii_and_wide() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 1));
        let next = buf.set_string(0, 0, "aあb", Style::new());
        assert_eq!(next, 4);
        assert_eq!(buf.line(0), "aあb  ");
        assert!(buf.get(2, 0).skip);
    }

    #[test]
    fn wide_char_is_not_split_at_right_edge() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));
        buf.set_string(0, 0, "aあい", Style::new());
        assert_eq!(buf.line(0), "aあ");
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 1));
        buf.set_string(0, 0, "aあい", Style::new());
        assert_eq!(buf.line(0), "aあ "); // 「い」は入りきらないので書かない
    }

    #[test]
    fn overwriting_half_of_wide_char_clears_the_other_half() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 1));
        buf.set_string(0, 0, "あい", Style::new());
        buf.set_string(1, 0, "x", Style::new()); // 「あ」の右半分を上書き
        assert_eq!(buf.line(0), " xい");
        buf.set_string(2, 0, "y", Style::new()); // 「い」の左半分を上書き
        assert_eq!(buf.line(0), " xy ");
    }

    #[test]
    fn set_stringn_limits_width() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        buf.set_stringn(1, 0, "abcdef", 3, Style::new());
        assert_eq!(buf.line(0), " abc      ");
    }

    #[test]
    fn set_style_patches_area() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 2));
        buf.set_string(0, 0, "abc", Style::new().fg(Color::Red));
        buf.set_style(Rect::new(1, 0, 5, 1), Style::new().bg(Color::Blue));
        assert_eq!(buf.get(0, 0).style.bg, None);
        assert_eq!(buf.get(1, 0).style, Style::new().fg(Color::Red).bg(Color::Blue));
        assert_eq!(buf.get(1, 1).style.bg, None);
    }

    #[test]
    fn diff_reports_only_changed_cells() {
        let area = Rect::new(0, 0, 5, 2);
        let prev = Buffer::empty(area);
        let mut next = Buffer::empty(area);
        next.set_string(1, 1, "hi", Style::new());
        let d = next.diff(&prev);
        let pos: Vec<_> = d.iter().map(|(x, y, c)| (*x, *y, c.ch)).collect();
        assert_eq!(pos, [(1, 1, 'h'), (2, 1, 'i')]);
        assert!(next.diff(&next.clone()).is_empty());
    }

    #[test]
    fn diff_skips_right_half_of_wide_chars() {
        let area = Rect::new(0, 0, 4, 1);
        let prev = Buffer::empty(area);
        let mut next = Buffer::empty(area);
        next.set_string(0, 0, "あ", Style::new());
        let pos: Vec<_> = next.diff(&prev).iter().map(|(x, _, c)| (*x, c.ch)).collect();
        assert_eq!(pos, [(0, 'あ')]);
    }

    #[test]
    fn diff_after_wide_char_replaced_by_narrow() {
        let area = Rect::new(0, 0, 4, 1);
        let mut prev = Buffer::empty(area);
        prev.set_string(0, 0, "あ", Style::new());
        let mut next = Buffer::empty(area);
        next.set_string(0, 0, "ab", Style::new());
        let pos: Vec<_> = next.diff(&prev).iter().map(|(x, _, c)| (*x, c.ch)).collect();
        assert_eq!(pos, [(0, 'a'), (1, 'b')]);
    }
}
