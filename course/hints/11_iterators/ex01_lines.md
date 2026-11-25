どの関数も「イテレータを作る → アダプタでつなぐ → `collect()` で消費」の 3 段構成です。
`text.lines()` は行のイテレータ、`lines.iter()` は `&String` のイテレータを返します。

---

- `non_empty_lines`: `.filter(|l| !l.trim().is_empty())`
- `grep`: `.enumerate()` は 0 始まりの `(i, l)` を作ります。`filter` の引数は `&(usize, &str)` なので `|(_, l)|` で分解できます。最後に `map(|(i, l)| (i + 1, l))`。
- `visible`: `.skip(scroll).take(height).map(|s| s.as_str())`
- `parse_numbers`: `map` の結果が `Result<i32, _>` のイテレータなら、`collect::<Result<Vec<i32>, _>>()` で「全部 Ok なら Ok(Vec)」になります。

---

```rust
pub fn grep<'a>(text: &'a str, pattern: &str) -> Vec<(usize, &'a str)> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| l.contains(pattern))
        .map(|(i, l)| (i + 1, l))
        .collect()
}

pub fn numbered(lines: &[&str]) -> Vec<String> {
    lines.iter().enumerate().map(|(i, l)| format!("{:>3} | {}", i + 1, l)).collect()
}

pub fn parse_numbers(s: &str) -> Result<Vec<i32>, ParseIntError> {
    s.split(',').map(|p| p.trim().parse::<i32>()).collect()
}
```
