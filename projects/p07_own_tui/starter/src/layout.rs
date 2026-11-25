//! 領域を縦または横に分割するレイアウト。

use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 上から下へ並べる (高さを分ける)
    Vertical,
    /// 左から右へ並べる (幅を分ける)
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    /// ちょうど n セル
    Length(u16),
    /// 全体の n %
    Percentage(u16),
    /// 残りを重み n で分け合う
    Fill(u16),
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub direction: Direction,
    pub constraints: Vec<Constraint>,
}

impl Layout {
    pub fn vertical(constraints: impl Into<Vec<Constraint>>) -> Self {
        Layout { direction: Direction::Vertical, constraints: constraints.into() }
    }

    pub fn horizontal(constraints: impl Into<Vec<Constraint>>) -> Self {
        Layout { direction: Direction::Horizontal, constraints: constraints.into() }
    }

    /// 1. Length と Percentage を前から順に確保する (足りなければ削る)
    /// 2. 残りを Fill の重みで分ける (端数は最後の Fill へ)
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        // TODO(STEP 6):
        //   1. total = 分割する方向の長さ (Vertical なら height)
        //   2. Length / Percentage の大きさを前から決める (残りより大きければ残りまで)
        //   3. 残りを Fill の重みで分ける。端数は最後の Fill に足す
        //   4. 大きさを順に並べて Rect にする
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Constraint::*;

    #[test]
    fn header_body_footer() {
        let rects = Layout::vertical([Length(1), Fill(1), Length(1)]).split(Rect::new(0, 0, 80, 24));
        assert_eq!(
            rects,
            [Rect::new(0, 0, 80, 1), Rect::new(0, 1, 80, 22), Rect::new(0, 23, 80, 1)]
        );
    }

    #[test]
    fn percentage_and_fill_horizontal() {
        let rects = Layout::horizontal([Percentage(30), Fill(1)]).split(Rect::new(10, 5, 100, 3));
        assert_eq!(rects, [Rect::new(10, 5, 30, 3), Rect::new(40, 5, 70, 3)]);
    }

    #[test]
    fn fill_weights_and_remainder() {
        let rects = Layout::horizontal([Fill(1), Fill(2), Fill(1)]).split(Rect::new(0, 0, 10, 1));
        let widths: Vec<_> = rects.iter().map(|r| r.width).collect();
        assert_eq!(widths, [2, 5, 3]); // 10*1/4=2, 10*2/4=5, 残り 3
        assert_eq!(widths.iter().sum::<u16>(), 10);
    }

    #[test]
    fn too_small_area_is_clamped() {
        let rects = Layout::vertical([Length(3), Length(3), Fill(1)]).split(Rect::new(0, 0, 5, 4));
        let heights: Vec<_> = rects.iter().map(|r| r.height).collect();
        assert_eq!(heights, [3, 1, 0]);
    }
}
