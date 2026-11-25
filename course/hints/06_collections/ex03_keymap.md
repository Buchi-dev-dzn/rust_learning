`parse_bindings` は 1 行ずつ: `trim` → 空行・`#` で始まる行を `continue` → `split_once('=')` で分ける →
キーと値を `trim` → どちらかが空なら `continue` → `map.insert(key.to_string(), cmd.to_string())`。
`insert` は上書きなので「後のものが勝つ」は何もしなくても実現できます。

---

- `command_for`: `map.get(key)` は `Option<&String>`。`.map(|s| s.as_str())` で `Option<&str>` にします。
- `keys_by_command`: `for (key, cmd) in map` で回し、`groups.entry(cmd.clone()).or_default().push(key.clone());`。
- HashMap の繰り返し順序は毎回違い得るので、最後に Vec にしてから並べ替えます。タプルの Vec は `sort()` で先頭要素順に並びます。

---

```rust
fn keys_by_command(map: &HashMap<String, String>) -> Vec<(String, Vec<String>)> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for (key, cmd) in map {
        groups.entry(cmd.clone()).or_default().push(key.clone());
    }
    let mut out: Vec<(String, Vec<String>)> = groups.into_iter().collect();
    for (_, keys) in &mut out {
        keys.sort();
    }
    out.sort();
    out
}
```
