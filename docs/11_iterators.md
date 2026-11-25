# 第11章 クロージャとイテレータ

## この章のゴール

- クロージャ `|x| x + 1` を書き、変数のキャプチャ (借用 / move) を理解する
- `Fn` / `FnMut` / `FnOnce` の違いを説明できる
- `map`・`filter`・`enumerate`・`skip`・`take`・`collect` などのアダプタを組み合わせられる
- 自分の型に `Iterator` を実装できる

TUI の描画処理は「データの一部を切り出して、整形して、画面に並べる」の繰り返しです。
たとえばログビューアなら「スクロール位置から `skip` して、画面の高さだけ `take` して、
検索語で `filter` して、行番号を `enumerate` で付ける」。
イテレータを使うと、これを短く・速く・バグなく書けます。

## クロージャ

クロージャは「その場で作る無名関数」で、周りの変数を **キャプチャ** できます。

```rust
fn main() {
    let add_one = |x: i32| x + 1;          // 型注釈は省略可能なことが多い
    let square = |x| x * x;
    println!("{} {}", add_one(2), square(5));

    let width = 10;
    let fits = |s: &str| s.len() <= width; // width をキャプチャ (借用)
    println!("{} {}", fits("short"), fits("this is too long"));

    // 複数行の本体は { } で
    let label = |name: &str, n: usize| {
        let s = format!("{name}: {n}");
        s.to_uppercase()
    };
    println!("{}", label("items", 3));
}
```

### キャプチャの方法と Fn / FnMut / FnOnce

クロージャが変数をどう使うかで、自動的に実装されるトレイトが決まります。

| トレイト | キャプチャした変数を | 何回呼べるか | 例 |
|---|---|---|---|
| `Fn` | 読むだけ (`&`) | 何回でも | `\|x\| x + offset` |
| `FnMut` | 変更する (`&mut`) | 何回でも (要 `mut`) | `\|\| count += 1` |
| `FnOnce` | 所有権ごと消費する | 1 回だけ | `\|\| drop(buf)` |

```rust
fn call_twice<F: FnMut()>(mut f: F) {
    f();
    f();
}

fn main() {
    let mut count = 0;
    let mut inc = || count += 1; // count を &mut で借用 → FnMut
    inc();
    call_twice(&mut inc);
    println!("count = {count}");

    let name = String::from("log.txt");
    let consume = move || name; // name の所有権をクロージャへ移動
    let got = consume();        // FnOnce: 1 回だけ呼べる
    println!("{got}");
}
```

`move` キーワードを付けると、キャプチャが **所有権の移動** になります。
第14章でスレッドにクロージャを渡すとき、`thread::spawn(move || ...)` の形で必ず使います。

### クロージャを引数・戻り値・フィールドにする

```rust
// 引数: ジェネリクスで受け取る
fn apply<F: Fn(i32) -> i32>(f: F, v: i32) -> i32 {
    f(v)
}

// 戻り値: impl Fn で返す (キャプチャするので move が必要)
fn make_scaler(k: i32) -> impl Fn(i32) -> i32 {
    move |x| x * k
}

// フィールド/コレクション: 型が違うクロージャを混ぜるなら Box<dyn Fn>
struct Command {
    name: &'static str,
    run: Box<dyn Fn(i32) -> i32>,
}

fn main() {
    println!("{}", apply(|x| x - 1, 10));
    let triple = make_scaler(3);
    println!("{}", triple(7));

    let commands = vec![
        Command { name: "double", run: Box::new(|x| x * 2) },
        Command { name: "negate", run: Box::new(|x| -x) },
        Command { name: "add 100", run: Box::new(make_scaler(1)) },
    ];
    for c in &commands {
        println!("{} -> {}", c.name, (c.run)(5)); // フィールドの関数呼び出しは (c.run)(..)
    }
}
```

`Box<dyn Fn>` を `HashMap<KeyCode, Box<dyn Fn(&mut App)>>` のように持てば、**キーバインド表** が作れます。

## イテレータの基本

イテレータは `Iterator` トレイトを実装した型で、核心は `next` だけです。

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // map, filter, collect ... など 70 以上のメソッドがデフォルト実装されている
}
```

コレクションからイテレータを得る方法は 3 つあります。

| 書き方 | 要素の型 | 元のコレクション |
|---|---|---|
| `v.iter()` | `&T` | そのまま使える |
| `v.iter_mut()` | `&mut T` | 要素を書き換えられる |
| `v.into_iter()` | `T` | 消費される (move) |

`for x in &v` は `v.iter()`、`for x in v` は `v.into_iter()` と同じ意味です。

### アダプタは遅延評価

`map` や `filter` は新しいイテレータを返すだけで、**まだ何も計算しません**。
`collect`・`sum`・`count`・`for` などで消費したときに初めて動きます。

```rust
fn main() {
    let lines = vec!["fn main() {", "", "    // comment", "    println!(\"hi\");", "}"];

    let code: Vec<String> = lines
        .iter()
        .enumerate()                                   // (index, &line)
        .filter(|(_, l)| !l.trim().is_empty())         // 空行を除く
        .filter(|(_, l)| !l.trim_start().starts_with("//")) // コメント行を除く
        .map(|(i, l)| format!("{:>3} | {}", i + 1, l)) // 行番号を付ける
        .collect();

    for line in &code {
        println!("{line}");
    }

    let total_chars: usize = lines.iter().map(|l| l.chars().count()).sum();
    println!("文字数合計: {total_chars}");
}
```

### よく使うアダプタと消費メソッド

```rust
fn main() {
    let log: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();

    // スクロール: 5 行目から画面の高さ 4 行分
    let visible: Vec<&String> = log.iter().skip(5).take(4).collect();
    println!("{:?}", visible);

    // 位置検索
    let pos = log.iter().position(|l| l.ends_with("13"));
    println!("{:?}", pos);

    // 条件判定
    println!("{}", log.iter().any(|l| l.contains("20")));
    println!("{}", log.iter().all(|l| l.starts_with("line")));

    // 最初に見つかったもの
    println!("{:?}", log.iter().find(|l| l.len() > 6));

    // map + Option の除外を同時に: filter_map
    let nums: Vec<u32> = "10 x 20 30 y".split(' ').filter_map(|s| s.parse().ok()).collect();
    println!("{:?}", nums);

    // 畳み込み
    let longest = log.iter().map(|l| l.len()).max().unwrap_or(0);
    let joined = ["a", "b", "c"].iter().fold(String::new(), |acc, s| acc + s);
    println!("{longest} {joined}");

    // 2 つを並べる / 平らにする
    let keys = ["q", "j", "k"];
    let desc = ["quit", "down", "up"];
    for (k, d) in keys.iter().zip(desc.iter()) {
        println!("{k}: {d}");
    }
    let words: Vec<&str> = ["a b", "c d e"].iter().flat_map(|s| s.split(' ')).collect();
    println!("{:?}", words);

    // 逆順・ステップ・ウィンドウ
    let v = [1, 2, 3, 4, 5, 6];
    println!("{:?}", v.iter().rev().step_by(2).collect::<Vec<_>>());
    println!("{:?}", v.chunks(4).collect::<Vec<_>>());
    println!("{:?}", v.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>());
}
```

`collect` は戻り値の型で何を作るか決まります。`Vec<_>`・`String`・`HashMap<_, _>`・
さらに `Result<Vec<_>, E>` (途中で `Err` があればそれを返す) も作れます。

```rust
fn main() {
    let ok: Result<Vec<i32>, _> = "1,2,3".split(',').map(|s| s.parse::<i32>()).collect();
    let ng: Result<Vec<i32>, _> = "1,x,3".split(',').map(|s| s.parse::<i32>()).collect();
    println!("{:?}", ok);
    println!("{}", ng.is_err());

    let s: String = vec!['h', 'i'].into_iter().collect();
    println!("{s}");
}
```

## 例: テキストの折り返し

TUI では「幅 W に収まるように単語単位で折り返す」処理が頻出です。
イテレータで単語を流し、行バッファに詰めていきます (ここでは簡単のため 1 文字 = 幅 1 とします。
日本語の全角文字の幅は第16章で扱います)。

```rust
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let needed = if current.is_empty() { word.len() } else { current.len() + 1 + word.len() };
        if needed > width && !current.is_empty() {
            lines.push(std::mem::take(&mut current)); // current を空にしつつ中身を取り出す
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn main() {
    let text = "Rust iterators are lazy and composable, which makes them great for rendering.";
    for line in wrap(text, 20) {
        println!("|{:<20}|", line);
    }
}
```

## 自分の型に Iterator を実装する

`next` を書くだけで、`map`・`filter`・`collect` などがすべて使えるようになります。

```rust
/// 0, 1, 2, ... と行番号を返しつつ、ページ (高さ height) ごとに区切る
struct Pages {
    total: usize,
    height: usize,
    start: usize,
}

impl Iterator for Pages {
    type Item = std::ops::Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.total {
            return None;
        }
        let end = (self.start + self.height).min(self.total);
        let page = self.start..end;
        self.start = end;
        Some(page)
    }
}

fn main() {
    let pages = Pages { total: 10, height: 4, start: 0 };
    for (i, p) in pages.enumerate() {
        println!("page {}: lines {:?}", i + 1, p);
    }
    let count = Pages { total: 100, height: 24, start: 0 }.count();
    println!("{count} pages");
}
```

## よくあるコンパイルエラーと読み方

### E0282: type annotations needed (collect)

```rust
let v = (1..4).map(|x| x * 2).collect();
```

`collect` は何を作るか分からないと決められません。
`let v: Vec<_> = ...` または `.collect::<Vec<_>>()` と書きます (`::<>` は「ターボフィッシュ」と呼ばれます)。

### イテレータを消費し忘れる (警告)

```text
warning: unused `Map` that must be used
  = note: iterators are lazy and do nothing unless consumed
```

`v.iter().map(|x| println!("{x}"));` は何も表示しません。副作用が目的なら `for` か `for_each` を使います。

### E0596 / E0499: クロージャと借用

```text
error[E0596]: cannot borrow `inc` as mutable, as it is not declared as mutable
```

`FnMut` なクロージャを呼ぶには、クロージャ自体を `let mut inc = || ...` と宣言します。
また、`FnMut` クロージャが変数を `&mut` 借用している間は、その変数を他で使えません。

### E0373: closure may outlive the current function

```text
error[E0373]: closure may outlive the current function, but it borrows `k`
help: to force the closure to take ownership of `k`, use the `move` keyword
```

クロージャを返したりスレッドに渡したりするときに出ます。`move |x| ...` にしましょう。

### filter の引数が `&&T` になる

`v.iter().filter(|x| ...)` の `x` は `&&T` です (iter が `&T`、filter がさらに参照を渡す)。
比較では `|&&x| x > 3` や `|x| **x > 3` と書きます。多くのメソッド呼び出しは自動参照外しで気にしなくて済みます。

## まとめ

- クロージャは環境をキャプチャする無名関数。`Fn` / `FnMut` / `FnOnce` はキャプチャの仕方で決まる
- `move` で所有権ごとキャプチャ (スレッドや戻り値で必須)
- イテレータは遅延評価。アダプタをつなげて最後に `collect` / `sum` / `for` で消費
- `skip`・`take`・`enumerate`・`filter`・`map` はスクロール付きリスト描画の基本セット
- `next` を実装すれば、自作の型でも全アダプタが使える

## 次へ

第12章では **スマートポインタ** (`Box`・`Rc`・`RefCell`) を学び、
UI 部品のツリー (親子関係) を表現します。
