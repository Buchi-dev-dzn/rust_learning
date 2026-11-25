# 第14章 並行処理 (thread, channel, Arc/Mutex)

## この章のゴール

- `std::thread::spawn` でスレッドを起動し、`join` で終了を待てる
- `move` クロージャでスレッドに値を渡す理由を説明できる
- `mpsc` チャネルで、複数のスレッドから 1 か所へイベントを送れる
- `Arc<Mutex<T>>` で状態を複数スレッドから安全に共有できる
- 「入力スレッド + tick スレッド → チャネル → メインループ」という TUI イベントループの骨格を書ける

TUI アプリは同時にいくつものことを待っています。
キー入力、一定間隔の再描画 (時計・プログレスバー)、バックグラウンドの処理 (ファイル読み込み、
コマンド実行) の完了…。これらを 1 本のループで素直に待つと、キー入力を待っている間は画面が止まってしまいます。
よく使われる解決策が **「各イベント源をスレッドにし、チャネルで 1 か所に集める」** 設計です。

```text
 [入力スレッド] --Key-->  \
 [tick スレッド] --Tick--> +--> mpsc チャネル --> [メインループ: 状態更新 → 描画]
 [作業スレッド] --Done--> /
```

## スレッドの起動と join

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("worker: {i}");
            thread::sleep(Duration::from_millis(5));
        }
        42 // スレッドの戻り値
    });

    println!("main: 待っている間も動ける");

    // join で終了を待ち、戻り値を受け取る (Result: スレッドが panic したら Err)
    let result = handle.join().unwrap();
    println!("worker の結果: {result}");
}
```

- `spawn` は `JoinHandle<T>` を返す。`join()` で終了を待つ
- `main` が終わると、他のスレッドは途中でも強制終了される。必要なら必ず `join` する
- 出力の順番はスケジューラ次第で、実行ごとに変わりうる

### move クロージャ

スレッドに外の変数を渡すときは `move` が必要です。

```rust
use std::thread;

fn main() {
    let name = String::from("loader");
    let files = vec!["a.txt", "b.txt"];

    // move しないと E0373: closure may outlive the current function
    let h = thread::spawn(move || {
        format!("{name} は {} 個のファイルを読む", files.len())
    });

    // name と files は move 済みなので、ここではもう使えない
    println!("{}", h.join().unwrap());
}
```

スレッドは「いつまで動くか分からない」ので、`main` のローカル変数を借用させられません。
`thread::spawn` のクロージャには `'static` 境界 (第10章) が付いていて、所有権ごと渡す必要があるのです。

### スコープ付きスレッド `thread::scope`

「このブロックの中で全部終わる」と保証できるなら、`thread::scope` でローカル変数を借用できます。

```rust
use std::thread;

fn main() {
    let lines: Vec<String> = (0..1000).map(|i| format!("line {i}")).collect();
    let (first, second) = lines.split_at(lines.len() / 2);

    let total = thread::scope(|s| {
        let a = s.spawn(|| first.iter().map(|l| l.len()).sum::<usize>());
        let b = s.spawn(|| second.iter().map(|l| l.len()).sum::<usize>());
        a.join().unwrap() + b.join().unwrap()
    }); // scope を抜ける時点で全スレッドの終了が保証される

    println!("合計 {total} バイト");
}
```

## チャネル (mpsc)

`std::sync::mpsc` は **multiple producer, single consumer** のチャネルです。
送信側 `Sender` は `clone` して何本でも作れ、受信側 `Receiver` は 1 つです。

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    for id in 0..3 {
        let tx = tx.clone(); // スレッドごとに Sender を複製
        thread::spawn(move || {
            for n in 0..2 {
                tx.send(format!("worker{id}: msg{n}")).unwrap();
            }
        }); // ここで tx が drop される
    }
    drop(tx); // main が持っている元の Sender も捨てる

    // すべての Sender が drop されると、for ループ (recv) が終わる
    let mut got: Vec<String> = rx.iter().collect();
    got.sort(); // 到着順は不定なので並べ替えて表示
    println!("{} 件: {:?}", got.len(), got);
}
```

受信のメソッド:

| メソッド | 動作 |
|---|---|
| `rx.recv()` | 届くまで待つ。全 Sender が消えたら `Err` |
| `rx.try_recv()` | 待たない。なければ `Err(TryRecvError::Empty)` |
| `rx.recv_timeout(d)` | 最大 `d` だけ待つ。`Err(RecvTimeoutError::Timeout)` |
| `for x in rx` / `rx.iter()` | 全 Sender が消えるまで受信し続ける |

**`drop(tx)` を忘れると、受信ループが永遠に終わりません。** よくあるハマりどころです。

## TUI イベントループの骨格

ここまでの部品を組み合わせます。本物の TUI では入力スレッドが端末からキーを読みますが、
ここでは決められたキー列を送るだけにします (標準入力は読まない)。

```rust
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Event {
    Key(char),
    Tick,
}

fn spawn_input(tx: Sender<Event>, keys: Vec<char>) {
    thread::spawn(move || {
        for k in keys {
            thread::sleep(Duration::from_millis(15)); // 人間のタイピングのつもり
            if tx.send(Event::Key(k)).is_err() {
                return; // 受信側が終了していたら抜ける
            }
        }
    });
}

fn spawn_ticker(tx: Sender<Event>, interval: Duration) {
    thread::spawn(move || {
        loop {
            thread::sleep(interval);
            if tx.send(Event::Tick).is_err() {
                return; // メインループが終わったので tick スレッドも終了
            }
        }
    });
}

struct App {
    text: String,
    ticks: u32,
    quit: bool,
}

fn main() {
    let (tx, rx) = mpsc::channel();
    spawn_input(tx.clone(), vec!['h', 'i', '!', 'q']);
    spawn_ticker(tx, Duration::from_millis(10));

    let mut app = App { text: String::new(), ticks: 0, quit: false };

    // メインループ: イベントを 1 つ受け取る → 状態を更新 → 描画
    while !app.quit {
        match rx.recv() {
            Ok(Event::Key('q')) => app.quit = true,
            Ok(Event::Key(c)) => app.text.push(c),
            Ok(Event::Tick) => app.ticks += 1,
            Err(_) => break, // 送信側が全滅
        }
        // 本物の TUI ならここで画面を描き直す
    }
    println!("text = {:?}, ticks > 0: {}", app.text, app.ticks > 0);
} // rx が drop される → tick スレッドの send が Err になり、スレッドも終わる
```

ポイント:

- 状態 (`App`) を触るのは **メインループだけ**。だからロックが要らない
- 入力スレッドと tick スレッドは「イベントを送る」だけ
- 終了は「受信側を drop → 送信側の `send` が `Err` → スレッドが自分で抜ける」で連鎖させる

この形は第19章 (crossterm) と第20章 (TUI アーキテクチャ) でそのまま使います。
`recv_timeout` を使えば、tick スレッドなしで「イベントがなければ一定時間で描画」もできます。

## 共有状態: `Arc<Mutex<T>>`

チャネルは「メッセージを渡す」方式ですが、複数スレッドから **同じデータを読み書きしたい** こともあります
(例: バックグラウンドの作業スレッドが進捗を書き、描画側がそれを読む)。

- `Arc<T>`: スレッド安全な `Rc` (Atomic Reference Counted)。共有所有する
- `Mutex<T>`: 同時に 1 スレッドだけが中身に触れるようにする鍵

```rust
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Default)]
struct Progress {
    done: usize,
    log: Vec<String>,
}

fn main() {
    let progress = Arc::new(Mutex::new(Progress::default()));
    let mut handles = Vec::new();

    for id in 0..4 {
        let p = Arc::clone(&progress);
        handles.push(thread::spawn(move || {
            for _ in 0..250 {
                let mut guard = p.lock().unwrap(); // ロック取得 (他のスレッドは待つ)
                guard.done += 1;
            } // guard が drop されるとロック解放
            p.lock().unwrap().log.push(format!("worker{id} finished"));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let p = progress.lock().unwrap();
    println!("done = {} / 1000", p.done);
    println!("log entries = {}", p.log.len());
}
```

- `lock()` は `Result<MutexGuard<T>, _>`。ロックを持ったスレッドが panic すると **毒化 (poison)** して `Err` になる
- `MutexGuard` は `Deref` で中身にアクセスでき、drop でロックを解放する
- **ロックを持ったまま重い処理や `send`・`sleep` をしない。** ガードのスコープを短くするのが鉄則
- 2 つの Mutex を逆順でロックし合うと **デッドロック** する。ロックの順番を決めておく

### 読み取りが多いなら RwLock、単純なフラグなら Atomic

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let counter = Arc::new(AtomicUsize::new(0));
    let config = Arc::new(RwLock::new(String::from("theme=dark")));

    let worker = {
        let running = Arc::clone(&running);
        let counter = Arc::clone(&counter);
        let config = Arc::clone(&config);
        thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let _cfg = config.read().unwrap(); // 読み取りは同時に何人でも
                if counter.fetch_add(1, Ordering::Relaxed) >= 999 {
                    break;
                }
            }
        })
    };

    *config.write().unwrap() = String::from("theme=light"); // 書き込みは 1 人だけ
    worker.join().unwrap();
    running.store(false, Ordering::Relaxed); // 終了フラグ (Ctrl-C ハンドラなどで使う)
    println!("count = {}", counter.load(Ordering::Relaxed));
    println!("{}", config.read().unwrap());
}
```

## Send と Sync

なぜ `Rc` はスレッドに渡せず `Arc` は渡せるのか。答えは 2 つのマーカートレイトです。

- `Send`: 値の **所有権を別スレッドに移して** よい
- `Sync`: `&T` を **複数スレッドで共有して** よい

コンパイラが自動で判定します。`Rc` や `RefCell` はスレッド安全でないため `Send`/`Sync` を実装しておらず、
`thread::spawn` に渡すとコンパイルエラーになります。**データ競合はコンパイル時に防がれる** のが Rust の並行処理の強みです。

| シングルスレッド | マルチスレッド |
|---|---|
| `Rc<T>` | `Arc<T>` |
| `RefCell<T>` | `Mutex<T>` / `RwLock<T>` |
| `Cell<usize>` | `AtomicUsize` |

## よくあるコンパイルエラーと読み方

### E0373: closure may outlive the current function

```text
error[E0373]: closure may outlive the current function, but it borrows `name`
help: to force the closure to take ownership of `name`, use the `move` keyword
```

`thread::spawn(move || ...)` にします。複数のスレッドで使うなら、先に `clone` してから move します。

### E0277: `Rc<...>` cannot be sent between threads safely

```text
error[E0277]: `Rc<Mutex<i32>>` cannot be sent between threads safely
  = help: the trait `Send` is not implemented for `Rc<Mutex<i32>>`
```

`Rc` を `Arc` に変えます。`RefCell` なら `Mutex` に変えます。

### E0382: use of moved value (ループ内の spawn)

```text
error[E0382]: use of moved value: `tx`
  |     value moved into closure here, in previous iteration of loop
```

1 回目のループで `tx` がクロージャに move されています。ループの中で `let tx = tx.clone();` してから move します。

### プログラムが終わらない (エラーではない)

- 受信ループが終わらない → どこかに `Sender` が残っている (`drop(tx)` を忘れている)
- 固まる → デッドロック。`lock()` のガードを持ったまま、同じ Mutex をもう一度 `lock()` していないか

## まとめ

- `thread::spawn(move || ...)` で起動、`join` で待つ。借用したいなら `thread::scope`
- `mpsc::channel` で複数のイベント源を 1 本の受信ループに集める。全 Sender の drop で終了
- TUI の基本形: 入力スレッド + tick スレッド → チャネル → メインループが状態を独占して更新・描画
- 共有状態は `Arc<Mutex<T>>`。ロックは短く。フラグやカウンタは Atomic
- `Send`/`Sync` のおかげで、データ競合はコンパイル時に防がれる

## 次へ

これで Stage 2 は終わりです。Stage 3 (第15章) では標準入出力・ファイル・引数・終了コードを扱い、
Unix 流儀の CLI ツールを作れるようになります。そこから先は、いよいよターミナルそのものの制御に進みます。
