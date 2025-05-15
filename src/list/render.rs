use crate::list::scrollable_list::ScrollableList;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, StatefulWidget};

/// Render a scrollable list
pub fn render_list<T, F>(
    list: &mut ScrollableList<T>,
    area: Rect,
    buf: &mut Buffer,
    normal_bg: Color,
    alt_bg: Color,
    selected_style: Style,
    header_style: Style,
    text_style: Style,
    item_to_text: F,
) where
    F: Fn(&T, usize) -> String,
{
    let block = Block::default()
        .title(format!(" {} ", list.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    // Calculate the inner area available for list items (after block borders)
    let list_content_area = block.inner(area);

    // Define the highlight symbol and its width (as used later in List::highlight_symbol)
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
            let item_text_from_fn = item_to_text(&list.items[actual_idx], actual_idx);

            let prefix = format!("{:>4}. ", display_idx + 1);
            let prefix_len = prefix.chars().count() as u16;

            // Width available for the item's text content (prefix + item_text_from_fn)
            // This is the width inside the list area, minus space for the highlight symbol.
            let available_width_for_item_content = list_content_area
                .width
                .saturating_sub(highlight_symbol_width);

            // Width available specifically for the `item_text_from_fn` part, after the prefix.
            let text_wrap_width = available_width_for_item_content.saturating_sub(prefix_len);

            let mut lines = Vec::new();

            if text_wrap_width > 0 {
                // Using textwrap crate for wrapping
                let wrapped_item_text_lines: Vec<String> =
                    textwrap::wrap(&item_text_from_fn, text_wrap_width as usize)
                        .into_iter()
                        .map(|s| s.into_owned())
                        .collect();

                if wrapped_item_text_lines.is_empty() {
                    // If item_text_from_fn was empty or only whitespace that got removed
                    lines.push(Line::from(prefix.clone()).style(text_style));
                } else {
                    // First line with prefix
                    lines.push(
                        Line::from(format!("{}{}", prefix, wrapped_item_text_lines[0]))
                            .style(text_style),
                    );
                    // Subsequent lines indented
                    let indent = " ".repeat(prefix.chars().count());
                    for text_line in wrapped_item_text_lines.iter().skip(1) {
                        lines
                            .push(Line::from(format!("{}{}", indent, text_line)).style(text_style));
                    }
                }
            } else {
                // Not enough space to wrap, or text_wrap_width is 0.
                // Show prefix and potentially a truncated version of the item text.
                let truncated_text = item_text_from_fn
                    .chars()
                    .take(
                        available_width_for_item_content
                            .saturating_sub(prefix_len)
                            .max(0) as usize,
                    )
                    .collect::<String>();
                lines.push(Line::from(format!("{}{}", prefix, truncated_text)).style(text_style));
            }

            ListItem::new(Text::from(lines)).bg(color)
        })
        .collect();

    let list_widget = List::new(items)
        .block(block)
        .highlight_style(selected_style)
        .highlight_symbol(highlight_symbol) // Use the defined symbol
        .repeat_highlight_symbol(true);

    StatefulWidget::render(list_widget, area, buf, &mut list.state);

    // Render status indicator at the bottom (remains the same)
    let status_text = if list.loading_more {
        // Assuming ScrollableList has these fields
        "Loading more items..."
    } else if list.has_more {
        "Press SPACE to load more items"
    } else if list.items.is_empty() {
        "No items found"
    } else {
        "End of list reached"
    };

    let status_area_width = area.width.saturating_sub(2); // Assuming status is inside borders if any apply here
    let status_x = area.x + 1 + (status_area_width.saturating_sub(status_text.len() as u16)) / 2;
    let status_y = area.y + area.height.saturating_sub(2); // Ensure this is within bounds

    // Ensure status text y is within the main area and above the bottom border if block has one
    if status_y >= area.y + 1 && status_y < area.y + area.height - 1 {
        for (i, c) in status_text.chars().enumerate() {
            let current_x = status_x + i as u16;
            if current_x >= area.x + 1 && current_x < area.x + area.width - 1 {
                // Stay within L/R borders
                buf.get_mut(current_x, status_y)
                    .set_char(c)
                    .set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }
    }
}
