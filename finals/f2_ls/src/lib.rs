//! F2: カラー対応の ls。
//!
//! 仕様は SPEC.md。`todo!()` を埋めて `cargo test -p f2_ls` を通そう。
//! 型と関数のシグネチャはテストが依存しているので変えないこと。

use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Dir,
    Symlink,
    Executable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub mode: u32,
    pub size: u64,
    pub nlink: u64,
    pub link_target: Option<String>,
}

impl Entry {
    /// 既定値で Entry を作る (SPEC 1章)。
    pub fn new(name: &str, kind: Kind) -> Entry {
        // TODO
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Options {
    pub all: bool,
    pub long: bool,
    pub color: bool,
}

/// 1 文字の表示幅 (0, 1, 2)。SPEC の表に従う。
pub fn char_width(c: char) -> usize {
    // TODO: `matches!(c as u32, 0x1100..=0x115F | ...)` が便利
    todo!()
}

pub fn str_width(s: &str) -> usize {
    // TODO
    todo!()
}

pub fn pad_right(s: &str, width: usize) -> String {
    // TODO
    todo!()
}

pub fn colorize(entry: &Entry) -> String {
    // TODO
    todo!()
}

/// 列優先の複数列表示。1 要素 = 1 行。
pub fn format_grid(entries: &[Entry], term_width: usize, color: bool) -> Vec<String> {
    // TODO: 列数 c を n から 1 まで試す
    todo!()
}

pub fn mode_string(entry: &Entry) -> String {
    // TODO
    todo!()
}

pub fn format_long(entries: &[Entry], color: bool) -> Vec<String> {
    // TODO
    todo!()
}

pub fn read_entries(dir: &Path, all: bool) -> io::Result<Vec<Entry>> {
    // TODO: fs::read_dir + fs::symlink_metadata
    //       (std::os::unix::fs::PermissionsExt / MetadataExt を use すると mode() / nlink() が使える)
    todo!()
}

pub fn parse_args(args: &[String]) -> Result<(Options, Vec<String>), String> {
    // TODO
    todo!()
}
