`spawn_square` は `thread::spawn(move || n * n)` だけです。`JoinHandle<u64>` の `u64` はクロージャの戻り値の型です。

`thread::scope` の中では、`s.spawn(...)` で起動したスレッドが外の変数 (`lines`・`pattern`) を **借用したまま** 使えます。
scope を抜ける前に全スレッドが終わることが保証されるからです。

---

`count_matches` の形:

```rust
thread::scope(|s| {
    let handles: Vec<_> = lines
        .chunks(size)
        .map(|chunk| s.spawn(move || chunk.iter().filter(|l| l.contains(pattern)).count()))
        .collect();   // ← いったん collect して全部起動してから
    handles.into_iter().map(|h| h.join().unwrap()).sum()
})
```

`collect` せずに `map(spawn).map(join)` とつなぐと、1 つ起動して終わるのを待ってから次を起動する
(= 並列にならない) ので注意。

`move` は `chunk` (と `pattern` という **参照**) をクロージャにコピーするためのもので、元のデータは移動しません。

---

`longest_line` は「塊ごとの最長」→「全体の最長」の 2 段階です。比較には `Option::is_none_or` が便利です。

```rust
let mut best: Option<&'a str> = None;
for &l in chunk {
    if best.is_none_or(|b| l.chars().count() > b.chars().count()) {
        best = Some(l);
    }
}
best
```

`>` (より大きい) で比べるので、同じ長さなら先のものが残ります。全体の集計も、ハンドルを塊の順に `join` して同じ比較をします。
