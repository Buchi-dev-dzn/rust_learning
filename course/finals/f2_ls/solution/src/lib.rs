//! F2: カラー対応の ls の模範解答。

use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
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
    /// テスト・手作業用の簡易コンストラクタ。
    pub fn new(name: &str, kind: Kind) -> Entry {
        let mode = match kind {
            Kind::Dir | Kind::Executable => 0o755,
            Kind::Symlink => 0o777,
            Kind::File => 0o644,
        };
        Entry {
            name: name.to_string(),
            kind,
            mode,
            size: 0,
            nlink: 1,
            link_target: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Options {
    pub all: bool,
    pub long: bool,
    pub color: bool,
}

pub fn char_width(c: char) -> usize {
    let u = c as u32;
    if u < 0x20 || (0x7F..0xA0).contains(&u) {
        return 0;
    }
    if (0x0300..=0x036F).contains(&u) || u == 0x200B {
        return 0;
    }
    let wide = matches!(u,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD);
    if wide { 2 } else { 1 }
}

pub fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

pub fn pad_right(s: &str, width: usize) -> String {
    let w = str_width(s);
    let mut out = s.to_string();
    for _ in w..width {
        out.push(' ');
    }
    out
}

pub fn colorize(entry: &Entry) -> String {
    let code = match entry.kind {
        Kind::Dir => "1;34",
        Kind::Executable => "1;32",
        Kind::Symlink => "1;36",
        Kind::File => return entry.name.clone(),
    };
    format!("\x1b[{code}m{}\x1b[0m", entry.name)
}

fn display(entry: &Entry, color: bool) -> String {
    if color { colorize(entry) } else { entry.name.clone() }
}

pub fn format_grid(entries: &[Entry], term_width: usize, color: bool) -> Vec<String> {
    let n = entries.len();
    if n == 0 {
        return Vec::new();
    }
    let widths: Vec<usize> = entries.iter().map(|e| str_width(&e.name)).collect();
    // 列数を多い方から試し、収まる最初の配置を採用する
    let mut layout = (n, vec![*widths.iter().max().unwrap()]); // (rows, col_widths) for c=1
    for c in (1..=n).rev() {
        let rows = n.div_ceil(c);
        let cols = n.div_ceil(rows);
        let col_widths: Vec<usize> = (0..cols)
            .map(|j| {
                widths[j * rows..((j + 1) * rows).min(n)]
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let total: usize = col_widths.iter().sum::<usize>() + 2 * (cols - 1);
        if total <= term_width || c == 1 {
            layout = (rows, col_widths);
            break;
        }
    }
    let (rows, col_widths) = layout;
    let cols = col_widths.len();
    let mut lines = Vec::with_capacity(rows);
    for r in 0..rows {
        let mut line = String::new();
        for (j, &cw) in col_widths.iter().enumerate() {
            let idx = j * rows + r;
            if idx >= n {
                break;
            }
            let e = &entries[idx];
            line.push_str(&display(e, color));
            let is_last = j + 1 == cols || (j + 1) * rows + r >= n;
            if !is_last {
                for _ in widths[idx]..cw + 2 {
                    line.push(' ');
                }
            }
        }
        lines.push(line);
    }
    lines
}

pub fn mode_string(entry: &Entry) -> String {
    let mut s = String::with_capacity(10);
    s.push(match entry.kind {
        Kind::Dir => 'd',
        Kind::Symlink => 'l',
        _ => '-',
    });
    let bits = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];
    for (bit, ch) in bits {
        s.push(if entry.mode & bit != 0 { ch } else { '-' });
    }
    s
}

pub fn format_long(entries: &[Entry], color: bool) -> Vec<String> {
    let lw = entries.iter().map(|e| e.nlink.to_string().len()).max().unwrap_or(0);
    let sw = entries.iter().map(|e| e.size.to_string().len()).max().unwrap_or(0);
    entries
        .iter()
        .map(|e| {
            let mut line = format!(
                "{} {:>lw$} {:>sw$} {}",
                mode_string(e),
                e.nlink,
                e.size,
                display(e, color)
            );
            if let Some(t) = &e.link_target {
                line.push_str(" -> ");
                line.push_str(t);
            }
            line
        })
        .collect()
}

pub fn read_entries(dir: &Path, all: bool) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        if !all && name.starts_with('.') {
            continue;
        }
        let meta = fs::symlink_metadata(item.path())?;
        let ft = meta.file_type();
        let mode = meta.permissions().mode() & 0o777;
        let (kind, link_target) = if ft.is_symlink() {
            let t = fs::read_link(item.path())
                .map(|p| p.to_string_lossy().into_owned())
                .ok();
            (Kind::Symlink, t)
        } else if ft.is_dir() {
            (Kind::Dir, None)
        } else if mode & 0o111 != 0 {
            (Kind::Executable, None)
        } else {
            (Kind::File, None)
        };
        entries.push(Entry {
            name,
            kind,
            mode,
            size: meta.len(),
            nlink: meta.nlink(),
            link_target,
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub fn parse_args(args: &[String]) -> Result<(Options, Vec<String>), String> {
    let mut opts = Options::default();
    let mut paths = Vec::new();
    for a in args {
        if a.len() > 1 && a.starts_with('-') {
            for ch in a[1..].chars() {
                match ch {
                    'a' => opts.all = true,
                    'l' => opts.long = true,
                    _ => return Err(format!("invalid option: -{ch}")),
                }
            }
        } else {
            paths.push(a.clone());
        }
    }
    if paths.is_empty() {
        paths.push(".".to_string());
    }
    Ok((opts, paths))
}
