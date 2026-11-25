//! 描画 (starter)。`App` の状態を読んで ratatui のウィジェットを組み立てるだけ。

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    // TODO(STEP 1): Layout::vertical で header / body / footer に分け、
    //               いまは header に "ファイルエクスプローラ" と表示するだけ
    // TODO(STEP 2): body を Layout::horizontal で left / right に分け、draw_list を呼ぶ
    // TODO(STEP 4): draw_preview
    // TODO(STEP 6): app.show_help なら draw_help
    frame.render_widget(Paragraph::new("TODO: STEP 1 から始めよう (q で終了)"), frame.area());
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    // TODO(STEP 1): タイトルとカレントディレクトリを色付きの 1 行で
    todo!()
}

fn draw_list(frame: &mut Frame, area: Rect, app: &mut App) {
    // TODO(STEP 2): entries から ListItem を作り、List を render_stateful_widget で描画
    // TODO(STEP 3): highlight_style / highlight_symbol("> ") で選択行を目立たせる
    todo!()
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App) {
    // TODO(STEP 4): Paragraph::new(preview).block(...).scroll((app.preview_scroll, 0))
    todo!()
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    // TODO(STEP 1): 操作説明 (status があればそちら) を反転表示
    todo!()
}

fn draw_help(frame: &mut Frame) {
    // TODO(STEP 6): area.centered(...) で中央に領域を取り、Clear → 枠付き Paragraph
    todo!()
}
