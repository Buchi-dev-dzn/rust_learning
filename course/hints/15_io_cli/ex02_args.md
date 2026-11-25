まず `Config { lines: 10, quiet: false, files: Vec::new() }` を作り、引数を 1 つずつ見て書き換えていきます。
`-n` は「次の引数」も必要なので、`for` ではなく `let mut it = args.iter(); while let Some(a) = it.next()` の形にすると、
ループの中で `it.next()` を呼んで値を取り出せます。

---

`match a.as_str()` で分岐します。順番が大事です:

1. `"-h" | "--help"` → `return Err(ArgError::Help)`
2. `"-q"`, `"-n"`
3. `"-"` → ファイル
4. `s if s.starts_with('-')` → 不明なオプション
5. `_` → ファイル

`Option` → `Result` の変換は `.ok_or(...)`、`parse` の失敗は `.map_err(|_| ...)` で変換して `?` で返せます。

---

```rust
"-n" => {
    let v = it.next().ok_or(ArgError::MissingValue("-n"))?;
    cfg.lines = v.parse().map_err(|_| ArgError::InvalidNumber(v.clone()))?;
}
```

ループ後に `if cfg.files.is_empty() { cfg.files.push("-".to_string()); }`。

`exit_code` は:

```rust
match r {
    Ok(_) | Err(ArgError::Help) => 0,
    Err(_) => 2,
}
```
