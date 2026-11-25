`byte_of` は `line.char_indices().nth(ci)` が `Option<(usize, char)>` を返すので、
`.map(|(b, _)| b).unwrap_or(line.len())` で書けます。

`col_of` は `line.chars().take(ci).map(char_width).sum()` です。

---

`char_at_col` は、各文字が「列 `x` から `x + w - 1` まで」を占めると考えます。
`for (i, c) in line.chars().enumerate()` で `x` を進めながら、`col < x + w` になった最初の `i` が答えです。
ループを抜けたら行末なので `line.chars().count()` を返します。

---

```rust
fn char_at_col(line: &str, col: usize) -> usize {
    let mut x = 0;
    for (i, c) in line.chars().enumerate() {
        let w = char_width(c);
        if col < x + w { return i; }
        x += w;
    }
    line.chars().count()
}

fn insert_at(line: &mut String, ci: usize, c: char) {
    let b = byte_of(line, ci);
    line.insert(b, c);
}
```
