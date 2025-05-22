use crate::app::AppMode;
use crate::app::state::AppState;
use crate::aws::s3_client::S3Client;
use crate::ui::ScrollableList;
use color_eyre::Result;
use color_eyre::eyre::Context;
use std::fs;
use std::path::{Path, PathBuf};

/// Handles all business logic actions for the application
#[derive(Debug)]
pub struct AppActions;

impl AppActions {
    pub fn new() -> Self {
        Self
    }

    /// Load S3 buckets into the application state
    pub async fn load_buckets(&self, state: &mut AppState) -> Result<()> {
        state.loading = true;
        let (buckets, next_token) = state
            .s3_client
            .get_bucket_list(state.bucket_continuation_token.clone())
            .await?;

        state.bucket_list.set_has_more(next_token.is_some());
        state.bucket_continuation_token = next_token;

        state.bucket_list.append_items(buckets);

        // Initialise filtered indices with all items
        state.bucket_list.filtered_indices = (0..state.bucket_list.items.len()).collect();

        state.bucket_list.first();
        state.loading = false;
        Ok(())
    }

    /// Load objects from the current bucket
    pub async fn load_objects(&self, state: &mut AppState) -> Result<()> {
        if let Some(bucket_name) = &state.current_bucket {
            state.loading = true;
            state.object_list.set_loading(true);

            // Get the prefix from the search bar if active
            let prefix = if !state.search_bar.query.is_empty() {
                Some(state.search_bar.query.clone())
            } else {
                state.current_prefix.clone()
            };

            let (objects, next_token) = state
                .s3_client
                .list_objects(
                    bucket_name,
                    state.object_continuation_token.clone(),
                    prefix.clone(),
                    20,
                )
                .await?;

            state.object_list.set_has_more(next_token.is_some());
            state.object_continuation_token = next_token;

            // If this is a new search/prefix, clear existing items
            if state.current_prefix != prefix {
                state.object_list.items.clear();
                state.current_prefix = prefix;
            }

            state.object_list.append_items(objects);

            // Initialize filtered indices with all items
            state.object_list.filtered_indices = (0..state.object_list.items.len()).collect();

            if state.object_list.selected_index().is_none()
                && !state.object_list.filtered_indices.is_empty()
            {
                state.object_list.first();
            }

            state.loading = false;
            state.object_list.set_loading(false);
        }
        Ok(())
    }

    /// Select a bucket and load its objects
    pub async fn select_bucket(&self, state: &mut AppState) -> Result<()> {
        if let Some(bucket) = state.bucket_list.selected_item() {
            if let Some(name) = &bucket.name {
                state.current_bucket = Some(name.clone());
                state.object_list = ScrollableList::new(format!("Contents of {}", name));
                state.object_continuation_token = None;
                state.current_prefix = None; // Reset prefix when changing buckets
                state.mode = AppMode::ObjectList; // Change to ObjectList mode
                self.load_objects(state).await?;
            }
        }
        Ok(())
    }

    /// Select an object and load its preview
    pub async fn select_object(&self, state: &mut AppState) -> Result<()> {
        if let Some(selected_s3_object) = state.object_list.selected_item() {
            if let (Some(bucket_name), Some(object_key)) =
                (&state.current_bucket, &selected_s3_object.key)
            {
                state.loading = true;
                state.object_preview = None;
                state.current_object_content_type = None;
                state.processed_preview_lines = None;

                let (content, content_type) = state
                    .s3_client
                    .get_object_content(bucket_name, object_key)
                    .await?;

                state.loading = false;
                state.current_object = Some(object_key.clone());
                state.object_preview = Some(content);
                state.current_object_content_type = content_type;

                // Mark that processed lines need regeneration
                state.processed_preview_lines = None;
                state.preview_scroll_offset = 0; // Reset scroll for new content
                state.mode = AppMode::PreviewObject;
            }
        }
        Ok(())
    }

    /// Refresh the preview of the current object
    pub async fn refresh_object_preview(
        &self,
        state: &mut AppState,
        bucket: &str,
        key: &str,
    ) -> Result<()> {
        state.loading = true;
        state.object_preview = None;
        state.current_object_content_type = None;
        state.processed_preview_lines = None;

        let (content, content_type) = state.s3_client.get_object_content(bucket, key).await?;

        state.loading = false;
        state.object_preview = Some(content);
        state.current_object_content_type = content_type;

        // Mark that processed lines need regeneration
        state.processed_preview_lines = None;
        state.preview_scroll_offset = 0; // Reset scroll for new content

        Ok(())
    }

    /// Download the currently selected object
    pub async fn download_object(&self, state: &mut AppState) -> Result<()> {
        let (bucket_name, object_key) = match (&state.current_bucket, &state.current_object) {
            (Some(bucket), Some(key)) => (bucket.clone(), key.clone()),
            _ => return Ok(()),
        };

        let original_filename = object_key.split('/').last().unwrap_or(&object_key);
        state.update_status(format!("Downloading {}...", original_filename));

        let download_dir = dirs::download_dir().unwrap_or_else(|| {
            state.update_status(
                "Could not determine the download directory. Using the current directory."
                    .to_string(),
            );
            PathBuf::from(".")
        });

        if !download_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&download_dir) {
                state.update_status(format!(
                    "Error creating download dir {}: {}",
                    download_dir.display(),
                    e
                ));
                return Ok(());
            }
        }

        // Handle potential filename conflicts
        let download_path = create_unique_filepath(&download_dir, original_filename);

        // Download and save the file
        match self
            .download_and_save(&state.s3_client, &bucket_name, &object_key, &download_path)
            .await
        {
            Ok(_) => state.update_status(format!("Downloaded to {}", download_path.display())),
            Err(e) => state.update_status(format!("Download failed: {}", e)),
        }

        Ok(())
    }

    async fn download_and_save(
        &self,
        s3_client: &S3Client,
        bucket_name: &str,
        object_key: &str,
        path: &Path,
    ) -> Result<()> {
        let content_bytes = s3_client
            .download_object_bytes(bucket_name, object_key)
            .await
            .wrap_err("Failed to download the object")?;

        fs::write(path, content_bytes)
            .wrap_err_with(|| format!("Failed to write the file to {}", path.display()))?;

        Ok(())
    }
}

fn create_unique_filepath(dir: &Path, filename: &str) -> PathBuf {
    let mut path = dir.join(filename);

    if !path.exists() {
        return path;
    }

    let (stem, extension) = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => (
            s.to_string(),
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string(),
        ),
        None => (filename.to_string(), String::new()),
    };

    let mut counter = 1;
    loop {
        let new_filename = if extension.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, extension)
        };

        path = dir.join(&new_filename);
        if !path.exists() {
            break;
        }
        counter += 1;
    }

    path
}
