# F4: 差分レンダラ

TUI ライブラリの心臓部である「**セルバッファ + 差分描画**」を作る。
画面全体をセル (文字 + スタイル) の 2 次元配列として持ち、前のフレームと比べて
**変わったセルだけ**を、なるべく短い ANSI エスケープシーケンスで出力する。

この課題で身につくこと: セルバッファの設計、全角文字 (2 セル幅) の扱い、
カーソル移動 (CUP) と SGR (色・太字) の最小化、出力を**文字列として**テストする方法。

外部クレートは `crossterm` のみ (main.rs のデモで raw モード・端末サイズ取得に使う)。
`src/lib.rs` は標準ライブラリだけで書く。テストは**生成された文字列を完全一致で比較する**ので、
下の規則を正確に守ること。

## 1. 型 (`src/lib.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color { #[default] Reset, Black, Red, Green, Yellow, Blue, Magenta, Cyan, White, Indexed(u8) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style { pub fg: Color, pub bg: Color, pub bold: bool }

impl Style {
    pub fn new() -> Style;                 // = Default
    pub fn fg(self, c: Color) -> Style;    // ビルダー: fg を変えたコピー
    pub fn bg(self, c: Color) -> Style;
    pub fn bold(self) -> Style;            // bold = true にしたコピー
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell { pub ch: char, pub style: Style }

impl Cell {
    pub const CONTINUATION_CHAR: char = '\0';  // 全角文字の右半分の印
    pub fn blank() -> Cell;                    // ' ' + Style::default()
    pub fn is_continuation(&self) -> bool;     // ch == CONTINUATION_CHAR
}
impl Default for Cell { /* blank() */ }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer { /* 自由 */ }

#[derive(Debug, Default)]
pub struct Renderer { /* 自由 */ }
```

## 2. 表示幅 `pub fn char_width(c: char) -> usize`

F2 と同じ規則:
`< 0x20`, `0x7F..=0x9F`, `0x0300..=0x036F`, `0x200B` → 0。
全角範囲 `1100-115F, 2E80-303E, 3041-33FF, 3400-4DBF, 4E00-9FFF, A000-A4CF, AC00-D7A3,
F900-FAFF, FE30-FE4F, FF00-FF60, FFE0-FFE6, 1F300-1F64F, 1F900-1F9FF, 20000-3FFFD` → 2。それ以外 → 1。

## 3. `Buffer`

```rust
pub fn new(width: usize, height: usize) -> Buffer;  // 全セル blank
pub fn width(&self) -> usize;
pub fn height(&self) -> usize;
pub fn get(&self, x: usize, y: usize) -> Cell;      // 範囲外はパニックしてよい
pub fn set(&mut self, x: usize, y: usize, ch: char, style: Style);
pub fn set_str(&mut self, x: usize, y: usize, s: &str, style: Style) -> usize;
pub fn clear(&mut self);                            // 全セル blank に
pub fn lines(&self) -> Vec<String>;                 // 各行の文字 (継続セルは飛ばす)
```

### `set(x, y, ch, style)` の規則

1. `(x, y)` が範囲外、または `char_width(ch) == 0` なら何もしない。
2. **書き込む前に、壊れる全角文字を片付ける**。セル `x` について:
   - `x` が継続セルなら、その左 `x-1` (全角の左半分) の `ch` を `' '` にする (スタイルはそのまま)。
   - `x+1` が継続セルなら (= `x` が全角の左半分)、`x+1` の `ch` を `' '` にする (スタイルはそのまま)。
3. 幅 1 の文字: セル `x` を `Cell { ch, style }` にする。
4. 幅 2 の文字:
   - `x + 1 >= width` (右端に入らない) なら、セル `x` を `Cell { ch: ' ', style }` にして終わり。
   - そうでなければ、セル `x+1` についても 2. の片付けを行ってから、
     セル `x` = `Cell { ch, style }`、セル `x+1` = `Cell { ch: CONTINUATION_CHAR, style }`。

### `set_str(x, y, s, style)`

`s` の文字を左から `set` していく。幅 0 の文字は飛ばす。書き込み位置が `width` 以上になったら打ち切る。
次の文字を書く位置 (= 書いた幅の合計を足した x、ただし `width` を上限とする) を返す。

## 4. SGR `pub fn sgr(style: &Style) -> String`

**常にリセットから始めて全属性を指定する**形式:

```
"\x1b[0" + (bold なら ";1") + (fg の指定) + (bg の指定) + "m"
```

| Color | fg の指定 | bg の指定 |
|---|---|---|
| Reset | (なし) | (なし) |
| Black, Red, Green, Yellow, Blue, Magenta, Cyan, White | `;30` 〜 `;37` | `;40` 〜 `;47` |
| Indexed(n) | `;38;5;n` | `;48;5;n` |

例: `sgr(&Style::default())` → `"\x1b[0m"`、太字・赤・背景 Indexed(236) → `"\x1b[0;1;31;48;5;236m"`

## 5. 差分 `pub fn diff(prev: &Buffer, next: &Buffer) -> String`

`prev` が今の画面、`next` が描きたい画面。`next` にするための出力を返す。

1. **サイズが違う場合**: 出力の先頭に `"\x1b[2J"` (画面消去) を付け、以降は
   `prev` の代わりに `Buffer::new(next.width(), next.height())` (全部 blank) と比較する。
2. 状態: 「現在のカーソル位置 = 不明」「現在のスタイル = `Style::default()`」から始める。
3. セルを **上の行から、各行は左から** 順に調べる。`next` のセルが継続セルなら飛ばす。
   `next` のセルが `prev` の同じ位置のセルと**等しければ飛ばす** (`Cell` の `==`)。
4. 違うセルを書くとき:
   1. カーソル位置が `(x, y)` でなければ `"\x1b[{y+1};{x+1}H"` を出力 (1 始まり)。
   2. セルのスタイルが現在のスタイルと違えば `sgr(&style)` を出力し、現在のスタイルを更新。
   3. 文字 `ch` を出力。
   4. カーソル位置を `(x + char_width(ch), y)` に更新。ただしそれが `width` 以上なら「不明」にする
      (端末は右端で折り返し待ち状態になり、位置があてにならない)。
5. 最後に、現在のスタイルが `Style::default()` でなければ `"\x1b[0m"` を出力。
6. 変更がなければ空文字列 (サイズ違いなら `"\x1b[2J"` + 必要な分)。カーソルを最後に移動したりはしない。

例 (5x2、`prev` は空、`next` は (1,0) から太字赤で `"hi"`):

```
"\x1b[1;2H" + "\x1b[0;1;31m" + "hi" + "\x1b[0m"
```

## 6. `Renderer`

```rust
pub fn new() -> Renderer;
pub fn render(&mut self, next: &Buffer) -> String;
pub fn invalidate(&mut self);
```

- 前回 `render` したバッファを覚えておき、`render` は `diff(前回, next)` を返して `next` のコピーを保存する。
- 最初の `render` と `invalidate()` 直後の `render` は「前回 = `Buffer::new(0, 0)`」として扱う
  (つまりサイズ違い → `\x1b[2J` + 空でないセルの描画)。

## 7. `src/main.rs` (テスト対象外・starter に同梱済み)

crossterm で raw モード + 代替画面にし、`Renderer` でアニメーションを描くデモ。
q / Esc / Ctrl-C で終了。パニック時も端末を元に戻す。lib.rs が出来上がれば `cargo run -p f4_diff_renderer` で動く。

## 合格条件

`cargo test -p f4_diff_renderer` がすべて成功すること。
