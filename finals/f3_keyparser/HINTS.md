設計の核心は「バッファの先頭からキーを 1 個だけ切り出す関数」を作ること。

```rust
enum Parsed {
    Key(Key, usize), // 解釈できたキーと消費バイト数
    Incomplete,      // 続きのバイトが必要
}

fn parse_one(buf: &[u8]) -> Parsed { /* buf は空でない */ }
```

`feed` はバッファに追記して、`Incomplete` が返るかバッファの終わりまで `parse_one` を繰り返し、
消費した分を `buf.drain(..pos)` で捨てるだけ。`parse` は `feed` + `flush`。

まずは ESC 以外の 1 バイト (表 2.1) を `match b { 0x0d | 0x0a => ..., 0x01..=0x1a => ..., }` で書いて、
`plain_ascii_and_control` を通そう。

---

ESC の処理は `buf.get(1)` で次のバイトを見る。`None` なら `Incomplete`。

CSI は「`0x20..=0x3F` を読み飛ばし、`0x40..=0x7E` で止まる」ループ:

```rust
let mut i = 2;
loop {
    let Some(&c) = buf.get(i) else { return Parsed::Incomplete };
    match c {
        0x20..=0x3f => i += 1,
        0x40..=0x7e => break,
        _ => return Parsed::Key(Key::Unknown(buf[..i].to_vec()), i),
    }
}
let params = std::str::from_utf8(&buf[2..i]).unwrap_or("");
let p: Option<u32> = params.split(';').next().and_then(|s| s.parse().ok());
```

`~` の F キーは番号が飛び飛び (16 と 22 が欠番) なので、`Some(n @ 11..=15) => F((n - 10) as u8)` のように範囲ごとに書く。

---

UTF-8: 先頭バイトから長さを決め、**届いている分の**継続バイトを先にチェックしてから「足りなければ Incomplete」とする。
順番を逆にすると `[0xE3, b'a']` が永遠に Incomplete になってしまう。

ペースト: `ESC[200~` を見つけたら、その後ろから `b"\x1b[201~"` を探す。

```rust
body.windows(6).position(|w| w == b"\x1b[201~")
```

見つからなければ `Incomplete` (開始シーケンスごと保持)。`flush` では `buf.starts_with(b"\x1b[200~")` のとき何もしない。

`flush` は保持中のバイト列に対する `match pending.as_slice() { [0x1b] => Esc, [0x1b, b'['] => Alt('['), ... }` で書ける。

`every_split_point_gives_same_result` が落ちる場合、「未完成なのに何かを確定させている」箇所がある。
