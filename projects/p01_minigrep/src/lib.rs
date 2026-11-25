//! minigrep: 小さな grep クローン (完成版)。
//!
//! ロジック (引数解析・検索) はこの lib.rs に置き、main.rs は
//! 「引数を集める → run を呼ぶ → 結果に応じて終了コードを返す」だけにする。

use std::error::Error;
use std::fmt;
use std::fs;

/// 実行時の設定。コマンドライン引数と環境変数から作る。
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub line_numbers: bool,
}

/// 引数の解析エラー。
#[derive(Debug, PartialEq, Eq)]
pub enum ArgsError {
    MissingQuery,
    MissingFile,
    UnknownOption(String),
    TooManyArgs,
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgsError::MissingQuery => write!(f, "検索文字列が指定されていません"),
            ArgsError::MissingFile => write!(f, "ファイル名が指定されていません"),
            ArgsError::UnknownOption(o) => write!(f, "不明なオプションです: {o}"),
            ArgsError::TooManyArgs => write!(f, "引数が多すぎます"),
        }
    }
}

impl Error for ArgsError {}

pub const USAGE: &str = "使い方: minigrep [-i] [-n] <query> <file>";

impl Config {
    /// `args` はプログラム名を除いた引数。`ignore_case_env` は環境変数 IGNORE_CASE が
    /// 設定されているかどうか (テストしやすいよう外から渡す)。
    pub fn build<I>(args: I, ignore_case_env: bool) -> Result<Config, ArgsError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut ignore_case = ignore_case_env;
        let mut line_numbers = false;
        let mut positional = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-i" | "--ignore-case" => ignore_case = true,
                "-n" | "--line-number" => line_numbers = true,
                s if s.starts_with('-') && s.len() > 1 => {
                    return Err(ArgsError::UnknownOption(arg));
                }
                _ => positional.push(arg),
            }
        }

        let mut it = positional.into_iter();
        let query = it.next().ok_or(ArgsError::MissingQuery)?;
        let file_path = it.next().ok_or(ArgsError::MissingFile)?;
        if it.next().is_some() {
            return Err(ArgsError::TooManyArgs);
        }

        Ok(Config {
            query,
            file_path,
            ignore_case,
            line_numbers,
        })
    }
}

/// 一致した行。行番号は 1 始まり。
#[derive(Debug, PartialEq, Eq)]
pub struct Match<'a> {
    pub line_no: usize,
    pub line: &'a str,
}

/// 大文字小文字を区別して検索する。
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<Match<'a>> {
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(query))
        .map(|(i, line)| Match { line_no: i + 1, line })
        .collect()
}

/// 大文字小文字を無視して検索する。
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<Match<'a>> {
    let query = query.to_lowercase();
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| line.to_lowercase().contains(&query))
        .map(|(i, line)| Match { line_no: i + 1, line })
        .collect()
}

/// 1 行分の出力を作る。
pub fn format_match(m: &Match<'_>, line_numbers: bool) -> String {
    if line_numbers {
        format!("{}:{}", m.line_no, m.line)
    } else {
        m.line.to_string()
    }
}

/// ファイルを読んで検索し、一致した行を返す (表示は呼び出し側)。
pub fn run(config: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)
        .map_err(|e| format!("{}: {e}", config.file_path))?;

    let matches = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    Ok(matches
        .iter()
        .map(|m| format_match(m, config.line_numbers))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENTS: &str = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.
Trust me.";

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn case_sensitive() {
        let r = search("duct", CONTENTS);
        assert_eq!(
            r,
            vec![Match {
                line_no: 2,
                line: "safe, fast, productive."
            }]
        );
    }

    #[test]
    fn case_insensitive() {
        let r = search_case_insensitive("rUsT", CONTENTS);
        let lines: Vec<_> = r.iter().map(|m| m.line_no).collect();
        assert_eq!(lines, vec![1, 5]);
    }

    #[test]
    fn no_match_is_empty() {
        assert!(search("zzz", CONTENTS).is_empty());
    }

    #[test]
    fn build_basic() {
        let c = Config::build(args(&["foo", "a.txt"]), false).unwrap();
        assert_eq!(
            c,
            Config {
                query: "foo".into(),
                file_path: "a.txt".into(),
                ignore_case: false,
                line_numbers: false
            }
        );
    }

    #[test]
    fn build_flags_and_env() {
        let c = Config::build(args(&["-n", "foo", "a.txt"]), true).unwrap();
        assert!(c.ignore_case);
        assert!(c.line_numbers);
        let c = Config::build(args(&["foo", "-i", "a.txt"]), false).unwrap();
        assert!(c.ignore_case);
    }

    #[test]
    fn build_errors() {
        assert_eq!(Config::build(args(&[]), false), Err(ArgsError::MissingQuery));
        assert_eq!(Config::build(args(&["foo"]), false), Err(ArgsError::MissingFile));
        assert_eq!(
            Config::build(args(&["a", "b", "c"]), false),
            Err(ArgsError::TooManyArgs)
        );
        assert_eq!(
            Config::build(args(&["-x", "a", "b"]), false),
            Err(ArgsError::UnknownOption("-x".into()))
        );
    }

    #[test]
    fn format_with_line_numbers() {
        let m = Match { line_no: 3, line: "abc" };
        assert_eq!(format_match(&m, true), "3:abc");
        assert_eq!(format_match(&m, false), "abc");
    }

    #[test]
    fn run_missing_file_is_error() {
        let c = Config::build(args(&["x", "/no/such/file"]), false).unwrap();
        assert!(run(&c).is_err());
    }
}
