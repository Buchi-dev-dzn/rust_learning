必要な状態は「現在位置 (col, row)」と「画面サイズ (cols, rows)」の 4 つの `u16` です。
`struct Cursor { col: u16, row: u16, cols: u16, rows: u16 }` のように書きます。
`new` は `Self { col: 0, row: 0, cols, rows }`。

---

- 読むだけの `col()` / `row()` は `&self`、動かすメソッドは `&mut self` です。
- 列の最大値は `cols - 1` ですが、`cols == 0` だと溢れます。`self.cols.saturating_sub(1)` を返す補助メソッド `max_col()` を作ると楽です。
- `right()` は `(self.col + 1).min(self.max_col())`、`left()` は `self.col.saturating_sub(1)`。

---

```rust
fn resize(&mut self, cols: u16, rows: u16) {
    self.cols = cols;
    self.rows = rows;
    self.col = self.col.min(self.max_col());
    self.row = self.row.min(self.max_row());
}
```

`down` / `up` も `right` / `left` と同じ形で書けます。
