`?` は `Result` の `Err` を見つけるとその場で `return Err(..)` します。
`Option` を `Result` に変えるには `ok_or(エラー値)` を使います:
`let (w, h) = s.split_once('x').ok_or(SizeError::NoSeparator)?;`

---

- `parse_dim`: `let s = s.trim(); s.parse::<u32>().map_err(|_| SizeError::NotANumber(s.to_string()))` (「読めなかった部分」は trim 後の文字列)。
- `check_dim`: `if n == 0 { Err(..) } else if n > MAX { Err(..) } else { Ok(n) }`。
- `parse_size`: 優先順位に注意。幅と高さの **両方を先に数値として読んでから** 範囲を調べます。

---

```rust
fn parse_size(s: &str) -> Result<(u32, u32), SizeError> {
    let (w, h) = s.split_once('x').ok_or(SizeError::NoSeparator)?;
    let w = parse_dim(w)?;
    let h = parse_dim(h)?;
    Ok((check_dim(w)?, check_dim(h)?))
}

fn size_or_default(s: &str) -> (u32, u32) {
    parse_size(s).unwrap_or((80, 24))
}
```
