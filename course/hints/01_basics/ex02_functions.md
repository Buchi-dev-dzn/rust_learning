`if` は値を持つ式です。`if a { x } else { y }` を関数の最後に書けば、そのまま戻り値になります。
`return` も `;` も要りません。

---

- `clamp`: `if value < min { min } else if value > max { max } else { value }`
- `move_left`: `u16` の引き算で負になるとパニックします。第1章の「整数のオーバーフロー」で紹介したメソッドを探しましょう。
- `center_pad`: 空き幅 (width - len) を 2 で割ります。ただし len > width のときも引き算が溢れないように。

---

```rust
fn move_left(col: u16, n: u16) -> u16 {
    col.saturating_sub(n)
}

fn center_pad(width: usize, len: usize) -> usize {
    width.saturating_sub(len) / 2
}
```
