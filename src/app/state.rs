use crate::app::AppMode;
use crate::aws::AWS;
use crate::aws::s3_client::S3Client;
use crate::search::SearchBar;
use crate::ui::ScrollableList;
use aws_sdk_s3::types::{Bucket, Object};
use std::time::Instant;

/// Represents the complete state of the application
#[derive(Debug)]
pub struct AppState {
    pub aws_config: AWS,
    pub mode: AppMode,
    pub bucket_list: ScrollableList<Bucket>,
    pub object_list: ScrollableList<Object>,
    pub current_bucket: Option<String>,
    pub current_prefix: Option<String>,
    pub object_continuation_token: Option<String>,
    pub s3_client: S3Client,
    pub exit: bool,
    pub loading: bool,
    pub search_bar: SearchBar,
    pub current_object: Option<String>,
    pub object_preview: Option<String>,
    pub current_object_content_type: Option<String>,
    pub processed_preview_lines: Option<Vec<String>>,
    pub preview_scroll_offset: usize,
    pub status_message: Option<String>,
    pub status_message_time: Option<Instant>,
}

impl AppState {
    pub fn new(aws_config: AWS, s3_client: S3Client) -> Self {
        Self {
            aws_config,
            mode: AppMode::BucketList,
            bucket_list: ScrollableList::new("S3 Buckets"),
            object_list: ScrollableList::new("Bucket Contents"),
            current_bucket: None,
            current_object: None,
            object_preview: None,
            current_object_content_type: None,
            processed_preview_lines: None,
            preview_scroll_offset: 0,
            status_message: None,
            current_prefix: None,
            object_continuation_token: None,
            s3_client,
            exit: false,
            loading: false,
            search_bar: SearchBar::default(),
            status_message_time: None,
        }
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

    /// Apply the current search query to the appropriate list based on the current mode
    pub fn apply_current_search(&mut self) {
        match self.mode {
            AppMode::BucketList => {
                self.bucket_list.apply_search(&self.search_bar, |bucket| {
                    bucket
                        .name
                        .clone()
                        .unwrap_or_else(|| "Unknown Bucket".to_string())
                });
            }
            AppMode::ObjectList => {
                self.object_list.apply_search(&self.search_bar, |object| {
                    object.key().unwrap_or("Unknown").to_string()
                });
            }
            AppMode::PreviewObject => {}
        }
    }

    pub fn update_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_message_time = Some(Instant::now());
    }
}
