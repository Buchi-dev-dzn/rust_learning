状態ごとに処理を分けます。`feed` の骨格は次の形です。

```rust
let state = std::mem::replace(&mut self.state, State::Ground);
let (next, key) = match state {
    State::Ground => { /* ... */ }
    State::Escape => { /* ... */ }
    State::Csi(mut params) => { /* ... */ }
    State::Ss3 => { /* ... */ }
    State::Utf8 { mut buf, need } => { /* ... */ }
};
self.state = next;
key
```

`mem::replace` で状態を取り出すと、`Csi(Vec<u8>)` の中の `Vec` を借用ではなく所有権ごと受け取れるので、
`push` してから新しい状態に入れ直すのが簡単になります。

---

- `Ground` は ex01 の `byte_to_key` とほぼ同じ。ただし `0x1b` と UTF-8 の先頭バイトは「まだ確定しない」ので `(State::Escape, None)` などを返す
- `Csi` は `0x30..=0x3f` なら `params.push(b)` して `(State::Csi(params), None)`。それ以外のバイトでキーを確定
- `~` の場合、最初の引数は `params.split(|&c| c == b';').next()` で取り出し、`std::str::from_utf8(..)?.parse::<u32>()` で数値にする
- `Unknown` 用のバイト列は `vec![0x1b, b'[']` に `params` と `b` を足して作る
- `Utf8` は `b & 0xc0 == 0x80` (続きのバイト) か確認し、`need` が 1 になったら `std::str::from_utf8(&buf)` で文字に変換

---

`~` の対応表は補助関数に切り出すと読みやすいです。

```rust
fn csi_final(params: &[u8], fin: u8) -> Option<Key> {
    let key = match fin {
        b'A' => Key::Up, b'B' => Key::Down, b'C' => Key::Right, b'D' => Key::Left,
        b'H' => Key::Home, b'F' => Key::End,
        b'~' => {
            let first = params.split(|&c| c == b';').next().unwrap_or(&[]);
            let n: u32 = std::str::from_utf8(first).ok()?.parse().ok()?;
            match n {
                1 | 7 => Key::Home, 2 => Key::Insert, 3 => Key::Delete, 4 | 8 => Key::End,
                5 => Key::PageUp, 6 => Key::PageDown, 15 => Key::F(5),
                17..=21 => Key::F((n - 11) as u8), 23 | 24 => Key::F((n - 12) as u8),
                _ => return None,
            }
        }
        _ => return None,
    };
    Some(key)
}
```

`flush` は `match std::mem::replace(&mut self.state, State::Ground) { State::Ground => None, State::Escape => Some(Key::Esc), ... }` です。
`Csi(params)` なら `ESC [` + `params`、`Ss3` なら `ESC O`、`Utf8 { buf, .. }` なら `buf` を `Unknown` に入れます。
