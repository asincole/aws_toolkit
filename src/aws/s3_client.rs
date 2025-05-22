use aws_sdk_s3::types::{Bucket, Object};
use aws_sdk_s3::Client;
use aws_types::SdkConfig;
use color_eyre::eyre::eyre;
use color_eyre::Result;

#[derive(Debug)]
pub struct S3Client {
    client: Client,
}

impl S3Client {
    pub fn new(config: &SdkConfig) -> Self {
        let client = Client::new(config);
        Self { client }
    }

    pub async fn get_bucket_list(
        &mut self,
        continuation_token: Option<String>,
    ) -> Result<(Vec<Bucket>, Option<String>)> {
        let mut request = self.client.list_buckets();
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let buckets_output = request.send().await?;

        let next_token = buckets_output.continuation_token;
        let buckets = buckets_output.buckets.unwrap_or_default();

        Ok((buckets, next_token))
    }

    pub async fn list_objects(
        &self,
        bucket: &str,
        continuation_token: Option<String>,
        prefix: Option<String>,
        max_keys: i32,
    ) -> Result<(Vec<Object>, Option<String>)> {
        let mut request = self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .max_keys(max_keys);

        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        if let Some(prefix_str) = prefix {
            request = request.prefix(prefix_str);
        }

        let response = request.send().await?;

        let objects = response.contents().to_vec();
        let next_token = response.next_continuation_token().map(String::from);

        Ok((objects, next_token))
    }

    /// Returns (content, content_type)
    pub async fn get_object_content(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(String, Option<String>)> {
        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get object: {}", e))?;

        let content_type = response.content_type().map(String::from);
        let body_bytes = response
            .body
            .collect()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to collect object body: {}", e))?
            .into_bytes();

        // Try to decode as UTF-8. If it fails, it's likely binary or a different encoding.
        let content_string = String::from_utf8(body_bytes.to_vec())
            .unwrap_or_else(|_| String::from("[Binary content - not valid UTF-8]"));

        Ok((content_string, content_type))
    }

    /// Fetches the raw bytes of an S3 object.
    pub async fn download_object_bytes(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|sdk_err| {
                eyre!(
                    "S3 SDK Error getting object for download (bucket: {}, key: {}): {}",
                    bucket,
                    key,
                    sdk_err
                )
            })?;

        let byte_stream = response.body;
        let collected_data = byte_stream.collect().await.map_err(|sdk_err| {
            eyre!(
                "S3 SDK Error collecting object body for download (bucket: {}, key: {}): {}",
                bucket,
                key,
                sdk_err
            )
        })?;

        Ok(collected_data.into_bytes().to_vec())
    }
}
