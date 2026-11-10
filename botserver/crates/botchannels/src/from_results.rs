use super::*;

impl MultiPostResult {
    /// Aggregates one post attempt per target.
    ///
    /// `default_channel` labels the synthesized result for an attempt that
    /// failed before reaching a provider (`Err(_)` carries no channel type).
    /// It used to be hardcoded to `Twitter`, so a failed Instagram post was
    /// reported against Twitter in the UI and in the audit trail.
    pub fn from_results(
        default_channel: ChannelType,
        results: Vec<Result<PostResult, ChannelError>>,
    ) -> Self {
        let total = results.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut post_results = Vec::with_capacity(total);

        for result in results {
            match result {
                Ok(r) => {
                    if r.success {
                        successful += 1;
                    } else {
                        failed += 1;
                    }
                    post_results.push(r);
                }
                Err(e) => {
                    failed += 1;
                    post_results.push(PostResult {
                        success: false,
                        channel_type: default_channel.clone(),
                        post_id: None,
                        url: None,
                        error: Some(e.to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        Self {
            total,
            successful,
            failed,
            results: post_results,
        }
    }
}

#[cfg(test)]
#[path = "result_tests.rs"]
mod result_tests;
