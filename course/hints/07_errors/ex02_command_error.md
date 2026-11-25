テストで使われているバリアントから型を逆算しましょう: `NoColon`, `Empty`, `Unknown(String)`,
`MissingArg(String)`, `BadNumber(ParseIntError)`。`#[derive(Debug, PartialEq)]` を付けます。

---

- `Display` は `impl fmt::Display for CmdError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self { ... => write!(f, "..."), } } }`。
- `impl std::error::Error for CmdError {}` は中身なしで OK。
- `From`: `impl From<ParseIntError> for CmdError { fn from(e: ParseIntError) -> Self { CmdError::BadNumber(e) } }`。
- 解析: `line.strip_prefix(':').ok_or(CmdError::NoColon)?` → `body.split_whitespace()` から `next()` でコマンド名 (なければ `Empty`) と引数。

---

```rust
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
            let n: usize = arg.parse()?;
            Ok(Command::Goto(n))
        }
        other => Err(CmdError::Unknown(other.to_string())),
    }
}
```
