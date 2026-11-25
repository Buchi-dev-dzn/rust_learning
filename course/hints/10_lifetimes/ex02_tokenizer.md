骨組みはこうです。

```rust
let s = self.rest.trim_start();
let first = s.chars().next()?;   // 空なら ? で None を返して終了
match first {
    '|' => { /* ... */ }
    '"' => { /* ... */ }
    _ => { /* ... */ }
}
```

`s` は `&'a str` (元の入力の一部) なので、`&s[..n]` も `&'a str` になります。

---

- `'|'`: `self.rest = &s[1..];` として `Some(Token::Pipe)`。
- `'"'`: `let body = &s[1..];` で引用符の後ろを取り、`body.find('"')` で閉じ引用符を探します。
  見つかったら中身は `&body[..end]`、残りは `&body[end + 1..]`。見つからなければ中身は `body` 全体、残りは `""`。
- 単語: `s.find(|c: char| c.is_whitespace() || c == '|' || c == '"').unwrap_or(s.len())` で終わりの位置が分かります。

`find` の返す位置は必ず文字境界なので、日本語でも安全にスライスできます。

---

```rust
_ => {
    let end = s
        .find(|c: char| c.is_whitespace() || c == '|' || c == '"')
        .unwrap_or(s.len());
    self.rest = &s[end..];
    Some(Token::Word(&s[..end]))
}
```
