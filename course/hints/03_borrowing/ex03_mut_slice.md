`&mut row[a..b]` で行の一部だけを可変で借りられます。ただし `a > b` や `b > row.len()` だとパニックするので、
スライスを作る **前に** 範囲を調整します。

---

- `fill_range`: `let end = end.min(row.len());` として、`start >= end` なら `return`。その後 `for cell in &mut row[start..end] { *cell = ch; }`。
- `put_str`: `text.chars().enumerate()` で `(i, ch)` を取り、`x + i` が `row.len()` 以上になったら `break`。
- `count_filled`: `for &c in row` と書くと `c` が `char` になります。

---

```rust
fn put_str(row: &mut [char], x: usize, text: &str) -> usize {
    let mut written = 0;
    for (i, ch) in text.chars().enumerate() {
        let pos = x + i;
        if pos >= row.len() {
            break;
        }
        row[pos] = ch;
        written += 1;
    }
    written
}
```
