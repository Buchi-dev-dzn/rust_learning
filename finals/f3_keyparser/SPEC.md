# F3: キー入力パーサ

raw モードの端末から読んだ**バイト列**を、`Key` の列に変換するパーサを作る。
crossterm の `event::read()` が内部でやっていることを自分で実装する課題。

この課題で身につくこと: エスケープシーケンス (CSI / SS3) の構造、UTF-8 のデコード、
**チャンク境界をまたぐインクリメンタル解析** (read() 1 回で 1 キーが揃うとは限らない)、
単独 ESC とシーケンス開始の曖昧さをタイムアウトで解決する設計。

外部クレートは `libc` のみ (main.rs の raw モード用)。`src/lib.rs` は標準ライブラリだけで書く。

## 1. 公開 API (`src/lib.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),      // 印字可能文字 (ASCII / UTF-8 マルチバイト)
    Ctrl(char),      // Ctrl + 文字 ('a'..='z' は小文字, ' ' '\\' ']' '^' '_')
    Alt(char),       // ESC + ASCII 印字可能文字
    Enter, Tab, BackTab, Backspace, Esc,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown, Insert, Delete,
    F(u8),           // F1..F12 → F(1)..F(12)
    Paste(String),   // ブラケットペーストの中身
    Unknown(Vec<u8>),// 解釈できなかったバイト列
}

#[derive(Debug, Default)]
pub struct Parser { /* 自由 (未処理バイトのバッファなど) */ }

impl Parser {
    pub fn new() -> Parser;
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Key>;
    pub fn flush(&mut self) -> Vec<Key>;
    pub fn has_pending(&self) -> bool;
}

pub fn parse(bytes: &[u8]) -> Vec<Key>; // = 新しい Parser で feed してから flush した結果を連結
```

## 2. `feed` の動作

`feed` は受け取ったバイトを内部バッファの末尾に足し、**先頭から完成したキーをすべて**取り出して返す。
最後にシーケンスの途中 (未完成) が残ったら、それはバッファに保持して次の `feed` を待つ。
`has_pending()` はバッファに未処理バイトが残っているときに `true`。

### 2.1 ESC 以外の 1 バイト

| バイト | Key |
|---|---|
| `0x0D`, `0x0A` | `Enter` |
| `0x09` | `Tab` |
| `0x7F`, `0x08` | `Backspace` |
| `0x00` | `Ctrl(' ')` |
| `0x01..=0x1A` (上記以外) | `Ctrl('a' + (b - 1))` 例: `0x01` → `Ctrl('a')`, `0x11` → `Ctrl('q')` |
| `0x1C` `0x1D` `0x1E` `0x1F` | `Ctrl('\\')` `Ctrl(']')` `Ctrl('^')` `Ctrl('_')` |
| `0x20..=0x7E` | `Char(b as char)` |
| `0x80..` | UTF-8 (2.2) |

### 2.2 UTF-8

- 先頭バイト `0xC2..=0xDF` → 2 バイト、`0xE0..=0xEF` → 3 バイト、`0xF0..=0xF4` → 4 バイト。
  それ以外 (`0x80..=0xC1`, `0xF5..`) → `Unknown(vec![b])` (1 バイト消費)。
- 届いている継続バイトのどれかが `0x80..=0xBF` でない → `Unknown(vec![先頭バイト])` (1 バイト消費) で、次のバイトから解析を続ける。
- 継続バイトがまだ全部届いていない → 未完成 (保持)。
- 揃ったら `std::str::from_utf8` でデコードし `Char(c)`。失敗したら `Unknown(vec![先頭バイト])` (1 バイト消費)。

### 2.3 ESC (`0x1B`) で始まる場合

ESC の次のバイトで分岐する。

| 次のバイト | 解釈 |
|---|---|
| (まだない) | 未完成 (保持) |
| `[` | CSI シーケンス (2.4) |
| `O` | SS3 シーケンス: さらに次の 1 バイトで決まる (なければ未完成)。下表 |
| `0x20..=0x7E` (`[` `O` 以外) | `Alt(その文字)` (2 バイト消費) |
| それ以外 (`ESC`, 制御文字, `0x80..`) | `Esc` (ESC 1 バイトだけ消費し、次のバイトから解析を続ける) |

SS3 (`ESC O x`): `A`→Up, `B`→Down, `C`→Right, `D`→Left, `H`→Home, `F`→End,
`P`/`Q`/`R`/`S`→`F(1)`..`F(4)`、それ以外→`Unknown(3 バイト)`。

### 2.4 CSI (`ESC [` ...)

- `ESC [` の後、`0x20..=0x3F` のバイト (パラメータ・中間バイト) が 0 個以上続き、
  `0x40..=0x7E` の**終端バイト**で終わる。終端がまだ来ていなければ未完成 (保持)。
- 終端の前に範囲外のバイト (制御文字など) が来たら、そこまで (範囲外バイトは含まない) を `Unknown` として消費し、
  範囲外バイトから解析を続ける。
- パラメータ文字列を `;` で区切った最初の数値を `p` とする (なければ無し)。**修飾キー (`;5` など) は無視する**。

| 終端 | Key |
|---|---|
| `A` `B` `C` `D` | Up, Down, Right, Left (例: `ESC[A`, `ESC[1;5A` どちらも Up) |
| `H` `F` | Home, End |
| `Z` | BackTab |
| `P` `Q` `R` `S` | F(1)..F(4) |
| `~` | `p` で決まる (下表) |
| その他 | `Unknown(シーケンス全体)` |

`~` の場合: `1`,`7`→Home, `2`→Insert, `3`→Delete, `4`,`8`→End, `5`→PageUp, `6`→PageDown,
`11..=15`→F(1)..F(5), `17..=21`→F(6)..F(10), `23`,`24`→F(11),F(12),
`200`→ブラケットペースト開始 (2.5), それ以外→`Unknown(シーケンス全体)`。

### 2.5 ブラケットペースト

`ESC[200~` から `ESC[201~` までの間のバイトは**キーとして解釈せず**、
`String::from_utf8_lossy` した文字列を `Paste(..)` として 1 つ返す (開始・終了シーケンス自体は含まない)。
終了シーケンスがまだ届いていなければ、開始シーケンスから全体を未完成として保持する。

## 3. `flush` (タイムアウト時に呼ぶ)

ESC 単独と「シーケンスの先頭」は、続きのバイトが来るまで区別できない。
呼び出し側は「一定時間 (例 50ms) 入力がなかったら `flush`」することで確定させる。

| 保持中のバイト | 返す Key | バッファ |
|---|---|---|
| (なし) | `[]` | 空のまま |
| `[ESC]` | `[Esc]` | 空にする |
| `[ESC, b'[']` | `[Alt('[')]` | 空にする |
| `[ESC, b'O']` | `[Alt('O')]` | 空にする |
| `ESC[200~` で始まる (ペースト途中) | `[]` | **保持したまま** |
| その他の未完成 | `[Unknown(保持中の全バイト)]` | 空にする |

## 4. 例

```text
parse(b"a\x1b[A\x03")             == [Char('a'), Up, Ctrl('c')]
parse("あ".as_bytes())            == [Char('あ')]
parse(b"\x1b")                    == [Esc]
parse(b"\x1bx")                   == [Alt('x')]
parse(b"\x1b[200~hi\x1b[A\x1b[201~") == [Paste("hi\x1b[A")]

let mut p = Parser::new();
p.feed(b"\x1b[")  == []          // 未完成
p.feed(b"5~")     == [PageUp]
```

## 5. `src/main.rs` (テスト対象外・starter に同梱済み)

starter の main.rs は完成している (読んで理解しよう)。lib.rs が出来上がれば、そのまま動く。

libc で raw モードにし (`tcgetattr` / `cfmakeraw` / `tcsetattr`)、stdin を `poll` で 50ms 待ちながら読む。
読めたら `feed`、タイムアウトで `has_pending()` なら `flush`。得られたキーを `{:?}` で 1 行ずつ表示し、
`Char('q')` か `Ctrl('c')` で終了。終了時 (パニック時も) は端末設定を必ず元に戻すこと。
`\x1b[?2004h` でブラケットペーストを有効化し、終了時に `\x1b[?2004l` で戻す。

## 合格条件

`cargo test -p f3_keyparser` がすべて成功すること。
完成したら `cargo run -p f3_keyparser` で実際のキーを押して確かめよう。
