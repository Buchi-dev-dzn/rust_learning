`truncate` は「ここまでの幅 `w`」を持ちながら `s.char_indices()` を回します。
`w + char_width(c) > max` になった瞬間、その文字の先頭バイト位置 `i` で `&s[..i]` を返します。
最後まで超えなければ `s` 全体を返します。

---

`fit` の考え方:

1. `width == 0` なら `String::new()`
2. `str_width(s) <= width` なら `s` + 空白 `(width - w)` 個
3. そうでなければ `truncate(s, width - 1)` に `…` を付け、`width - 1 - 切り詰め後の幅` 個の空白を足す

空白は `" ".repeat(n)` で作れます。

---

```rust
fn fit(s: &str, width: usize) -> String {
    if width == 0 { return String::new(); }
    let w = str_width(s);
    if w <= width {
        return format!("{s}{}", " ".repeat(width - w));
    }
    let t = truncate(s, width - 1);
    format!("{t}…{}", " ".repeat(width - 1 - str_width(t)))
}
```

`fit_right` は収まる場合だけ `format!("{}{s}", " ".repeat(width - w))`、収まらなければ `fit(s, width)` を返します。
