use crate::app::AppMode;
use crate::app::actions::AppActions;
use crate::app::state::{LoadingState, NotificationMessage};
use crate::aws::s3_client::S3Client;
use crate::search::SearchBar;
use crate::ui::ScrollableList;
use crate::util::create_unique_filepath;
use aws_sdk_s3::types::Object;
use color_eyre::eyre::Context;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug)]
pub struct S3ObjectState {
    pub current_bucket: String,
    pub object_list: ScrollableList<Object>,
    pub current_object: Option<String>,
    pub current_prefix: Option<String>,
    pub object_continuation_token: Option<String>,
    pub current_object_content_type: Option<String>,
    pub preview_object: bool,
    pub object_preview: Option<String>,
    pub processed_preview_lines: Option<Vec<String>>,
    pub preview_scroll_offset: usize,
    pub search_bar: SearchBar,
    pub s3_client: Arc<S3Client>,
    pub notification: Option<NotificationMessage>,
    pub loading_state: LoadingState,
}

impl S3ObjectState {
    pub fn new(s3_client: Arc<S3Client>) -> Self {
        Self {
            current_bucket: String::default(),
            object_list: ScrollableList::new("Bucket Contents"),
            current_object: None,
            preview_object: false,
            object_preview: None,
            current_object_content_type: None,
            processed_preview_lines: None,
            preview_scroll_offset: 0,
            current_prefix: None,
            object_continuation_token: None,
            s3_client,
            search_bar: SearchBar::default(),
            notification: None,
            loading_state: LoadingState::Idle,
        }
    }

    /// Load S3 bucket objects into the application state
    pub async fn load_objects(&mut self) -> color_eyre::Result<()> {
        self.loading_state = LoadingState::Loading;
        let (buckets, next_token) = self
            .s3_client
            .list_objects(
                &self.current_bucket,
                self.object_continuation_token.clone(),
                match !self.search_bar.query.is_empty() {
                    true => Some(self.search_bar.query.clone()),
                    false => None,
                },
                20,
            )
            .await?;

        self.object_list.set_has_more(next_token.is_some());
        self.object_continuation_token = next_token;

        self.object_list.append_items(buckets);

        self.object_list.filtered_indices = (0..self.object_list.items.len()).collect();

        self.object_list.first();
        self.loading_state = LoadingState::Loaded;
        Ok(())
    }

    pub fn update_status(&mut self, message: String) {
        self.notification = Some(NotificationMessage {
            status_message: message,
            status_message_time: Instant::now(),
        })
    }

    /// Apply the current search query to the appropriate list based on the current mode
    pub fn apply_current_search(&mut self) {
        self.object_list.apply_search(&self.search_bar, |object| {
            object.key().unwrap_or("Unknown").to_string()
        });
    }

    /// Download the currently selected object
    pub async fn download_object(&mut self) -> color_eyre::Result<()> {
        let (bucket_name, object_key) = match (&self.current_bucket, &self.current_object) {
            (bucket, Some(key)) => (bucket.clone(), key.clone()),
            _ => return Ok(()),
        };

        let original_filename = object_key.split('/').last().unwrap_or(&object_key);
        self.update_status(format!("Downloading {}...", original_filename));

        let download_dir = dirs::download_dir().unwrap_or_else(|| {
            self.update_status(
                "Could not determine the download directory. Using the current directory."
                    .to_string(),
            );
            PathBuf::from(".")
        });

        if !download_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&download_dir) {
                self.update_status(format!(
                    "Error creating download dir {}: {}",
                    download_dir.display(),
                    e
                ));
                return Ok(());
            }
        }

        let download_path = create_unique_filepath(&download_dir, original_filename);

        match self
            .download_and_save(&self.s3_client, &bucket_name, &object_key, &download_path)
            .await
        {
            Ok(_) => self.update_status(format!("Downloaded to {}", download_path.display())),
            Err(e) => self.update_status(format!("Download failed: {}", e)),
        }

        Ok(())
    }

    async fn download_and_save(
        &self,
        s3_client: &S3Client,
        bucket_name: &str,
        object_key: &str,
        path: &Path,
    ) -> color_eyre::Result<()> {
        let content_bytes = s3_client
            .download_object_bytes(bucket_name, object_key)
            .await
            .wrap_err("Failed to download the object")?;

        fs::write(path, content_bytes)
            .wrap_err_with(|| format!("Failed to write the file to {}", path.display()))?;

        Ok(())
    }

    /// Prepares display lines for the preview pane based on the content type and available width
    pub fn prepare_display_lines_for_preview(&mut self, available_width: u16) {
        if self.object_preview.is_none() {
            self.processed_preview_lines = None;
            return;
        }

        let raw_content = self
            .object_preview
            .as_ref()
            .expect("object_preview is Some as checked above");
        let mut text_to_wrap = raw_content.clone(); // Start with raw content

        let mut is_structured_text = false;

        if let Some(content_type) = &self.current_object_content_type {
            if content_type.starts_with("application/json")
                && raw_content != "[Binary content - not valid UTF-8]"
            {
                match serde_json::from_str::<serde_json::Value>(raw_content) {
                    Ok(json_value) => {
                        if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                            text_to_wrap = pretty_json;
                            is_structured_text = true;
                        }
                        // If pretty printing fails, text_to_wrap remains raw_content
                    }
                    Err(_) => {
                        // Not valid JSON, keep raw_content
                        text_to_wrap = format!(
                            "[Content-Type: application/json, but failed to parse as JSON]\n\n{}",
                            raw_content
                        );
                    }
                }
            } else if content_type.starts_with("image/") {
                text_to_wrap = format!("[Image: {}]", content_type);
            } else if content_type.starts_with("application/octet-stream")
                || raw_content == "[Binary content - not valid UTF-8]"
            {
                // Ensure the placeholder for binary is used if from_utf8 failed
                if raw_content == "[Binary content - not valid UTF-8]" {
                    text_to_wrap = raw_content.clone();
                } else {
                    text_to_wrap = format!("[Binary Data: {}]", content_type);
                }
            } else if !content_type.starts_with("text/") {
                // For other non-text application types, show a message
                text_to_wrap = format!(
                    "[Preview for Content-Type: {}]\n\n{}",
                    content_type, raw_content
                );
            }
            // For "text/*", text_to_wrap remains raw_content by default
        }
        // If no content_type, text_to_wrap is still raw_content

        if available_width == 0 {
            self.processed_preview_lines = Some(vec![text_to_wrap]); // Avoid panic with textwrap
            return;
        }

        let wrapped_lines: Vec<String> = textwrap::wrap(&text_to_wrap, available_width as usize)
            .into_iter()
            .map(|s| s.into_owned())
            .collect();

        self.processed_preview_lines = Some(wrapped_lines);
    }

    /// Select an object and load its preview
    pub async fn select_object(&mut self) -> color_eyre::Result<()> {
        if let Some(selected_s3_object) = self.object_list.selected_item() {
            if let (bucket_name, Some(object_key)) = (&self.current_bucket, &selected_s3_object.key)
            {
                self.loading_state = LoadingState::Loading;
                self.object_preview = None;
                self.current_object_content_type = None;
                self.processed_preview_lines = None;

                let (content, content_type) = self
                    .s3_client
                    .get_object_content(bucket_name, object_key)
                    .await?;

                self.loading_state = LoadingState::Loaded;
                self.current_object = Some(object_key.clone());
                self.object_preview = Some(content);
                self.current_object_content_type = content_type;

                // Mark that processed lines need regeneration
                self.processed_preview_lines = None;
                self.preview_scroll_offset = 0; // Reset scroll for new content
                self.preview_object = true;
            }
        }
        Ok(())
    }

    pub async fn handle_object_action(
        &mut self,
        action: AppActions,
        app_mode: &mut AppMode,
    ) -> color_eyre::Result<()> {
        if self.search_bar.active {
            match action {
                AppActions::SearchInput(c) => {
                    self.search_bar.input(c);
                    self.apply_current_search();
                }
                AppActions::SearchDelete => {
                    self.search_bar.delete();
                    self.apply_current_search();
                }
                AppActions::Enter => {
                    self.apply_current_search();
                    self.search_bar.toggle();
                }
                AppActions::GoBack => {
                    self.search_bar.toggle();
                    self.apply_current_search();
                }
                _ => {}
            }
            return Ok(());
        }

        match action {
            AppActions::GoBack => {
                *app_mode = AppMode::BucketList;
                self.object_list = ScrollableList::new("");
                self.current_object = None;
            }
            AppActions::StartSearch => {
                self.search_bar.toggle();
            }
            AppActions::MoveDown => self.object_list.next(),
            AppActions::MoveUp => self.object_list.previous(),
            AppActions::MoveToTop => self.object_list.first(),
            AppActions::MoveToBottom => self.object_list.last(),
            AppActions::PageDown => {
                for _ in 0..10 {
                    self.object_list.next();
                }
            }
            AppActions::PageUp => {
                for _ in 0..10 {
                    self.object_list.previous();
                }
            }
            AppActions::HalfPageDown => {
                for _ in 0..5 {
                    self.object_list.next();
                }
            }
            AppActions::HalfPageUp => {
                for _ in 0..5 {
                    self.object_list.previous();
                }
            }
            AppActions::Enter => {
                self.select_object().await?;
            }
            AppActions::Download => {
                self.download_object().await?;
            }
            AppActions::Refresh => {
                self.object_list = ScrollableList::new("Objects");
                self.object_continuation_token = None;
                self.load_objects().await?;
            }
            AppActions::LoadMore => {
                if self.object_continuation_token.is_some() {
                    self.load_objects().await?;
                }
            }
            AppActions::ClearSearch => {
                self.search_bar.clear();
                self.object_list
                    .apply_search(&self.search_bar, |object| "Unknown".to_string());
            }
            _ => {} // Ignore actions not relevant to bucket mode
        }

        Ok(())
    }

    /// Refresh the preview of the current object
    pub async fn refresh_object_preview(
        &mut self,
        // state: &mut AppState,
        // bucket: &str,
        key: &str,
    ) -> color_eyre::Result<()> {
        self.loading_state = LoadingState::Loading;
        self.object_preview = None;
        self.current_object_content_type = None;
        self.processed_preview_lines = None;

        let (content, content_type) = self
            .s3_client
            .get_object_content(&self.current_bucket, key)
            .await?;

        self.loading_state = LoadingState::Loaded;
        self.object_preview = Some(content);
        self.current_object_content_type = content_type;

        // Mark that processed lines need regeneration
        self.processed_preview_lines = None;
        self.preview_scroll_offset = 0; // Reset scroll for new content

        Ok(())
    }

    pub async fn handle_preview_action(&mut self, action: AppActions) -> color_eyre::Result<()> {
        match action {
            AppActions::GoBack => {
                self.preview_object = false;
            }
            AppActions::MoveDown => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_add(1);
            }
            AppActions::MoveUp => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(1);
            }
            AppActions::MoveToTop => {
                self.preview_scroll_offset = 0;
            }
            AppActions::MoveToBottom => {
                if let Some(lines) = &self.processed_preview_lines {
                    self.preview_scroll_offset = lines.len().saturating_sub(1);
                }
            }
            AppActions::PageDown => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_add(10);
            }
            AppActions::PageUp => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(10);
            }
            AppActions::HalfPageDown => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_add(5);
            }
            AppActions::HalfPageUp => {
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(5);
            }
            AppActions::Download => {
                self.download_object().await?;
            }
            AppActions::Refresh => {
                if let Some(key) = &self.current_object.clone() {
                    self.refresh_object_preview(key).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
