use crate::app::actions::AppActions;
use crate::app::events::EventHandler;
use crate::app::state::AppState;
use crate::aws::AWS;
use crate::aws::s3_client::S3Client;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::time::Duration;

pub mod actions;
pub mod events;
pub mod state;

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    BucketList,
    ObjectList,
    PreviewObject,
}

#[derive(Debug)]
pub struct App {
    pub state: AppState,
    pub actions: AppActions,
    event_handler: EventHandler,
}

impl App {
    pub async fn new() -> Result<Self> {
        let aws_config = AWS::new().await;
        let s3_client = S3Client::new(&aws_config.config);

        let mut state = AppState::new(aws_config, s3_client);

        // Load initial buckets
        state.loading = true;
        let buckets = state.s3_client.get_bucket_list().await?;
        state.bucket_list.append_items(buckets);
        state.bucket_list.filtered_indices = (0..state.bucket_list.items.len()).collect();
        state.bucket_list.first();
        state.loading = false;

        Ok(Self {
            state,
            actions: AppActions::new(),
            event_handler: EventHandler::new(),
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.state.exit {
            self.update();
            terminal.draw(|frame| crate::ui::render(self, frame))?;

            if self
                .event_handler
                .handle_events(&mut self.state, &mut self.actions)
                .await?
            {
                // If true was returned, we need to redraw immediately
                continue;
            }

            // Small delay to prevent CPU hogging
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }

    fn update(&mut self) {
        // Clear status message after 3 seconds
        if let Some(time) = self.state.status_message_time {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.state.status_message = None;
                self.state.status_message_time = None;
            }
        }
    }
}
