エラーを 1 つずつ読みましょう。E0502 は「共有借用が生きている間に可変借用した」、
E0499 は「可変借用が同時に 2 つ」、E0596 は「共有参照越しに書き換えようとした」です。

---

- `append_and_report`: `last` は `lines` の中を指す参照です。`format!` で報告文字列 (所有する `String`) を **先に** 作ってしまえば、その後は `last` を使わないので `push` できます。
- `swap_ends`: `v.swap(0, n - 1)` を使えば参照を 2 つ作る必要がありません。
- `shout_in_place`: 引数の型を `&mut String` にします。

---

```rust
let report = format!("{last} -> {new_line}");
lines.push(new_line.to_string());
report
```

```rust
v.swap(0, n - 1);
```

```rust
fn shout_in_place(s: &mut String) {
    s.make_ascii_uppercase();
}
```
