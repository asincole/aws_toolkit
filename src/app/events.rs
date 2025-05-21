use crate::app::AppMode;
use crate::app::actions::AppActions;
use crate::app::state::AppState;
use crate::ui::ScrollableList;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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
    pub async fn handle_events(
        &self,
        state: &mut AppState,
        actions: &mut AppActions,
    ) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event, state, actions).await?;
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    /// Handle keyboard input events
    async fn handle_key_event(
        &self,
        key_event: KeyEvent,
        state: &mut AppState,
        actions: &mut AppActions,
    ) -> Result<()> {
        // If search is active, handle search input first
        if state.search_bar.active {
            match key_event.code {
                KeyCode::Char(c) => {
                    state.search_bar.input(c);
                    // For bucket list, filter locally
                    if state.mode == AppMode::BucketList {
                        state.apply_current_search();
                    }
                }
                KeyCode::Backspace => {
                    state.search_bar.delete();
                    if state.mode == AppMode::BucketList {
                        state.apply_current_search();
                    }
                }
                KeyCode::Esc => {
                    state.search_bar.toggle();
                    if state.mode == AppMode::BucketList {
                        state.apply_current_search();
                    }
                }
                KeyCode::Enter => {
                    // For object list, perform a server-side search
                    if state.mode == AppMode::ObjectList {
                        state.object_continuation_token = None;
                        actions.load_objects(state).await?;
                    }
                    state.search_bar.toggle();
                }
                _ => {}
            }
            return Ok(());
        }

        // Otherwise handle normal navigation
        match key_event.code {
            KeyCode::Char('q') => state.exit = true,
            KeyCode::Char('/') => {
                state.search_bar.toggle();
            }
            KeyCode::Esc => {
                if state.mode == AppMode::ObjectList {
                    state.mode = AppMode::BucketList;
                    state.object_list = ScrollableList::new("Bucket Contents");
                    state.current_bucket = None;
                } else if state.mode == AppMode::PreviewObject {
                    state.mode = AppMode::ObjectList;
                }
            }
            KeyCode::Down => match state.mode {
                AppMode::BucketList => state.bucket_list.next(),
                AppMode::ObjectList => state.object_list.next(),
                AppMode::PreviewObject => {
                    // Scroll preview down
                    state.preview_scroll_offset = state.preview_scroll_offset.saturating_add(1);
                }
            },
            KeyCode::Up => match state.mode {
                AppMode::BucketList => state.bucket_list.previous(),
                AppMode::ObjectList => state.object_list.previous(),
                AppMode::PreviewObject => {
                    // Scroll preview up
                    state.preview_scroll_offset = state.preview_scroll_offset.saturating_sub(1);
                }
            },
            KeyCode::Home => match state.mode {
                AppMode::BucketList => state.bucket_list.first(),
                AppMode::ObjectList => state.object_list.first(),
                AppMode::PreviewObject => {
                    // Go to top of preview
                    state.preview_scroll_offset = 0;
                }
            },
            KeyCode::End => match state.mode {
                AppMode::BucketList => state.bucket_list.last(),
                AppMode::ObjectList => state.object_list.last(),
                AppMode::PreviewObject => {
                    // Go to bottom of preview if we have processed lines
                    if let Some(lines) = &state.processed_preview_lines {
                        state.preview_scroll_offset = lines.len().saturating_sub(1);
                    }
                }
            },
            KeyCode::Char('g') => match state.mode {
                AppMode::BucketList => state.bucket_list.first(),
                AppMode::ObjectList => state.object_list.first(),
                AppMode::PreviewObject => {
                    // Go to top of preview
                    state.preview_scroll_offset = 0;
                }
            },
            KeyCode::Char('G') => match state.mode {
                AppMode::BucketList => state.bucket_list.last(),
                AppMode::ObjectList => state.object_list.last(),
                AppMode::PreviewObject => {
                    // Go to bottom of preview if we have processed lines
                    if let Some(lines) = &state.processed_preview_lines {
                        state.preview_scroll_offset = lines.len().saturating_sub(1);
                    }
                }
            },
            // Add download functionality
            KeyCode::Char('d') => {
                if state.mode == AppMode::PreviewObject {
                    actions.download_object(state).await?;
                }
            }
            KeyCode::Enter => {
                if state.mode == AppMode::BucketList {
                    actions.select_bucket(state).await?;
                } else if state.mode == AppMode::ObjectList {
                    actions.select_object(state).await?;
                }
            }
            KeyCode::Char(' ') => {
                // Load more items
                match state.mode {
                    AppMode::BucketList => {
                        // For buckets, we'd need to implement continuation if AWS supports it
                        // Currently S3 doesn't support pagination for bucket listing in a standard way
                    }
                    AppMode::ObjectList => {
                        if state.object_continuation_token.is_some() {
                            actions.load_objects(state).await?;
                        }
                    }
                    AppMode::PreviewObject => {}
                }
            }
            // Additional navigation keys
            KeyCode::PageDown => {
                // Jump down by 10 items
                match state.mode {
                    AppMode::BucketList => {
                        for _ in 0..10 {
                            state.bucket_list.next();
                        }
                    }
                    AppMode::ObjectList => {
                        for _ in 0..10 {
                            state.object_list.next();
                        }
                    }
                    AppMode::PreviewObject => {
                        state.preview_scroll_offset =
                            state.preview_scroll_offset.saturating_add(10);
                    }
                }
            }
            KeyCode::PageUp => {
                // Jump up by 10 items
                match state.mode {
                    AppMode::BucketList => {
                        for _ in 0..10 {
                            state.bucket_list.previous();
                        }
                    }
                    AppMode::ObjectList => {
                        for _ in 0..10 {
                            state.object_list.previous();
                        }
                    }
                    AppMode::PreviewObject => {
                        state.preview_scroll_offset =
                            state.preview_scroll_offset.saturating_sub(10);
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh the current view
                match state.mode {
                    AppMode::BucketList => {
                        state.bucket_list = ScrollableList::new("S3 Buckets");
                        actions.load_buckets(state).await?;
                    }
                    AppMode::ObjectList => {
                        if let Some(bucket_name) = &state.current_bucket {
                            state.object_list =
                                ScrollableList::new(format!("Contents of {}", bucket_name));
                            state.object_continuation_token = None;
                            actions.load_objects(state).await?;
                        }
                    }
                    AppMode::PreviewObject => {
                        // Refresh the current object preview
                        if let (Some(bucket), Some(key)) =
                            (&state.current_bucket.clone(), &state.current_object.clone())
                        {
                            actions.refresh_object_preview(state, bucket, key).await?;
                        }
                    }
                }
            }
            KeyCode::Char('c') => {
                // Clear search if there's any
                if !state.search_bar.query.is_empty() {
                    state.search_bar.clear();
                    state.apply_current_search();
                }
            }
            _ => {}
        }
        Ok(())
    }
}
