- `sum_to`: `let mut total = 0;` を用意し、`for i in 1..=n` で足していきます。`..=` は終端を含む範囲です。
- `digits`: 10 で割るたびに 1 桁減ります。0 になるまで割った回数が桁数です。ただし 0 自身は 1 桁。

---

- `digits` は「まず 1 回割ってから 0 かどうか調べる」形にすると、0 の特別扱いが要りません。`loop { ...; if n == 0 { break count; } }`
- `fizzbuzz` は `(n % 3, n % 5)` というタプルで `match` し、`(0, 0)` を一番上に書きます。
- `hline` は `i == 0 || i == width - 1` なら `'+'`。`width == 0` のときはループが 1 回も回らないので `width - 1` は評価されません。

---

```rust
fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}
```
