# 第20章 TUI アーキテクチャ (イベントループ / 状態 / 描画)

## この章のゴール

- TUI アプリを「状態 (Model)・メッセージ (Message)・更新 (update)・描画 (view)」に分けて設計できる
- 入力とタイマー (tick) を 1 つのイベントループで扱える
- スレッドとチャネル (第14章) で入力待ちと定期処理を両立できる
- 端末に依存しない「純粋なロジック」を切り出して単体テストできる
- 必要なときだけ再描画する、副作用 (保存など) を update の外に出す、といった実践的な工夫を知る

小さな TUI なら `main` に全部書いても動きます。しかし機能が増えると、キー処理と描画と状態が
絡み合ってすぐに手に負えなくなります。この章の構造は、ratatui (第21章) でも自作フレームワーク
(第22章) でも、卒業課題でもそのまま使います。

## TUI プログラムの骨格

どんな TUI も、本質的には次のループです。

```
初期化 (raw モード, 代替スクリーン)
loop {
    画面を描く          ← 状態 → 画面
    イベントを待つ       ← キー, リサイズ, タイマー
    状態を更新する       ← イベント → 新しい状態
    終了なら break
}
後片付け
```

このうち「状態を更新する」と「状態から画面を作る」を **純粋な関数** にするのがポイントです。
純粋な関数 = 引数だけから結果が決まり、端末やファイルに触らない関数。テストが簡単に書けます。

## Elm アーキテクチャ

Web フレームワーク Elm に由来する設計で、Rust の TUI 界隈 (ratatui の公式ガイドなど) でも定番です。

| 要素 | Rust での形 | 役割 |
|---|---|---|
| Model | `struct App { ... }` | アプリの状態すべて |
| Message | `enum Msg { ... }` | 「何が起きたか」 (キー入力を解釈したもの、タイマー等) |
| update | `fn update(&mut self, msg: Msg)` | Message を受けて Model を変える。唯一の状態変更場所 |
| view | `fn view(&self, frame)` | Model を読んで画面を作る。状態は変えない |

```
            ┌────────────── view(&model) ──────────────┐
            │                                          ▼
     ┌─────────────┐                            ┌──────────┐
     │    Model    │                            │   画面    │
     └─────────────┘                            └──────────┘
            ▲                                          │ キー入力
            └──── update(&mut model, msg) ◀── Msg ◀────┘ (map_key)
```

データが一方向にしか流れないので、「どこで状態が変わったか」を追うのが簡単になります。

```rust
#[derive(Debug, Default)]
struct Model {
    count: i64,
    quit: bool,
}

#[derive(Debug, Clone, Copy)]
enum Msg {
    Increment,
    Decrement,
    Reset,
    Quit,
}

fn update(m: &mut Model, msg: Msg) {
    match msg {
        Msg::Increment => m.count += 1,
        Msg::Decrement => m.count -= 1,
        Msg::Reset => m.count = 0,
        Msg::Quit => m.quit = true,
    }
}

fn view(m: &Model) -> String {
    format!("count = {}  (+/- で増減, r でリセット, q で終了)", m.count)
}

/// 入力層: キー (ここでは char) → Msg。知らないキーは None
fn map_key(c: char) -> Option<Msg> {
    match c {
        '+' => Some(Msg::Increment),
        '-' => Some(Msg::Decrement),
        'r' => Some(Msg::Reset),
        'q' => Some(Msg::Quit),
        _ => None,
    }
}

fn main() {
    let mut model = Model::default();
    // 本物の TUI ならここが event::read() になる。今回はキー列で代用
    for key in "++x+-q+".chars() {
        if let Some(msg) = map_key(key) {
            update(&mut model, msg);
        }
        println!("{}", view(&model));
        if model.quit {
            break;
        }
    }
}
```

`main` を見ると、端末に関わる部分は「キーを読む」と「文字列を表示する」だけです。
`update` と `view` は端末なしでテストできます。

```rust,noplayground
#[test]
fn reset_sets_zero() {
    let mut m = Model { count: 5, quit: false };
    update(&mut m, Msg::Reset);
    assert_eq!(m.count, 0);
    assert!(view(&m).contains("count = 0"));
}
```

## キー → Message を分ける理由

キー入力を直接 `update` に渡さず、いったん `Msg` に変換するのはなぜでしょう。

- **同じ操作を複数のキーに割り当てられる** (`↓` と `j` がどちらも `Msg::Down`)
- **モードによって解釈を変えられる** (挿入モードでは `j` は文字入力)
- **キー以外からも同じ操作を起こせる** (マウスのスクロールで `Msg::Down`、タイマーで `Msg::Tick`)
- **テストが読みやすい** (`update(&mut m, Msg::Down)` は `KeyEvent { code: Down, ... }` より意図が明確)

第19章の `map_key` がまさにこの変換層です。

## イベントループ: 入力とタイマー

時計・プログレスバー・システムモニタのように、キーを押さなくても画面が変わるアプリでは、
「キー入力を待つ」と「一定時間ごとに更新する」を両立させる必要があります。方法は 2 つです。

### 方法 1: poll のタイムアウト

crossterm の `event::poll(timeout)` で「次の tick までの残り時間」だけ待ちます。

```rust,noplayground
let tick_rate = Duration::from_millis(250);
let mut last_tick = Instant::now();
loop {
    terminal.draw(|f| view(&app, f))?;

    let timeout = tick_rate.saturating_sub(last_tick.elapsed());
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if let Some(msg) = map_key(key) {
                app.update(msg);
            }
        }
    }
    if last_tick.elapsed() >= tick_rate {
        app.update(Msg::Tick);
        last_tick = Instant::now();
    }
    if app.quit {
        break;
    }
}
```

シンプルで、多くの ratatui アプリがこの形です。

### 方法 2: スレッド + チャネル

入力スレッド・タイマースレッド・(必要なら) バックグラウンド処理スレッドが、
すべて 1 本のチャネルに `Event` を送り、メインループはそれを受け取るだけにします。

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Event {
    Key(char),
    Tick,
    Loaded(String), // バックグラウンド処理の結果
}

fn main() {
    let (tx, rx) = mpsc::channel::<Event>();

    // 入力スレッド (本物なら event::read() をループ。ここではキー列を流す)
    let tx_key = tx.clone();
    thread::spawn(move || {
        for c in ['a', 'b', 'q'] {
            thread::sleep(Duration::from_millis(30));
            if tx_key.send(Event::Key(c)).is_err() {
                break; // メインが終了した
            }
        }
    });

    // タイマースレッド
    let tx_tick = tx.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(20));
        if tx_tick.send(Event::Tick).is_err() {
            break;
        }
    });

    // 重い処理 (ファイル読み込みなど) も別スレッドで
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        let _ = tx.send(Event::Loaded("data.txt の中身".into()));
    });

    let mut ticks = 0;
    for ev in rx {
        match ev {
            Event::Tick => ticks += 1,
            Event::Key('q') => {
                println!("終了 (tick は {ticks} 回くらい)");
                break;
            }
            other => println!("{other:?}"),
        }
    }
}
```

メインループが受け取る `Event` を 1 つの enum にまとめると、`update` は
「何がどこから来たか」を気にせず処理できます。重い処理を別スレッドにすれば、
その間も画面が固まりません (卒業課題 F6 のシステムモニタで使います)。

> 注意: crossterm の `event::read()` を複数のスレッドから同時に呼んではいけません。
> 入力を読むのは 1 スレッドだけにしましょう。

## 必要なときだけ描く

毎回のループで描き直すのは簡単ですが、tick が速いと CPU を無駄に使います。
`update` が「画面が変わったか」を返すようにすれば、変化があったときだけ `view` を呼べます。

```rust,noplayground
let dirty = app.update(msg); // bool を返す
if dirty {
    terminal.draw(|f| view(&app, f))?;
}
```

ratatui や第22章の自作フレームワークは内部で差分描画をするので、毎回描いても
端末への出力量は小さく抑えられます。それでも `view` 自体の計算が重いアプリでは有効です。
(演習 ex03 で実装します。)

## 副作用を update から追い出す

「保存」「ファイルを開く」「外部コマンドを実行」のような副作用を `update` の中で直接行うと、
テストでファイルが書かれてしまいます。代わりに **やるべきこと (Command / Effect) を返す** 形にします。

```rust
#[derive(Debug, PartialEq)]
enum Effect {
    None,
    SaveFile { path: String, contents: String },
    Quit,
}

struct Editor {
    path: String,
    text: String,
    dirty: bool,
}

enum Msg {
    Insert(char),
    Save,
    Quit,
}

impl Editor {
    fn update(&mut self, msg: Msg) -> Effect {
        match msg {
            Msg::Insert(c) => {
                self.text.push(c);
                self.dirty = true;
                Effect::None
            }
            Msg::Save => {
                self.dirty = false;
                Effect::SaveFile { path: self.path.clone(), contents: self.text.clone() }
            }
            Msg::Quit => Effect::Quit,
        }
    }
}

fn main() {
    let mut ed = Editor { path: "memo.txt".into(), text: String::new(), dirty: false };
    for msg in [Msg::Insert('h'), Msg::Insert('i'), Msg::Save, Msg::Quit] {
        match ed.update(msg) {
            Effect::None => {}
            // 本番ではここで std::fs::write(path, contents) を呼ぶ
            Effect::SaveFile { path, contents } => println!("save {path}: {contents:?}"),
            Effect::Quit => break,
        }
    }
    println!("dirty = {}", ed.dirty);
}
```

テストでは「`Msg::Save` を送ったら `Effect::SaveFile` が返る」ことだけを確かめればよく、
実際のファイル操作は `main` 側の小さなコードに閉じ込められます。

## 状態の設計のコツ

- **表示に必要な情報はすべて Model に入れる**。`view` が Model 以外 (グローバル変数・現在時刻など) を
  読むと、テストで結果が変わってしまう。時刻が必要なら `Msg::Tick(now)` で渡して Model に保存する
- **ありえない状態を型で防ぐ**。「編集中かどうか」と「入力中の文字列」を別々のフィールドにするより、
  `enum Mode { Normal, Editing { input: String } }` にすると、編集中でないのに入力がある状態を作れない
- **インデックスの範囲を update で守る**。リストの選択位置は、削除したり件数が 0 になったときに
  範囲外にならないよう、`update` の中で必ず補正する (`selected.min(len - 1)` など)
- **画面サイズも Model に持つ**。`Msg::Resize(w, h)` で更新し、スクロール量の計算などに使う

## よくあるコンパイルエラーと読み方

**`error[E0502]: cannot borrow `app` as mutable because it is also borrowed as immutable`**
→ `for item in &app.items { app.update(...) }` のように、読みながら書こうとしています。
Elm アーキテクチャでは update の外で Model を借用し続けないようにします。必要な値を先にコピーしましょう。

**`error[E0004]: non-exhaustive patterns: `Msg::Tick` not covered`**
→ `Msg` にバリアントを追加したのに `update` の `match` を直していません。これは **Rust の強み** です。
新しい Message を足すと、処理し忘れの場所をコンパイラが全部教えてくれます。

**`error[E0382]: use of moved value: `tx``**
→ チャネルの送信側を複数スレッドに渡すときは `tx.clone()` してから `move` します。

**`error[E0373]: closure may outlive the current function, but it borrows ...`**
→ `thread::spawn` のクロージャに `move` を付け忘れています。

## まとめ

- Model (状態) / Msg (何が起きたか) / update (唯一の状態変更) / view (状態 → 画面) に分ける
- キー → Msg の変換層を挟むと、キー割り当て・モード・テストが楽になる
- tick とキー入力は `poll(timeout)` か「スレッド + チャネル」で 1 つのループにまとめる
- 副作用は `Effect` として返し、実行は main 側で行う
- 端末に触らない部分はすべて単体テストできる

## 次へ

第21章では ratatui を使い、`view` の中身 (レイアウト・枠・リスト・段落) を宣言的に書く方法を学びます。
