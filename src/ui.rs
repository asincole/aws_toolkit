use crate::app::{App, AppMode};
#[cfg(feature = "logging")]
use crate::ui::components::render_logger;
use crate::ui::components::{
    render_footer, render_header, render_notification_area, render_search_bar,
};
use crate::ui::list::{render_bucket_list, render_object_list, render_preview};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::palette::tailwind::{BLUE, SLATE};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use ratatui::Frame;

mod components;
mod list;

pub use list::scrollable_list::ScrollableList;

// UI constants
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;

/// Render the application UI
pub fn render(app: &mut App, frame: &mut Frame) {
    frame.render_widget(app, frame.area());
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        #[cfg(feature = "logging")]
        let [main_app_area, logger] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);

        #[cfg(feature = "logging")]
        render_logger(logger, buf);

        #[cfg(not(feature = "logging"))]
        let [main_app_area] = Layout::horizontal([Constraint::Fill(1)]).areas(area);

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
        .areas(main_app_area);

        render_header(header_area, buf);
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
                render_bucket_list(self, main_area, buf);
            }
            AppMode::ObjectList => {
                if self.state.s3_object.preview_object == true {
                    let [list_area, preview_content_area] =
                        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                            .areas(main_area);

                    render_object_list(self, list_area, buf);
                    render_preview(self, preview_content_area, buf);
                } else {
                    let [list_area, preview_content_area] =
                        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                            .areas(main_area);

                    render_object_list(self, list_area, buf);
                    render_preview(self, preview_content_area, buf);
                }
            }
        }
    }
}
