まずは `Buffer` の基本 (`new` / `get` / 幅 1 の `set` / `set_str` / `clear` / `lines`) と `sgr` を作り、
全角を除くテストを通そう。

```rust
pub fn new(width: usize, height: usize) -> Buffer {
    Buffer { width, height, cells: vec![Cell::blank(); width * height] }
}
```

`sgr` は `String::from("\x1b[0")` に `;1` や `;31` を `push_str` していき、最後に `m`。
色の番号は `Black=0 .. White=7` として `30 + k` / `40 + k`、`Indexed(n)` は `;38;5;n` / `;48;5;n`。

---

全角文字の `set` は「書く前に壊れるものを片付ける」補助関数を作ると見通しがよい:

```rust
fn break_wide_at(&mut self, x: usize, y: usize) {
    let i = y * self.width + x;
    if self.cells[i].is_continuation() && x > 0 {
        self.cells[i - 1].ch = ' ';           // 自分が右半分 → 左半分を消す
    }
    if x + 1 < self.width && self.cells[i + 1].is_continuation() {
        self.cells[i + 1].ch = ' ';           // 自分が左半分 → 右半分を消す
    }
}
```

幅 2 の文字なら `x` と `x+1` の両方についてこれを呼んでから書き込む。

---

`diff` は 2 つの状態変数を持ってセルを走査するだけ:

```rust
let mut cursor: Option<(usize, usize)> = None;
let mut current = Style::default();
for y in 0..next.height() {
    for x in 0..next.width() {
        let n = next.get(x, y);
        if n.is_continuation() || n == prev.get(x, y) { continue; }
        if cursor != Some((x, y)) { /* "\x1b[{};{}H", y + 1, x + 1 */ }
        if n.style != current { /* sgr を出して current を更新 */ }
        out.push(n.ch);
        let nx = x + char_width(n.ch);
        cursor = if nx >= next.width() { None } else { Some((nx, y)) };
    }
}
if current != Style::default() { out.push_str("\x1b[0m"); }
```

サイズが違うときは `"\x1b[2J"` を出してから、`Buffer::new(next.width(), next.height())` を `prev` の代わりに使う。
`Renderer::render` は `self.prev` が `None` なら `Buffer::new(0, 0)` と比べればよい (サイズ違いの規則がそのまま効く)。

`String` に `write!` で書き込むには `use std::fmt::Write as _;` が必要。
