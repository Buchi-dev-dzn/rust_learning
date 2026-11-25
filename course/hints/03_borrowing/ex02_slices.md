`trim()` も `&s[a..b]` も新しい文字列を作らず、元の文字列の一部を指す `&str` を返します。
戻り値の型が `&str` なので、`String` を作って返すことはできません (作っても関数を抜けると消えてしまう)。

---

- `split_command`: まず `let line = line.trim();`。`line.find(' ')` は `Option<usize>` (最初の空白のバイト位置) を返すので、`match` で `Some(i)` と `None` に分けます。引数側は `line[i + 1..].trim()` で余分な空白を除きます。
- `truncate_chars`: `s.char_indices().nth(max_chars)` は、`max_chars` 文字を過ぎた次の文字の `(バイト位置, 文字)` を返します。なければ `None` (文字数が足りない)。

---

```rust
fn split_command(line: &str) -> (&str, &str) {
    let line = line.trim();
    match line.find(' ') {
        Some(i) => (&line[..i], line[i + 1..].trim()),
        None => (line, ""),
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((byte_pos, _)) => &s[..byte_pos],
        None => s,
    }
}
```
