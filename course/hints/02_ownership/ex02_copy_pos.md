エラーは `move occurs because p has type Pos, which does not implement the Copy trait` と言っています。
関数本体を変えずに直すには、`Pos` 自体を `Copy` にするしかありません。

---

`Pos` のフィールドは `u16` だけで、ヒープを所有していません。こういう型は `#[derive(...)]` で `Copy` にできます。
`Copy` は `Clone` を前提にしている (supertrait) ので、両方を並べる必要があります。

---

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
struct Pos {
    x: u16,
    y: u16,
}
```

もし `Pos` に `String` のフィールドがあったら `Copy` は付けられません (試してみよう)。
