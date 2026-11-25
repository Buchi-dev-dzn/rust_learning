//! F1: wc / cat / head を自作する。
//!
//! 仕様は SPEC.md を参照。`todo!()` を埋めて `cargo test -p f1_coreutils` を通そう。
//! 関数のシグネチャ (名前・引数・戻り値) はテストが依存しているので変えないこと。
//! 補助関数は自由に追加してよい。

use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WcFlags {
    pub lines: bool,
    pub words: bool,
    pub chars: bool,
    pub bytes: bool,
}

/// 行数・単語数・文字数・バイト数を数える (SPEC 1章)。
pub fn count(input: &[u8]) -> Counts {
    // TODO: 1 バイトずつ見て数える
    todo!()
}

/// wc の 1 行分を整形する (改行なし)。
pub fn format_counts(counts: &Counts, flags: &WcFlags, name: Option<&str>) -> String {
    // TODO: flags が全部 false ならデフォルト (lines, words, bytes)
    todo!()
}

/// 先頭 n 行ぶんのスライスを返す。
pub fn head_lines(input: &[u8], n: usize) -> &[u8] {
    // TODO
    todo!()
}

/// cat -n 相当。戻り値は (出力, 次の行番号)。
pub fn number_lines(input: &[u8], start: usize) -> (Vec<u8>, usize) {
    // TODO: `split_inclusive` が便利
    todo!()
}

/// コマンド全体。args はプログラム名を除いた引数。戻り値は終了コード。
pub fn run(
    args: &[String],
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    // TODO: サブコマンドで分岐し、オプションとファイルを解析する
    todo!()
}
