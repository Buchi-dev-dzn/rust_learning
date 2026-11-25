# 第6章 コレクション (Vec, String, HashMap)

## この章のゴール

- `Vec<T>` の作成・追加・削除・挿入・添字アクセス・安全なアクセス (`get`) ができる
- `String` を組み立て、文字単位・行単位で操作できる。`String` に整数で添字アクセスできない理由がわかる
- `HashMap<K, V>` で検索・集計ができ、`entry` API を使える
- イテレータの基本 (`iter` / `iter_mut` / `into_iter` / `map` / `filter` / `collect`) を読み書きできる
- テキストエディタの「行バッファ」やキーバインド表をこれらで表現できる

## Vec<T>

`Vec<T>` は同じ型の値を並べる、伸び縮みする配列です。中身はヒープにあります。

```rust
fn main() {
    let mut lines: Vec<String> = Vec::new();
    lines.push(String::from("first"));
    lines.push(String::from("second"));
    let nums = vec![1, 2, 3];          // vec! マクロ
    let zeros = vec![0u8; 5];          // 同じ値を 5 個

    println!("{lines:?} {nums:?} {zeros:?}");
    println!("len = {}, empty? {}", lines.len(), lines.is_empty());

    lines.insert(0, String::from("zeroth")); // 途中に挿入
    let removed = lines.remove(1);           // 途中を削除して返す
    let last = lines.pop();                  // 末尾を取り出す (Option)
    println!("removed={removed} last={last:?} rest={lines:?}");
}
```

| メソッド | 動作 | 計算量 |
|----------|------|--------|
| `push(x)` / `pop()` | 末尾に追加 / 末尾を取り出し (`Option<T>`) | O(1) |
| `insert(i, x)` / `remove(i)` | 位置 i に挿入 / 位置 i を削除 | O(n) |
| `v[i]` | 添字アクセス。範囲外は **パニック** | O(1) |
| `v.get(i)` | 安全なアクセス (`Option<&T>`) | O(1) |
| `truncate(n)` / `clear()` | 長さ n に切る / 空にする | |
| `extend(iter)` | まとめて追加 | |
| `contains(&x)` | 含むか | O(n) |
| `sort()` / `dedup()` / `reverse()` | 並べ替え / 連続重複除去 / 反転 | |

### 添字と get

```rust
fn main() {
    let v = vec![10, 20, 30];
    println!("{}", v[1]);
    // println!("{}", v[10]); // パニック: index out of bounds
    match v.get(10) {
        Some(x) => println!("{x}"),
        None => println!("範囲外"),
    }
    // 最後の要素
    println!("{:?} {:?}", v.last(), v.first());
}
```

カーソル位置がずれて範囲外を読むのは TUI で最もよくあるバグの 1 つです。
「必ず範囲内」と言い切れないところでは `get` を使いましょう。

### Vec と借用

`Vec` の要素への参照を持っている間は、`push` などで `Vec` を変更できません (第3章)。

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    // 読む: &v (各要素は &i32)
    for x in &v {
        print!("{x} ");
    }
    println!();
    // 書き換える: &mut v (各要素は &mut i32)
    for x in &mut v {
        *x *= 10;
    }
    println!("{v:?}");
    // 消費する: v そのもの (各要素の所有権を受け取る)
    for x in v {
        print!("{x} ");
    }
    println!();
}
```

## イテレータの基本

`for` ループの背後にあるのが **イテレータ** です。メソッドをつなげてデータを変換できます。
詳しくは第11章ですが、ここでよく使う形だけ覚えておきましょう。

```rust
fn main() {
    let words = vec!["apple", "banana", "cherry", "date"];

    // map で変換して collect で Vec に集める
    let lens: Vec<usize> = words.iter().map(|w| w.len()).collect();
    println!("{lens:?}");

    // filter で絞る
    let long: Vec<&&str> = words.iter().filter(|w| w.len() > 5).collect();
    println!("{long:?}");

    // enumerate で番号付き
    for (i, w) in words.iter().enumerate() {
        println!("{:>2}: {w}", i + 1);
    }

    // sum / count / max / any / all / position
    let total: usize = lens.iter().sum();
    let max = lens.iter().max();
    let has_b = words.iter().any(|w| w.starts_with('b'));
    let pos = words.iter().position(|&w| w == "cherry");
    println!("total={total} max={max:?} has_b={has_b} pos={pos:?}");
}
```

| 呼び出し | 要素の型 | 元のコレクション |
|----------|----------|------------------|
| `v.iter()` (`for x in &v`) | `&T` | そのまま使える |
| `v.iter_mut()` (`for x in &mut v`) | `&mut T` | そのまま使える |
| `v.into_iter()` (`for x in v`) | `T` | ムーブされる |

## String

`String` は UTF-8 のバイト列を持つ `Vec<u8>` のようなものです。

```rust
fn main() {
    let mut s = String::new();
    s.push_str("Hello");
    s.push(',');
    s.push(' ');
    s += "世界"; // += で &str を追加
    let t = format!("{s}! ({} bytes, {} chars)", s.len(), s.chars().count());
    println!("{t}");

    // 変換
    let n = 42;
    let from_num = n.to_string();
    let from_str: String = "abc".into();
    println!("{from_num} {from_str}");

    // よく使うメソッド
    let line = "  Cargo.toml  ";
    println!("[{}]", line.trim());
    println!("{}", line.trim().to_uppercase());
    println!("{}", "a-b-c".replace('-', "/"));
    println!("{}", "main.rs".ends_with(".rs"));
    println!("{:?}", "key=value".split_once('='));
    println!("{}", "ab".repeat(3));
}
```

### 文字列に添字でアクセスできない

```rust
fn demo() {
    let s = String::from("hello");
    let c = s[0]; // error[E0277]: the type `str` cannot be indexed by `{integer}`
}
```

UTF-8 では 1 文字が 1〜4 バイトなので、「0 番目」がバイトなのか文字なのかが曖昧です。
Rust は曖昧な操作を許さず、意図を明示させます。

```rust
fn main() {
    let s = "aあ😀";
    println!("bytes: {:?}", s.as_bytes());          // バイト列
    println!("chars: {:?}", s.chars().collect::<Vec<_>>()); // 文字 (Unicode スカラー値)
    println!("3 番目の文字: {:?}", s.chars().nth(2));
    for (byte_pos, ch) in s.char_indices() {
        println!("{byte_pos}: {ch}");
    }
}
```

エディタの行は `String` で持ちつつ、カーソルは「何文字目か」で管理し、
必要なときに `char_indices` でバイト位置に変換する、というのがよくある設計です。
(さらに、全角文字は画面上で 2 マス使うという「表示幅」の問題があります。第16章で扱います)

### 文字単位の挿入と削除

```rust
/// 文字位置 → バイト位置 (末尾なら len)
fn byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(s.len())
}

fn main() {
    let mut line = String::from("こんにちは");
    let at = byte_index(&line, 2);
    line.insert(at, '★');         // insert はバイト位置を取る
    println!("{line}");
    let at = byte_index(&line, 0);
    let removed = line.remove(at); // remove もバイト位置。取り除いた char を返す
    println!("{removed} を削除 → {line}");
}
```

### 行に分ける・つなぐ

```rust
fn main() {
    let text = "line one\nline two\n\nline four";
    let lines: Vec<&str> = text.lines().collect();
    println!("{} 行: {lines:?}", lines.len());

    let words: Vec<&str> = "  split   by whitespace ".split_whitespace().collect();
    println!("{words:?}");

    let joined = lines.join(" | ");
    println!("{joined}");

    // Vec<String> は所有する行バッファ
    let owned: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    println!("{owned:?}");
}
```

## HashMap<K, V>

キーから値を引く表です。`use std::collections::HashMap;` が必要です。

```rust
use std::collections::HashMap;

fn main() {
    let mut bindings: HashMap<char, &str> = HashMap::new();
    bindings.insert('q', "quit");
    bindings.insert('s', "save");
    bindings.insert('j', "down");

    // get は Option<&V>
    match bindings.get(&'s') {
        Some(cmd) => println!("s → {cmd}"),
        None => println!("未割り当て"),
    }
    println!("{:?}", bindings.get(&'x'));
    println!("q あり? {}", bindings.contains_key(&'q'));

    // 上書き: insert は古い値を Option で返す
    let old = bindings.insert('q', "force-quit");
    println!("old = {old:?}");

    bindings.remove(&'j');
    // 繰り返しの順序は不定 (順序が必要なら Vec に入れて sort する)
    let mut keys: Vec<&char> = bindings.keys().collect();
    keys.sort();
    println!("{keys:?}");
}
```

### entry API で集計

「なければ作って、あれば更新する」は `entry` で書きます。

```rust
use std::collections::HashMap;

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the end";
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    // 回数の多い順、同数なら辞書順に並べる
    let mut list: Vec<(&str, usize)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (w, n) in list.iter().take(3) {
        println!("{w}: {n}");
    }

    // グループ分け: キーごとに Vec を持つ
    let mut by_ext: HashMap<&str, Vec<&str>> = HashMap::new();
    for f in ["a.rs", "b.toml", "c.rs", "README.md"] {
        let ext = f.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        by_ext.entry(ext).or_default().push(f);
    }
    println!("rs: {:?}", by_ext["rs"]);
}
```

- `entry(k).or_insert(v)` は値への `&mut V` を返します。`*` で参照外しして更新します。
- `or_default()` は `V::default()` (`0`、空の `Vec` など) で作ります。
- キーにする型は `Eq + Hash` が必要です。自作の型なら `#[derive(PartialEq, Eq, Hash)]`。

### 所有権と HashMap

`insert` するとキーと値の所有権は HashMap に移ります。

```rust
use std::collections::HashMap;

fn main() {
    let name = String::from("theme");
    let value = String::from("dark");
    let mut config = HashMap::new();
    config.insert(name, value);
    // println!("{name}"); // エラー: name はムーブ済み
    // String のキーで引くときは &str を渡せる
    println!("{:?}", config.get("theme"));
}
```

## その他のコレクション

| 型 | 用途 |
|----|------|
| `VecDeque<T>` | 両端キュー。入力イベントのキュー、ログのリングバッファ |
| `BTreeMap<K, V>` | キー順に並ぶマップ。順序付きの表示 |
| `HashSet<T>` / `BTreeSet<T>` | 重複のない集合。「選択済みの項目」など |

```rust
use std::collections::{BTreeMap, HashSet, VecDeque};

fn main() {
    let mut history: VecDeque<&str> = VecDeque::new();
    for cmd in ["ls", "cd src", "cargo build", "cargo test"] {
        history.push_back(cmd);
        if history.len() > 3 {
            history.pop_front(); // 古いものを捨てる
        }
    }
    println!("{history:?}");

    let mut marked: HashSet<usize> = HashSet::new();
    marked.insert(3);
    marked.insert(3);
    println!("marked = {marked:?}");

    let mut sorted = BTreeMap::new();
    sorted.insert("zeta", 1);
    sorted.insert("alpha", 2);
    println!("{sorted:?}"); // キー順
}
```

## よくあるコンパイルエラーと読み方

| コード | 状況 | 直し方 |
|--------|------|--------|
| E0277 `cannot be indexed by {integer}` | `s[0]` で文字列に添字 | `s.chars().nth(0)` / `s.as_bytes()[0]` |
| E0502 | 要素を借りたまま `push` | 必要な値を先にコピー/clone、または借用を先に終える |
| E0507 `cannot move out of index` | `let x = v[0];` (非 Copy) | `&v[0]` / `v[0].clone()` / `v.remove(0)` |
| E0282 `type annotations needed` | `collect()` の型がわからない | `let v: Vec<_> = ...collect();` または `collect::<Vec<_>>()` |
| E0599 `no method named entry` | `HashMap` の `use` を忘れ別の型に | `use std::collections::HashMap;` |

実行時エラー (パニック) にも注意:

- `index out of bounds` — `v[i]` の範囲外。`get` を使う。
- `byte index N is not a char boundary` — 文字列スライスを文字の途中で切った。`char_indices` を使う。

## まとめ

- `Vec<T>`: `push` / `pop` / `insert` / `remove`。範囲外が怖い場所では `get`。
- `String` は UTF-8。整数添字は不可。文字は `chars()`、位置は `char_indices()`。
- `HashMap`: `get` は `Option`、集計は `entry().or_insert()`。順序は不定。
- `iter` / `iter_mut` / `into_iter` の違いは「借りる / 可変で借りる / 消費する」。
- `map` / `filter` / `collect` でデータ変換をつなげられる。

## 次へ

第7章では `Result` と `?` 演算子によるエラー処理を学び、「失敗するかもしれない処理」を安全に組み立てます。
