`let Some(x) = 式 else { return None; };` と書くと、`Some` のときは中身が `x` に入り、
`None` のときは関数から抜けます。3 回並べれば `parse_size` が書けます。

---

- `parse_size`: `s.split_once('x')` → `(w, h)`、`w.parse::<u16>().ok()` → `Option<u16>`。
- `selected_label`: `match selected { Some(i) => match items.get(i) { Some(s) => s, None => "(none)" }, None => "(none)" }` でも書けます。`&String` は `&str` として返せます。
- `select_next`: `match (selected, len)` で、まず `(_, 0) => None` を一番上に。

---

```rust
fn selected_label(items: &[String], selected: Option<usize>) -> &str {
    selected
        .and_then(|i| items.get(i))
        .map(|s| s.as_str())
        .unwrap_or("(none)")
}

fn select_next(selected: Option<usize>, len: usize) -> Option<usize> {
    match (selected, len) {
        (_, 0) => None,
        (None, _) => Some(0),
        (Some(i), len) => Some((i + 1).min(len - 1)),
    }
}
```
