//! アプリの状態とキー操作 (starter)。描画は `ui.rs` に分ける。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;

/// プレビューで読む最大バイト数
const PREVIEW_LIMIT: u64 = 64 * 1024;

/// ディレクトリ内の 1 項目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// ディレクトリの中身を「ディレクトリが先、その中は名前順 (大文字小文字を区別しない)」で読む
pub fn read_entries(dir: &Path) -> io::Result<Vec<Entry>> {
    // TODO(STEP 2): fs::read_dir で Entry を集め、sort_by で並べ替える
    todo!()
}

/// 右ペインに表示する内容を作る
pub fn load_preview(entry: &Entry) -> String {
    // TODO(STEP 4): ディレクトリなら子の名前一覧 (空なら "(空のディレクトリ)")、
    //               ファイルなら先頭 PREVIEW_LIMIT バイト。NUL を含めば "(バイナリファイル: N バイト)"
    todo!()
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
        // TODO(STEP 2): read_entries で読んでから (失敗したら何も変えずに Err)、
        //               cwd / entries を更新し、list_state.select(...) する
        // TODO(STEP 4): 最後に update_preview()
        // いまは cwd を覚えるだけの仮実装 (STEP 1 で App::new がパニックしないように)
        self.cwd = dir;
        Ok(())
    }

    pub fn selected(&self) -> Option<&Entry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    fn update_preview(&mut self) {
        // TODO(STEP 4): selected() の load_preview を self.preview に入れ、スクロールを 0 に戻す
    }

    fn select(&mut self, index: usize) {
        // TODO(STEP 3): 範囲内に収めて list_state.select(Some(..))、update_preview()
        todo!()
    }

    /// 次の項目へ (末尾なら先頭へ戻る)
    pub fn next(&mut self) {
        // TODO(STEP 3)
        todo!()
    }

    /// 前の項目へ (先頭なら末尾へ)
    pub fn previous(&mut self) {
        // TODO(STEP 3)
        todo!()
    }

    /// 選択中のディレクトリに入る
    pub fn enter(&mut self) {
        // TODO(STEP 5): ディレクトリなら change_dir。ファイルなら status にメッセージ
        todo!()
    }

    /// 親ディレクトリへ。元いたディレクトリを選択状態にする
    pub fn go_up(&mut self) {
        // TODO(STEP 5): cwd.parent() へ change_dir(parent, Some(元のディレクトリ名))
        todo!()
    }

    pub fn preview_lines(&self) -> usize {
        self.preview.lines().count()
    }

    pub fn scroll_preview(&mut self, delta: i32) {
        // TODO(STEP 4): 0..=(行数 - 1) の範囲で preview_scroll を増減
        todo!()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        // TODO(STEP 6): show_help 中は ? / Esc で閉じる、q で終了、それ以外は無視
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            // TODO(STEP 3): j/k/↓/↑ で next/previous、g/G で先頭/末尾
            // TODO(STEP 4): J/K/PageDown/PageUp でプレビューをスクロール
            // TODO(STEP 5): Enter/l で enter、Backspace/h で go_up
            // TODO(STEP 6): ? でヘルプ
            _ => {}
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn fixture(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("p06_starter_test_{}_{name}", std::process::id()));
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
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_follows_selection() {
        let root = fixture("preview");
        let mut app = App::new(&root).unwrap();
        assert_eq!(app.preview, "inner.txt");
        app.next();
        assert_eq!(app.preview, "line1\nline2\nline3");
        app.next();
        assert!(app.preview.contains("バイナリ"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_scroll_is_clamped() {
        let root = fixture("scroll");
        let mut app = App::new(&root).unwrap();
        app.next();
        app.handle_key(key(KeyCode::PageDown));
        assert_eq!(app.preview_scroll, 2);
        app.handle_key(key(KeyCode::PageUp));
        assert_eq!(app.preview_scroll, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enter_and_go_up() {
        let root = fixture("nav");
        let mut app = App::new(&root).unwrap();
        app.handle_key(key(KeyCode::Enter));
        assert!(app.cwd.ends_with("sub"));
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.selected().unwrap().name, "sub");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn help_popup_and_quit() {
        let root = fixture("help");
        let mut app = App::new(&root).unwrap();
        app.handle_key(key(KeyCode::Char('?')));
        assert!(app.show_help);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.list_state.selected(), Some(0));
        app.handle_key(key(KeyCode::Esc));
        assert!(!app.show_help);
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
        fs::remove_dir_all(root).unwrap();
    }
}
