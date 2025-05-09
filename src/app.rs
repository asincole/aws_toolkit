use crate::aws::s3_client::S3Client;
use crate::aws::AWS;
use aws_sdk_s3::types::Bucket;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::{Constraint, Layout};
use ratatui::style::palette::tailwind::{BLUE, GREEN, SLATE};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{
    Borders, HighlightSpacing, List, ListItem, ListState, Padding, StatefulWidget, Wrap,
};
use ratatui::{
    buffer::Buffer, layout::Rect,
    style::Stylize,
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal,
    Frame,
};

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    Config,
}

#[derive(Debug)]
pub struct App {
    aws_config: AWS,
    pub current_screen: CurrentScreen,
    s3_bucket_list: S3BucketList,
    exit: bool,
}

#[derive(Debug)]
struct S3BucketList {
    buckets: Vec<Bucket>,
    state: ListState,
}

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

impl App {
    pub async fn new() -> Result<Self> {
        let aws_config = AWS::new().await;
        let mut s3_client = S3Client::new(&aws_config.config);
        let buckets = s3_client.get_bucket_list().await?;
        Ok(Self {
            aws_config,
            current_screen: CurrentScreen::default(),
            s3_bucket_list: S3BucketList {
                buckets,
                state: Default::default(),
            },
            exit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            // KeyCode::Left => self.decrement_counter()?,
            // KeyCode::Right => self.increment_counter()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("AWS Toolkit")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("S3 Bucket List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .s3_bucket_list
            .buckets
            .iter()
            .enumerate()
            .map(|(i, bucket)| {
                let color = alternate_colors(i);
                ListItem::from(ListItem::new(Line::styled(
                    format!("{:?}", bucket.name),
                    TEXT_FG_COLOR,
                )))
                .bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.s3_bucket_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.s3_bucket_list.state.selected() {
            // match self.s3_bucket_list.buckets[i].status {
            //     Status::Completed => format!("✓ DONE: {}", self.todo_list.items[i].info),
            //     Status::Todo => format!("☐ TODO: {}", self.todo_list.items[i].info),
            // }
            "Nothing selected...".to_string()
        } else {
            "Nothing selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("S3 Bucket Content").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl Widget for &mut App {
    // fn render(self, area: Rect, buf: &mut Buffer) {
    //     let title = Line::from(" AWS Toolkit ".bold());
    //     let instructions = Line::from(vec![
    //         // " Decrement ".into(),
    //         // "<Left>".blue().bold(),
    //         // " Increment ".into(),
    //         // "<Right>".blue().bold(),
    //         " Quit ".into(),
    //         "<Q> ".blue().bold(),
    //     ]);
    //     let block = Block::bordered()
    //         .title(title.centered())
    //         .title_bottom(instructions.centered())
    //         .border_set(border::THICK);
    //
    //     let counter_text = Text::from(vec![Line::from(vec![
    //         "Value: ".into(),
    //         3.to_string().yellow(),
    //     ])]);
    //
    //     Paragraph::new(counter_text)
    //         // .centered()
    //         .block(block)
    //         .render(area, buf);
    // }

    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
