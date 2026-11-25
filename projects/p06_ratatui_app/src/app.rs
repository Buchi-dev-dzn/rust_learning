//! アプリの状態とキー操作。描画 (ratatui の Widget) は `ui.rs` に分ける。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;

/// プレビューで読む最大バイト数 (巨大なファイルで固まらないように)
const PREVIEW_LIMIT: u64 = 64 * 1024;
/// ディレクトリのプレビューで表示する最大件数
const DIR_PREVIEW_LIMIT: usize = 200;

/// ディレクトリ内の 1 項目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// ディレクトリの中身を「ディレクトリが先、その中は名前順」で読む
pub fn read_entries(dir: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let item = item?;
        let path = item.path();
        // シンボリックリンクはリンク先で判定する (壊れたリンクはファイル扱い)
        let meta = fs::metadata(&path).or_else(|_| item.metadata())?;
        entries.push(Entry {
            name: item.file_name().to_string_lossy().into_owned(),
            path,
            is_dir: meta.is_dir(),
            size: meta.len(),
        });
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

/// 右ペインに表示する内容を作る
pub fn load_preview(entry: &Entry) -> String {
    if entry.is_dir {
        return match read_entries(&entry.path) {
            Ok(children) if children.is_empty() => "(空のディレクトリ)".to_string(),
            Ok(children) => {
                let mut lines: Vec<String> = children
                    .iter()
                    .take(DIR_PREVIEW_LIMIT)
                    .map(|c| if c.is_dir { format!("{}/", c.name) } else { c.name.clone() })
                    .collect();
                if children.len() > DIR_PREVIEW_LIMIT {
                    lines.push(format!("... ほか {} 件", children.len() - DIR_PREVIEW_LIMIT));
                }
                lines.join("\n")
            }
            Err(e) => format!("(読めません: {e})"),
        };
    }
    let bytes = match fs::File::open(&entry.path) {
        Ok(f) => {
            let mut buf = Vec::new();
            match io::Read::read_to_end(&mut io::Read::take(f, PREVIEW_LIMIT), &mut buf) {
                Ok(_) => buf,
                Err(e) => return format!("(読めません: {e})"),
            }
        }
        Err(e) => return format!("(読めません: {e})"),
    };
    if bytes.contains(&0) {
        return format!("(バイナリファイル: {} バイト)", entry.size);
    }
    // 途中で切ったときに末尾の UTF-8 が壊れても表示できるよう lossy で変換
    let text = String::from_utf8_lossy(&bytes);
    // タブは表示幅が定まらないので空白にしておく
    let mut text = text.replace('\t', "    ");
    if entry.size > PREVIEW_LIMIT {
        text.push_str("\n... (以下省略)");
    }
    text
}

#[derive(Debug)]
pub struct App {
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub list_state: ListState,
    pub preview: String,
    pub preview_scroll: u16,
    pub show_help: bool,
    pub should_quit: bool,
    pub status: String,
}

impl App {
    pub fn new(dir: impl AsRef<Path>) -> io::Result<Self> {
        let cwd = fs::canonicalize(dir)?;
        let mut app = App {
            cwd: PathBuf::new(),
            entries: Vec::new(),
            list_state: ListState::default(),
            preview: String::new(),
            preview_scroll: 0,
            show_help: false,
            should_quit: false,
            status: String::new(),
        };
        app.change_dir(cwd, None)?;
        Ok(app)
    }

    /// ディレクトリを移動する。`select_name` があればその項目を選択する
    fn change_dir(&mut self, dir: PathBuf, select_name: Option<&str>) -> io::Result<()> {
        let entries = read_entries(&dir)?; // 失敗したら状態を変えない
        self.cwd = dir;
        self.entries = entries;
        let index = select_name
            .and_then(|name| self.entries.iter().position(|e| e.name == name))
            .unwrap_or(0);
        self.list_state
            .select(if self.entries.is_empty() { None } else { Some(index) });
        self.update_preview();
        Ok(())
    }

    pub fn selected(&self) -> Option<&Entry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    fn update_preview(&mut self) {
        self.preview = match self.selected() {
            Some(entry) => load_preview(entry),
            None => "(空のディレクトリ)".to_string(),
        };
        self.preview_scroll = 0;
    }

    fn select(&mut self, index: usize) {
        if self.entries.is_empty() {
            return;
        }
        self.list_state.select(Some(index.min(self.entries.len() - 1)));
        self.update_preview();
    }

    /// 次の項目へ (末尾なら先頭へ戻る)
    pub fn next(&mut self) {
        let len = self.entries.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().map_or(0, |i| (i + 1) % len);
        self.select(i);
    }

    /// 前の項目へ (先頭なら末尾へ)
    pub fn previous(&mut self) {
        let len = self.entries.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().map_or(0, |i| (i + len - 1) % len);
        self.select(i);
    }

    /// 選択中のディレクトリに入る
    pub fn enter(&mut self) {
        let Some(entry) = self.selected().cloned() else {
            return;
        };
        if !entry.is_dir {
            self.status = format!("{} はファイルです (右のプレビューを J/K でスクロール)", entry.name);
            return;
        }
        match self.change_dir(entry.path, None) {
            Ok(()) => self.status.clear(),
            Err(e) => self.status = format!("開けません: {e}"),
        }
    }

    /// 親ディレクトリへ。元いたディレクトリを選択状態にする
    pub fn go_up(&mut self) {
        let Some(parent) = self.cwd.parent().map(Path::to_path_buf) else {
            self.status = "これ以上は上がれません".to_string();
            return;
        };
        let from = self
            .cwd
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        match self.change_dir(parent, from.as_deref()) {
            Ok(()) => self.status.clear(),
            Err(e) => self.status = format!("開けません: {e}"),
        }
    }

    pub fn preview_lines(&self) -> usize {
        self.preview.lines().count()
    }

    pub fn scroll_preview(&mut self, delta: i32) {
        let max = self.preview_lines().saturating_sub(1) as i32;
        let next = (self.preview_scroll as i32 + delta).clamp(0, max);
        self.preview_scroll = next as u16;
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        if self.show_help {
            // ヘルプ表示中は閉じる操作と終了だけ受け付ける
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter => self.show_help = false,
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('k') | KeyCode::Up => self.previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select(0),
            KeyCode::Char('G') | KeyCode::End => self.select(self.entries.len().saturating_sub(1)),
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.enter(),
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => self.go_up(),
            KeyCode::Char('J') => self.scroll_preview(1),
            KeyCode::Char('K') => self.scroll_preview(-1),
            KeyCode::PageDown => self.scroll_preview(10),
            KeyCode::PageUp => self.scroll_preview(-10),
            _ => {}
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// テスト用のディレクトリを作る:
    ///   root/
    ///     sub/inner.txt
    ///     a.txt   ("line1\nline2\nline3")
    ///     B.bin   (NUL を含む)
    pub(crate) fn fixture(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("p06_test_{}_{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/inner.txt"), "inner").unwrap();
        fs::write(root.join("a.txt"), "line1\nline2\nline3").unwrap();
        fs::write(root.join("B.bin"), [0u8, 1, 2, 3]).unwrap();
        root
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn entries_are_sorted_dirs_first() {
        let root = fixture("sort");
        let names: Vec<_> = read_entries(&root).unwrap().into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["sub", "a.txt", "B.bin"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn move_selection_wraps() {
        let root = fixture("wrap");
        let mut app = App::new(&root).unwrap();
        assert_eq!(app.list_state.selected(), Some(0));
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.list_state.selected(), Some(2));
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.list_state.selected(), Some(0));
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.selected().unwrap().name, "a.txt");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_follows_selection() {
        let root = fixture("preview");
        let mut app = App::new(&root).unwrap();
        assert_eq!(app.preview, "inner.txt"); // sub/ の中身
        app.next();
        assert_eq!(app.preview, "line1\nline2\nline3");
        app.next();
        assert!(app.preview.contains("バイナリ"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enter_and_go_up() {
        let root = fixture("nav");
        let mut app = App::new(&root).unwrap();
        app.handle_key(key(KeyCode::Enter));
        assert!(app.cwd.ends_with("sub"));
        assert_eq!(app.selected().unwrap().name, "inner.txt");
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.cwd, fs::canonicalize(&root).unwrap());
        // 戻ったら元いた sub が選択されている
        assert_eq!(app.selected().unwrap().name, "sub");
        // ファイルで Enter してもディレクトリは変わらない
        app.next();
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.cwd, fs::canonicalize(&root).unwrap());
        assert!(!app.status.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_scroll_is_clamped() {
        let root = fixture("scroll");
        let mut app = App::new(&root).unwrap();
        app.next(); // a.txt (3 行)
        app.handle_key(key(KeyCode::Char('J')));
        assert_eq!(app.preview_scroll, 1);
        app.handle_key(key(KeyCode::PageDown));
        assert_eq!(app.preview_scroll, 2);
        app.handle_key(key(KeyCode::PageUp));
        assert_eq!(app.preview_scroll, 0);
        // 選択を変えるとスクロールは先頭に戻る
        app.scroll_preview(1);
        app.previous();
        assert_eq!(app.preview_scroll, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn help_popup_and_quit() {
        let root = fixture("help");
        let mut app = App::new(&root).unwrap();
        app.handle_key(key(KeyCode::Char('?')));
        assert!(app.show_help);
        app.handle_key(key(KeyCode::Char('j'))); // ヘルプ中は移動しない
        assert_eq!(app.list_state.selected(), Some(0));
        app.handle_key(key(KeyCode::Esc));
        assert!(!app.show_help);
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn empty_directory() {
        let root = std::env::temp_dir().join(format!("p06_test_{}_empty", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut app = App::new(&root).unwrap();
        assert_eq!(app.list_state.selected(), None);
        app.next();
        app.enter();
        assert_eq!(app.preview, "(空のディレクトリ)");
        fs::remove_dir_all(root).unwrap();
    }
}
