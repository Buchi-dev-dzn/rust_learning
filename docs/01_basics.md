# 第1章 変数・型・関数・制御フロー

## この章のゴール

- `let` / `let mut` / シャドーイングの違いを説明できる
- 整数・浮動小数点・`bool`・`char`・タプル・配列といった基本の型を使い分けられる
- 引数と戻り値を持つ関数を書け、「式」と「文」の違いがわかる
- `if` / `loop` / `while` / `for` / `match` の基本形を書ける
- コンパイルエラーのメッセージを落ち着いて読める

Rust は「コンパイラと会話しながら書く」言語です。最初のうちはエラーが多く出ますが、
メッセージはとても親切で、多くの場合「こう直せばよい」まで教えてくれます。
この章では、それを読むための語彙をそろえます。

## Hello, world とプログラムの形

```rust
fn main() {
    println!("Hello, world!");
    let name = "Rust";
    println!("こんにちは、{name}!");
    println!("{} + {} = {}", 1, 2, 1 + 2);
}
```

- プログラムは `fn main()` から始まります。
- `println!` の `!` は **マクロ** であることを示します。関数ではないので、可変個の引数をとれます。
- `{}` に値が埋め込まれます。`{name}` のように変数名を直接書くこともできます。
- デバッグ表示は `{:?}`、整形付きは `{:#?}` です。

後で TUI を作るときも、最終的には「文字列を組み立ててターミナルに書く」ことの繰り返しです。
`format!` / `print!` の書式指定はずっと使います。

```rust
fn main() {
    let width = 8;
    // 右寄せ・左寄せ・中央寄せ・ゼロ埋め
    println!("[{:>8}]", "right");
    println!("[{:<8}]", "left");
    println!("[{:^8}]", "mid");
    println!("[{:08}]", 42);
    // 幅を変数で指定する
    println!("[{:>width$}]", "w");
    // 小数点以下の桁数
    println!("{:.2}", 3.14159);
    // 文字列として作るだけなら format!
    let line = format!("{:-<10}", "");
    println!("{line}");
}
```

## 変数とミュータビリティ

Rust の変数は **デフォルトで不変 (immutable)** です。

```rust
fn main() {
    let x = 5;
    println!("x = {x}");
    let mut y = 10; // mut を付けると変更できる
    y += 1;
    println!("y = {y}");
}
```

`mut` なしで再代入するとコンパイルエラーになります。

```rust
fn demo() {
    let x = 5;
    x = 6; // error[E0384]: cannot assign twice to immutable variable `x`
}
```

エラーメッセージには `help: consider making this binding mutable: mut x` のように
修正案が出ます。「不変がデフォルト」なのは、どこで値が変わるかを読み手が一目で追えるようにするためです。

### シャドーイング

同じ名前で `let` し直すと、新しい変数で前の変数を **覆い隠す (shadowing)** ことができます。
型を変えてもかまいません。

```rust
fn main() {
    let input = "  42  ";
    let input = input.trim();          // &str のまま
    let input: i32 = input.parse().unwrap(); // i32 に変わる
    println!("{}", input * 2);
}
```

`mut` は「同じ変数の値を変える」、シャドーイングは「新しい変数を作る」です。
入力文字列を段階的に変換するときによく使います。

### 定数

```rust
const MAX_WIDTH: u16 = 80;

fn main() {
    println!("最大幅は {MAX_WIDTH}");
}
```

`const` は型注釈が必須で、名前は大文字スネークケースにします。

## 基本の型

| 種類 | 型 | 例 |
|------|----|----|
| 符号付き整数 | `i8` `i16` `i32` `i64` `i128` `isize` | `-3`, `42i64` |
| 符号なし整数 | `u8` `u16` `u32` `u64` `u128` `usize` | `255u8` |
| 浮動小数点 | `f32` `f64` | `1.5` |
| 真偽値 | `bool` | `true` |
| 文字 | `char` (Unicode スカラー値, 4 バイト) | `'あ'`, `'\n'` |
| タプル | `(T, U, ...)` | `(1, 'a')` |
| 配列 | `[T; N]` (長さ固定) | `[0; 4]` |

- 整数リテラルの既定は `i32`、小数の既定は `f64` です。
- **添字や長さは `usize`** です。`vec[i]` の `i` は `usize` でなければなりません。
- ターミナルの座標やサイズは慣習的に `u16` を使います (crossterm や ratatui もそうです)。

```rust
fn main() {
    let cols: u16 = 80;
    let rows: u16 = 24;
    let cells = cols as usize * rows as usize; // as で型変換
    println!("{cols}x{rows} = {cells} セル");

    let pos = (3, 5);            // タプル
    let (x, y) = pos;            // 分解
    println!("x={x} y={y} / pos.0={}", pos.0);

    let palette = [30, 31, 32, 33]; // 配列
    println!("先頭 {} / 長さ {}", palette[0], palette.len());

    let c = 'あ';
    println!("{c} は {} バイト (UTF-8)", c.len_utf8());
}
```

### 型変換は明示的に

Rust は異なる整数型を **自動では変換しません**。`u16 + usize` はコンパイルエラーです。

```rust
fn demo() {
    let a: u16 = 10;
    let b: usize = 20;
    let c = a + b; // error[E0308]: mismatched types
}
```

`as` で変換するか、`usize::from(a)` のような変換関数を使います。
`as` は大きい型から小さい型へ変換すると黙って切り捨てるので注意してください。

### 整数のオーバーフロー

デバッグビルドでは整数のオーバーフローは **パニック** (実行時エラー) になります。
意図的に扱いたい場合は専用メソッドを使います。

```rust
fn main() {
    let x: u8 = 250;
    println!("{:?}", x.checked_add(10));   // None (溢れたら None)
    println!("{}", x.saturating_add(10));  // 255 (上限で止まる)
    println!("{}", x.wrapping_add(10));    // 4 (一周する)
    let col: u16 = 0;
    println!("{}", col.saturating_sub(1)); // 0 (カーソルを左に動かすときに便利)
}
```

`saturating_sub` は「カーソルを左へ。ただし 0 より左には行かない」のような
TUI の座標計算で頻出します。

## 関数

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b // セミコロンなし = この値を返す
}

fn greet(name: &str) {
    println!("やあ、{name}");
}

fn main() {
    println!("{}", add(2, 3));
    greet("Ferris");
}
```

- 引数には **必ず型注釈** が要ります。
- 戻り値の型は `-> 型` で書きます。書かなければ `()` (ユニット型、「何もない」) を返します。
- 関数の最後の **式** がそのまま戻り値になります。途中で返すときは `return x;` を使います。

### 式と文

Rust ではほとんどのものが **式 (値を持つ)** です。`if` もブロック `{}` も値を持ちます。

```rust
fn main() {
    let n = 7;
    let kind = if n % 2 == 0 { "偶数" } else { "奇数" };
    let squared = {
        let t = n * n;
        t + 0 // ブロックの最後の式がブロックの値
    };
    println!("{n} は{kind}、2 乗は {squared}");
}
```

最後の式に `;` を付けると、それは「文」になり値は `()` になります。
次のエラーはこの典型です。

```rust
fn double(x: i32) -> i32 {
    x * 2; // error[E0308]: mismatched types ... expected `i32`, found `()`
}
```

コンパイラは `help: remove this semicolon to return this value` と教えてくれます。

## 制御フロー

### if

条件は `bool` でなければなりません (`if 1 { }` は書けません)。括弧は不要です。

### loop / while / for

```rust
fn main() {
    // loop: 無限ループ。break で値を返せる
    let mut count = 0;
    let result = loop {
        count += 1;
        if count == 5 {
            break count * 10;
        }
    };
    println!("result = {result}");

    // while
    let mut n = 3;
    while n > 0 {
        print!("{n} ");
        n -= 1;
    }
    println!("発射!");

    // for + 範囲 (0..5 は 0〜4、0..=5 は 0〜5)
    for i in 0..5 {
        print!("{i} ");
    }
    println!();

    // 配列を順に
    for c in ['a', 'b', 'c'] {
        print!("{c}");
    }
    println!();

    // 逆順・ステップ
    for i in (0..10).rev().step_by(3) {
        print!("{i} ");
    }
    println!();
}
```

TUI アプリの心臓部は「イベントを待つ → 状態を更新 → 描画」を延々と繰り返す
`loop` です。終了キーが押されたら `break` する、という形は第20章でそのまま使います。

ネストしたループから一気に抜けたいときはラベルを付けます。

```rust
fn main() {
    'outer: for y in 0..5 {
        for x in 0..5 {
            if x * y == 6 {
                println!("見つけた: ({x}, {y})");
                break 'outer;
            }
        }
    }
}
```

### match (さわりだけ)

`match` は値をパターンで振り分けます。**すべての場合を網羅しないとコンパイルエラー** になります。
詳しくは第5章で扱います。

```rust
fn describe(n: i32) -> &'static str {
    match n {
        0 => "ゼロ",
        1 | 2 | 3 => "小さい",
        4..=9 => "一桁",
        _ if n < 0 => "負",
        _ => "大きい",
    }
}

fn main() {
    for n in [0, 2, 7, -1, 100] {
        println!("{n}: {}", describe(n));
    }
}
```

## 小さなプログラム: テキストの箱を描く

ここまでの知識で、ターミナルに枠を描いてみましょう。後の TUI で描く「ボーダー付きの箱」の原型です。

```rust
fn draw_box(width: usize, height: usize) {
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let ch = match (x, y) {
                (0, 0) => '┌',
                (x, 0) if x == width - 1 => '┐',
                (0, y) if y == height - 1 => '└',
                (x, y) if x == width - 1 && y == height - 1 => '┘',
                (_, 0) => '─',
                (_, y) if y == height - 1 => '─',
                (0, _) => '│',
                (x, _) if x == width - 1 => '│',
                _ => ' ',
            };
            line.push(ch);
        }
        println!("{line}");
    }
}

fn main() {
    draw_box(12, 4);
}
```

## よくあるコンパイルエラーと読み方

Rust のエラーは次の形をしています。

```text
error[E0384]: cannot assign twice to immutable variable `x`
 --> src/main.rs:3:5
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut x = 5;
  |         +++
```

1. 1 行目: エラーコード (`E0384`) と要約。`rustc --explain E0384` で詳しい説明が読めます。
2. `-->`: ファイル名:行:列。
3. `^^^` と `-`: 問題の場所と、それに関係する場所。
4. `help:`: 修正案。まずここを読みましょう。

この章で出会いやすいもの:

| コード | 意味 | よくある原因 |
|--------|------|--------------|
| E0384 | 不変変数への再代入 | `mut` の付け忘れ |
| E0308 | 型が合わない | 最後の式に `;`、整数型の混在 |
| E0425 | 名前が見つからない | タイプミス、スコープ外 |
| E0004 | match が網羅的でない | `_` の書き忘れ |

## まとめ

- 変数は既定で不変。変えたいなら `mut`、型や意味を変えるならシャドーイング。
- 整数型は自動変換されない。添字は `usize`、端末座標は `u16` が慣習。
- `saturating_sub` などで溢れを明示的に扱う。
- 関数の最後の式が戻り値。`;` を付けると `()` になる。
- `if` もブロックも式。`loop` は `break 値` で値を返せる。

## 次へ

演習で `mut`・関数・ループを手に馴染ませたら、第2章で Rust 最大の特徴である **所有権** に進みます。
