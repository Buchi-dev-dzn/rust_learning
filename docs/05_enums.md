# 第5章 enum とパターンマッチ / Option

## この章のゴール

- データを持つ enum (代数的データ型) を定義できる
- `match` で全パターンを網羅して処理を分けられる
- `if let` / `let else` / `matches!` を使い分けられる
- `Option<T>` で「値がないかもしれない」を安全に表現し、`map` / `unwrap_or` などで扱える
- キー入力やアプリの状態を enum で設計できる

## enum の基本

enum (列挙型) は「いくつかの種類のうち、どれか 1 つ」を表す型です。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

fn mode_label(m: Mode) -> &'static str {
    match m {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Command => "COMMAND",
    }
}

fn main() {
    let mut mode = Mode::Normal;
    println!("{}", mode_label(mode));
    mode = Mode::Insert;
    println!("{} / {:?}", mode_label(mode), mode);
}
```

C の enum と違い、Rust の enum は各 **バリアント (variant)** にデータを持たせられます。

## データを持つ enum

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Key {
    Char(char),          // タプル風: 1 つの値を持つ
    Ctrl(char),
    Enter,               // データなし
    Esc,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    F(u8),               // F1〜F12
}

#[derive(Debug)]
enum Event {
    Key(Key),
    Resize { cols: u16, rows: u16 }, // 構造体風: 名前付きフィールド
    Tick,
}

fn main() {
    let events = [
        Event::Key(Key::Char('a')),
        Event::Key(Key::Ctrl('c')),
        Event::Resize { cols: 120, rows: 40 },
        Event::Tick,
        Event::Key(Key::F(5)),
    ];
    for e in &events {
        println!("{e:?}");
    }
}
```

これは crossterm の `Event` / `KeyCode` とほぼ同じ形です。
「キー入力・リサイズ・タイマー」のように **種類ごとに持つデータが違う** ものを、
1 つの型で安全に表せるのが enum の強みです。

enum の値は「どのバリアントか (タグ)」と「そのデータ」を持ち、サイズは最大のバリアントに合わせて決まります。

## match

`match` はパターンで値を分解しながら分岐します。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Key {
    Char(char),
    Ctrl(char),
    Enter,
    Esc,
    Up,
    Down,
}

#[derive(Debug, PartialEq)]
enum Action {
    Quit,
    Insert(char),
    Move(i32),
    Submit,
    None,
}

fn to_action(key: Key) -> Action {
    match key {
        Key::Ctrl('q') | Key::Esc => Action::Quit,   // | で複数パターン
        Key::Ctrl(_) => Action::None,                 // _ は「何でもよい」
        Key::Char(c) if c.is_ascii_digit() => Action::None, // ガード (if 条件)
        Key::Char(c) => Action::Insert(c),            // 中身を c に束縛
        Key::Up => Action::Move(-1),
        Key::Down => Action::Move(1),
        Key::Enter => Action::Submit,
    }
}

fn main() {
    for k in [Key::Char('x'), Key::Ctrl('q'), Key::Up, Key::Char('7'), Key::Enter] {
        println!("{k:?} => {:?}", to_action(k));
    }
}
```

ポイント:

- **網羅性**: すべてのバリアントを扱わないとコンパイルエラー (E0004) になります。
  後で `Key` に新しいバリアントを足すと、扱い忘れの `match` をコンパイラが全部教えてくれます。
- 上から順に試され、最初に一致した腕が実行されます。`Key::Ctrl('q')` を `Key::Ctrl(_)` より上に書くのはそのためです。
- `match` も式なので値を返せます。

### いろいろなパターン

```rust
fn classify(p: (i32, i32)) -> String {
    match p {
        (0, 0) => "原点".to_string(),
        (x, 0) | (0, x) => format!("軸上 ({x})"),
        (x, y) if x == y => "対角線上".to_string(),
        (1..=9, 1..=9) => "1〜9 の範囲".to_string(),
        _ => "その他".to_string(),
    }
}

struct Size {
    w: u16,
    h: u16,
}

fn main() {
    for p in [(0, 0), (3, 0), (0, -2), (4, 4), (2, 5), (20, 1)] {
        println!("{p:?}: {}", classify(p));
    }
    // 構造体の分解。.. で残りを無視
    let s = Size { w: 80, h: 24 };
    let Size { w, .. } = s;
    println!("w = {w}, h = {}", s.h);
    // @ で値を束縛しつつ範囲チェック
    let n = 42;
    match n {
        small @ 0..=9 => println!("一桁: {small}"),
        big @ 10.. => println!("二桁以上: {big}"),
        _ => println!("負"),
    }
}
```

## Option: null の代わり

Rust には null がありません。「値がないかもしれない」は `Option<T>` で表します。

```rust
// 標準ライブラリでの定義 (自分で書く必要はない)
enum Option<T> {
    None,
    Some(T),
}
```

`Option<T>` と `T` は別の型なので、「値がないかもしれないのにチェックせず使う」ことが型レベルでできません。

```rust
fn find_char(s: &str, target: char) -> Option<usize> {
    for (i, c) in s.chars().enumerate() {
        if c == target {
            return Some(i);
        }
    }
    None
}

fn main() {
    let line = "hello: world";
    match find_char(line, ':') {
        Some(i) => println!("':' は {i} 文字目"),
        None => println!("見つからない"),
    }
    // 標準ライブラリにも Option を返すメソッドがたくさんある
    let v = vec![10, 20, 30];
    println!("{:?} {:?}", v.get(1), v.get(99));   // Some(20) None
    println!("{:?}", v.first());                   // Some(10)
    println!("{:?}", "42".parse::<i32>().ok());    // Some(42)
}
```

### Option を扱う便利メソッド

毎回 `match` を書かなくても、よくある操作はメソッドで書けます。

```rust
fn main() {
    let some: Option<u16> = Some(5);
    let none: Option<u16> = None;

    // 値がなければ既定値
    println!("{} {}", some.unwrap_or(0), none.unwrap_or(0));
    // 中身を変換 (None は None のまま)
    println!("{:?} {:?}", some.map(|x| x * 2), none.map(|x| x * 2));
    // 条件で絞る
    println!("{:?}", some.filter(|&x| x > 10));
    // Option を返す処理をつなぐ
    println!("{:?}", some.and_then(|x| x.checked_sub(10)));
    // 値があるか
    println!("{} {}", some.is_some(), none.is_none());
    // 別の Option で埋める
    println!("{:?}", none.or(Some(1)));
    // unwrap: None ならパニック。テストやプロトタイプ以外では避ける
    println!("{}", some.unwrap());
    // expect: パニック時のメッセージを付けた unwrap
    println!("{}", some.expect("値があるはず"));
}
```

`|x| x * 2` は **クロージャ** (無名関数) です。詳しくは第11章で扱いますが、
「引数 x を受け取って x * 2 を返す関数」と読めば十分です。

### Option と参照

`Option<String>` の中身を借りたいときは `as_ref()` や `as_deref()` を使います。

```rust
fn main() {
    let filename: Option<String> = Some(String::from("main.rs"));
    // as_deref(): Option<String> -> Option<&str>。filename はムーブされない
    let shown = filename.as_deref().unwrap_or("[無題]");
    println!("{shown}");
    println!("まだ使える: {filename:?}");

    // take(): 中身を取り出して None にする (std::mem::take の Option 版)
    let mut pending: Option<String> = Some(String::from("保存しました"));
    if let Some(msg) = pending.take() {
        println!("通知: {msg}");
    }
    println!("pending = {pending:?}");
}
```

## if let / let else / matches!

1 つのパターンだけに興味があるときは `match` より短く書けます。

```rust
#[derive(Debug, Clone, Copy)]
enum Key {
    Char(char),
    Enter,
    Esc,
}

fn main() {
    let keys = [Key::Char('h'), Key::Enter, Key::Char('i'), Key::Esc];
    let mut typed = String::new();
    for key in keys {
        // if let: 一致したときだけ実行
        if let Key::Char(c) = key {
            typed.push(c);
        }
    }
    println!("typed = {typed}");

    // matches!: パターンに一致するかを bool で
    let quit_count = keys.iter().filter(|k| matches!(k, Key::Esc)).count();
    println!("Esc は {quit_count} 回");
}

fn first_char_upper(s: &str) -> Option<char> {
    // let else: 一致しなければ else 節で早期リターン (else 節は必ず抜ける)
    let Some(c) = s.chars().next() else {
        return None;
    };
    Some(c.to_ascii_uppercase())
}
```

`let else` は「ここから先は値がある前提で書きたい」ときに便利で、ネストを浅く保てます。

## enum にメソッドを付ける

enum にも `impl` でメソッドを付けられます。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    White,
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    /// 前景色を設定する ANSI エスケープシーケンスのパラメータ部分
    fn fg_code(self) -> String {
        match self {
            Color::Black => "30".into(),
            Color::Red => "31".into(),
            Color::Green => "32".into(),
            Color::Yellow => "33".into(),
            Color::Blue => "34".into(),
            Color::White => "37".into(),
            Color::Ansi256(n) => format!("38;5;{n}"),
            Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }
}

fn main() {
    for c in [Color::Red, Color::Ansi256(208), Color::Rgb(80, 160, 255)] {
        // \x1b[...m で色を変え、\x1b[0m で戻す
        println!("\x1b[{}m{:?}\x1b[0m", c.fg_code(), c);
    }
}
```

第17章では、こうした enum から実際にエスケープシーケンスを組み立てて端末を彩ります。

## 状態を enum で表す

アプリの状態も enum で表すと、「ありえない状態」を型で作れなくできます。

```rust
#[derive(Debug)]
enum Screen {
    List { selected: usize },
    Detail { index: usize, scroll: u16 },
    Confirm { message: String },
}

fn title(s: &Screen) -> String {
    match s {
        Screen::List { selected } => format!("一覧 ({selected} 番目を選択)"),
        Screen::Detail { index, scroll } => format!("詳細 #{index} (scroll {scroll})"),
        Screen::Confirm { message } => format!("確認: {message}"),
    }
}

fn main() {
    let screens = vec![
        Screen::List { selected: 2 },
        Screen::Detail { index: 2, scroll: 0 },
        Screen::Confirm { message: String::from("削除しますか?") },
    ];
    for s in &screens {
        println!("{}", title(s));
    }
}
```

`match s` で `s` が `&Screen` のとき、パターン中の `selected` などは自動的に **参照** として束縛されます
(デフォルト束縛モード)。だから `message` の `String` はムーブされません。

## よくあるコンパイルエラーと読み方

```text
error[E0004]: non-exhaustive patterns: `Key::Esc` not covered
```

`match` で扱っていないバリアントがあります。メッセージに足りないパターンが書いてあるので、
腕を追加するか、意図的にまとめるなら `_ => ...` を書きます。
ただし `_` を使うと、後からバリアントを足したときにコンパイラが警告してくれなくなる点に注意。

| コード | 状況 | 直し方 |
|--------|------|--------|
| E0004 | match が網羅的でない | 足りない腕を追加 / `_` |
| E0308 `expected Option<..>, found ..` | `Some(..)` で包み忘れ | `Some(x)` を返す |
| E0308 `expected i32, found Option<i32>` | Option のまま使った | `match` / `unwrap_or` / `?` で取り出す |
| E0507 `cannot move out of ... which is behind a shared reference` | `&Option<String>` から中身をムーブ | `as_ref()` / `as_deref()` / `ref` で借りる |
| warning `unreachable pattern` | 上の腕がすべてを受けている | 腕の順序を見直す |

## まとめ

- enum は「どれか 1 つ」を表し、バリアントごとに違うデータを持てる。
- `match` は網羅的。バリアント追加時の修正漏れをコンパイラが見つけてくれる。
- `Option<T>` で null を安全に置き換える。`map` / `unwrap_or` / `and_then` / `as_deref` / `take` が便利。
- 1 パターンだけなら `if let`、早期リターンには `let else`、判定だけなら `matches!`。
- キー・イベント・色・画面状態など、TUI の多くは enum で設計する。

## 次へ

第6章では `Vec` / `String` / `HashMap` を本格的に使い、行バッファやキーバインド表を作ります。
