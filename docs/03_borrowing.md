# 第3章 借用と参照・スライス

## この章のゴール

- `&T` (共有参照) と `&mut T` (可変参照) を使い分けられる
- 借用のルール「共有参照はいくつでも、可変参照は 1 つだけ」を説明できる
- `String` と `&str`、`Vec<T>` と `&[T]` の関係がわかる
- 文字列スライス・配列スライスを安全に作れる
- `E0502` / `E0499` / `E0106` などの借用エラーを読んで直せる

前章では、値を関数に渡すと所有権がムーブしてしまうことを学びました。
「見るだけ」「ちょっと書き換えるだけ」なら所有権を渡す必要はありません。
値を **借りる (borrow)** のが参照です。

## 共有参照 `&T`

```rust
fn char_count(s: &String) -> usize {
    s.chars().count()
} // s は参照なので、ここで drop されるのは参照だけ (中身は解放されない)

fn main() {
    let title = String::from("ファイル一覧");
    let n = char_count(&title); // & で参照を作って渡す
    println!("{title} は {n} 文字"); // title はまだ使える
}
```

- `&title` は「title を指す参照」を作ります。所有権は移りません。
- 参照を通して値を **読む** ことはできますが、変更はできません。
- 参照を作ることを「借用する」と言います。

## 可変参照 `&mut T`

```rust
fn add_border(s: &mut String) {
    s.insert(0, '[');
    s.push(']');
}

fn main() {
    let mut label = String::from("OK"); // 元の変数も mut である必要がある
    add_border(&mut label);
    println!("{label}");
}
```

`&mut` で借りると、借りた先で値を書き換えられます。
TUI では「画面バッファ `&mut Buffer` を描画関数に貸して、そこに書き込んでもらう」形を何度も使います。

## 借用のルール

ある時点で、1 つの値に対して

1. **共有参照 `&T` がいくつあってもよい**、または
2. **可変参照 `&mut T` がちょうど 1 つだけ**

のどちらかしか許されません。そして、参照は常に有効な値を指していなければなりません。

これは「誰かが読んでいる最中に、別の誰かが書き換える」データ競合をコンパイル時に防ぐためのルールです。
たとえば `Vec` に要素を追加すると、中のメモリが再確保されて場所が変わることがあります。
その最中に古い場所を指す参照が残っていたら危険です。

```rust
fn demo() {
    let mut lines = vec![String::from("a")];
    let first = &lines[0];          // 共有借用
    lines.push(String::from("b"));  // error[E0502]: cannot borrow `lines` as mutable
                                    // because it is also borrowed as immutable
    println!("{first}");            // 共有借用はここまで生きている
}
```

### 借用の「生存期間」は最後に使われるまで

参照が有効なのは、**最後に使われる場所まで** です (NLL: Non-Lexical Lifetimes)。
したがって、次のコードは問題ありません。

```rust
fn main() {
    let mut lines = vec![String::from("a")];
    let first = &lines[0];
    println!("先頭は {first}"); // first の最後の使用
    lines.push(String::from("b")); // もう first は使われないので OK
    println!("{lines:?}");
}
```

借用エラーが出たら「その参照を最後に使うのはどこか」を見ると、
順序を入れ替えるだけで直ることがよくあります。

### 可変参照は同時に 1 つ

```rust
fn demo() {
    let mut s = String::new();
    let a = &mut s;
    let b = &mut s; // error[E0499]: cannot borrow `s` as mutable more than once at a time
    a.push('x');
    b.push('y');
}
```

## 参照の参照外し `*`

参照の先の値そのものを扱うには `*` を使います。メソッド呼び出しや比較では自動で参照外しされることが多いので、
明示的に書くのは主に「参照の先に代入する」ときです。

```rust
fn increment(count: &mut u32) {
    *count += 1; // 参照の先の値を書き換える
}

fn main() {
    let mut c = 0;
    increment(&mut c);
    increment(&mut c);
    println!("c = {c}");

    let v = vec![1, 2, 3];
    let mut total = 0;
    for x in &v {      // x は &i32
        total += *x;   // *x で i32 を取り出す (total += x でも動く)
    }
    println!("total = {total}");
}
```

## スライス

**スライス** は「連続したデータの一部を指す参照」です。所有権は持ちません。

### 文字列スライス `&str`

```rust
fn main() {
    let s = String::from("hello world");
    let hello: &str = &s[0..5];   // 0 バイト目から 5 バイト目の手前まで
    let world: &str = &s[6..];    // 6 バイト目から最後まで
    let all: &str = &s[..];       // 全体
    println!("{hello} / {world} / {all}");

    let literal: &str = "文字列リテラルも &str"; // プログラムに埋め込まれた文字列への参照
    println!("{literal}");
}
```

`String` は「所有する、伸び縮みする文字列」、`&str` は「どこかにある文字列の一部を借りたもの」です。

**関数の引数には `&String` より `&str` を使いましょう。**
`&String` は自動的に `&str` に変換される (Deref 型強制) ので、`&str` を受け取る関数は
`String` にもリテラルにも使えます。

```rust
fn first_word(s: &str) -> &str {
    // bytes() と enumerate() で 1 バイトずつ見て、空白の位置で切る
    for (i, b) in s.bytes().enumerate() {
        if b == b' ' {
            return &s[..i];
        }
    }
    s
}

fn main() {
    let owned = String::from("quit now");
    println!("{}", first_word(&owned));     // &String -> &str に自動変換
    println!("{}", first_word("open file")); // リテラルはそのまま &str
    println!("{}", first_word("single"));
}
```

戻り値の `&str` は引数 `s` から借りたものです。だから `owned` が生きている間しか使えません。
この「どの引数から借りたか」はコンパイラが推論してくれます (ライフタイム省略規則, 第10章)。

### UTF-8 と境界

`&s[a..b]` の `a`, `b` は **バイト位置** です。日本語の文字は UTF-8 で 3 バイトなので、
文字の途中で切るとパニックします。

```rust
fn main() {
    let s = "日本語";
    println!("バイト数 {} / 文字数 {}", s.len(), s.chars().count());
    println!("{}", &s[0..3]); // "日" (OK: 文字の境界)
    // println!("{}", &s[0..1]); // パニック: byte index 1 is not a char boundary
    // 文字単位で扱うときは chars() を使う
    let first_two: String = s.chars().take(2).collect();
    println!("{first_two}");
    // 境界かどうかを調べる
    println!("{} {}", s.is_char_boundary(1), s.is_char_boundary(3));
}
```

エディタでカーソルを動かすときは、この「バイト位置と文字位置のずれ」が大問題になります。
第16章で表示幅と合わせて詳しく扱います。

### 配列・Vec のスライス `&[T]`

```rust
fn sum(xs: &[i32]) -> i32 {
    let mut total = 0;
    for x in xs {
        total += x;
    }
    total
}

fn main() {
    let arr = [1, 2, 3, 4, 5];
    let v = vec![10, 20, 30];
    println!("{}", sum(&arr));       // 配列全体
    println!("{}", sum(&arr[1..3])); // 一部: [2, 3]
    println!("{}", sum(&v));         // &Vec<i32> -> &[i32]
}
```

`&[T]` を受け取る関数は配列にも `Vec` にも使えます。`&mut [T]` なら中身を書き換えられます。

```rust
fn fill(row: &mut [char], ch: char) {
    for cell in row.iter_mut() {
        *cell = ch;
    }
}

fn main() {
    let mut line = ['.'; 10];
    fill(&mut line[2..5], '#'); // 一部分だけを貸す
    let s: String = line.iter().collect();
    println!("{s}");
}
```

この「1 行分のセル配列の一部を塗りつぶす」操作は、第22章の自作セルバッファで実際に使います。

### スクロールとスライス

画面に収まる行だけを取り出すのもスライスでできます。

```rust
fn visible(lines: &[String], offset: usize, height: usize) -> &[String] {
    let start = offset.min(lines.len());
    let end = (offset + height).min(lines.len());
    &lines[start..end]
}

fn main() {
    let lines: Vec<String> = (1..=10).map(|i| format!("line {i}")).collect();
    for l in visible(&lines, 7, 5) {
        println!("{l}");
    }
}
```

## ダングリング参照は作れない

関数の中で作った値への参照を返すことはできません。関数を抜けると値が drop され、
参照が無効になるからです。

```rust
fn dangle() -> &String { // error[E0106]: missing lifetime specifier
    let s = String::from("hi");
    &s
}
```

この場合は参照ではなく `String` そのもの (所有権) を返します。

## よくあるコンパイルエラーと読み方

| コード | 意味 | 直し方 |
|--------|------|--------|
| E0502 | 共有借用中に可変借用した (またはその逆) | 参照を最後に使う位置の後に変更を移す、必要な値をコピーしておく |
| E0499 | 可変借用が同時に 2 つ | 片方の使用を終えてから次を作る |
| E0596 | `cannot borrow as mutable` | 元の変数を `let mut` にする / 引数を `&mut` にする |
| E0106 | `missing lifetime specifier` | ローカル値への参照を返そうとしていないか確認し、所有権で返す |
| E0308 | `expected &str, found String` | `&s` や `s.as_str()` で参照にする |

E0502 のメッセージは 3 か所を指します:

```text
3 |     let first = &lines[0];
  |                  ----- immutable borrow occurs here
4 |     lines.push(String::from("b"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |               ------- immutable borrow later used here
```

「借りた場所」「衝突した場所」「借用がまだ生きている理由 (later used here)」の 3 つです。
最後の 1 つを前に動かすか、衝突する操作を後ろに動かすのが基本の直し方です。

## まとめ

- `&T` は読むだけの借用、`&mut T` は書き換えられる借用。
- 共有参照は何個でも、可変参照は 1 個だけ。同時には混ぜられない。
- 借用は最後に使われるところまで有効。順序の入れ替えでエラーが消えることが多い。
- 引数は `&String` より `&str`、`&Vec<T>` より `&[T]`。
- 文字列スライスの添字はバイト位置。文字の境界に注意。

## 次へ

第4章では構造体を作り、メソッドの `&self` / `&mut self` / `self` という形で借用と所有権を使い分けます。
