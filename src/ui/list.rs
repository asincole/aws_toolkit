use super::{ALT_ROW_BG_COLOR, NORMAL_ROW_BG, SELECTED_STYLE, TEXT_FG_COLOR};
use crate::app::App;
use crate::app::state::LoadingState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use render::render_list;

mod render;
pub mod scrollable_list;

pub fn render_bucket_list(app: &mut App, area: Rect, buf: &mut Buffer) {
    render_list(
        &mut app.state.s3_bucket.bucket_list,
        area,
        buf,
        NORMAL_ROW_BG,
        ALT_ROW_BG_COLOR,
        SELECTED_STYLE,
        Style::default().fg(TEXT_FG_COLOR),
        |bucket, _| {
            bucket
                .name
                .clone()
                .unwrap_or_else(|| "Unknown Bucket".to_string())
        },
    );
}

pub fn render_object_list(app: &mut App, area: Rect, buf: &mut Buffer) {
    render_list(
        &mut app.state.s3_object.object_list,
        area,
        buf,
        NORMAL_ROW_BG,
        ALT_ROW_BG_COLOR,
        SELECTED_STYLE,
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

pub fn render_preview(app: &mut App, area: Rect, buf: &mut Buffer) {
    let object_name_for_title = app
        .state
        .s3_object
        .current_object
        .as_deref()
        .unwrap_or("N/A");
    let content_type_for_title = app
        .state
        .s3_object
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

    let preview_inner_drawing_area = preview_block.inner(area);
    preview_block.render(area, buf);

    // Prepare display lines if they haven't been, or if width might have changed
    if app.state.s3_object.object_preview.is_some()
        && app.state.s3_object.processed_preview_lines.is_none()
    {
        app.state
            .s3_object
            .prepare_display_lines_for_preview(preview_inner_drawing_area.width);
    }

    if matches!(app.state.s3_object.loading_state, LoadingState::Loading)
        && app.state.s3_object.current_object.is_some()
    {
        Paragraph::new("Loading preview...")
            .style(Style::default().fg(TEXT_FG_COLOR))
            .centered()
            .render(preview_inner_drawing_area, buf);
    } else if let Some(display_lines) = &app.state.s3_object.processed_preview_lines {
        if display_lines.is_empty() {
            Paragraph::new("[No content to display or empty object]")
                .style(Style::default().fg(TEXT_FG_COLOR))
                .centered()
                .render(preview_inner_drawing_area, buf);
            return; // Nothing more to render for preview
        }

        let visible_lines_count = preview_inner_drawing_area.height as usize;

        // Ensure scroll offset is valid
        if app.state.s3_object.preview_scroll_offset >= display_lines.len() {
            app.state.s3_object.preview_scroll_offset = display_lines.len().saturating_sub(1);
        }

        let start_line_idx = app.state.s3_object.preview_scroll_offset.min(
            display_lines
                .len()
                .saturating_sub(visible_lines_count)
                .max(0),
        );

        let end_line_idx = (start_line_idx + visible_lines_count).min(display_lines.len());

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
                let scroll_percentage = start_line_idx as f64 / scrollable_content_height as f64;
                let scrollbar_track_height =
                    preview_inner_drawing_area.height.saturating_sub(0) as f64;

                let mut scroll_thumb_pos =
                    (scrollbar_track_height * scroll_percentage).round() as u16;
                scroll_thumb_pos =
                    scroll_thumb_pos.min(preview_inner_drawing_area.height.saturating_sub(1));

                let scrollbar_x = preview_inner_drawing_area.x
                    + preview_inner_drawing_area.width.saturating_sub(1);
                if scrollbar_x >= preview_inner_drawing_area.x {
                    buf.cell_mut((scrollbar_x, preview_inner_drawing_area.y + scroll_thumb_pos))
                        .map(|cell| {
                            cell.set_char('█')
                                .set_style(Style::default().fg(super::BLUE.c500));
                        });
                }
            }
        }
    } else if app.state.s3_object.current_object.is_some() {
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
