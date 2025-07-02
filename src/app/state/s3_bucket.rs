use crate::app::AppMode;
use crate::app::actions::AppActions;
use crate::app::state::LoadingState;
use crate::app::state::s3_object::S3ObjectState;
use crate::aws::s3_client::S3Client;
use crate::search::SearchBar;
use crate::ui::ScrollableList;
use aws_sdk_s3::types::Bucket;
use std::sync::Arc;

#[derive(Debug)]
pub struct S3BucketState {
    pub bucket_list: ScrollableList<Bucket>,
    pub bucket_continuation_token: Option<String>,
    pub current_bucket: Option<String>,
    pub search_bar: SearchBar,
    pub s3_client: Arc<S3Client>,
    loading_state: LoadingState,
}

impl S3BucketState {
    pub fn new(s3_client: Arc<S3Client>) -> Self {
        Self {
            bucket_list: ScrollableList::new("S3 Buckets"),
            bucket_continuation_token: None,
            current_bucket: None,
            search_bar: SearchBar::default(),
            s3_client,
            loading_state: LoadingState::Idle,
        }
    }

    /// Load S3 buckets into the application state
    pub async fn load_buckets(&mut self) -> color_eyre::Result<()> {
        self.loading_state = LoadingState::Loading;
        let (buckets, next_token) = self
            .s3_client
            .get_bucket_list(self.bucket_continuation_token.clone())
            .await?;

        self.bucket_list.set_has_more(next_token.is_some());
        self.bucket_continuation_token = next_token;

        self.bucket_list.append_items(buckets);

        // Initialise filtered indices with all items
        self.bucket_list.filtered_indices = (0..self.bucket_list.items.len()).collect();

        self.bucket_list.first();
        self.loading_state = LoadingState::Loaded;
        Ok(())
    }

    /// Select a bucket and load its objects
    pub async fn select_bucket(&mut self) -> color_eyre::Result<()> {
        if let Some(bucket) = self.bucket_list.selected_item() {
            if let Some(name) = &bucket.name {
                self.current_bucket = Some(name.clone());
            }
        }
        Ok(())
    }

    /// Apply the current search query to the appropriate list based on the current mode
    pub fn apply_current_search(&mut self) {
        self.bucket_list.apply_search(&self.search_bar, |bucket| {
            bucket
                .name
                .clone()
                .unwrap_or_else(|| "Unknown Bucket".to_string())
        });
    }

    pub async fn handle_bucket_action(
        &mut self,
        action: AppActions,
        app_mode: &mut AppMode,
        s3_object_state: &mut S3ObjectState,
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
            AppActions::StartSearch => {
                self.search_bar.toggle();
            }
            AppActions::MoveDown => self.bucket_list.next(),
            AppActions::MoveUp => self.bucket_list.previous(),
            AppActions::MoveToTop => self.bucket_list.first(),
            AppActions::MoveToBottom => self.bucket_list.last(),
            AppActions::PageDown => {
                for _ in 0..10 {
                    self.bucket_list.next();
                }
            }
            AppActions::PageUp => {
                for _ in 0..10 {
                    self.bucket_list.previous();
                }
            }
            AppActions::HalfPageDown => {
                for _ in 0..5 {
                    self.bucket_list.next();
                }
            }
            AppActions::HalfPageUp => {
                for _ in 0..5 {
                    self.bucket_list.previous();
                }
            }
            AppActions::Enter => {
                self.select_bucket().await?;
                if let Some(bucket_name) = &self.current_bucket {
                    s3_object_state.object_list =
                        ScrollableList::new(format!("Contents of {}", bucket_name));
                    s3_object_state.current_bucket = bucket_name.clone();
                    *app_mode = AppMode::ObjectList; // Change to ObjectList mode
                    s3_object_state.load_objects().await?;
                }
            }
            AppActions::Refresh => {
                self.bucket_list = ScrollableList::new("S3 Buckets");
                self.bucket_continuation_token = None;
                self.load_buckets().await?;
            }
            AppActions::LoadMore => {
                if self.bucket_continuation_token.is_some() {
                    self.load_buckets().await?;
                }
            }
            AppActions::ClearSearch => {
                self.search_bar.clear();
                self.bucket_list
                    .apply_search(&self.search_bar, |object| "Unknown".to_string());
            }
            _ => {}
        }

        Ok(())
    }
}
