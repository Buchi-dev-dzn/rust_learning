//! # Box で再帰的なレイアウト木を作る
//!
//! 画面分割を表す木 `Layout` を作り、各ペインの位置 (Rect) を計算しよう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `Layout::Split(Dir, 子A, 子B)` は領域を 2 つに分ける
//!   - `Dir::Horizontal`: 左右に分ける。左の幅は `width / 2`、右は残り
//!   - `Dir::Vertical`: 上下に分ける。上の高さは `height / 2`、下は残り
//! - `count_leaves`: 葉 (ペイン) の数
//! - `depth`: 木の深さ (葉だけなら 1)
//! - `compute`: 各葉の `(名前, Rect)` を左 (上) から順に返す
//! - 関連: 第12章「Box<T>: ヒープに置く」

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dir {
    Horizontal,
    Vertical,
}

// 子を Box で持つことで Layout のサイズが決まる
#[derive(Debug)]
pub enum Layout {
    Leaf(String),
    Split(Dir, Box<Layout>, Box<Layout>),
}

impl Layout {
    pub fn leaf(name: &str) -> Layout {
        Layout::Leaf(name.to_string())
    }

    pub fn split(dir: Dir, a: Layout, b: Layout) -> Layout {
        Layout::Split(dir, Box::new(a), Box::new(b))
    }

    pub fn count_leaves(&self) -> usize {
        match self {
            Layout::Leaf(_) => 1,
            Layout::Split(_, a, b) => a.count_leaves() + b.count_leaves(),
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            Layout::Leaf(_) => 1,
            Layout::Split(_, a, b) => 1 + a.depth().max(b.depth()),
        }
    }

    pub fn compute(&self, area: Rect) -> Vec<(String, Rect)> {
        let mut out = Vec::new();
        self.compute_into(area, &mut out);
        out
    }

    // 再帰の途中結果を 1 つの Vec に貯める補助関数
    fn compute_into(&self, area: Rect, out: &mut Vec<(String, Rect)>) {
        match self {
            Layout::Leaf(name) => out.push((name.clone(), area)),
            Layout::Split(dir, a, b) => {
                let (ra, rb) = split_rect(area, *dir);
                a.compute_into(ra, out);
                b.compute_into(rb, out);
            }
        }
    }
}

pub fn split_rect(area: Rect, dir: Dir) -> (Rect, Rect) {
    match dir {
        Dir::Horizontal => {
            let left = area.width / 2;
            (
                Rect { width: left, ..area },
                Rect { x: area.x + left, width: area.width - left, ..area },
            )
        }
        Dir::Vertical => {
            let top = area.height / 2;
            (
                Rect { height: top, ..area },
                Rect { y: area.y + top, height: area.height - top, ..area },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect { x, y, width, height }
    }

    fn sample() -> Layout {
        // +--------+--------+
        // |        | editor |
        // | files  +--------+
        // |        | status |
        // +--------+--------+
        Layout::split(
            Dir::Horizontal,
            Layout::leaf("files"),
            Layout::split(Dir::Vertical, Layout::leaf("editor"), Layout::leaf("status")),
        )
    }

    #[test]
    fn counts() {
        assert_eq!(Layout::leaf("x").count_leaves(), 1);
        assert_eq!(Layout::leaf("x").depth(), 1);
        assert_eq!(sample().count_leaves(), 3);
        assert_eq!(sample().depth(), 3);
    }

    #[test]
    fn split_rect_odd_sizes() {
        assert_eq!(split_rect(r(0, 0, 5, 3), Dir::Horizontal), (r(0, 0, 2, 3), r(2, 0, 3, 3)));
        assert_eq!(split_rect(r(1, 1, 4, 5), Dir::Vertical), (r(1, 1, 4, 2), r(1, 3, 4, 3)));
    }

    #[test]
    fn compute_sample() {
        let panes = sample().compute(r(0, 0, 80, 24));
        assert_eq!(
            panes,
            vec![
                ("files".to_string(), r(0, 0, 40, 24)),
                ("editor".to_string(), r(40, 0, 40, 12)),
                ("status".to_string(), r(40, 12, 40, 12)),
            ]
        );
    }

    #[test]
    fn compute_single_leaf() {
        assert_eq!(Layout::leaf("only").compute(r(3, 4, 5, 6)), vec![("only".to_string(), r(3, 4, 5, 6))]);
    }

    #[test]
    fn deep_tree() {
        // 左に 4 回分割し続けると、一番左のペインは幅 80/16 = 5
        let mut l = Layout::leaf("leftmost");
        for i in 0..4 {
            l = Layout::split(Dir::Horizontal, l, Layout::leaf(&format!("r{i}")));
        }
        assert_eq!(l.depth(), 5);
        let panes = l.compute(r(0, 0, 80, 1));
        assert_eq!(panes[0], ("leftmost".to_string(), r(0, 0, 5, 1)));
        let total: u16 = panes.iter().map(|(_, rect)| rect.width).sum();
        assert_eq!(total, 80);
    }

    #[test]
    fn uses_box_directly() {
        let l = Layout::Split(Dir::Vertical, Box::new(Layout::leaf("a")), Box::new(Layout::leaf("b")));
        assert_eq!(l.count_leaves(), 2);
    }
}
