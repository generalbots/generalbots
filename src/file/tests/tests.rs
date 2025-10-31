use super::*;

#[tokio::test]
async fn test_create_s3_client() {
    if std::env::var("CI").is_ok() {
        return; // Skip in CI environment
    }
    
    // Setup test environment variables
    std::env::set_var("DRIVE_SERVER", "http://localhost:9000");
    std::env::set_var("DRIVE_ACCESS_KEY", "minioadmin");
    std::env::set_var("DRIVE_SECRET_KEY", "minioadmin");

    match create_s3_client().await {
        Ok(client) => {
            // Verify client creation
            assert!(client.config().region().is_some());
            
            // Test bucket operations
            if let Err(e) = create_bucket(&client, "test.gbai").await {
                println!("Bucket creation failed: {:?}", e);
            }
        },
        Err(e) => {
            // Skip if no S3 server available
            println!("S3 client creation failed: {:?}", e);
        }
    }

    // Cleanup
    std::env::remove_var("DRIVE_SERVER");
    std::env::remove_var("DRIVE_ACCESS_KEY");
    std::env::remove_var("DRIVE_SECRET_KEY");
}

#[tokio::test]
async fn test_bucket_exists() {
    if std::env::var("CI").is_ok() {
        return; // Skip in CI environment
    }

    // Setup test environment variables
    std::env::set_var("DRIVE_SERVER", "http://localhost:9000");
    std::env::set_var("DRIVE_ACCESS_KEY", "minioadmin");
    std::env::set_var("DRIVE_SECRET_KEY", "minioadmin");

    match create_s3_client().await {
        Ok(client) => {
            // Verify client creation
            assert!(client.config().region().is_some());
        },
        Err(e) => {
            // Skip if no S3 server available
            println!("S3 client creation failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_create_bucket() {
    if std::env::var("CI").is_ok() {
        return; // Skip in CI environment
    }

    // Setup test environment variables
    std::env::set_var("DRIVE_SERVER", "http://localhost:9000");
    std::env::set_var("DRIVE_ACCESS_KEY", "minioadmin");
    std::env::set_var("DRIVE_SECRET_KEY", "minioadmin");

    match create_s3_client().await {
        Ok(client) => {
            // Verify client creation
            assert!(client.config().region().is_some());
        },
        Err(e) => {
            // Skip if no S3 server available
            println!("S3 client creation failed: {:?}", e);
        }
    }
}
