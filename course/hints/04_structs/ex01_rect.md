`new` は `Self { x, y, width, height }` の 1 行で書けます (フィールド初期化の省略記法)。
`area` は `self.width as u32 * self.height as u32`。

---

- `contains`: 範囲は「左端・上端を含み、右端・下端を含まない」半開区間です。`px >= self.x && px < self.x + self.width` を y についても。
- `split_left`: まず `let w = w.min(self.width);`。左は幅 `w`、残りは `x` が `self.x + w` で幅 `self.width - w`。
- `intersects`: x 方向の重なり `self.x < other.x + other.width && other.x < self.x + self.width` と、同じ形の y 方向の重なりの両方が必要です。ただし幅 0 の Rect はこの式だと「重なる」と判定されてしまうので、先に `self.area() == 0 || other.area() == 0` なら `false` を返します。

---

```rust
fn split_left(&self, w: u16) -> (Rect, Rect) {
    let w = w.min(self.width);
    let left = Rect::new(self.x, self.y, w, self.height);
    let rest = Rect::new(self.x + w, self.y, self.width - w, self.height);
    (left, rest)
}

fn intersects(&self, other: &Rect) -> bool {
    if self.area() == 0 || other.area() == 0 {
        return false;
    }
    let x_overlap = self.x < other.x + other.width && other.x < self.x + self.width;
    let y_overlap = self.y < other.y + other.height && other.y < self.y + self.height;
    x_overlap && y_overlap
}
```
