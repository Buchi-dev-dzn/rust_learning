まずテストを読みましょう。`#[should_panic(expected = "...")]` の文字列は、
パニックメッセージに **含まれて** いなければなりません。
`todo!()` もパニックしますが、メッセージが "not yet implemented" なのでテストは失敗します。

`assert!(条件, "メッセージ {width}x{height}")` と書けば、条件が偽のときそのメッセージでパニックします。

---

`area` は `self.width as u32 * self.height as u32`。先に `u32` に変換するのがポイントです
(`(self.width * self.height) as u32` だと u16 のままオーバーフローします)。

`parse_size` の流れ:

1. `s.split_once('x')` は `Option<(&str, &str)>`。`.ok_or(SizeError::MissingX)?` で `Result` にして `?`
2. `w.parse::<u16>()` の `Err` を `.map_err(|_| SizeError::BadNumber(w.to_string()))?` で変換
3. どちらかが 0 なら `Err(SizeError::Zero)`

---

```rust
pub fn parse_size(s: &str) -> Result<(u16, u16), SizeError> {
    let (w, h) = s.split_once('x').ok_or(SizeError::MissingX)?;
    let w: u16 = w.parse().map_err(|_| SizeError::BadNumber(w.to_string()))?;
    let h: u16 = h.parse().map_err(|_| SizeError::BadNumber(h.to_string()))?;
    if w == 0 || h == 0 {
        return Err(SizeError::Zero);
    }
    Ok((w, h))
}
```
