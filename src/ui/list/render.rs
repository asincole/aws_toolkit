use crate::ui::ScrollableList;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, StatefulWidget, Widget},
};

pub fn render_list<T, F>(
    list: &mut ScrollableList<T>,
    area: Rect,
    buf: &mut Buffer,
    normal_bg: Color,
    alt_bg: Color,
    selected_style: Style,
    text_style: Style,
    item_to_text: F,
) where
    F: Fn(&T, usize) -> String,
{
    let outer_block = Block::default()
        .title(format!(" {} ", list.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner_area = outer_block.inner(area);

    let [list_area, info_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(inner_area);

    outer_block.render(area, buf);

    let highlight_symbol = " > ";
    let highlight_symbol_width = highlight_symbol.chars().count() as u16;

    let items: Vec<ListItem> = list
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(display_idx, &actual_idx)| {
            let color = if display_idx % 2 == 0 {
                normal_bg
            } else {
                alt_bg
            };

            let item_data = &list.items[actual_idx];
            let item_content_as_string = item_to_text(item_data, actual_idx);

            let prefix = format!("{:>4}. ", display_idx + 1);
            let prefix_char_count = prefix.chars().count();
            let prefix_render_width = prefix_char_count as u16;

            // Calculate the width available for the item's text content,
            // after accounting for the highlight symbol and the prefix.
            let text_content_render_width = list_area
                .width
                .saturating_sub(highlight_symbol_width)
                .saturating_sub(prefix_render_width);

            let mut rendered_lines_for_item: Vec<Line> = Vec::new();

            if text_content_render_width > 0 {
                let wrapped_content_segments =
                    textwrap::wrap(&item_content_as_string, text_content_render_width as usize);

                if wrapped_content_segments.is_empty() {
                    rendered_lines_for_item.push(Line::from(prefix.clone()).style(text_style));
                } else {
                    rendered_lines_for_item.push(
                        Line::from(format!("{}{}", prefix, wrapped_content_segments[0]))
                            .style(text_style),
                    );
                    if wrapped_content_segments.len() > 1 {
                        let indent = " ".repeat(prefix_char_count);
                        for content_segment in wrapped_content_segments.iter().skip(1) {
                            rendered_lines_for_item.push(
                                Line::from(format!("{}{}", indent, content_segment))
                                    .style(text_style),
                            );
                        }
                    }
                }
            } else {
                // Not enough space for any text content alongside the prefix. Display only the prefix.
                // The List widget itself will handle truncation if the prefix is too long
                // for the available line width (list_area.width - highlight_symbol_width).
                rendered_lines_for_item.push(Line::from(prefix.clone()).style(text_style));
            }

            ListItem::new(Text::from(rendered_lines_for_item)).bg(color)
        })
        .collect();

    let list_widget = List::new(items)
        .highlight_style(selected_style)
        .highlight_symbol(highlight_symbol)
        .repeat_highlight_symbol(false);

    StatefulWidget::render(list_widget, list_area, buf, &mut list.state);

    let status_text = if list.loading_more {
        "Loading more items..."
    } else if list.has_more {
        "Press SPACE to load more items"
    } else if list.items.is_empty() {
        "No items found"
    } else {
        "End of list reached"
    };

    Paragraph::new(status_text)
        .bold()
        .alignment(Alignment::Center)
        .render(info_area, buf);
}
