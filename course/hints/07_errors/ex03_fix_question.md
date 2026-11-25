エラーは 3 種類です。

1. `` `?` couldn't convert the error to `LoadError` `` — `From` が足りない
2. `` mismatched types: expected `Result<Vec<i64>, LoadError>`, found `Vec<i64>` `` — `Ok` で包み忘れ
3. `` the `?` operator can only be used in a function that returns `Result` or `Option` `` — 戻り値の型

---

- 1: `impl From<ParseIntError> for LoadError { fn from(e: ParseIntError) -> Self { ... } }` を書きます。
- 2: 最後の `out` を `Ok(out)` に。
- 3: `fn total(s: &str) -> Result<i64, LoadError>`。

---

```rust
impl From<ParseIntError> for LoadError {
    fn from(e: ParseIntError) -> Self {
        LoadError::Parse(e)
    }
}
```
