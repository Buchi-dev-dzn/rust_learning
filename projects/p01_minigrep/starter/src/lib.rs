//! minigrep: 小さな grep クローン (starter)。
//!
//! STEPS.md の順に `TODO(STEP n)` を埋めていこう。
//! `cargo test -p p01_minigrep_starter` でテストを実行できる。

use std::error::Error;
use std::fmt;

/// 実行時の設定。
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
        // TODO(STEP 2): 各バリアントに日本語のメッセージを付ける (match self { ... })
        write!(f, "{self:?}")
    }
}

impl Error for ArgsError {}

pub const USAGE: &str = "使い方: minigrep [-i] [-n] <query> <file>";

impl Config {
    /// `args` はプログラム名を除いた引数。
    pub fn build<I>(args: I, ignore_case_env: bool) -> Result<Config, ArgsError>
    where
        I: IntoIterator<Item = String>,
    {
        // TODO(STEP 2): 位置引数 2 つ (query, file) を取り出す。足りなければ Err。
        // TODO(STEP 4): -i / -n オプションと ignore_case_env を反映する。
        let _ = (args, ignore_case_env);
        todo!("Config::build")
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
    // TODO(STEP 3): contents.lines().enumerate() で各行を調べる
    let _ = (query, contents);
    todo!("search")
}

/// 大文字小文字を無視して検索する。
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<Match<'a>> {
    // TODO(STEP 4): to_lowercase() を使って比較する
    let _ = (query, contents);
    todo!("search_case_insensitive")
}

/// 1 行分の出力を作る。line_numbers が true なら "3:本文" の形式。
pub fn format_match(m: &Match<'_>, line_numbers: bool) -> String {
    // TODO(STEP 5): 行番号付き表示
    let _ = line_numbers;
    m.line.to_string()
}

/// ファイルを読んで検索し、一致した行 (表示用文字列) を返す。
pub fn run(config: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    // TODO(STEP 3): fs::read_to_string → search → format_match
    let _ = config;
    todo!("run")
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
    fn step2_build_basic() {
        let c = Config::build(args(&["foo", "a.txt"]), false).unwrap();
        assert_eq!(c.query, "foo");
        assert_eq!(c.file_path, "a.txt");
        assert_eq!(Config::build(args(&["foo"]), false), Err(ArgsError::MissingFile));
    }

    #[test]
    fn step3_case_sensitive() {
        let r = search("duct", CONTENTS);
        assert_eq!(r, vec![Match { line_no: 2, line: "safe, fast, productive." }]);
    }

    #[test]
    fn step4_case_insensitive() {
        let r = search_case_insensitive("rUsT", CONTENTS);
        let lines: Vec<_> = r.iter().map(|m| m.line_no).collect();
        assert_eq!(lines, vec![1, 5]);
        let c = Config::build(args(&["-i", "foo", "a.txt"]), false).unwrap();
        assert!(c.ignore_case);
    }

    #[test]
    fn step5_format() {
        let m = Match { line_no: 3, line: "abc" };
        assert_eq!(format_match(&m, true), "3:abc");
    }
}
