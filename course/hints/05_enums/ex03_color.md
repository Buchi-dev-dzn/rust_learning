まず型を定義します。データを持たないバリアントとデータを持つバリアントを混ぜられます。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Reset,
    Red,
    Green,
    Blue,
    Indexed(u8),
    Rgb(u8, u8, u8),
}
```

---

- `fg` / `bg` は `match self { Color::Red => "31".to_string(), Color::Indexed(n) => format!("38;5;{n}"), ... }`。
- `from_name` は `name.to_ascii_lowercase()` で小文字にしてから `match lower.as_str() { "red" => ..., _ => { /* #rrggbb の処理 */ } }`。
- `#rrggbb` は `lower.strip_prefix('#')` (`Option<&str>`) で `#` を外し、長さ 6 を確認して `&hex[0..2]` などを `u8::from_str_radix(.., 16).ok()` で変換します。

---

```rust
_ => {
    let Some(hex) = lower.strip_prefix('#') else { return None; };
    if hex.len() != 6 {
        return None;
    }
    let (Ok(r), Ok(g), Ok(b)) = (
        u8::from_str_radix(&hex[0..2], 16),
        u8::from_str_radix(&hex[2..4], 16),
        u8::from_str_radix(&hex[4..6], 16),
    ) else {
        return None;
    };
    Some(Color::Rgb(r, g, b))
}
```

(`hex` が ASCII 以外を含むと `&hex[0..2]` がパニックし得ます。気になる人は `hex.is_ascii()` も確認しよう)
