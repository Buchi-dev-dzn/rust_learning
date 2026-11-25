//! todo: サブコマンド付き ToDo 管理 CLI (starter)。
//!
//! STEPS.md の順に `TODO(STEP n)` を埋めていこう。
//! `cargo test -p p02_todo_cli_starter` でテストを実行できる。

use std::fmt;
use std::io;
use std::path::Path;

/// 解析済みのサブコマンド。
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add(String),
    List,
    Done(u32),
    Remove(u32),
    Clear,
    Help,
}

/// このツールで起こりうるエラー。
#[derive(Debug)]
pub enum TodoError {
    Usage(String),
    InvalidId(String),
    NotFound(u32),
    Parse { line: usize, reason: String },
    Io(io::Error),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(STEP 2): バリアントごとに日本語メッセージを書く
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TodoError {}

// TODO(STEP 5): impl From<io::Error> for TodoError を書いて `?` で変換できるようにする

pub const USAGE: &str = "\
使い方:
  todo add <内容...>   ToDo を追加
  todo list            一覧を表示
  todo done <ID>       完了にする
  todo remove <ID>     削除する
  todo clear           完了済みをすべて削除
  todo help            この説明を表示
保存先: 環境変数 TODO_FILE (未設定なら ./todo.txt)";

/// プログラム名を除いた引数を `Command` に変換する。
pub fn parse_command(args: &[String]) -> Result<Command, TodoError> {
    // TODO(STEP 2): args.split_first() で先頭のサブコマンドを取り出し、match で振り分ける
    let _ = args;
    todo!("parse_command")
}

/// ToDo 1 件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: u32,
    pub done: bool,
    pub text: String,
}

/// ToDo の一覧。ID は削除しても再利用しない。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TodoList {
    items: Vec<Todo>,
}

impl TodoList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(&self) -> &[Todo] {
        &self.items
    }

    /// 追加して、割り当てた ID (既存の最大 ID + 1) を返す。
    pub fn add(&mut self, text: &str) -> u32 {
        // TODO(STEP 3)
        let _ = text;
        todo!("add")
    }

    pub fn mark_done(&mut self, id: u32) -> Result<(), TodoError> {
        // TODO(STEP 3): iter_mut().find(...) を使う。無ければ NotFound
        let _ = id;
        todo!("mark_done")
    }

    pub fn remove(&mut self, id: u32) -> Result<Todo, TodoError> {
        // TODO(STEP 3): iter().position(...) と Vec::remove
        let _ = id;
        todo!("remove")
    }

    /// 完了済みを削除し、削除した件数を返す。
    pub fn clear_done(&mut self) -> usize {
        // TODO(STEP 3): Vec::retain
        todo!("clear_done")
    }

    /// 保存形式 (1 行 1 件、`ID<TAB>0or1<TAB>内容`) に変換する。
    pub fn to_tsv(&self) -> String {
        // TODO(STEP 4)
        todo!("to_tsv")
    }

    /// 保存形式から読み込む。空行は無視する。
    pub fn from_tsv(s: &str) -> Result<Self, TodoError> {
        // TODO(STEP 4): splitn(3, '\t') で列に分ける。壊れていたら Parse { line, reason }
        let _ = s;
        todo!("from_tsv")
    }

    /// 一覧表示用の文字列。
    pub fn render(&self) -> String {
        // TODO(STEP 3): 空なら "ToDo はありません"、それ以外は "  1. [x] 内容" を改行でつなぐ
        todo!("render")
    }
}

/// ファイルから読み込む。ファイルが無ければ空の一覧とみなす。
pub fn load(path: &Path) -> Result<TodoList, TodoError> {
    // TODO(STEP 5): fs::read_to_string。ErrorKind::NotFound は空の一覧に
    let _ = path;
    todo!("load")
}

/// ファイルへ保存する。
pub fn save(path: &Path, list: &TodoList) -> Result<(), TodoError> {
    // TODO(STEP 5): fs::write
    let _ = (path, list);
    todo!("save")
}

/// コマンドを一覧に適用し、表示メッセージと「保存が必要か」を返す。
pub fn apply(list: &mut TodoList, cmd: Command) -> Result<(String, bool), TodoError> {
    // TODO(STEP 6): match cmd { ... }
    let _ = (list, cmd);
    todo!("apply")
}

/// 読み込み → 適用 → (必要なら) 保存 までを行う。
pub fn execute(path: &Path, cmd: Command) -> Result<String, TodoError> {
    // TODO(STEP 6)
    let _ = (path, cmd);
    todo!("execute")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn step2_parse() {
        assert_eq!(
            parse_command(&args(&["add", "牛乳を", "買う"])).unwrap(),
            Command::Add("牛乳を 買う".into())
        );
        assert_eq!(parse_command(&args(&["done", "3"])).unwrap(), Command::Done(3));
        assert!(matches!(parse_command(&args(&["done", "x"])), Err(TodoError::InvalidId(_))));
        assert!(matches!(parse_command(&args(&["fly"])), Err(TodoError::Usage(_))));
    }

    #[test]
    fn step3_operations() {
        let mut l = TodoList::new();
        assert_eq!(l.add("a"), 1);
        assert_eq!(l.add("b"), 2);
        l.mark_done(1).unwrap();
        assert!(matches!(l.mark_done(9), Err(TodoError::NotFound(9))));
        assert_eq!(l.clear_done(), 1);
        assert_eq!(l.add("c"), 3);
        assert_eq!(l.remove(2).unwrap().text, "b");
        assert_eq!(l.render(), "  3. [ ] c");
    }

    #[test]
    fn step4_tsv() {
        let mut l = TodoList::new();
        l.add("一つ目");
        l.add("二つ目");
        l.mark_done(2).unwrap();
        let s = l.to_tsv();
        assert_eq!(s, "1\t0\t一つ目\n2\t1\t二つ目\n");
        assert_eq!(TodoList::from_tsv(&s).unwrap(), l);
        assert!(matches!(
            TodoList::from_tsv("x\t0\tbad\n"),
            Err(TodoError::Parse { line: 1, .. })
        ));
    }

    #[test]
    fn step6_execute() {
        let path = std::env::temp_dir().join(format!("p02_starter_test_{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        execute(&path, Command::Add("x".into())).unwrap();
        execute(&path, Command::Done(1)).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "1\t1\tx\n");
        let _ = std::fs::remove_file(&path);
    }
}
