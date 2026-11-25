//! # Rect: 画面上の長方形
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: `impl` ブロックに関連関数とメソッドを書く。`&self` で読み、`Self` を返す
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第4章「メソッドと impl」「実例: Rect」
//!
//! 座標系はターミナルと同じく、左上が (0, 0)、右が x+、下が y+。
//! `Rect { x, y, width, height }` は x..x+width, y..y+height のセルを覆う。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    /// 関連関数 (コンストラクタ)
    fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        // TODO
        todo!()
    }

    /// 覆うセルの数
    fn area(&self) -> u32 {
        // TODO: u16 * u16 は溢れることがあるので u32 に変換してから掛ける
        todo!()
    }

    /// (px, py) のセルがこの Rect の中にあるか
    fn contains(&self, px: u16, py: u16) -> bool {
        // TODO
        todo!()
    }

    /// 左から `w` 列を切り出し、(左, 残り) を返す。`w` が幅より大きければ全体が左になる。
    fn split_left(&self, w: u16) -> (Rect, Rect) {
        // TODO
        todo!()
    }

    /// 2 つの Rect が 1 セル以上重なっているか。
    /// 幅か高さが 0 の Rect (空の Rect) は何とも重ならない。
    fn intersects(&self, other: &Rect) -> bool {
        // TODO: x 方向と y 方向の区間がどちらも重なっていれば true
        //       区間 [a1, a2) と [b1, b2) が重なる条件は a1 < b2 && b1 < a2
        //       (空の Rect は先に除外しておく)
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs() {
        let r = Rect::new(1, 2, 3, 4);
        assert_eq!(r, Rect { x: 1, y: 2, width: 3, height: 4 });
    }

    #[test]
    fn computes_area() {
        assert_eq!(Rect::new(0, 0, 80, 24).area(), 1920);
        assert_eq!(Rect::new(0, 0, 1000, 1000).area(), 1_000_000);
        assert_eq!(Rect::new(5, 5, 0, 10).area(), 0);
    }

    #[test]
    fn contains_points() {
        let r = Rect::new(2, 3, 4, 2); // x: 2..6, y: 3..5
        assert!(r.contains(2, 3));
        assert!(r.contains(5, 4));
        assert!(!r.contains(6, 4));
        assert!(!r.contains(5, 5));
        assert!(!r.contains(1, 3));
        assert!(!Rect::new(0, 0, 0, 0).contains(0, 0));
    }

    #[test]
    fn splits_left() {
        let r = Rect::new(10, 1, 30, 5);
        let (l, rest) = r.split_left(12);
        assert_eq!(l, Rect::new(10, 1, 12, 5));
        assert_eq!(rest, Rect::new(22, 1, 18, 5));
        let (l, rest) = r.split_left(100);
        assert_eq!(l, r);
        assert_eq!(rest.width, 0);
        assert_eq!(rest.x, 40);
    }

    #[test]
    fn intersections() {
        let a = Rect::new(0, 0, 10, 10);
        assert!(a.intersects(&Rect::new(5, 5, 10, 10)));
        assert!(a.intersects(&Rect::new(2, 2, 1, 1)));
        assert!(!a.intersects(&Rect::new(10, 0, 5, 5))); // 右隣でぴったり接するだけ
        assert!(!a.intersects(&Rect::new(0, 10, 5, 5))); // 下で接するだけ
        assert!(!a.intersects(&Rect::new(3, 3, 0, 4))); // 幅 0
    }
}
