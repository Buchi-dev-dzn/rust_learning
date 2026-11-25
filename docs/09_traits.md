# 第9章 トレイトとジェネリクス

## この章のゴール

- トレイト (trait) で「型が持つべき振る舞い」を定義し、構造体に実装できる
- ジェネリクス (`<T>`) とトレイト境界 (`T: Display`) で、型に依存しない関数を書ける
- `Box<dyn Trait>` (トレイトオブジェクト) で、**違う型を 1 つの `Vec` にまとめて扱える**
- `Display`・`Default`・`From` など標準ライブラリの代表的なトレイトを実装できる

TUI ライブラリ (ratatui や、第22章で作る自作フレームワーク) の中心にあるのは
「画面に描けるもの = `Widget` トレイトを実装した型」という考え方です。
ボタンもラベルもリストも「`render` できる」という共通点だけで扱われます。
この章はその土台です。

## トレイトとは

トレイトは「この型はこういうメソッドを持っている」という約束です。
Java/C# のインターフェース、Go の interface、TypeScript の interface に近いものです。

```rust
trait Describe {
    // 実装側が必ず書くメソッド
    fn name(&self) -> String;

    // デフォルト実装付きのメソッド (上書きしてもよい)
    fn describe(&self) -> String {
        format!("これは {} です", self.name())
    }
}

struct Button {
    label: String,
}

struct Spacer;

impl Describe for Button {
    fn name(&self) -> String {
        format!("ボタン「{}」", self.label)
    }
}

impl Describe for Spacer {
    fn name(&self) -> String {
        "スペーサー".to_string()
    }
    // describe を上書き
    fn describe(&self) -> String {
        "(空白)".to_string()
    }
}

fn main() {
    let b = Button { label: "OK".to_string() };
    let s = Spacer;
    println!("{}", b.describe());
    println!("{}", s.describe());
}
```

ポイント:

- `trait 名前 { ... }` でメソッドのシグネチャを並べる
- `impl トレイト for 型 { ... }` で実装する
- 本体付きのメソッドは **デフォルト実装**。実装側で書かなければそれが使われる

Go と違い、Rust のトレイト実装は **明示的** です。メソッド名が一致しているだけでは実装したことになりません。

## ジェネリクスとトレイト境界

「`Describe` を実装している型なら何でも受け取る関数」は、ジェネリクスで書けます。

```rust
use std::fmt::Display;

// T は Display を実装している型なら何でもよい
fn boxed<T: Display>(value: T) -> String {
    let inner = format!(" {} ", value);
    let line = "-".repeat(inner.chars().count());
    format!("+{line}+\n|{inner}|\n+{line}+")
}

// 最大値を返す。比較できて (PartialOrd)、コピーできる (Copy) 型に限る
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter();
    let mut max = *iter.next()?;
    for &x in iter {
        if x > max {
            max = x;
        }
    }
    Some(max)
}

fn main() {
    println!("{}", boxed("hello"));
    println!("{}", boxed(42));
    println!("{:?}", largest(&[3, 9, 2]));
    println!("{:?}", largest(&[1.5, -2.0]));
    println!("{:?}", largest::<i32>(&[]));
}
```

- `<T: Display>` が **トレイト境界**。「T は Display を実装していること」という条件
- 複数の境界は `+` でつなぐ: `T: PartialOrd + Copy`
- 境界が長くなったら `where` 句に分けられる

```rust
use std::fmt::Debug;

fn show_all<T>(items: &[T]) -> String
where
    T: Debug,
{
    items.iter().map(|x| format!("{:?}", x)).collect::<Vec<_>>().join(", ")
}

fn main() {
    println!("{}", show_all(&["a", "b"]));
}
```

引数位置では `impl Trait` という短い書き方もできます。意味は `<T: Trait>` とほぼ同じです。

```rust
use std::fmt::Display;

fn print_twice(x: impl Display) {
    println!("{x} {x}");
}

fn main() {
    print_twice("やあ");
    print_twice(3.14);
}
```

### ジェネリックな構造体

構造体もジェネリクスにできます。`impl<T> ...` の `<T>` を忘れないこと。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pos<T> {
    x: T,
    y: T,
}

impl<T: std::ops::Add<Output = T> + Copy> Pos<T> {
    fn offset(self, dx: T, dy: T) -> Pos<T> {
        Pos { x: self.x + dx, y: self.y + dy }
    }
}

fn main() {
    let p = Pos { x: 1u16, y: 2u16 };
    println!("{:?}", p.offset(3, 4));
    let q = Pos { x: 0.5, y: 1.0 };
    println!("{:?}", q.offset(0.5, 0.5));
}
```

ジェネリクスはコンパイル時に型ごとの専用コードが生成されます (**単相化**)。
実行時のコストはゼロですが、「1 つの Vec に違う型を混ぜる」ことはできません。そこで次の仕組みが必要になります。

## トレイトオブジェクト `dyn Trait`

TUI の画面は「ラベル」「プログレスバー」「枠線」など **違う型の部品の集まり** です。
`Vec<Label>` では `Bar` を入れられません。そこで `Box<dyn Widget>` を使います。

```rust
// 描画先: 文字の 2 次元配列
struct Buffer {
    width: usize,
    cells: Vec<Vec<char>>,
}

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        Buffer { width, cells: vec![vec![' '; width]; height] }
    }
    fn set(&mut self, x: usize, y: usize, c: char) {
        if y < self.cells.len() && x < self.width {
            self.cells[y][x] = c;
        }
    }
    fn to_string(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: usize,
    y: usize,
    width: usize,
}

trait Widget {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

struct Label(String);
struct Bar {
    ratio: f64,
}

impl Widget for Label {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        for (i, c) in self.0.chars().take(area.width).enumerate() {
            buf.set(area.x + i, area.y, c);
        }
    }
}

impl Widget for Bar {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let filled = (area.width as f64 * self.ratio).round() as usize;
        for i in 0..area.width {
            buf.set(area.x + i, area.y, if i < filled { '#' } else { '.' });
        }
    }
}

fn main() {
    // 違う型を 1 つの Vec に入れられる
    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Label("Downloading...".to_string())),
        Box::new(Bar { ratio: 0.4 }),
        Box::new(Label("q: quit".to_string())),
    ];

    let mut buf = Buffer::new(20, 3);
    for (row, w) in widgets.iter().enumerate() {
        w.render(Rect { x: 0, y: row, width: 20 }, &mut buf);
    }
    println!("{}", buf.to_string());
}
```

- `dyn Widget` は「Widget を実装した何かの型 (コンパイル時には不明)」
- サイズが決まらないので `Box` (ヒープ) や `&` (参照) 越しに扱う: `Box<dyn Widget>`, `&dyn Widget`
- メソッド呼び出しは実行時に vtable 経由で解決される (**動的ディスパッチ**)

### ジェネリクスと dyn の使い分け

| | ジェネリクス `<T: Widget>` | トレイトオブジェクト `dyn Widget` |
|---|---|---|
| 解決 | コンパイル時 | 実行時 |
| 速度 | 最速 (インライン化可) | わずかに遅い (間接呼び出し) |
| 違う型を混ぜる | できない | できる |
| 典型用途 | `fn draw<W: Widget>(w: W)` | `Vec<Box<dyn Widget>>` |

ratatui では `frame.render_widget(widget, area)` がジェネリクス、
自作フレームワークの「子ウィジェットの一覧」は `Vec<Box<dyn Widget>>` になります。両方使えるようになりましょう。

### dyn にできないトレイト

トレイトオブジェクトにするには、トレイトが **dyn 互換 (object safe)** である必要があります。
代表的な制限は「メソッドがジェネリクスを持たない」「`Self` を値で返さない」です。
`Clone` は `fn clone(&self) -> Self` を持つので `dyn Clone` にはできません。

## 標準ライブラリのトレイトを実装する

### `Display` — `{}` で表示

```rust
use std::fmt;

struct Size {
    w: u16,
    h: u16,
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.w, self.h)
    }
}

fn main() {
    let s = Size { w: 80, h: 24 };
    println!("端末サイズ: {s}");
    let text: String = s.to_string(); // Display を実装すると to_string() も使える
    println!("{}", text.len());
}
```

### `derive` で自動実装

`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `PartialOrd`, `Ord` は
`#[derive(...)]` で自動実装できます。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Color {
    #[default]
    Reset,
    Red,
    Green,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Style {
    fg: Color,
    bold: bool,
}

fn main() {
    let s = Style::default();
    println!("{:?}", s);
    let t = Style { fg: Color::Red, ..s.clone() };
    println!("{}", s == t);
}
```

### `From` / `Into` — 型変換

```rust
struct Line(String);

impl From<&str> for Line {
    fn from(s: &str) -> Self {
        Line(s.to_string())
    }
}

fn show(line: impl Into<Line>) {
    let line: Line = line.into();
    println!("[{}]", line.0);
}

fn main() {
    let a = Line::from("abc");
    println!("{}", a.0);
    show("xyz"); // From を実装すると Into も自動で使える
}
```

ratatui の `Line::from("text")` や `Span::from(...)` もこの仕組みです。

### 演算子のオーバーロード

`std::ops::Add` などを実装すると `+` が使えます。座標計算で便利です。

```rust
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Add for Pos {
    type Output = Pos; // 関連型: 演算結果の型
    fn add(self, o: Pos) -> Pos {
        Pos { x: self.x + o.x, y: self.y + o.y }
    }
}

fn main() {
    let cursor = Pos { x: 3, y: 1 } + Pos { x: 1, y: 0 };
    println!("{:?}", cursor);
}
```

`type Output = Pos;` のようにトレイト内で宣言する型を **関連型** と呼びます。
第11章の `Iterator` トレイトでも `type Item` が出てきます。

## よくあるコンパイルエラーと読み方

### E0277: トレイト境界を満たしていない

```rust
fn show<T>(x: T) {
    println!("{}", x);
}
```

```text
error[E0277]: `T` doesn't implement `std::fmt::Display`
help: consider restricting type parameter `T` with trait `Display`
  |
1 | fn show<T: std::fmt::Display>(x: T) {
```

`T` が何でもあり得るので `{}` で表示できる保証がない、という意味です。
コンパイラの `help` の通りに境界を足せば直ります。

### E0046: メソッドの実装漏れ

```text
error[E0046]: not all trait items implemented, missing: `render`
```

`impl Widget for X` の中で、デフォルト実装のないメソッドを書き忘れています。

### E0038: dyn 互換でない

```text
error[E0038]: the trait `Clone` is not dyn compatible
```

`Box<dyn Clone>` のように、トレイトオブジェクトにできないトレイトを使おうとしています。
設計を見直し、`dyn` にしたいトレイトからジェネリックなメソッドや `-> Self` を外しましょう。

### `Box<dyn Widget>` にしたいのに型が合わない

```text
error[E0308]: mismatched types
expected `Box<dyn Widget>`, found `Box<Label>`
```

`vec![...]` の要素型を推論させると最初の要素の型 (`Box<Label>`) に決まってしまうことがあります。
`let widgets: Vec<Box<dyn Widget>> = ...` と型を明示しましょう。

## まとめ

- トレイト = 振る舞いの約束。`impl Trait for Type` で明示的に実装する
- ジェネリクス + トレイト境界で、型に依存しない関数・構造体を書ける (静的ディスパッチ)
- `Box<dyn Trait>` で違う型をまとめて扱える (動的ディスパッチ)。TUI の部品リストの基本形
- `Display`, `Default`, `From`, `Add` などを実装すると、Rust の書き味に自然に溶け込む型になる

## 次へ

次の第10章では **ライフタイム** を学びます。
`&str` を返す関数や、`&str` を持つ構造体 (トークナイザなど) を書くときに必要になる知識です。
演習では `Widget` トレイトを実際に実装して、画面バッファに描画してみましょう。
