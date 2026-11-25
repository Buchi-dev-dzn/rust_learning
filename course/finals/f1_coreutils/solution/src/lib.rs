//! F1: wc / cat / head の模範解答。
//!
//! I/O を `&mut dyn Read` / `&mut dyn Write` で受け取ることで、
//! 本物の標準入出力でもメモリ上のバッファでも同じコードが動く (= テストしやすい)。

use std::fs::File;
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

const USAGE: &str = "usage: f1_coreutils <wc|head|cat> [OPTIONS] [FILE...]";

fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

pub fn count(input: &[u8]) -> Counts {
    let mut c = Counts {
        bytes: input.len(),
        ..Counts::default()
    };
    let mut in_word = false;
    for &b in input {
        if b == b'\n' {
            c.lines += 1;
        }
        if b & 0xC0 != 0x80 {
            c.chars += 1;
        }
        if is_space(b) {
            in_word = false;
        } else if !in_word {
            in_word = true;
            c.words += 1;
        }
    }
    c
}

pub fn format_counts(counts: &Counts, flags: &WcFlags, name: Option<&str>) -> String {
    let mut f = *flags;
    if !(f.lines || f.words || f.chars || f.bytes) {
        f = WcFlags {
            lines: true,
            words: true,
            chars: false,
            bytes: true,
        };
    }
    let mut out = String::new();
    for (on, n) in [
        (f.lines, counts.lines),
        (f.words, counts.words),
        (f.chars, counts.chars),
        (f.bytes, counts.bytes),
    ] {
        if on {
            out.push_str(&format!("{:>8}", n));
        }
    }
    if let Some(name) = name {
        out.push(' ');
        out.push_str(name);
    }
    out
}

pub fn head_lines(input: &[u8], n: usize) -> &[u8] {
    if n == 0 {
        return &input[..0];
    }
    let mut seen = 0;
    for (i, &b) in input.iter().enumerate() {
        if b == b'\n' {
            seen += 1;
            if seen == n {
                return &input[..=i];
            }
        }
    }
    input
}

pub fn number_lines(input: &[u8], start: usize) -> (Vec<u8>, usize) {
    let mut out = Vec::with_capacity(input.len() + input.len() / 4);
    let mut n = start;
    // split_inclusive は改行を含めて分割し、最後の改行のない行もそのまま返す
    for line in input.split_inclusive(|&b| b == b'\n') {
        out.extend_from_slice(format!("{:>6}\t", n).as_bytes());
        out.extend_from_slice(line);
        n += 1;
    }
    (out, n)
}

/// 入力ソース 1 つ分を読む。`-` は標準入力。
fn read_source(path: &str, stdin: &mut dyn Read) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    if path == "-" {
        stdin.read_to_end(&mut buf)?;
    } else {
        File::open(path)?.read_to_end(&mut buf)?;
    }
    Ok(buf)
}

fn display_name(path: &str) -> &str {
    if path == "-" { "standard input" } else { path }
}

pub fn run(
    args: &[String],
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let Some(cmd) = args.first() else {
        let _ = writeln!(stderr, "{USAGE}");
        return 2;
    };
    let rest = &args[1..];
    let result = match cmd.as_str() {
        "wc" => cmd_wc(rest, stdin, stdout, stderr),
        "head" => cmd_head(rest, stdin, stdout, stderr),
        "cat" => cmd_cat(rest, stdin, stdout, stderr),
        _ => {
            let _ = writeln!(stderr, "{USAGE}");
            return 2;
        }
    };
    let _ = stdout.flush();
    result
}

fn is_option(a: &str) -> bool {
    a.len() > 1 && a.starts_with('-')
}

fn invalid_option(stderr: &mut dyn Write, a: &str) -> i32 {
    let _ = writeln!(stderr, "f1_coreutils: invalid option: {a}");
    2
}

fn files_or_stdin(files: Vec<String>) -> Vec<String> {
    if files.is_empty() { vec!["-".to_string()] } else { files }
}

fn cmd_wc(args: &[String], stdin: &mut dyn Read, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let mut flags = WcFlags::default();
    let mut files = Vec::new();
    for a in args {
        if is_option(a) {
            for ch in a[1..].chars() {
                match ch {
                    'l' => flags.lines = true,
                    'w' => flags.words = true,
                    'm' => flags.chars = true,
                    'c' => flags.bytes = true,
                    _ => return invalid_option(err, a),
                }
            }
        } else {
            files.push(a.clone());
        }
    }
    let show_total = files.len() >= 2;
    let mut code = 0;
    let mut total = Counts::default();
    for path in files_or_stdin(files) {
        match read_source(&path, stdin) {
            Ok(data) => {
                let c = count(&data);
                total.lines += c.lines;
                total.words += c.words;
                total.chars += c.chars;
                total.bytes += c.bytes;
                let name = if path == "-" { None } else { Some(path.as_str()) };
                let _ = writeln!(out, "{}", format_counts(&c, &flags, name));
            }
            Err(e) => {
                let _ = writeln!(err, "f1_coreutils: {path}: {e}");
                code = 1;
            }
        }
    }
    if show_total {
        let _ = writeln!(out, "{}", format_counts(&total, &flags, Some("total")));
    }
    code
}

fn cmd_head(args: &[String], stdin: &mut dyn Read, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let mut n = 10usize;
    let mut files = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if is_option(a) {
            if let Some(v) = a.strip_prefix("-n") {
                let value = if v.is_empty() {
                    i += 1;
                    args.get(i).cloned().unwrap_or_default()
                } else {
                    v.to_string()
                };
                match value.parse::<usize>() {
                    Ok(v) => n = v,
                    Err(_) => {
                        let _ = writeln!(err, "f1_coreutils: invalid number of lines: {value}");
                        return 2;
                    }
                }
            } else {
                return invalid_option(err, a);
            }
        } else {
            files.push(a.clone());
        }
        i += 1;
    }
    let headers = files.len() >= 2;
    let mut code = 0;
    let mut printed = 0;
    for path in files_or_stdin(files) {
        match read_source(&path, stdin) {
            Ok(data) => {
                if headers {
                    if printed > 0 {
                        let _ = out.write_all(b"\n");
                    }
                    let _ = writeln!(out, "==> {} <==", display_name(&path));
                }
                let _ = out.write_all(head_lines(&data, n));
                printed += 1;
            }
            Err(e) => {
                let _ = writeln!(err, "f1_coreutils: {path}: {e}");
                code = 1;
            }
        }
    }
    code
}

fn cmd_cat(args: &[String], stdin: &mut dyn Read, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let mut number = false;
    let mut files = Vec::new();
    for a in args {
        if is_option(a) {
            if a == "-n" {
                number = true;
            } else {
                return invalid_option(err, a);
            }
        } else {
            files.push(a.clone());
        }
    }
    let mut code = 0;
    let mut next = 1;
    for path in files_or_stdin(files) {
        match read_source(&path, stdin) {
            Ok(data) => {
                if number {
                    let (numbered, n) = number_lines(&data, next);
                    next = n;
                    let _ = out.write_all(&numbered);
                } else {
                    let _ = out.write_all(&data);
                }
            }
            Err(e) => {
                let _ = writeln!(err, "f1_coreutils: {path}: {e}");
                code = 1;
            }
        }
    }
    code
}
