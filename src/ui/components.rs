use crate::app::state::AppState;
use crate::app::AppMode;
use crate::search::SearchBar;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
#[cfg(feature = "logging")]
use tui_logger::{LogFormatter, TuiLoggerLevelOutput, TuiLoggerWidget};

/// Render the header
pub fn render_header(area: Rect, buf: &mut Buffer) {
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
pub fn render_search_bar(app_mode: &AppMode, search_bar: &SearchBar, area: Rect, buf: &mut Buffer) {
    if !search_bar.active && search_bar.query.is_empty() {
        return;
    }

    let search_text = if search_bar.active {
        match app_mode {
            AppMode::BucketList => format!("Search buckets: {}_", search_bar.query),
            AppMode::ObjectList => {
                format!("Filter by prefix: {}_", search_bar.query)
            }
        }
    } else {
        match app_mode {
            AppMode::BucketList => format!("Search buckets: {}", search_bar.query),
            AppMode::ObjectList => format!("Prefix: {}", search_bar.query),
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

/// Render the footer with context-aware help based on app state
pub fn render_footer(area: Rect, buf: &mut Buffer, helper_text: &str) {
    Paragraph::new(helper_text).centered().render(area, buf);
}

#[cfg(feature = "logging")]
pub fn render_logger(area: Rect, buf: &mut Buffer) {
    let formatter: Option<Box<dyn LogFormatter>> = None;
    TuiLoggerWidget::default()
        .block(Block::bordered().title("Unfiltered TuiLoggerWidget"))
        .opt_formatter(formatter)
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style(Style::default().fg(Color::White))
        .render(area, buf);
}
