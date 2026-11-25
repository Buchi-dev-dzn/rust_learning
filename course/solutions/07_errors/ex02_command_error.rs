//! # 独自エラー型を設計する
//!
//! Vim 風のコマンドライン (`:q`, `:w file`, `:goto 12`) を解析する関数と、そのエラー型を作ろう。
//!
//! - ゴール: エラー enum を設計し、`Display` / `std::error::Error` / `From` を実装して `?` で使う
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第7章「独自のエラー型」「From と ? による自動変換」
//!
//! 仕様:
//! - 入力は ':' で始まる。始まらなければ `CmdError::NoColon`。
//! - ':' の後を空白で区切り、最初の語がコマンド名。
//!   - `q` / `quit` → `Command::Quit` (引数があってもよい。無視する)
//!   - `w` / `write` → `Command::Write(Option<String>)` (ファイル名は任意)
//!   - `goto` → `Command::Goto(usize)`。引数がなければ `CmdError::MissingArg("goto")`、
//!     数値として読めなければ `CmdError::BadNumber(ParseIntError)` (← `From` で自動変換すること)
//!   - コマンド名が空なら `CmdError::Empty`
//!   - それ以外は `CmdError::Unknown(コマンド名)`
//! - `Display` の文言 (テストで確認する):
//!   - NoColon      → "コマンドは ':' で始めてください"
//!   - Empty        → "コマンドが空です"
//!   - Unknown(c)   → "不明なコマンド: c"
//!   - MissingArg(c)→ "c には引数が必要です"
//!   - BadNumber(e) → "数値が不正です: e"   (e は ParseIntError の Display)

use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
enum Command {
    Quit,
    Write(Option<String>),
    Goto(usize),
}

#[derive(Debug, PartialEq)]
enum CmdError {
    NoColon,
    Empty,
    Unknown(String),
    MissingArg(String),
    BadNumber(ParseIntError), // 元のエラーを包んで保持する
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CmdError::NoColon => write!(f, "コマンドは ':' で始めてください"),
            CmdError::Empty => write!(f, "コマンドが空です"),
            CmdError::Unknown(c) => write!(f, "不明なコマンド: {c}"),
            CmdError::MissingArg(c) => write!(f, "{c} には引数が必要です"),
            CmdError::BadNumber(e) => write!(f, "数値が不正です: {e}"),
        }
    }
}

impl std::error::Error for CmdError {}

impl From<ParseIntError> for CmdError {
    fn from(e: ParseIntError) -> Self {
        CmdError::BadNumber(e)
    }
}

fn parse_command(line: &str) -> Result<Command, CmdError> {
    let body = line.strip_prefix(':').ok_or(CmdError::NoColon)?;
    let mut words = body.split_whitespace();
    let name = words.next().ok_or(CmdError::Empty)?;
    let arg = words.next();
    match name {
        "q" | "quit" => Ok(Command::Quit),
        "w" | "write" => Ok(Command::Write(arg.map(|s| s.to_string()))),
        "goto" => {
            let arg = arg.ok_or_else(|| CmdError::MissingArg(name.to_string()))?;
            let n: usize = arg.parse()?; // ParseIntError → CmdError (From による自動変換)
            Ok(Command::Goto(n))
        }
        other => Err(CmdError::Unknown(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands() {
        assert_eq!(parse_command(":q"), Ok(Command::Quit));
        assert_eq!(parse_command(":quit now"), Ok(Command::Quit));
        assert_eq!(parse_command(":w"), Ok(Command::Write(None)));
        assert_eq!(parse_command(":write  out.txt"), Ok(Command::Write(Some("out.txt".into()))));
        assert_eq!(parse_command(":goto 12"), Ok(Command::Goto(12)));
    }

    #[test]
    fn errors() {
        assert_eq!(parse_command("q"), Err(CmdError::NoColon));
        assert_eq!(parse_command(":"), Err(CmdError::Empty));
        assert_eq!(parse_command(":   "), Err(CmdError::Empty));
        assert_eq!(parse_command(":fly"), Err(CmdError::Unknown("fly".into())));
        assert_eq!(parse_command(":goto"), Err(CmdError::MissingArg("goto".into())));
        assert!(matches!(parse_command(":goto x"), Err(CmdError::BadNumber(_))));
    }

    #[test]
    fn displays() {
        assert_eq!(CmdError::NoColon.to_string(), "コマンドは ':' で始めてください");
        assert_eq!(CmdError::Empty.to_string(), "コマンドが空です");
        assert_eq!(CmdError::Unknown("fly".into()).to_string(), "不明なコマンド: fly");
        assert_eq!(CmdError::MissingArg("goto".into()).to_string(), "goto には引数が必要です");
        let e = parse_command(":goto -3").unwrap_err();
        assert_eq!(e.to_string(), "数値が不正です: invalid digit found in string");
    }

    #[test]
    fn is_std_error() {
        // std::error::Error を実装していれば Box<dyn Error> に入れられる
        let boxed: Box<dyn std::error::Error> = Box::new(CmdError::Empty);
        assert_eq!(boxed.to_string(), "コマンドが空です");
    }
}
