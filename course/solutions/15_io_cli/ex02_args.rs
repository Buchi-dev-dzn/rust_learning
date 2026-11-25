//! # 引数解析と終了コード
//!
//! 小さな `head` 風コマンドの引数解析を作ろう。`std::env::args()` から受け取った
//! 引数 (プログラム名を除いたもの) を `&[String]` として受け取り、`Config` を返す。
//!
//! 仕様:
//! - `-n <数>` … 表示する行数 (省略時 10)
//! - `-q` … ファイル名ヘッダを出さない (`quiet = true`)
//! - `-h` / `--help` … `Err(ArgError::Help)`
//! - `-` 以外の `-` で始まる引数 … `Err(ArgError::UnknownOption(その文字列))`
//! - `-n` の後ろに値がない … `Err(ArgError::MissingValue("-n"))`
//! - `-n` の値が数値でない … `Err(ArgError::InvalidNumber(値))`
//! - それ以外 (`-` 単体を含む) … ファイル名として `files` に順番に追加
//! - ファイルが 1 つもなければ `files = vec!["-"]` (標準入力の意味)
//!
//! さらに `exit_code` で、結果を終了コードに変換する:
//! 成功 = 0、`Help` = 0、それ以外のエラー = 2 (Unix の慣習で「使い方の誤り」は 2)。
//!
//! - 関連: 第15章「コマンドライン引数」「終了コード」
//!
//! テスト (`mod tests`) は編集しないこと。

use std::process::ExitCode;

#[derive(Debug, PartialEq)]
struct Config {
    lines: usize,
    quiet: bool,
    files: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum ArgError {
    Help,
    UnknownOption(String),
    MissingValue(&'static str),
    InvalidNumber(String),
}

fn parse_args(args: &[String]) -> Result<Config, ArgError> {
    let mut cfg = Config { lines: 10, quiet: false, files: Vec::new() };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => return Err(ArgError::Help),
            "-q" => cfg.quiet = true,
            "-n" => {
                // 次の要素を値として消費する。イテレータを手で進めるのがコツ
                let v = it.next().ok_or(ArgError::MissingValue("-n"))?;
                cfg.lines = v.parse().map_err(|_| ArgError::InvalidNumber(v.clone()))?;
            }
            "-" => cfg.files.push(a.clone()),
            s if s.starts_with('-') => return Err(ArgError::UnknownOption(s.to_string())),
            _ => cfg.files.push(a.clone()),
        }
    }
    if cfg.files.is_empty() {
        cfg.files.push("-".to_string());
    }
    Ok(cfg)
}

fn exit_code(r: &Result<Config, ArgError>) -> u8 {
    match r {
        Ok(_) | Err(ArgError::Help) => 0,
        Err(_) => 2,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = parse_args(&args);
    match &r {
        Ok(cfg) => println!("{cfg:?}"),
        Err(ArgError::Help) => println!("usage: head [-n N] [-q] [FILE...]"),
        Err(e) => eprintln!("error: {e:?}"),
    }
    ExitCode::from(exit_code(&r))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn defaults() {
        assert_eq!(
            parse_args(&s(&[])),
            Ok(Config { lines: 10, quiet: false, files: s(&["-"]) })
        );
    }

    #[test]
    fn options_and_files() {
        assert_eq!(
            parse_args(&s(&["-n", "3", "a.txt", "-q", "b.txt"])),
            Ok(Config { lines: 3, quiet: true, files: s(&["a.txt", "b.txt"]) })
        );
    }

    #[test]
    fn dash_is_a_file() {
        assert_eq!(parse_args(&s(&["-"])).unwrap().files, s(&["-"]));
    }

    #[test]
    fn errors() {
        assert_eq!(parse_args(&s(&["--help"])), Err(ArgError::Help));
        assert_eq!(parse_args(&s(&["a", "-h"])), Err(ArgError::Help));
        assert_eq!(parse_args(&s(&["-x"])), Err(ArgError::UnknownOption("-x".into())));
        assert_eq!(parse_args(&s(&["-n"])), Err(ArgError::MissingValue("-n")));
        assert_eq!(parse_args(&s(&["-n", "abc"])), Err(ArgError::InvalidNumber("abc".into())));
    }

    #[test]
    fn exit_codes() {
        assert_eq!(exit_code(&parse_args(&s(&["a"]))), 0);
        assert_eq!(exit_code(&parse_args(&s(&["-h"]))), 0);
        assert_eq!(exit_code(&parse_args(&s(&["-x"]))), 2);
        assert_eq!(exit_code(&parse_args(&s(&["-n"]))), 2);
    }
}
