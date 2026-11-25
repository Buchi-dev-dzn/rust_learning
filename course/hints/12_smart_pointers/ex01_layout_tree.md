E0072 は「`Layout` の中に `Layout` が直接入っているので、サイズが無限になる」というエラーです。
`Split(Dir, Box<Layout>, Box<Layout>)` にし、`split` 関数では `Box::new(a)` のように包みます。

---

再帰関数は `match self` で書きます。`Box<Layout>` に対してもメソッドはそのまま呼べます (Deref)。

```rust
match self {
    Layout::Leaf(_) => 1,
    Layout::Split(_, a, b) => a.count_leaves() + b.count_leaves(),
}
```

`depth` は `1 + a.depth().max(b.depth())`。

---

```rust
pub fn split_rect(area: Rect, dir: Dir) -> (Rect, Rect) {
    match dir {
        Dir::Horizontal => {
            let left = area.width / 2;
            (
                Rect { width: left, ..area },
                Rect { x: area.x + left, width: area.width - left, ..area },
            )
        }
        Dir::Vertical => { /* y と height で同様に */ }
    }
}

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
```
