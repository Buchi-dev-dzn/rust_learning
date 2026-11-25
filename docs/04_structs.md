# 第4章 構造体とメソッド

## この章のゴール

- 名前付きフィールドの構造体・タプル構造体・ユニット構造体を定義できる
- `impl` ブロックでメソッドと関連関数 (`new` など) を書ける
- `self` / `&self` / `&mut self` の違いを、所有権と借用の言葉で説明できる
- `#[derive(Debug, Clone, Copy, PartialEq, Default)]` を使い分けられる
- TUI の基本部品である `Pos` / `Size` / `Rect` / `Cursor` を自分で設計できる

## 構造体の定義と作成

```rust
#[derive(Debug)]
struct Window {
    title: String,
    width: u16,
    height: u16,
    visible: bool,
}

fn main() {
    let mut w = Window {
        title: String::from("main"),
        width: 80,
        height: 24,
        visible: true,
    };
    w.width = 100; // 変数が mut ならフィールドを書き換えられる
    println!("{} {}x{} visible={}", w.title, w.width, w.height, w.visible);
    println!("{w:?}");   // Debug 表示 (derive(Debug) が必要)
    println!("{w:#?}");  // 整形した Debug 表示
}
```

- フィールドごとに `mut` を付けることはできません。変更可否は **変数 (束縛) 単位** です。
- `#[derive(Debug)]` を付けると `{:?}` で表示できるようになります。

### 省略記法と構造体更新記法

```rust
#[derive(Debug)]
struct Style {
    fg: u8,
    bg: u8,
    bold: bool,
    underline: bool,
}

fn new_style(fg: u8, bg: u8) -> Style {
    // 変数名とフィールド名が同じなら `fg: fg` を `fg` と書ける
    Style { fg, bg, bold: false, underline: false }
}

fn main() {
    let base = new_style(7, 0);
    // ..base で「残りは base と同じ」
    let title = Style { bold: true, ..base };
    println!("{title:?}");
}
```

構造体更新記法 `..base` は、`Copy` でないフィールドがあると **base からムーブ** します。
`String` を含む構造体で使った後に `base` が使えなくなるのはそのためです。

## タプル構造体とユニット構造体

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rgb(u8, u8, u8); // フィールドに名前がない

#[derive(Debug)]
struct Quit; // フィールドが 1 つもない

fn main() {
    let red = Rgb(255, 0, 0);
    println!("r={} g={} b={}", red.0, red.1, red.2);
    let Rgb(r, g, b) = red; // 分解
    println!("{r},{g},{b} / {:?}", Quit);
}
```

タプル構造体は「同じ `u16` でも意味が違う」ものを型で区別したいときにも使えます
(ニュータイプパターン)。例: `struct Col(u16); struct Row(u16);` とすれば、行と列を取り違えると
コンパイルエラーになります。

## メソッドと impl

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Size {
    width: u16,
    height: u16,
}

impl Size {
    // 関連関数 (self を取らない)。Size::new(..) で呼ぶ。コンストラクタの慣習名は new
    fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    // &self: 読むだけのメソッド
    fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    // &mut self: 自分を書き換えるメソッド
    fn shrink(&mut self, n: u16) {
        self.width = self.width.saturating_sub(n);
        self.height = self.height.saturating_sub(n);
    }

    // self: 自分を消費して新しい値を返すメソッド
    fn transposed(self) -> Size {
        Size { width: self.height, height: self.width }
    }
}

fn main() {
    let mut s = Size::new(80, 24);
    println!("面積 {}", s.area());
    s.shrink(2);
    println!("{s:?}");
    let t = s.transposed();
    println!("{t:?}");
}
```

| レシーバ | 意味 | 呼び出し元の条件 |
|----------|------|------------------|
| `&self` | 共有借用。読むだけ | なし |
| `&mut self` | 可変借用。自分を書き換える | 変数が `mut` |
| `self` | 所有権を受け取る (消費する) | 呼んだ後は元の変数は使えない (Copy なら使える) |

- `Self` は「この impl の型」の別名です。
- `s.area()` は実際には `Size::area(&s)` の糖衣構文です。Rust は必要に応じて自動で `&` や `&mut` を付けてくれます (自動参照)。
- `impl` ブロックは複数に分けて書いてもかまいません。

### ビルダー風のメソッドチェーン

`self` を受け取って `Self` を返すメソッドを並べると、設定を連鎖して書けます。
ratatui の `Block::default().title("x").borders(Borders::ALL)` はこの形です。

```rust
#[derive(Debug, Default)]
struct Block {
    title: String,
    border: bool,
    padding: u16,
}

impl Block {
    fn title(mut self, t: &str) -> Self {
        self.title = t.to_string();
        self
    }
    fn border(mut self, on: bool) -> Self {
        self.border = on;
        self
    }
    fn padding(mut self, p: u16) -> Self {
        self.padding = p;
        self
    }
}

fn main() {
    let b = Block::default().title("Files").border(true).padding(1);
    println!("{b:?}");
}
```

`#[derive(Default)]` は、各フィールドの既定値 (`0`, `false`, 空文字列…) で作る `Block::default()` を自動生成します。

## derive でよく使うトレイト

| derive | できるようになること | 条件 |
|--------|----------------------|------|
| `Debug` | `{:?}` で表示 | 全フィールドが Debug |
| `Clone` | `.clone()` で複製 | 全フィールドが Clone |
| `Copy` | 代入でコピー (ムーブしない) | 全フィールドが Copy、かつ Clone も derive |
| `PartialEq` / `Eq` | `==` で比較 | 全フィールドが PartialEq |
| `Default` | `T::default()` | 全フィールドが Default |
| `Hash` | `HashMap` のキーにできる | 全フィールドが Hash (Eq も必要) |

座標やサイズのような小さな値型には `Debug, Clone, Copy, PartialEq, Eq, Default` を
まとめて付けるのが定番です。テストの `assert_eq!` には `Debug` と `PartialEq` が必要です。

## 実例: Rect

TUI のレイアウトはすべて「長方形 (Rect) を分割していく」ことで行います。
ratatui にも同じ名前の型があります。自分で作ってみましょう。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }
    fn right(&self) -> u16 {
        self.x + self.width // 右端の 1 つ外側
    }
    fn bottom(&self) -> u16 {
        self.y + self.height
    }
    fn contains(&self, px: u16, py: u16) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
    /// 周囲を margin だけ縮めた内側の Rect
    fn inner(&self, margin: u16) -> Rect {
        let w = self.width.saturating_sub(margin * 2);
        let h = self.height.saturating_sub(margin * 2);
        Rect::new(self.x + margin, self.y + margin, w, h)
    }
    /// 上から height 行を切り出し、(上, 残り) を返す
    fn split_top(&self, height: u16) -> (Rect, Rect) {
        let h = height.min(self.height);
        let top = Rect::new(self.x, self.y, self.width, h);
        let rest = Rect::new(self.x, self.y + h, self.width, self.height - h);
        (top, rest)
    }
}

fn main() {
    let screen = Rect::new(0, 0, 80, 24);
    let (header, body) = screen.split_top(1);
    let content = body.inner(1);
    println!("header  = {header:?}");
    println!("body    = {body:?}");
    println!("content = {content:?}");
    println!("(5,5) は content 内? {}", content.contains(5, 5));
    println!("(0,1) は content 内? {}", content.contains(0, 1));
}
```

`Rect` は `Copy` なので、`split_top` などに `&self` で渡しても、値で返しても気軽に扱えます。

## 構造体と所有権

`String` や `Vec` をフィールドに持つ構造体は、それらを **所有** します。
構造体が drop されると中身も一緒に drop されます。

```rust
#[derive(Debug)]
struct StatusLine {
    left: String,
    right: String,
}

impl StatusLine {
    fn new() -> Self {
        Self { left: String::new(), right: String::new() }
    }
    fn set_left(&mut self, s: &str) {
        self.left.clear();
        self.left.push_str(s);
    }
    /// 幅 width に収めて 1 行の文字列にする (ASCII 前提)
    fn render(&self, width: usize) -> String {
        let gap = width.saturating_sub(self.left.len() + self.right.len());
        format!("{}{}{}", self.left, " ".repeat(gap), self.right)
    }
}

fn main() {
    let mut st = StatusLine::new();
    st.set_left("NORMAL main.rs");
    st.right = String::from("12:4");
    println!("[{}]", st.render(30));
}
```

### メソッドからフィールドへの参照を返す

```rust
struct Editor {
    lines: Vec<String>,
}

impl Editor {
    fn line(&self, i: usize) -> &str {
        &self.lines[i] // self から借りた参照を返す
    }
    fn line_mut(&mut self, i: usize) -> &mut String {
        &mut self.lines[i]
    }
}

fn main() {
    let mut ed = Editor { lines: vec![String::from("hello")] };
    ed.line_mut(0).push_str(", world");
    println!("{}", ed.line(0));
}
```

## よくあるコンパイルエラーと読み方

| コード | 状況 | 直し方 |
|--------|------|--------|
| E0277 `doesn't implement Debug` | `{:?}` で表示しようとした | `#[derive(Debug)]` を付ける |
| E0369 `binary operation == cannot be applied` | `==` で比較した | `#[derive(PartialEq)]` |
| E0596 `cannot borrow as mutable` | `&mut self` メソッドを不変変数で呼んだ | `let mut` にする |
| E0063 `missing field` | 構造体作成時にフィールドが足りない | 全フィールドを書くか `..Default::default()` |
| E0204 `the trait Copy cannot be implemented` | `String` などを含む型に `Copy` | `Copy` を外して `Clone` のみにする |
| E0382 `use of moved value` | `self` を取るメソッドを呼んだ後に使った | `&self` にする、または Copy にする |

## まとめ

- 構造体は関連するデータを 1 つにまとめる。`impl` でメソッドを付ける。
- `&self` は読む、`&mut self` は書き換える、`self` は消費する。
- `Self` は自分の型。コンストラクタは慣習的に `new`。
- 小さな値型には `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]`。
- `self` を返すメソッドを並べるとビルダー風に書ける (ratatui の書き方)。

## 次へ

第5章では「いくつかの種類のうちどれか 1 つ」を表す **enum** と、それを安全に取り出す `match` を学びます。
キー入力 (`Key::Char('q')`, `Key::Up`, ...) を表すのにぴったりの道具です。
