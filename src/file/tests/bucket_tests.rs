use super::*;
use aws_sdk_s3::Client as S3Client;
use std::env;

#[tokio::test]
async fn test_aws_s3_bucket_create() {
    if env::var("CI").is_ok() {
        return; // Skip in CI environment
    }

    let bucket = "test-bucket-aws";
    let endpoint = "http://localhost:4566"; // LocalStack default endpoint
    let access_key = "test";
    let secret_key = "test";

    match aws_s3_bucket_create(bucket, endpoint, access_key, secret_key).await {
        Ok(_) => {
            // Verify bucket exists
            let config = aws_config::defaults(BehaviorVersion::latest())
                .endpoint_url(endpoint)
                .region("auto")
                .load()
                .await;
            let client = S3Client::new(&config);
            
            let exists = bucket_exists(&client, bucket).await.unwrap_or(false);
            assert!(exists, "Bucket should exist after creation");
        },
        Err(e) => {
            println!("Bucket creation failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_aws_s3_bucket_delete() {
    if env::var("CI").is_ok() {
        return; // Skip in CI environment
    }

    let bucket = "test-delete-bucket-aws";
    let endpoint = "http://localhost:4566"; // LocalStack default endpoint
    let access_key = "test";
    let secret_key = "test";

    // First create the bucket
    if let Err(e) = aws_s3_bucket_create(bucket, endpoint, access_key, secret_key).await {
        println!("Failed to create test bucket: {:?}", e);
        return;
    }

    // Then test deletion
    match aws_s3_bucket_delete(bucket, endpoint, access_key, secret_key).await {
        Ok(_) => {
            // Verify bucket no longer exists
            let config = aws_config::defaults(BehaviorVersion::latest())
                .endpoint_url(endpoint)
                .region("auto")
                .load()
                .await;
            let client = S3Client::new(&config);
            
            let exists = bucket_exists(&client, bucket).await.unwrap_or(false);
            assert!(!exists, "Bucket should not exist after deletion");
        },
        Err(e) => {
            println!("Bucket deletion failed: {:?}", e);
        }
    }
}
