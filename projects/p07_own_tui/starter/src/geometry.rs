//! 画面上の長方形領域。

/// 左上 (x, y) と幅・高さで表す長方形 (単位はセル)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Rect { x, y, width, height }
    }

    /// 右端の「次」の列 (この列は含まない)
    pub const fn right(&self) -> u16 {
        self.x + self.width
    }

    /// 下端の「次」の行 (この行は含まない)
    pub const fn bottom(&self) -> u16 {
        self.y + self.height
    }

    pub const fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// 四方を `margin` セルずつ縮めた領域
    pub fn inner(&self, margin: u16) -> Rect {
        // TODO(STEP 2): x, y を margin 増やし、width, height を margin * 2 減らす。
        //               足りなければ幅 0・高さ 0 の Rect を返す
        todo!()
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// 2 つの長方形の重なり (重ならなければ幅か高さが 0)
    pub fn intersection(&self, other: Rect) -> Rect {
        // TODO(STEP 2): 左上は max、右下は min。saturating_sub で負にならないように
        todo!()
    }

    /// 中央に `width` x `height` の領域を取る (ポップアップ用)
    pub fn centered(&self, width: u16, height: u16) -> Rect {
        let w = width.min(self.width);
        let h = height.min(self.height);
        Rect::new(self.x + (self.width - w) / 2, self.y + (self.height - h) / 2, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_and_edges() {
        let r = Rect::new(2, 3, 10, 5);
        assert_eq!(r.right(), 12);
        assert_eq!(r.bottom(), 8);
        assert_eq!(r.inner(1), Rect::new(3, 4, 8, 3));
        assert!(r.inner(3).is_empty());
    }

    #[test]
    fn intersection_and_centered() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.intersection(b), Rect::new(5, 5, 5, 5));
        assert!(a.intersection(Rect::new(20, 20, 1, 1)).is_empty());
        assert_eq!(a.centered(4, 2), Rect::new(3, 4, 4, 2));
    }
}
