テストの `use` 文が「完成形のパス」を教えてくれています:
`crate::app`、`crate::geometry::Rect`、`crate::render::draw_box`。
まず空のモジュール `mod geometry {}` / `mod render {}` / `mod app {}` を作り、中身を 1 つずつ足しましょう。

---

- `geometry`: `pub struct Rect { pub x: u16, ... }` と `impl Rect { pub fn new(..), pub fn split_top(..) }`。テストで `Rect { x: .., .. }` と直接作っているのでフィールドも `pub` が必要です。
- `render` の中からは `Rect` は見えません。`use super::geometry::Rect;` (または `use crate::geometry::Rect;`) を書きます。
- `app::layout` は `Rect::new(0, 0, width, height).split_top(1)` の 1 行で書けます。
- ルートに `pub use geometry::Rect;`。

---

```rust
mod render {
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
```
