use crate::app::AppMode;
use crate::app::state::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

/// Render the header
pub fn render_header(state: &AppState, area: Rect, buf: &mut Buffer) {
    Paragraph::new("AWS S3 Browser")
        .bold()
        .centered()
        .render(area, buf);
}

/// Render the notification area
pub fn render_notification_area(state: &AppState, area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!(
        "Currently in {:?} Mode. Current status message: {:?}",
        state.mode, state.status_message
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Current State"),
    )
    .bold()
    .render(area, buf);
}

/// Render the search bar
pub fn render_search_bar(state: &AppState, area: Rect, buf: &mut Buffer) {
    if !state.search_bar.active && state.search_bar.query.is_empty() {
        return;
    }

    let search_text = if state.search_bar.active {
        match state.mode {
            AppMode::BucketList => format!("Search buckets: {}_", state.search_bar.query),
            AppMode::ObjectList => format!("Filter by prefix: {}_", state.search_bar.query),
            AppMode::PreviewObject => String::from(""),
        }
    } else {
        match state.mode {
            AppMode::BucketList => format!("Search buckets: {}", state.search_bar.query),
            AppMode::ObjectList => format!("Prefix: {}", state.search_bar.query),
            AppMode::PreviewObject => String::from(""),
        }
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(SLATE.c800));

    let search_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };

    block.render(search_area, buf);

    let text_x = area.x + 1;
    for (i, c) in search_text.chars().enumerate() {
        if (text_x + i as u16) < area.x + area.width {
            buf.cell_mut(Position::new(text_x + i as u16, area.y))
                .map(|cell| cell.set_char(c).set_style(Style::default().fg(SLATE.c100)));
        }
    }
}

/// Render the footer
pub fn render_footer(area: Rect, buf: &mut Buffer, mode: &AppMode, status_message: Option<&str>) {
    let help_text = match mode {
        AppMode::BucketList => {
            "↑/↓: Navigate  Enter: Select  Space: Load More  /: Search  g/G: Top/Bottom  q: Quit"
        }
        AppMode::ObjectList => {
            "↑/↓: Navigate  Space: Load More  /: Filter by Prefix  Enter: Preview  Esc: Back  q: Quit"
        }
        AppMode::PreviewObject => {
            "↑/↓: Scroll  PgUp/PgDn: Scroll Fast  d: Download  Esc: Back  q: Quit"
        }
    };

    let display_text = if let Some(msg) = status_message {
        msg
    } else {
        help_text
    };

    Paragraph::new(display_text).centered().render(area, buf);
}
