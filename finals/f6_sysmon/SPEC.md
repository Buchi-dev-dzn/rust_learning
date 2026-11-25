# F6: htop 風システムモニタ (ratatui)

プロセス一覧と CPU 使用率の履歴グラフを表示する、htop 風のモニタを ratatui で作る。

```
┌ CPU 合計 23.4% ─────────────────────────────┐
│        ▂▃▅▂▁▁▂▇█▅▃▂                         │
└─────────────────────────────────────────────┘
┌ プロセス 412/412 ───────────────────────────┐
│PID    CPU%▼ MEM%  COMMAND                   │
│812     12.3   1.2 /Applications/...         │  ← 選択行は反転
│...                                          │
└─────────────────────────────────────────────┘
 q:終了 j/k:移動 c/m/p/n:ソート /:絞り込み Space:一時停止
```

この課題で身につくこと: **状態 (App) と描画・I/O の分離**、データ源の注入 (Snapshot)、
スレッド + チャネルでのバックグラウンド取得、ソート・フィルタ・選択の維持・スクロールといった
「リスト UI の定番ロジック」、リングバッファ。

外部クレートは `ratatui` のみ (`ratatui::crossterm` で crossterm も使える)。
受け入れテストは `src/lib.rs` の**状態ロジックだけ**を対象にする。

## 1. 構成

- `src/lib.rs` … データ型・`parse_ps`・`History`・`App` (テスト対象)。**端末にもプロセス取得にも依存しない**。
- `src/main.rs` … 取得スレッド (`ps` を 1 秒ごとに実行して `Snapshot` を mpsc で送る) とイベントループと描画。
  starter にはイベントループと取得スレッドが用意してあり、描画 (`ui` 関数) だけが TODO。

## 2. データと `parse_ps`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo { pub pid: u32, pub cpu: f32, pub mem: f32, pub command: String }
impl ProcessInfo { pub fn new(pid: u32, cpu: f32, mem: f32, command: &str) -> ProcessInfo; }

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Snapshot { pub processes: Vec<ProcessInfo> }
impl Snapshot { pub fn total_cpu(&self) -> f32; } // 全プロセスの cpu の合計

pub fn parse_ps(output: &str) -> Vec<ProcessInfo>;
```

`parse_ps` は `ps -axo pid,pcpu,pmem,comm` の出力を解析する。各行について:

- 空白区切りで先頭から 3 フィールド (pid, cpu, mem) を取り出し、**残り全体を trim したもの**を command とする
  (command は空白を含んでよい: `/Applications/Google Chrome.app/...`)。
- pid が `u32`、cpu と mem が `f32` として解析できない行 (ヘッダ行 `PID %CPU ...` など) は**飛ばす**。
- フィールドが足りない行、command が空の行も飛ばす。
- 順序は入力の順。

## 3. `History` — CPU 履歴のリングバッファ

```rust
pub fn new(capacity: usize) -> History;
pub fn push(&mut self, v: f32);        // 満杯なら最も古い値を捨ててから追加。capacity 0 なら何もしない
pub fn len(&self) -> usize;
pub fn is_empty(&self) -> bool;
pub fn capacity(&self) -> usize;
pub fn values(&self) -> Vec<f32>;      // 古い順
pub fn latest(&self) -> Option<f32>;   // 最新の値
pub fn max(&self) -> f32;              // 最大値。空なら 0.0
pub fn sparkline_data(&self) -> Vec<u64>; // 古い順に、各値を 0 未満は 0 にして round() した u64
```

## 4. `App`

```rust
pub enum SortKey { Pid, Cpu, Mem, Command }
pub enum Mode { Normal, Filter }
pub enum Input { Char(char), Up, Down, PageUp, PageDown, Home, End, Enter, Esc, Backspace }
```

(すべて `Debug, Clone, Copy, PartialEq, Eq`)

### 4.1 初期状態 `App::new(history_capacity)`

プロセスなし、履歴は容量 `history_capacity` で空、ソートは `Cpu` の降順 (`sort_desc() == true`)、
フィルタ空、`Mode::Normal`、一時停止していない、`should_quit() == false`、`selected() == 0`、スクロール位置 0。

読み取り用メソッド: `history()`, `sort_key()`, `sort_desc()`, `filter()`, `mode()`, `paused()`,
`should_quit()`, `selected()` (表示リスト中の選択 index), `process_count()` (フィルタ前の総数)。

### 4.2 表示リスト `visible(&self) -> Vec<&ProcessInfo>`

1. **フィルタ**: `filter` が空でなければ、`command` に `filter` を**大文字小文字を区別せず**部分文字列として含むものだけ。
2. **ソート**: `sort_key` で比較 (Command は `String` の通常の順序)。`sort_desc` なら逆順。
   **キーが等しいときは常に pid の昇順** (降順ソートでも pid は昇順)。

`selected_process()` は `visible()` の `selected` 番目 (空なら `None`)。

### 4.3 状態更新

- `update(snapshot)`:
  - 一時停止中 (`paused()`) なら**何もしない** (プロセスも履歴も変えない)。
  - そうでなければ、`snapshot.total_cpu()` を履歴に push し、プロセス一覧を `snapshot.processes` で**置き換える**。
- `set_sort(key)`: 現在と同じキーなら昇順/降順を反転。違うキーなら、そのキーにして既定の向き
  (Cpu・Mem は降順、Pid・Command は昇順) にする。
- `set_filter(s)`: フィルタ文字列を `s` にする。

**選択の維持**: 上の 3 つの操作の後、表示リストが変わっても、
操作前に選択していたプロセスと**同じ pid** が新しい表示リストにあれば、その位置を選択する。
なければ、操作前の index を `表示リストの長さ - 1` で頭打ちにしたもの (空なら 0) を選択する。

### 4.4 選択とスクロール

- `select_next` / `select_prev`: 1 つ下/上。端で止まる (ラップしない)。
- `page_down(n)` / `page_up(n)`: n 個下/上。端で止まる。
- `select_first` / `select_last`: 先頭 / 末尾 (空なら 0)。
- `scroll_offset(height) -> usize`: 表示行数 `height` のとき、何行目から表示するかを返す。
  App は前回のオフセットを覚えておき、選択行が見えるように**最小限**だけ動かす:
  - `height == 0` なら前回の値をそのまま返す。
  - `selected < offset` なら `offset = selected`。
  - `selected >= offset + height` なら `offset = selected + 1 - height`。
  - それ以外は変えない。

### 4.5 キー入力 `handle_input(input, page_size)`

モードに関係なく: `Up`→`select_prev`, `Down`→`select_next`, `PageUp`→`page_up(page_size)`,
`PageDown`→`page_down(page_size)`, `Home`→`select_first`, `End`→`select_last`。

Normal モード:

| 入力 | 動作 |
|---|---|
| `Char('q')` | `should_quit` を true に |
| `Char('j')` / `Char('k')` | 下 / 上 |
| `Char('g')` / `Char('G')` | 先頭 / 末尾 |
| `Char('c')` `Char('m')` `Char('p')` `Char('n')` | `set_sort(Cpu / Mem / Pid / Command)` |
| `Char('/')` | Filter モードへ |
| `Char(' ')` | 一時停止の切り替え |
| `Esc` | フィルタを空に (`set_filter("")`) |
| その他 | 無視 |

Filter モード:

| 入力 | 動作 |
|---|---|
| `Char(c)` (どの文字でも。`q` も) | フィルタの末尾に `c` を追加 (`set_filter` と同じく選択を維持) |
| `Backspace` | フィルタの最後の 1 文字を削除 |
| `Enter` | フィルタを保ったまま Normal へ |
| `Esc` | フィルタを空にして Normal へ |
| その他 | 無視 |

## 5. `src/main.rs` (テスト対象外)

- `ratatui::init()` / `ratatui::restore()` で端末を準備・復元 (init はパニック時の復元フックも入れる)。
- 取得スレッド: `Command::new("ps").args(["-axo", "pid,pcpu,pmem,comm"]).env("LC_ALL", "C")` を 1 秒ごとに実行し、
  `parse_ps` した `Snapshot` を `mpsc::Sender` で送る。
- メインループ: `try_recv` で届いたスナップショットを `update` → `terminal.draw(...)` →
  `event::poll(100ms)` でキーがあれば `handle_input`。`should_quit()` か Ctrl-C で抜ける。
- 描画: 上に `Sparkline` (履歴)、中央に `Table` (`visible()` を `scroll_offset` から表示し、選択行を反転、
  ソート中の列見出しに ▼/▲)、下にステータス行 (Filter モードなら `/フィルタ`)。

## 合格条件

`cargo test -p f6_sysmon` がすべて成功すること。完成したら `cargo run -p f6_sysmon` で動かしてみよう。
