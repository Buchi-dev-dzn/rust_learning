まず `truncate` から。`s.chars().count() <= width` ならそのまま `to_string()`。
超えるなら `s.chars().take(width - 1).collect::<String>()` に `'…'` を push します
(`width == 0` のときは `width - 1` がオーバーフローするので先に分岐)。

`wrap` は解説の `wrap` 関数がベースです。ただし `word.len()` はバイト数なので、
日本語のテストが通るよう `word.chars().count()` を使いましょう。

---

`wrap` の流れ:

1. `for line in text.lines()` — 空白だけの行なら `String::new()` を push して次へ
2. 行ごとに `current: String` と `cur_len: usize` (文字数) を用意
3. `for word in line.split_whitespace()`:
   - `needed = if cur_len == 0 { wlen } else { cur_len + 1 + wlen }`
   - `needed > width` なら `current` を out に出して、`current = word`
   - そうでなければ (必要なら空白を足して) `current` に追加
4. 行の終わりで `current` が空でなければ push

`std::mem::take(&mut current)` を使うと「中身を取り出して current を空にする」が 1 行で書けます。

---

長すぎる単語 (`wlen > width`) の処理:

```rust
if wlen > width {
    if cur_len > 0 {
        out.push(std::mem::take(&mut current));
    }
    let chars: Vec<char> = word.chars().collect();
    let mut pieces: Vec<String> = chars.chunks(width).map(|c| c.iter().collect()).collect();
    current = pieces.pop().unwrap_or_default(); // 最後のかけらは次の単語と同じ行に続けられる
    cur_len = current.chars().count();
    out.extend(pieces);
    continue;
}
```
