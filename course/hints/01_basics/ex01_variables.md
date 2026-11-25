エラー `E0384: cannot assign twice to immutable variable` は「不変の変数に再代入した」という意味です。
コンパイラの `help:` 行に修正案が書いてあります。

---

`move_right` では `let col` を `let mut col` にします。
`parse_and_double` の `value * 2` は `&str * i32` になっているのでエラーです (E0369)。
`value` を数値として作り直す必要があります。

---

`let value = input.trim();` の次の行に

```rust
let value: i32 = value.parse().unwrap();
```

を追加します。同じ名前で新しい変数を作るのがシャドーイングです。
