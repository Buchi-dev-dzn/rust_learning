//! # モジュールで TUI の部品を分ける
//!
//! 小さな TUI の「レイアウト」と「描画」を、モジュールに分けて **自分で** 組み立てよう。
//! 1 ファイルの中で `mod name { ... }` を使う (実際のプロジェクトでは各モジュールが別ファイルになる)。
//!
//! - ゴール: モジュールの木を設計し、`pub` / `use` / `super` / `crate` を正しく使う
//! - テスト (`mod tests`) は編集しないこと。テストが使っているパスと名前に合わせて作る。
//! - 関連: 第8章「mod と pub」「パスと use」「pub use で再公開」
//!
//! 作るもの:
//!
//! ```text
//! crate
//! ├── geometry
//! │   └── Rect          (pub struct。フィールド x, y, width, height: u16 はすべて pub)
//! │                      derive(Debug, Clone, Copy, PartialEq)
//! │       Rect::new(x, y, width, height) -> Rect
//! │       Rect::split_top(&self, h: u16) -> (Rect, Rect)   上から h 行と残り (h は高さで頭打ち)
//! ├── render
//! │   └── draw_box(r: &Rect) -> Vec<String>
//! │          幅 r.width、高さ r.height の枠を文字列の行で返す。
//! │          角は '+'、上下は '-'、左右は '|'、中は ' '。
//! │          幅か高さが 2 未満なら空の Vec。(位置 x, y は無視してよい)
//! └── app
//!     └── layout(width: u16, height: u16) -> (Rect, Rect)
//!            画面全体を (ヘッダ 1 行, 残り本体) に分ける。geometry の Rect を使うこと。
//! ```
//!
//! さらに、クレートのルートで `pub use geometry::Rect;` して、`crate::Rect` とも書けるようにする。

mod geometry {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Rect {
        pub x: u16,
        pub y: u16,
        pub width: u16,
        pub height: u16,
    }

    impl Rect {
        pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
            Self { x, y, width, height }
        }

        pub fn split_top(&self, h: u16) -> (Rect, Rect) {
            let h = h.min(self.height);
            (
                Rect::new(self.x, self.y, self.width, h),
                Rect::new(self.x, self.y + h, self.width, self.height - h),
            )
        }
    }
}

mod render {
    // 兄弟モジュールへは super (親) 経由でたどる。crate::geometry::Rect でも同じ。
    use super::geometry::Rect;

    pub fn draw_box(r: &Rect) -> Vec<String> {
        let (w, h) = (r.width as usize, r.height as usize);
        if w < 2 || h < 2 {
            return Vec::new();
        }
        let edge = format!("+{}+", "-".repeat(w - 2));
        let middle = format!("|{}|", " ".repeat(w - 2));
        let mut lines = vec![edge.clone()];
        for _ in 0..h - 2 {
            lines.push(middle.clone());
        }
        lines.push(edge);
        lines
    }
}

mod app {
    use crate::geometry::Rect;

    pub fn layout(width: u16, height: u16) -> (Rect, Rect) {
        Rect::new(0, 0, width, height).split_top(1)
    }
}

// 利用者は crate::Rect と短く書ける。
pub use geometry::Rect;

#[cfg(test)]
mod tests {
    use crate::app;
    use crate::geometry::Rect;
    use crate::render::draw_box;

    #[test]
    fn rect_split() {
        let r = Rect::new(0, 0, 10, 5);
        let (top, rest) = r.split_top(2);
        assert_eq!(top, Rect { x: 0, y: 0, width: 10, height: 2 });
        assert_eq!(rest, Rect { x: 0, y: 2, width: 10, height: 3 });
        let (top, rest) = r.split_top(9);
        assert_eq!(top, r);
        assert_eq!(rest.height, 0);
    }

    #[test]
    fn draws_boxes() {
        assert_eq!(draw_box(&Rect::new(3, 3, 4, 3)), vec!["+--+", "|  |", "+--+"]);
        assert_eq!(draw_box(&Rect::new(0, 0, 2, 2)), vec!["++", "++"]);
        assert!(draw_box(&Rect::new(0, 0, 1, 5)).is_empty());
    }

    #[test]
    fn app_layout() {
        let (header, body) = app::layout(80, 24);
        assert_eq!(header, Rect::new(0, 0, 80, 1));
        assert_eq!(body, Rect::new(0, 1, 80, 23));
    }

    #[test]
    fn reexported() {
        // crate::Rect と crate::geometry::Rect は同じ型
        let a: crate::Rect = Rect::new(1, 2, 3, 4);
        assert_eq!(a.width, 3);
    }
}
