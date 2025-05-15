use crate::app::{App, AppMode};
use crate::list::render_list;
use crate::ui::components::{
    render_footer, render_header, render_notification_area, render_search_bar,
};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::palette::tailwind::{BLUE, SLATE};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use ratatui::Frame;

mod components;

// UI constants
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);

/// Render the application UI
pub fn render(app: &mut App, frame: &mut Frame) {
    frame.render_widget(app, frame.area());
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [
            header_area,
            notification_area,
            search_area,
            main_area,
            footer_area,
        ] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        render_header(&self.state, header_area, buf);
        render_notification_area(&self.state, notification_area, buf);
        render_search_bar(&self.state, search_area, buf);
        render_footer(
            footer_area,
            buf,
            &self.state.mode,
            self.state.status_message.as_deref(),
        );

        match self.state.mode {
            AppMode::BucketList => {
                render_list(
                    &mut self.state.bucket_list,
                    main_area,
                    buf,
                    NORMAL_ROW_BG,
                    ALT_ROW_BG_COLOR,
                    SELECTED_STYLE,
                    HEADER_STYLE,
                    Style::default().fg(TEXT_FG_COLOR),
                    |bucket, _| {
                        bucket
                            .name
                            .clone()
                            .unwrap_or_else(|| "Unknown Bucket".to_string())
                    },
                );
            }
            AppMode::ObjectList => {
                let [list_area, _] =
                    Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

                render_list(
                    &mut self.state.object_list,
                    list_area,
                    buf,
                    NORMAL_ROW_BG,
                    ALT_ROW_BG_COLOR,
                    SELECTED_STYLE,
                    HEADER_STYLE,
                    Style::default().fg(TEXT_FG_COLOR),
                    |object, _| {
                        let key = object.key().unwrap_or("Unknown");
                        let size = object.size().unwrap_or(0);
                        let size_str = if size < 1024 {
                            format!("{}B", size)
                        } else if size < 1024 * 1024 {
                            format!("{:.2}KB", size as f64 / 1024.0)
                        } else if size < 1024 * 1024 * 1024 {
                            format!("{:.2}MB", size as f64 / (1024.0 * 1024.0))
                        } else {
                            format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                        };

                        format!("{} ({})", key, size_str)
                    },
                );
            }
            AppMode::PreviewObject => {
                let [list_area, preview_content_area] =
                    Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

                // TODO: this duplicates `AppMode::ObjectList` mode... fix later
                // Render the object list on the left
                render_list(
                    &mut self.state.object_list,
                    list_area,
                    buf,
                    NORMAL_ROW_BG,
                    ALT_ROW_BG_COLOR,
                    SELECTED_STYLE,
                    HEADER_STYLE,
                    Style::default().fg(TEXT_FG_COLOR),
                    |object, _| {
                        let key = object.key().unwrap_or("Unknown");
                        let size = object.size().unwrap_or(0);
                        let size_str = if size < 1024 {
                            format!("{}B", size)
                        } else if size < 1024 * 1024 {
                            format!("{:.2}KB", size as f64 / 1024.0)
                        } else if size < 1024 * 1024 * 1024 {
                            format!("{:.2}MB", size as f64 / (1024.0 * 1024.0))
                        } else {
                            format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                        };
                        format!("{} ({})", key, size_str)
                    },
                );

                // --- Preview Pane Rendering ---
                let object_name_for_title = self.state.current_object.as_deref().unwrap_or("N/A");
                let content_type_for_title = self
                    .state
                    .current_object_content_type
                    .as_deref()
                    .unwrap_or("Unknown");
                let preview_title = format!(
                    " Preview: {} ({}) ",
                    object_name_for_title, content_type_for_title
                );

                let preview_block = Block::default()
                    .title(preview_title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded);

                // Get the actual inner area for content AFTER the block is defined
                let preview_inner_drawing_area = preview_block.inner(preview_content_area);
                preview_block.render(preview_content_area, buf); // Render the block frame

                // Prepare display lines if they haven't been, or if the width might have changed
                if self.state.object_preview.is_some()
                    && self.state.processed_preview_lines.is_none()
                {
                    self.state
                        .prepare_display_lines_for_preview(preview_inner_drawing_area.width);
                }

                if self.state.loading && self.state.current_object.is_some() {
                    Paragraph::new("Loading preview...")
                        .style(Style::default().fg(TEXT_FG_COLOR))
                        .centered()
                        .render(preview_inner_drawing_area, buf);
                } else if let Some(display_lines) = &self.state.processed_preview_lines {
                    if display_lines.is_empty() {
                        Paragraph::new("[No content to display or an empty object]")
                            .style(Style::default().fg(TEXT_FG_COLOR))
                            .centered()
                            .render(preview_inner_drawing_area, buf);
                        return;
                    }

                    let visible_lines_count = preview_inner_drawing_area.height as usize;

                    // Ensure the scroll offset is valid
                    if self.state.preview_scroll_offset >= display_lines.len() {
                        self.state.preview_scroll_offset = display_lines.len().saturating_sub(1);
                    }

                    let start_line_idx = self.state.preview_scroll_offset.min(
                        display_lines
                            .len()
                            .saturating_sub(visible_lines_count)
                            .max(0),
                    );

                    let end_line_idx =
                        (start_line_idx + visible_lines_count).min(display_lines.len());

                    for (i, line_text) in display_lines[start_line_idx..end_line_idx]
                        .iter()
                        .enumerate()
                    {
                        Paragraph::new(line_text.as_str())
                            .style(Style::default().fg(TEXT_FG_COLOR))
                            .render(
                                Rect {
                                    x: preview_inner_drawing_area.x,
                                    y: preview_inner_drawing_area.y + i as u16,
                                    width: preview_inner_drawing_area.width,
                                    height: 1,
                                },
                                buf,
                            );
                    }

                    // Scroll Indicator
                    if display_lines.len() > visible_lines_count {
                        let scrollable_content_height = display_lines.len() - visible_lines_count;
                        if scrollable_content_height > 0 {
                            let scroll_percentage =
                                start_line_idx as f64 / scrollable_content_height as f64;
                            let scrollbar_track_height =
                                preview_inner_drawing_area.height.saturating_sub(0) as f64;

                            let mut scroll_thumb_pos =
                                (scrollbar_track_height * scroll_percentage).round() as u16;
                            scroll_thumb_pos = scroll_thumb_pos
                                .min(preview_inner_drawing_area.height.saturating_sub(1));

                            let scrollbar_x = preview_inner_drawing_area.x
                                + preview_inner_drawing_area.width.saturating_sub(1);
                            if scrollbar_x >= preview_inner_drawing_area.x {
                                buf.cell_mut(Position::new(
                                    scrollbar_x,
                                    preview_inner_drawing_area.y + scroll_thumb_pos,
                                ))
                                .map(|cell| {
                                    cell.set_char('█').set_style(Style::default().fg(BLUE.c500))
                                });
                            }
                        }
                    }
                } else if self.state.current_object.is_some() {
                    // Object selected, but no preview content (e.g. still loading initial)
                    Paragraph::new("Processing preview...")
                        .style(Style::default().fg(TEXT_FG_COLOR))
                        .centered()
                        .render(preview_inner_drawing_area, buf);
                } else {
                    // No object selected for preview
                    Paragraph::new("Select an object from the list to preview.")
                        .style(Style::default().fg(TEXT_FG_COLOR))
                        .centered()
                        .render(preview_inner_drawing_area, buf);
                }
            }
        }
    }
}
