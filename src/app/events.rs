use crate::app::AppMode;
use crate::app::actions::AppActions;
use crate::app::state::AppState;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;

/// Handles all event processing for the application
#[derive(Debug)]
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle events from the terminal
    /// Returns true if a redraw is needed immediately
    pub async fn handle_events(&self, state: &mut AppState) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event.into(), state).await?;
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    /// Handle keyboard input events
    async fn handle_key_event(&self, action: AppActions, state: &mut AppState) -> Result<()> {
        match action {
            AppActions::Exit => {
                state.exit = true;
                return Ok(());
            }
            AppActions::StartSearch => {
                match state.mode {
                    AppMode::BucketList => {
                        state.s3_bucket.search_bar.toggle();
                    }
                    AppMode::ObjectList => {
                        state.s3_object.search_bar.toggle();
                    }
                }
                return Ok(());
            }
            _ => match state.mode {
                AppMode::BucketList => {
                    state
                        .s3_bucket
                        .handle_bucket_action(action, &mut state.mode, &mut state.s3_object)
                        .await?;
                }
                AppMode::ObjectList => {
                    if state.s3_object.preview_object == true {
                        state.s3_object.handle_preview_action(action).await?;
                    } else {
                        state
                            .s3_object
                            .handle_object_action(action, &mut state.mode)
                            .await?;
                    }
                }
            },
        }
        Ok(())
    }
}
