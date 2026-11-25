`longer` のエラー E0106 は「戻り値が `a` と `b` のどちらから借りているか分からない」という意味です。
`fn longer<'a>(...)` とライフタイム引数を宣言し、`a`・`b`・戻り値に `'a` を付けます。

`after_prefix` は引数が 2 つの参照なので省略規則が効きません。
戻り値に関係するのは `text` だけなので、`text` と戻り値にだけ `'a` を付けます。

---

- `strip_comment`: `line.find('#')` は `Option<usize>` (バイト位置) を返します。
  `Some(i)` なら `&line[..i]`、`None` なら `line` 全体。最後に `.trim_end()`。
  `trim_end` もスライスを返すのでコピーは起きません。
- `nth_line`: `text.lines()` はイテレータなので `.nth(n)` が使えます。
- `after_prefix`: `str::strip_prefix` がそのまま `Option<&str>` を返します。

---

```rust
pub fn longer<'a>(a: &'a str, b: &'a str) -> &'a str { /* 本体はそのまま */ }

pub fn strip_comment(line: &str) -> &str {
    let code = match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    };
    code.trim_end()
}

pub fn after_prefix<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.strip_prefix(prefix)
}
```
