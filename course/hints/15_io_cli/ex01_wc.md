`String` を 1 つ用意し、`r.read_line(&mut buf)` をループで呼びます。戻り値は読んだバイト数で、`0` なら EOF です。
ループの先頭で `buf.clear()` を忘れずに。

---

- バイト数: `read_line` の戻り値を足していく
- 行数: `buf.ends_with('\n')` のときだけ +1
- 単語数: `buf.split_whitespace().count()`

`report` は `writeln!(w, "{:>7} ...", ...)` で書けます。`{:>7}` が幅 7 の右寄せです。

---

```rust
loop {
    buf.clear();
    let n = r.read_line(&mut buf)?;
    if n == 0 { break; }
    stats.bytes += n;
    if buf.ends_with('\n') { stats.lines += 1; }
    stats.words += buf.split_whitespace().count();
}
Ok(stats)
```

```rust
writeln!(w, "{:>7} {:>7} {:>7} {}", s.lines, s.words, s.bytes, name)
```
