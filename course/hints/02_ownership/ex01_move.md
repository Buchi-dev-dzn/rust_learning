エラーメッセージの `value moved here` が「どこでムーブされたか」、
`value used here after move` が「どこでムーブ後に使ったか」を指しています。
3 つの関数それぞれで、この 2 か所を見つけましょう。

---

- `original_and_upper`: `shout` には所有権を渡す必要がありますが、`text` は後でも使いたい。複製を渡せば元は残ります。
- `title_with_len`: `usize` は `Copy` です。長さを **先に** 取り出しておけば、後で `title` をムーブしても問題ありません。
- `triple_bang`: `append_bang` は加工した `String` を返します。その戻り値を `s` に代入し直します (`s` には `mut` が必要)。

---

```rust
let upper = shout(text.clone());
```

```rust
let len = title.len();
(len, title)
```

```rust
let mut s = String::from("go");
for _ in 0..3 {
    s = append_bang(s);
}
s
```
