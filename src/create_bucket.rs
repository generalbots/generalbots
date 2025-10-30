use std::fs;
use std::path::Path;

pub fn create_bucket(bucket_name: &str) -> std::io::Result<()> {
    let bucket_path = Path::new("buckets").join(bucket_name);
    fs::create_dir_all(&bucket_path)?;
    Ok(())
}
