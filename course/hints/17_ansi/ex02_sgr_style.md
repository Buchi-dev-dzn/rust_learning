`fg_params` / `bg_params` は `match c { Color::Basic(n) => ..., ... }` で 4 通りに分けます。
`Basic(n)` は `format!("{}", 30 + n)` のように数値を足してから文字列にします。

---

`sgr` では引数を `Vec<String>` に集めるのがコツです。

```rust
let mut params = vec!["0".to_string()];
if style.bold { params.push("1".into()); }
// underline, reverse も同様
if let Some(c) = style.fg { params.push(fg_params(c)); }
```

最後に `params.join(";")` でつなぎます。

---

```rust
format!("\x1b[{}m", params.join(";"))
```

`paint` は `format!("{}{text}\x1b[0m", sgr(style))` です。
