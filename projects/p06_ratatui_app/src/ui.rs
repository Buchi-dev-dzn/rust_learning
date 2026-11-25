//! 描画。`App` の状態を読んで ratatui のウィジェットを組み立てるだけ。

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};

use crate::app::App;

const HELP: &[(&str, &str)] = &[
    ("j / ↓", "次の項目"),
    ("k / ↑", "前の項目"),
    ("g / G", "先頭 / 末尾"),
    ("Enter / l", "ディレクトリを開く"),
    ("Backspace / h", "親ディレクトリへ"),
    ("J / K", "プレビューを 1 行スクロール"),
    ("PageDown / PageUp", "プレビューを 10 行スクロール"),
    ("?", "このヘルプを開く / 閉じる"),
    ("q", "終了"),
];

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(body);

    draw_header(frame, header, app);
    draw_list(frame, left, app);
    draw_preview(frame, right, app);
    draw_footer(frame, footer, app);
    if app.show_help {
        draw_help(frame);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let line = Line::from(vec![
        Span::from(" ファイルエクスプローラ ").bold(),
        Span::from(app.cwd.display().to_string()),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::new().fg(Color::Black).bg(Color::Cyan)),
        area,
    );
}

fn draw_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|e| {
            if e.is_dir {
                ListItem::new(format!("{}/", e.name)).style(Style::new().fg(Color::Blue).bold())
            } else {
                ListItem::new(e.name.as_str())
            }
        })
        .collect();
    let title = format!(" {} 件 ", app.entries.len());
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    // ListState を渡すと、選択位置のハイライトと自動スクロールを ratatui がやってくれる
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.selected() {
        Some(e) => format!(" {} ", e.name),
        None => " プレビュー ".to_string(),
    };
    let position = format!(" {}/{} ", app.preview_scroll as usize + 1, app.preview_lines().max(1));
    let preview = Paragraph::new(app.preview.as_str())
        .block(Block::bordered().title(title).title_bottom(Line::from(position).right_aligned()))
        .scroll((app.preview_scroll, 0));
    frame.render_widget(preview, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.status.is_empty() {
        " j/k:移動  Enter:開く  h:上へ  J/K:プレビュー  ?:ヘルプ  q:終了".to_string()
    } else {
        format!(" {}", app.status)
    };
    frame.render_widget(Paragraph::new(text).reversed(), area);
}

fn draw_help(frame: &mut Frame) {
    let area = frame
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(HELP.len() as u16 + 2));
    let lines: Vec<Line> = HELP
        .iter()
        .map(|(k, desc)| Line::from(vec![Span::from(format!(" {k:<18}")).yellow(), Span::from(*desc)]))
        .collect();
    // Clear で下に描かれたものを消してから、ポップアップを重ねる
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" ヘルプ (Esc で閉じる) ")),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::fixture;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// TestBackend に描画して、画面の文字だけを行ごとの文字列にする
    fn render_to_lines(app: &mut App, w: u16, h: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        let buf = terminal.backend().buffer();
        (0..h)
            .map(|y| {
                let mut line = String::new();
                let mut x = 0;
                while x < w {
                    let sym = buf[(x, y)].symbol();
                    line.push_str(sym);
                    // 全角文字の右隣のセルは空きなので飛ばす
                    x += Span::raw(sym).width().max(1) as u16;
                }
                line
            })
            .collect()
    }

    #[test]
    fn renders_list_and_preview() {
        let root = fixture("ui");
        let mut app = App::new(&root).unwrap();
        app.next(); // a.txt
        let lines = render_to_lines(&mut app, 80, 10);
        assert!(lines[0].contains("ファイルエクスプローラ"));
        let all = lines.join("\n");
        assert!(all.contains("sub/"));
        assert!(all.contains("> a.txt"));
        assert!(all.contains("line1"));
        assert!(lines[9].contains("q:終了"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn renders_help_popup() {
        let root = fixture("ui_help");
        let mut app = App::new(&root).unwrap();
        app.show_help = true;
        let all = render_to_lines(&mut app, 80, 20).join("\n");
        assert!(all.contains("ヘルプ"));
        assert!(all.contains("親ディレクトリへ"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
