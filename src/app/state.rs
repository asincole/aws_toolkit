use crate::app::AppMode;
use crate::app::state::s3_bucket::S3BucketState;
use crate::app::state::s3_object::S3ObjectState;
use crate::aws::AWS;
use crate::aws::s3_client::S3Client;
use std::sync::Arc;
use std::time::Instant;

mod s3_bucket;
mod s3_object;

#[derive(Debug)]
pub enum LoadingState {
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug)]
struct NotificationMessage {
    status_message: String,
    status_message_time: Instant,
}

/// Represents the complete state of the application
#[derive(Debug)]
pub struct AppState {
    pub aws_config: AWS,
    pub mode: AppMode,
    pub s3_client: Arc<S3Client>,
    pub s3_bucket: S3BucketState,
    pub s3_object: S3ObjectState,
    pub status_message: Option<String>,
    pub status_message_time: Option<Instant>,
    pub exit: bool,
}

impl AppState {
    pub fn new(aws_config: AWS, s3_client: Arc<S3Client>) -> Self {
        Self {
            aws_config,
            mode: AppMode::BucketList,
            status_message: None,
            s3_bucket: S3BucketState::new(s3_client.clone()),
            s3_object: S3ObjectState::new(s3_client.clone()),
            s3_client,
            exit: false,
            status_message_time: None,
        }
    }
}
