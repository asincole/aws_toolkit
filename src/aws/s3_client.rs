use aws_sdk_s3::types::Bucket;
use aws_sdk_s3::Client;
use aws_types::SdkConfig;
use color_eyre::eyre::bail;
use color_eyre::Result;
use std::collections::HashMap;

pub struct S3Client {
    client: Client,
    buckets_map: HashMap<String, Bucket>,
    bucket_continuation_token: Option<String>,
}

impl S3Client {
    pub fn new(config: &SdkConfig) -> Self {
        let client = Client::new(config);
        Self {
            client,
            buckets_map: Default::default(),
            bucket_continuation_token: None,
        }
    }

    pub async fn get_bucket_list(&mut self) -> Result<Vec<Bucket>> {
        let buckets_output = self
            .client
            .list_buckets()
            .into_paginator()
            .send()
            .next()
            .await
            .transpose()?;

        let mut buckets = Vec::new();

        if let Some(output) = buckets_output {
            self.bucket_continuation_token = output.continuation_token;

            if let Some(bucket_list) = output.buckets {
                for bucket in bucket_list {
                    let bucket_name = bucket
                        .name
                        .clone()
                        .unwrap_or_else(|| "Unknown Bucket".to_string());
                    self.buckets_map.insert(bucket_name, bucket.clone());
                    buckets.push(bucket);
                }
            }
        }

        Ok(buckets)
    }

    pub async fn list_objects(&self, bucket: &str) -> Result<()> {
        let mut response = self
            .client
            .list_objects_v2()
            .bucket(bucket.to_owned())
            .max_keys(20)
            .into_paginator()
            .send();

        while let Some(result) = response.next().await {
            match result {
                Ok(output) => {
                    for object in output.contents() {
                        println!(" - {}", object.key().unwrap_or("Unknown"));
                    }
                }
                Err(err) => {
                    bail!(
                        "encountered error while getting objects for bucket {bucket} with error: {err:?}"
                    )
                }
            }
        }

        Ok(())
    }
}
