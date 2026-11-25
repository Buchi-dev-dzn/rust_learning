`format!` の中で定数 `ESC` を `{ESC}` と書けます。`move_to` は `format!("{ESC}[{};{}H", y + 1, x + 1)` です。
**y (行) が先** である点に注意。

---

`move_by` は空の `String` を作り、条件ごとに `push_str` / `+=` で足していきます。
負の値は `-dy` で正に直してから埋め込みます (`ESC[-3A` のような書き方はできません)。

---

```rust
let mut s = String::new();
if dy < 0 { s += &format!("{ESC}[{}A", -dy); } else if dy > 0 { s += &format!("{ESC}[{dy}B"); }
if dx > 0 { s += &format!("{ESC}[{dx}C"); } else if dx < 0 { s += &format!("{ESC}[{}D", -dx); }
s
```

`enter_tui` は `format!("{ESC}[?1049h{ESC}[?25l{}", clear_screen())`、
`leave_tui` は `format!("{ESC}[?25h{ESC}[?1049l")` です。
