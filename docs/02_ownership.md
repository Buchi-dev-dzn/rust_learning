# 第2章 所有権 (move / clone / Copy)

## この章のゴール

- 所有権の 3 つのルールを言える
- 「ムーブ (move)」が起きる場面と、その後に元の変数が使えなくなる理由がわかる
- `clone` と `Copy` の違いを説明できる
- 関数に値を渡す・関数から値を返すときに所有権がどう移るか追える
- `E0382: borrow of moved value` を自分で直せる

所有権は Rust を Rust たらしめる仕組みです。ガベージコレクタなしで、
メモリの解放忘れや二重解放をコンパイル時に防ぎます。
TUI アプリでも「画面バッファは誰が持っているか」「入力文字列は誰が持っているか」を
常に意識することになるので、ここでしっかり体に入れましょう。

## スタックとヒープ

- **スタック**: サイズがコンパイル時に決まる値 (`i32`, `bool`, `(u16, u16)` など) を置く。速い。関数を抜けると自動で消える。
- **ヒープ**: 実行時にサイズが変わる値 (`String` の中身, `Vec` の中身) を置く。確保と解放が必要。

`String` は次の 3 つの値 (ポインタ・長さ・容量) をスタックに持ち、文字データ本体はヒープにあります。

```text
 スタック (s)            ヒープ
+----------+--------+   +---+---+---+---+---+
| ptr      | ------------>| h | e | l | l | o |
| len      | 5      |   +---+---+---+---+---+
| capacity | 5      |
+----------+--------+
```

ヒープの領域は「いつ」「誰が」解放するのかを決めなければなりません。
Rust はそれを **所有権 (ownership)** で決めます。

## 所有権の 3 つのルール

1. Rust の各値には **所有者 (owner)** と呼ばれる変数がちょうど 1 つある。
2. 所有者は同時に 1 つだけ。
3. 所有者がスコープを抜けると、値は **破棄 (drop)** される。

```rust
fn main() {
    {
        let s = String::from("hello"); // s が所有者になる
        println!("{s}");
    } // ここで s のスコープが終わり、ヒープの "hello" が解放される
    // println!("{s}"); // ここでは s はもう存在しない
    println!("ブロックを抜けました");
}
```

`drop` はスコープを抜けるときに自動で呼ばれます。C の `free` を書く必要はありません。
後の章で「ターミナルを raw モードにして、スコープを抜けたら自動で元に戻す」
という仕組みを作りますが、これはこのルール (と `Drop` トレイト) を利用したものです。

## ムーブ

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1 の所有権が s2 に「ムーブ」する
    println!("{s2}");
    // println!("{s1}"); // error[E0382]: borrow of moved value: `s1`
}
```

`let s2 = s1;` はヒープのデータをコピーしません。スタック上の (ptr, len, capacity) だけをコピーし、
**s1 を無効にします**。もし両方が有効なままだと、スコープを抜けたときに同じヒープ領域を 2 回解放してしまうからです。

コメントを外すと、コンパイラは次のように教えてくれます。

```text
error[E0382]: borrow of moved value: `s1`
 --> src/main.rs:4:15
  |
2 |     let s1 = String::from("hello");
  |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
3 |     let s2 = s1;
  |              -- value moved here
4 |     println!("{s1}");
  |               ^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let s2 = s1.clone();
  |                ++++++++
```

読み方:

- `move occurs because ... does not implement the Copy trait` — `String` は `Copy` でないのでムーブされた。
- `value moved here` — ムーブが起きた場所。
- `value borrowed here after move` — ムーブ後に使おうとした場所。
- `help:` — 直し方の候補 (ここでは `clone`)。

## clone: 明示的な深いコピー

ヒープのデータごと複製したいときは `.clone()` を呼びます。

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // ヒープの中身も複製する
    println!("s1 = {s1}, s2 = {s2}");
}
```

`clone` は名前のとおりコストがかかる処理です。Rust は「高くつく処理は目に見える形で書かせる」方針なので、
暗黙の深いコピーは起きません。コードに `clone()` が見えたら「ここで複製している」と読めます。

ただし `clone` は「とりあえずエラーを消す道具」として乱用しがちです。
多くの場合、本当に必要なのは次章の **借用** です。

## Copy: スタックだけで完結する型

整数などはムーブ後も元の変数が使えます。

```rust
fn main() {
    let x = 5;
    let y = x; // コピーされる。x はそのまま使える
    println!("x = {x}, y = {y}");

    let pos = (3u16, 4u16);
    let pos2 = pos; // タプルも中身がすべて Copy なら Copy
    println!("{pos:?} {pos2:?}");
}
```

`Copy` トレイトを実装した型は、代入や関数呼び出しで **ビット単位でコピー** されます。
ヒープを所有しない小さな型だけが `Copy` になれます。

| Copy である | Copy でない |
|-------------|-------------|
| 整数, 浮動小数点, `bool`, `char` | `String` |
| 中身がすべて Copy のタプル・配列 | `Vec<T>` |
| `&T` (共有参照) | `Box<T>` |
| `#[derive(Clone, Copy)]` を付けた自作の小さな構造体 | `&mut T` |

第4章では、ターミナルの座標 `Pos { x: u16, y: u16 }` のような小さな構造体に
`#[derive(Clone, Copy)]` を付けて、整数のように気軽に渡せるようにします。

## 関数と所有権

関数に値を渡すのは、変数への代入と同じです。`Copy` でない値は **関数にムーブ** されます。

```rust
fn takes_ownership(s: String) {
    println!("受け取った: {s}");
} // ここで s が drop される

fn makes_copy(n: i32) {
    println!("受け取った: {n}");
}

fn main() {
    let s = String::from("hello");
    takes_ownership(s); // s はムーブされた
    // println!("{s}"); // エラー: s はもう使えない

    let n = 5;
    makes_copy(n);
    println!("n はまだ使える: {n}");
}
```

### 戻り値で所有権を返す

関数から値を返すと、所有権は呼び出し元にムーブします。

```rust
fn make_title(name: &str) -> String {
    let mut t = String::from("== ");
    t.push_str(name);
    t.push_str(" ==");
    t // 所有権を呼び出し元に渡す
}

fn add_suffix(mut s: String) -> String {
    s.push_str("!");
    s // 受け取ったものに手を加えて返す
}

fn main() {
    let title = make_title("menu");
    let title = add_suffix(title); // ムーブして、返ってきたものを受け取り直す
    println!("{title}");
}
```

「受け取って、加工して、返す」は所有権を明示する素直なスタイルです。
ただ、値を見るだけなのに毎回受け取って返すのは面倒です。それを解決するのが次章の借用 (`&`) です。

## ムーブが起きる場所いろいろ

### ループの中

```rust
fn consume(s: String) -> usize {
    s.len()
}

fn demo() {
    let s = String::from("abc");
    for _ in 0..2 {
        consume(s); // error[E0382]: use of moved value: `s`
                    // 1 回目でムーブ済みなので 2 回目で使えない
    }
}
```

エラーメッセージに `value moved here, in previous iteration of loop` と出ます。

### Vec から取り出す

```rust
fn demo() {
    let v = vec![String::from("a"), String::from("b")];
    let first = v[0]; // error[E0507]: cannot move out of index of `Vec<String>`
}
```

`v[0]` を所有権ごと取り出すと Vec の中に「穴」があいてしまうので禁止されています。
`v[0].clone()` で複製するか、`&v[0]` で借用 (次章) します。

### for ループで Vec を回す

```rust
fn main() {
    let names = vec![String::from("alice"), String::from("bob")];
    for name in names { // names はここでムーブされ、各要素の所有権が name に渡る
        println!("{name}");
    }
    // println!("{:?}", names); // エラー: names はムーブ済み
    let names2 = vec![String::from("carol")];
    for name in &names2 { // & を付ければ借用するだけ
        println!("{name}");
    }
    println!("まだ使える: {names2:?}");
}
```

## 部分的なムーブと構造体

構造体のフィールドを取り出すと、その構造体は「部分的にムーブされた」状態になります。

```rust
struct Line {
    text: String,
    number: usize,
}

fn main() {
    let line = Line { text: String::from("fn main() {}"), number: 1 };
    let number = line.number; // usize は Copy なのでコピー
    let text = line.text;     // String はムーブ
    println!("{number}: {text}");
    // line.text はもう使えないが、line.number はまだ使える
    println!("{}", line.number);
}
```

## std::mem::take と replace

「所有権を取り出したいが、元の場所を空にはできない」ときの道具です。
エディタのバッファを丸ごと差し替えるような場面で役立ちます。

```rust
fn main() {
    let mut buffer = String::from("old text");
    // buffer を空文字列に置き換え、元の中身を所有権ごと受け取る
    let old = std::mem::take(&mut buffer);
    println!("old = {old:?}, buffer = {buffer:?}");

    let mut status = String::from("Ready");
    let prev = std::mem::replace(&mut status, String::from("Saving..."));
    println!("prev = {prev}, now = {status}");
}
```

## よくあるコンパイルエラーと読み方

| コード | メッセージ | 直し方の候補 |
|--------|------------|--------------|
| E0382 | `borrow of moved value` / `use of moved value` | ムーブ前に `clone()`、借用 `&` に変える、関数から返してもらう |
| E0507 | `cannot move out of ...` | `clone()`、`&` で借用、`std::mem::take` |
| E0505 | `cannot move out of ... because it is borrowed` | 借用が終わってからムーブする (次章) |

E0382 を見たら、まず「どこでムーブされたか (`value moved here`)」を探してください。
そのうえで、

1. 本当に所有権を渡す必要があるか? → 不要なら **借用** (次章)
2. 両方で別々の値を持つ必要があるか? → **clone**
3. 渡した先から返してもらえるか? → **戻り値で返す**

の順に検討するのが定石です。

## まとめ

- 値には所有者が 1 つだけあり、所有者がスコープを抜けると drop される。
- `Copy` でない値の代入・関数呼び出しは **ムーブ**。ムーブ元は使えなくなる。
- `clone()` は明示的な深いコピー。コストが見える。
- 整数・`bool`・`char`、それらだけでできたタプルなどは `Copy`。
- 関数の戻り値で所有権を呼び出し元に返せる。
- `std::mem::take` / `replace` で「中身だけ抜き取る」ことができる。

## 次へ

「見るだけなのにムーブされて困る」問題を解決するのが第3章の **借用と参照** です。
