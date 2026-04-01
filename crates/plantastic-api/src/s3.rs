//! S3 helper functions for object storage operations.

use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::time::Duration;

/// Create an S3 client from environment configuration.
///
/// On Lambda, credentials come from the execution role.
/// Locally, uses AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY env vars
/// or ~/.aws/credentials.
pub async fn create_s3_client() -> Client {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    Client::new(&config)
}

/// Upload bytes to S3.
///
/// # Errors
///
/// Returns `S3Error` if the upload fails.
pub async fn upload_bytes(
    client: &Client,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<(), S3Error> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(bytes))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| S3Error(format!("upload to {key}: {e}")))?;
    Ok(())
}

/// Download bytes from S3.
///
/// # Errors
///
/// Returns `S3Error` if the download fails.
pub async fn download_bytes(client: &Client, bucket: &str, key: &str) -> Result<Vec<u8>, S3Error> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| S3Error(format!("download {key}: {e}")))?;

    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| S3Error(format!("read body for {key}: {e}")))?
        .into_bytes()
        .to_vec();

    Ok(bytes)
}

/// Generate a presigned GET URL for an S3 object.
///
/// # Errors
///
/// Returns `S3Error` if presigning fails.
pub async fn presigned_get_url(
    client: &Client,
    bucket: &str,
    key: &str,
    expires_secs: u64,
) -> Result<String, S3Error> {
    let presign_config = PresigningConfig::expires_in(Duration::from_secs(expires_secs))
        .map_err(|e| S3Error(format!("presign config: {e}")))?;

    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presign_config)
        .await
        .map_err(|e| S3Error(format!("presign {key}: {e}")))?;

    Ok(presigned.uri().to_string())
}

/// S3 operation error.
#[derive(Debug)]
pub struct S3Error(pub String);

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S3 error: {}", self.0)
    }
}

impl std::error::Error for S3Error {}
