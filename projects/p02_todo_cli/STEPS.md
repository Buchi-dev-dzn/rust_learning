# todo: サブコマンド付き CLI

`git` や `cargo` のように **サブコマンド** を持つ ToDo 管理ツールを、引数解析ライブラリ (clap) を使わずに作ります。
データはテキストファイル (タブ区切り / TSV) に保存し、次に起動したときも残っています。

## 完成イメージ

```text
$ cargo run -q -p p02_todo_cli -- add 牛乳を買う
追加しました: 1
$ cargo run -q -p p02_todo_cli -- add Rust の本を読む
追加しました: 2
$ cargo run -q -p p02_todo_cli -- done 1
完了にしました: 1
$ cargo run -q -p p02_todo_cli -- list
  1. [x] 牛乳を買う
  2. [ ] Rust の本を読む
$ cargo run -q -p p02_todo_cli -- clear
完了済みを 1 件削除しました
$ cargo run -q -p p02_todo_cli -- done 9
todo: ID 9 の ToDo はありません
```

| サブコマンド | 動作 |
|---|---|
| `add <内容...>` | 追加 (残りの引数を空白でつなげて 1 件に) |
| `list` (`ls`) | 一覧 |
| `done <ID>` | 完了にする |
| `remove <ID>` (`rm`) | 削除 |
| `clear` | 完了済みをまとめて削除 |
| `help` | 使い方 |

保存先は環境変数 `TODO_FILE`、未設定ならカレントディレクトリの `todo.txt` です。
学習サイトのボタンから実行するとカレントディレクトリはリポジトリのルートなので、そこに `todo.txt` ができます
(`.gitignore` には入っていないので、気になる場合は `TODO_FILE=/tmp/todo.txt` を使うか、終わったら削除してください)。
`list` だけならファイルは作られません。

終了コード: `0` = 成功、`1` = 実行時エラー (ID が無い・ファイルが壊れている等)、`2` = 使い方の誤り。

## 学ぶこと

- `enum` でコマンドとエラーを表現し、`match` で網羅的に処理する
- スライスパターン `split_first` / `let-else`、`str::parse` による数値変換
- 独自エラー型に `Display` と `From<io::Error>` を実装して `?` を気持ちよく使う
- `Vec` の `iter_mut().find` / `position` / `retain`
- 簡単なファイル形式の設計 (シリアライズ / デシリアライズ) と、そのラウンドトリップテスト

作業は `projects/p02_todo_cli/starter/` で行います。テストは `cargo test -p p02_todo_cli_starter`。

## STEP 1: 設計を読む

やること
- `starter/src/lib.rs` の型 (`Command`, `TodoError`, `Todo`, `TodoList`) と関数の役割を確認する。

ポイント
- 処理は 3 層に分かれています。
  1. `parse_command`: `&[String]` → `Command` (ファイルに触れない)
  2. `TodoList` のメソッド: メモリ上の一覧を操作・TSV 文字列と相互変換 (ファイルに触れない)
  3. `load` / `save` / `execute`: ファイル入出力
- 1 と 2 は純粋な関数なので、一時ファイルなしで簡単にテストできます。I/O を端に寄せる設計は TUI でも同じです。

確認方法
- `cargo build -p p02_todo_cli_starter` が通る。

## STEP 2: サブコマンドを解析する

やること
- `parse_command` を実装する。先頭をサブコマンド名、残りを引数として扱う。
- ID は `u32` に変換し、0 や数値でないものは `InvalidId`。
- `TodoError` の `Display` を実装する。

コード例
```rust,noplayground
pub fn parse_command(args: &[String]) -> Result<Command, TodoError> {
    let Some((cmd, rest)) = args.split_first() else {
        return Err(TodoError::Usage("サブコマンドを指定してください".into()));
    };
    match cmd.as_str() {
        "add" => {
            let text = rest.join(" ");
            if text.trim().is_empty() {
                Err(TodoError::Usage("add には内容が必要です".into()))
            } else {
                Ok(Command::Add(text))
            }
        }
        "list" | "ls" => Ok(Command::List),
        "done" => parse_id(rest.first(), "done").map(Command::Done),
        // remove / clear / help も同様に
        other => Err(TodoError::Usage(format!("不明なサブコマンドです: {other}"))),
    }
}

fn parse_id(arg: Option<&String>, cmd: &str) -> Result<u32, TodoError> {
    let s = arg.ok_or_else(|| TodoError::Usage(format!("{cmd} には ID が必要です")))?;
    match s.parse::<u32>() {
        Ok(id) if id > 0 => Ok(id),
        _ => Err(TodoError::InvalidId(s.clone())),
    }
}
```

ポイント
- `let Some(x) = ... else { return ...; };` (let-else) は「失敗したら早期 return」を 1 文で書けます。
- `.map(Command::Done)` — タプル型バリアントは関数としても使えます (`|id| Command::Done(id)` と同じ)。
- 余分な引数 (`list foo`) もエラーにすると親切です (完成版参照)。

確認方法
- `cargo test -p p02_todo_cli_starter step2` が通る。

## STEP 3: 一覧を操作する

やること
- `add`, `mark_done`, `remove`, `clear_done`, `render` を実装する。
- ID は「既存の最大 ID + 1」。削除しても番号を詰めない (ユーザが覚えた番号が変わらないように)。

コード例
```rust,noplayground
pub fn add(&mut self, text: &str) -> u32 {
    let id = self.items.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    // タブや改行は保存形式を壊すので空白に
    let text = text.replace(['\t', '\n', '\r'], " ").trim().to_string();
    self.items.push(Todo { id, done: false, text });
    id
}

pub fn mark_done(&mut self, id: u32) -> Result<(), TodoError> {
    let todo = self.items.iter_mut().find(|t| t.id == id).ok_or(TodoError::NotFound(id))?;
    todo.done = true;
    Ok(())
}

pub fn clear_done(&mut self) -> usize {
    let before = self.items.len();
    self.items.retain(|t| !t.done);
    before - self.items.len()
}
```

ポイント
- `iter_mut()` は `&mut Todo` を返すので、見つけた要素をその場で書き換えられます。
- `render` は `format!("{:>3}. [{}] {}", ...)` で ID を右寄せ 3 桁に。`collect::<Vec<_>>().join("\n")` でつなぎます。

確認方法
- `cargo test -p p02_todo_cli_starter step3` が通る。

## STEP 4: TSV 形式で保存・復元する

やること
- `to_tsv`: 1 件を `ID<TAB>0 or 1<TAB>内容\n` に。
- `from_tsv`: 逆変換。空行は無視し、壊れた行は `Parse { line, reason }` (行番号は 1 始まり)。

コード例
```rust,noplayground
for (i, line) in s.lines().enumerate() {
    if line.trim().is_empty() {
        continue;
    }
    let err = |reason: &str| TodoError::Parse { line: i + 1, reason: reason.to_string() };
    let mut cols = line.splitn(3, '\t');
    let id = cols
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .ok_or_else(|| err("ID が数値ではありません"))?;
    let done = match cols.next() {
        Some("0") => false,
        Some("1") => true,
        _ => return Err(err("完了フラグは 0 か 1 です")),
    };
    let text = cols.next().ok_or_else(|| err("内容がありません"))?;
    items.push(Todo { id, done, text: text.to_string() });
}
```

ポイント
- `splitn(3, '\t')` は最大 3 つに分けるので、もし内容にタブが混ざっても 3 列目に全部入ります。
- 「書き出して読み戻すと元に戻る」ラウンドトリップテストは、ファイル形式を作るときの定番テストです。

確認方法
- `cargo test -p p02_todo_cli_starter step4` が通る。

## STEP 5: ファイルの読み書きと From<io::Error>

やること
- `impl From<io::Error> for TodoError` を書く。
- `load`: ファイルを読み `from_tsv`。**ファイルが無いのはエラーではなく空の一覧**。
- `save`: `fs::write(path, list.to_tsv())?`。
- `main.rs` の `todo_path` で `TODO_FILE` を読む。

コード例
```rust,noplayground
impl From<io::Error> for TodoError {
    fn from(e: io::Error) -> Self {
        TodoError::Io(e)
    }
}

pub fn load(path: &Path) -> Result<TodoList, TodoError> {
    match fs::read_to_string(path) {
        Ok(s) => TodoList::from_tsv(&s),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(TodoList::new()),
        Err(e) => Err(e.into()),
    }
}

fn todo_path() -> PathBuf {
    env::var_os("TODO_FILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("todo.txt"))
}
```

ポイント
- `From` を実装すると、`?` が `io::Error` を自動で `TodoError` に変換してくれます。
- `io::Error::kind()` でエラーの種類を見分けられます。

確認方法
- `cargo build -p p02_todo_cli_starter` が通る (ここでは主にコンパイルが通ることを確認)。

## STEP 6: コマンドを実行する

やること
- `apply`: `Command` ごとに一覧を操作し、`(表示メッセージ, 保存が必要か)` を返す。
- `execute`: `load` → `apply` → 変更があれば `save`。

コード例
```rust,noplayground
pub fn apply(list: &mut TodoList, cmd: Command) -> Result<(String, bool), TodoError> {
    Ok(match cmd {
        Command::Add(text) => {
            let id = list.add(&text);
            (format!("追加しました: {id}"), true)
        }
        Command::List => (list.render(), false),
        Command::Done(id) => {
            list.mark_done(id)?;
            (format!("完了にしました: {id}"), true)
        }
        // Remove / Clear / Help
        _ => todo!(),
    })
}

pub fn execute(path: &Path, cmd: Command) -> Result<String, TodoError> {
    let mut list = load(path)?;
    let (msg, changed) = apply(&mut list, cmd)?;
    if changed {
        save(path, &list)?;
    }
    Ok(msg)
}
```

ポイント
- `match` が網羅的でないとコンパイルエラーになります。`Command` にバリアントを足したとき、
  直し忘れをコンパイラが教えてくれるのが enum の強みです。

確認方法
- `cargo test -p p02_todo_cli_starter` の全テストが通る。

## STEP 7: main でエラーを報告する

やること
- `main` の `unwrap` をやめ、`parse_command(&args).and_then(|cmd| execute(&todo_path(), cmd))` の結果を `match` する。
- エラーは `eprintln!("todo: {e}")`。`Usage` / `InvalidId` は終了コード 2、それ以外は 1。

コード例
```rust,noplayground
match parse_command(&args).and_then(|cmd| execute(&todo_path(), cmd)) {
    Ok(msg) => {
        println!("{msg}");
        ExitCode::SUCCESS
    }
    Err(e) => {
        eprintln!("todo: {e}");
        match e {
            TodoError::Usage(_) | TodoError::InvalidId(_) => ExitCode::from(2),
            _ => ExitCode::FAILURE,
        }
    }
}
```

確認方法
```text
$ export TODO_FILE=/tmp/my_todo.txt
$ cargo run -q -p p02_todo_cli_starter -- add テスト
$ cargo run -q -p p02_todo_cli_starter -- list
$ cargo run -q -p p02_todo_cli_starter -- done 99; echo $?    # → 1
$ cargo run -q -p p02_todo_cli_starter -- fly; echo $?        # → 2 と使い方
$ cat /tmp/my_todo.txt                                       # タブ区切りで保存されている
```
完成版 `cargo run -p p02_todo_cli -- ...` と同じ動きになれば完成です。

## 発展課題

- `edit <ID> <内容...>` と `undone <ID>` を追加する。
- 保存を「一時ファイルに書いてから `fs::rename`」にして、書き込み途中で落ちてもデータが壊れないようにする。
- 期限 (`--due 2025-01-31`) や優先度の列を追加し、古い形式のファイルも読めるようにする (後方互換)。
- `list` で完了済みを ANSI エスケープで灰色表示する (`\x1b[90m` ... `\x1b[0m`)。
