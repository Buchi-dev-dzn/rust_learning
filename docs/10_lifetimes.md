# 第10章 ライフタイム

## この章のゴール

- 「参照は、参照先より長生きしてはいけない」というルールを説明できる
- `fn f<'a>(x: &'a str) -> &'a str` のようなライフタイム注釈を読み書きできる
- ライフタイム省略規則を知り、「注釈が要るとき / 要らないとき」を判断できる
- `&str` を持つ構造体 (`struct Tokenizer<'a>`) を書ける

ターミナルアプリはテキスト処理の塊です。
入力行を単語に分ける、ファイルの中身を行に分ける、キー入力のバイト列を区切る…。
このとき **コピーせずに元の文字列の一部 (`&str` スライス) を返す** と速くてメモリも節約できますが、
「その `&str` はいつまで有効か」をコンパイラに伝える必要があります。それがライフタイムです。

## なぜライフタイムが必要か

次のコードはコンパイルできません (実行ボタンなしの断片です)。

```rust
let r;
{
    let s = String::from("hello");
    r = &s;          // s を借用
}                    // ここで s は破棄される
println!("{}", r);   // 破棄済みのメモリを指す参照 (ダングリング参照)!
```

```text
error[E0597]: `s` does not live long enough
  |
4 |     r = &s;
  |         ^^ borrowed value does not live long enough
5 | }
  | - `s` dropped here while still borrowed
6 | println!("{}", r);
  |                - borrow later used here
```

C/C++ ならこれは実行時の未定義動作ですが、Rust は **借用チェッカ** がコンパイル時に見つけます。
借用チェッカは各参照が有効な範囲 = **ライフタイム** を追跡しています。

関数の中だけならコンパイラが自動で追跡できます。
問題になるのは **関数の境界をまたぐとき** です。

## 関数のライフタイム注釈

2 つの文字列のうち長い方を返す関数を考えます。

```rust
fn longer(a: &str, b: &str) -> &str {
    if a.len() >= b.len() { a } else { b }
}
```

```text
error[E0106]: missing lifetime specifier
  |
1 | fn longer(a: &str, b: &str) -> &str {
  |              ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature
          does not say whether it is borrowed from `a` or `b`
```

戻り値の `&str` が `a` から来るのか `b` から来るのか、シグネチャだけでは分かりません。
呼び出し側は「戻り値をいつまで使ってよいか」を知る必要があるので、注釈で明示します。

```rust
// 'a は「a と b の両方が有効な期間」。戻り値はその期間だけ有効
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    let s1 = String::from("long string");
    let result;
    {
        let s2 = String::from("xyz");
        result = longer(&s1, &s2);
        println!("{}", result); // OK: s1 も s2 も生きている
    }
    // println!("{}", result); // NG: s2 が破棄済みなので使えない
}
```

読み方:

- `<'a>` でライフタイム引数を宣言 (ジェネリクスの `<T>` と同じ場所)
- `&'a str` は「少なくとも `'a` の間は有効な `&str`」
- 戻り値 `&'a str` は「`a` と `b` の **短い方** の寿命まで使える」

**注釈は寿命を変えません。** 「この参照とあの参照の寿命はこういう関係」と説明するだけです。
コンパイラはその説明が正しいか検査し、呼び出し側の使い方もその説明に照らして検査します。

### 戻り値が片方にしか依存しないとき

```rust
// 戻り値は text の一部。prefix とは関係ない
fn strip<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

fn main() {
    let line = String::from("> echo hi");
    let cmd;
    {
        let p = String::from("> ");
        cmd = strip(&line, &p);
    } // p は破棄されるが、cmd は line にだけ依存するので OK
    println!("{cmd}");
}
```

戻り値と関係ない引数にまで `'a` を付けると、呼び出し側に不要な制約がかかります。
「戻り値はどの引数から借りているか」を正確に書くのがコツです。

## ライフタイム省略規則

毎回 `'a` を書くのは大変なので、よくあるパターンでは省略できます。

1. 参照の引数それぞれに別々のライフタイムが割り当てられる
2. 参照の引数が **1 つだけ** なら、戻り値の参照はそれと同じライフタイム
3. メソッドで `&self` / `&mut self` があれば、戻り値の参照は `self` と同じライフタイム

```rust
// 規則 2: 引数の参照が 1 つなので注釈不要
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

struct Editor {
    lines: Vec<String>,
}

impl Editor {
    // 規則 3: 戻り値は self から借りたもの
    fn line(&self, n: usize) -> &str {
        self.lines.get(n).map(|s| s.as_str()).unwrap_or("")
    }
}

fn main() {
    println!("{}", first_word("cargo run --release"));
    let ed = Editor { lines: vec!["fn main() {".into(), "}".into()] };
    println!("{}", ed.line(0));
}
```

`longer` のように参照の引数が 2 つあり、`self` もない場合は省略できないので注釈が必要です。

## 構造体に参照を持たせる

構造体のフィールドに参照を入れるときは、必ずライフタイムを書きます。
「この構造体は `'a` より長生きできない」という意味です。

```rust
/// 元の文字列をコピーせずに 1 行ずつ切り出す
struct Lines<'a> {
    rest: &'a str,
}

impl<'a> Lines<'a> {
    fn new(text: &'a str) -> Self {
        Lines { rest: text }
    }

    // 戻り値は self ではなく元の text ('a) から借りている
    fn next_line(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            return None;
        }
        match self.rest.find('\n') {
            Some(i) => {
                let line = &self.rest[..i];
                self.rest = &self.rest[i + 1..];
                Some(line)
            }
            None => {
                let line = self.rest;
                self.rest = "";
                Some(line)
            }
        }
    }
}

fn main() {
    let text = String::from("first\nsecond\nthird");
    let mut lines = Lines::new(&text);
    let a = lines.next_line();
    let b = lines.next_line();
    // lines を使い終わっても、a と b は text が生きている限り使える
    println!("{:?} {:?} {:?}", a, b, lines.next_line());
    println!("{:?}", lines.next_line());
}
```

ここで `next_line` の戻り値を `Option<&str>` (省略形) にすると、規則 3 により
「`self` (= `&mut Lines`) と同じ寿命」になってしまい、`a` を持ったまま次の `next_line` を呼べなくなります。
`Option<&'a str>` と書くことで「元のテキストから借りている」ことを正しく伝えています。

### Iterator を実装する

第11章で詳しく扱いますが、上の `Lines` は `Iterator` にすると `for` で回せます。

```rust
struct Words<'a> {
    rest: &'a str,
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let s = self.rest.trim_start();
        if s.is_empty() {
            return None;
        }
        let end = s.find(char::is_whitespace).unwrap_or(s.len());
        let (word, rest) = s.split_at(end);
        self.rest = rest;
        Some(word)
    }
}

fn main() {
    let input = "  :w   file.txt  ";
    let words: Vec<&str> = Words { rest: input }.collect();
    println!("{:?}", words);
}
```

`str::find` が返すのは **バイト位置** です。`&s[..i]` でスライスするとき、
マルチバイト文字 (日本語など) の途中で切ると panic します。
`find` や `char_indices` が返す位置は必ず文字境界なので安全です (第16章で詳しく扱います)。

## `'static`

`'static` は「プログラム終了まで有効」という特別なライフタイムです。
文字列リテラル `"hello"` の型は `&'static str` です (バイナリに埋め込まれているため)。

```rust
fn key_name(code: u8) -> &'static str {
    match code {
        13 => "Enter",
        27 => "Esc",
        127 => "Backspace",
        _ => "Other",
    }
}

fn main() {
    println!("{}", key_name(27));
}
```

第14章でスレッドに値を渡すとき `'static` 境界がまた登場します
(スレッドはいつまで動くか分からないので、借用した値を渡せない)。

エラーを黙らせるために何でも `'static` にするのは悪手です。
困ったら `String` (所有) で持つ、という選択肢も覚えておきましょう。
**最初は所有型で書き、性能が必要な箇所だけ参照にする** のが Rust の現実的な進め方です。

## よくあるコンパイルエラーと読み方

### E0106: missing lifetime specifier

参照を返す関数・参照を持つ構造体で、ライフタイムが決められないときに出ます。
「戻り値はどの引数から借りているか」を考えて `'a` を付けます。

```rust
struct Token {
    text: &str, // E0106: 構造体のフィールドには必ず注釈が要る
}
```

→ `struct Token<'a> { text: &'a str }`

### E0597: borrowed value does not live long enough

借用先が先に破棄されるのに、参照がまだ使われています。
`- 's' dropped here while still borrowed` と `borrow later used here` の行を見比べ、
値の宣言位置を外側に移すか、所有型 (`String`) にします。

### E0515: cannot return reference to local variable

```rust
fn make() -> &'static str {
    let s = String::from("temp");
    &s // 関数を抜けると s は破棄される
}
```

関数内で作った値への参照は返せません。`String` をそのまま返しましょう。
ライフタイム注釈ではこのエラーは直せません (注釈は寿命を延ばさない)。

### lifetime may not live long enough

```text
error: lifetime may not live long enough
  |
  | fn pick<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { y }
  |                                                      ^ returning this value requires that `'b` must outlive `'a`
```

宣言した関係と実装が食い違っています。`y` を返すなら戻り値は `'b` にすべきです。

## まとめ

- ライフタイムは「参照が有効な期間」。借用チェッカがダングリング参照を防ぐ
- 注釈 `'a` は寿命を変えず、**参照同士の関係** を説明するだけ
- 省略規則で多くの場合は書かなくてよい。引数が複数の参照で戻り値も参照のときに必要
- `&'a str` を持つ構造体で、コピーなしのトークナイザや行分割が書ける
- 迷ったら所有型 (`String`) で書いてよい

## 次へ

第11章では **クロージャとイテレータ** を学びます。
この章の `Words` のような「`&str` を順に返すもの」を、`map`・`filter`・`collect` で組み合わせて
テキスト折り返しや行フィルタを書きます。
