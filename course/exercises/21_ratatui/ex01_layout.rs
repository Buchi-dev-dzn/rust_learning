//! # Layout の領域分割を考える
//!
//! ratatui の `Layout` を使う前に、矩形の座標計算を純粋な関数で練習しよう。
//! `split_vertical(area, header, footer)` は上からヘッダー、本文、フッターの順で返す。
//! 高さが足りない場合はヘッダーを優先し、残りからフッターを取る。幅と x 座標は維持する。
//! `header` と `footer` は希望する高さで、実際の高さは領域内に収める。
//!
//! - 関連: 第21章「Layout と Constraint」
//! - テストは編集しないこと。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect { x: u16, y: u16, width: u16, height: u16 }

// TODO: 領域からはみ出さない 3 つの矩形を返す。
fn split_vertical(area: Rect, header: u16, footer: u16) -> [Rect; 3] {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary() {
        assert_eq!(split_vertical(Rect { x: 2, y: 4, width: 30, height: 10 }, 2, 3), [
            Rect { x: 2, y: 4, width: 30, height: 2 },
            Rect { x: 2, y: 6, width: 30, height: 5 },
            Rect { x: 2, y: 11, width: 30, height: 3 },
        ]);
    }
    #[test]
    fn cramped_and_empty() {
        let area = Rect { x: 1, y: 7, width: 0, height: 2 };
        assert_eq!(split_vertical(area, 4, 3), [
            Rect { x: 1, y: 7, width: 0, height: 2 },
            Rect { x: 1, y: 9, width: 0, height: 0 },
            Rect { x: 1, y: 9, width: 0, height: 0 },
        ]);
    }
    #[test]
    fn high_coordinates() {
        let area = Rect { x: 65535, y: 65534, width: 1, height: 1 };
        assert_eq!(split_vertical(area, 0, 1)[2], Rect { x: 65535, y: 65534, width: 1, height: 1 });
    }
}
