//! todo: サブコマンド付き ToDo 管理 CLI (完成版)。
//!
//! - `parse_command`: 引数 → `Command` (純粋関数)
//! - `TodoList`: 追加・完了・削除などの操作と、TSV 形式との相互変換 (純粋関数)
//! - `load` / `save`: ファイル入出力
//! - `execute`: コマンドを実行し、表示する文字列を返す

use std::fmt;
use std::fs;
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
    /// サブコマンドや引数の使い方が間違っている
    Usage(String),
    /// ID が数値として解釈できない
    InvalidId(String),
    /// 指定 ID の ToDo が存在しない
    NotFound(u32),
    /// 保存ファイルの形式が壊れている
    Parse { line: usize, reason: String },
    /// ファイル入出力の失敗
    Io(io::Error),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::Usage(msg) => write!(f, "{msg}\n{USAGE}"),
            TodoError::InvalidId(s) => write!(f, "ID は正の整数で指定してください: {s:?}"),
            TodoError::NotFound(id) => write!(f, "ID {id} の ToDo はありません"),
            TodoError::Parse { line, reason } => {
                write!(f, "保存ファイルの {line} 行目が読めません: {reason}")
            }
            TodoError::Io(e) => write!(f, "ファイル入出力エラー: {e}"),
        }
    }
}

impl std::error::Error for TodoError {}

// `?` で io::Error を TodoError に自動変換できるようにする
impl From<io::Error> for TodoError {
    fn from(e: io::Error) -> Self {
        TodoError::Io(e)
    }
}

pub const USAGE: &str = "\
使い方:
  todo add <内容...>   ToDo を追加
  todo list            一覧を表示
  todo done <ID>       完了にする
  todo remove <ID>     削除する
  todo clear           完了済みをすべて削除
  todo help            この説明を表示
保存先: 環境変数 TODO_FILE (未設定なら ./todo.txt)";

fn parse_id(arg: Option<&String>, cmd: &str) -> Result<u32, TodoError> {
    let s = arg.ok_or_else(|| TodoError::Usage(format!("{cmd} には ID が必要です")))?;
    match s.parse::<u32>() {
        Ok(id) if id > 0 => Ok(id),
        _ => Err(TodoError::InvalidId(s.clone())),
    }
}

/// プログラム名を除いた引数を `Command` に変換する。
pub fn parse_command(args: &[String]) -> Result<Command, TodoError> {
    let Some((cmd, rest)) = args.split_first() else {
        return Err(TodoError::Usage("サブコマンドを指定してください".into()));
    };
    let expect_no_args = |c: Command| {
        if rest.is_empty() {
            Ok(c)
        } else {
            Err(TodoError::Usage(format!("{cmd} は引数を取りません")))
        }
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
        "list" | "ls" => expect_no_args(Command::List),
        "done" => {
            if rest.len() > 1 {
                return Err(TodoError::Usage("done の ID は 1 つだけです".into()));
            }
            parse_id(rest.first(), "done").map(Command::Done)
        }
        "remove" | "rm" => {
            if rest.len() > 1 {
                return Err(TodoError::Usage("remove の ID は 1 つだけです".into()));
            }
            parse_id(rest.first(), "remove").map(Command::Remove)
        }
        "clear" => expect_no_args(Command::Clear),
        "help" | "-h" | "--help" => Ok(Command::Help),
        other => Err(TodoError::Usage(format!("不明なサブコマンドです: {other}"))),
    }
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

    fn next_id(&self) -> u32 {
        self.items.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    /// 追加して、割り当てた ID を返す。タブと改行は保存形式を壊すので空白に置き換える。
    pub fn add(&mut self, text: &str) -> u32 {
        let id = self.next_id();
        let text = text.replace(['\t', '\n', '\r'], " ").trim().to_string();
        self.items.push(Todo { id, done: false, text });
        id
    }

    pub fn mark_done(&mut self, id: u32) -> Result<(), TodoError> {
        let todo = self
            .items
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(TodoError::NotFound(id))?;
        todo.done = true;
        Ok(())
    }

    pub fn remove(&mut self, id: u32) -> Result<Todo, TodoError> {
        let pos = self
            .items
            .iter()
            .position(|t| t.id == id)
            .ok_or(TodoError::NotFound(id))?;
        Ok(self.items.remove(pos))
    }

    /// 完了済みを削除し、削除した件数を返す。
    pub fn clear_done(&mut self) -> usize {
        let before = self.items.len();
        self.items.retain(|t| !t.done);
        before - self.items.len()
    }

    /// 保存形式 (1 行 1 件、`ID<TAB>0or1<TAB>内容`) に変換する。
    pub fn to_tsv(&self) -> String {
        self.items
            .iter()
            .map(|t| format!("{}\t{}\t{}\n", t.id, if t.done { 1 } else { 0 }, t.text))
            .collect()
    }

    /// 保存形式から読み込む。空行は無視する。
    pub fn from_tsv(s: &str) -> Result<Self, TodoError> {
        let mut items = Vec::new();
        for (i, line) in s.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let err = |reason: &str| TodoError::Parse {
                line: i + 1,
                reason: reason.to_string(),
            };
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
            items.push(Todo {
                id,
                done,
                text: text.to_string(),
            });
        }
        Ok(TodoList { items })
    }

    /// 一覧表示用の文字列。
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return "ToDo はありません".to_string();
        }
        self.items
            .iter()
            .map(|t| format!("{:>3}. [{}] {}", t.id, if t.done { "x" } else { " " }, t.text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// ファイルから読み込む。ファイルが無ければ空の一覧とみなす。
pub fn load(path: &Path) -> Result<TodoList, TodoError> {
    match fs::read_to_string(path) {
        Ok(s) => TodoList::from_tsv(&s),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(TodoList::new()),
        Err(e) => Err(e.into()),
    }
}

/// ファイルへ保存する。
pub fn save(path: &Path, list: &TodoList) -> Result<(), TodoError> {
    fs::write(path, list.to_tsv())?;
    Ok(())
}

/// コマンドを一覧に適用し、表示メッセージと「保存が必要か」を返す。
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
        Command::Remove(id) => {
            let t = list.remove(id)?;
            (format!("削除しました: {} {}", t.id, t.text), true)
        }
        Command::Clear => {
            let n = list.clear_done();
            (format!("完了済みを {n} 件削除しました"), n > 0)
        }
        Command::Help => (USAGE.to_string(), false),
    })
}

/// 読み込み → 適用 → (必要なら) 保存 までを行う。
pub fn execute(path: &Path, cmd: Command) -> Result<String, TodoError> {
    let mut list = load(path)?;
    let (msg, changed) = apply(&mut list, cmd)?;
    if changed {
        save(path, &list)?;
    }
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_subcommands() {
        assert_eq!(
            parse_command(&args(&["add", "牛乳を", "買う"])).unwrap(),
            Command::Add("牛乳を 買う".into())
        );
        assert_eq!(parse_command(&args(&["list"])).unwrap(), Command::List);
        assert_eq!(parse_command(&args(&["done", "3"])).unwrap(), Command::Done(3));
        assert_eq!(parse_command(&args(&["rm", "2"])).unwrap(), Command::Remove(2));
        assert_eq!(parse_command(&args(&["clear"])).unwrap(), Command::Clear);
        assert_eq!(parse_command(&args(&["--help"])).unwrap(), Command::Help);
    }

    #[test]
    fn parse_errors() {
        assert!(matches!(parse_command(&[]), Err(TodoError::Usage(_))));
        assert!(matches!(parse_command(&args(&["add"])), Err(TodoError::Usage(_))));
        assert!(matches!(parse_command(&args(&["done"])), Err(TodoError::Usage(_))));
        assert!(matches!(
            parse_command(&args(&["done", "abc"])),
            Err(TodoError::InvalidId(_))
        ));
        assert!(matches!(
            parse_command(&args(&["done", "0"])),
            Err(TodoError::InvalidId(_))
        ));
        assert!(matches!(parse_command(&args(&["list", "x"])), Err(TodoError::Usage(_))));
        assert!(matches!(parse_command(&args(&["fly"])), Err(TodoError::Usage(_))));
    }

    #[test]
    fn add_done_remove_clear() {
        let mut l = TodoList::new();
        assert_eq!(l.add("a"), 1);
        assert_eq!(l.add("b\tc"), 2);
        assert_eq!(l.items()[1].text, "b c");
        l.mark_done(1).unwrap();
        assert!(l.items()[0].done);
        assert!(matches!(l.mark_done(9), Err(TodoError::NotFound(9))));
        assert_eq!(l.clear_done(), 1);
        assert_eq!(l.items().len(), 1);
        assert_eq!(l.add("d"), 3); // ID は最大値 + 1
        assert_eq!(l.remove(2).unwrap().text, "b c");
        assert!(matches!(l.remove(2), Err(TodoError::NotFound(2))));
    }

    #[test]
    fn tsv_roundtrip() {
        let mut l = TodoList::new();
        l.add("一つ目");
        l.add("二つ目");
        l.mark_done(2).unwrap();
        let s = l.to_tsv();
        assert_eq!(s, "1\t0\t一つ目\n2\t1\t二つ目\n");
        assert_eq!(TodoList::from_tsv(&s).unwrap(), l);
    }

    #[test]
    fn tsv_parse_error_has_line_number() {
        let e = TodoList::from_tsv("1\t0\tok\n\nx\t0\tbad\n").unwrap_err();
        assert!(matches!(e, TodoError::Parse { line: 3, .. }));
        assert!(e.to_string().contains("3 行目"));
    }

    #[test]
    fn render_list() {
        let mut l = TodoList::new();
        assert_eq!(l.render(), "ToDo はありません");
        l.add("a");
        l.mark_done(1).unwrap();
        assert_eq!(l.render(), "  1. [x] a");
    }

    #[test]
    fn execute_persists_to_file() {
        let path = std::env::temp_dir().join(format!("p02_todo_test_{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        assert_eq!(execute(&path, Command::List).unwrap(), "ToDo はありません");
        assert!(!path.exists(), "list だけならファイルは作らない");
        execute(&path, Command::Add("x".into())).unwrap();
        execute(&path, Command::Done(1)).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "1\t1\tx\n");
        assert!(matches!(execute(&path, Command::Remove(5)), Err(TodoError::NotFound(5))));
        let _ = fs::remove_file(&path);
    }
}
