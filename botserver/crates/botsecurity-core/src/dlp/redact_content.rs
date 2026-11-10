use super::*;

impl DlpManager {
    /// Rebuilds the content with every match replaced by its redaction label.
    ///
    /// #2066 — the previous implementation rewrote the string in place from
    /// the last match to the first. Detection patterns overlap (a card number
    /// also matches the SSN shape), so a replacement changed the length and a
    /// later `replace_range` could slice past the end or inside a multi-byte
    /// character: a panic, and the release profile aborts the process on
    /// panic. Rebuilding against the original offsets is overlap-safe.
    fn redact_with<F>(content: &str, matches: &[DlpMatch], replacement: F) -> String
    where
        F: Fn(&DlpMatch) -> String,
    {
        let mut sorted_matches: Vec<&DlpMatch> = matches.iter().collect();
        // Widest span first at each start: a card number also matches the SSN
        // shape, and redacting the longer span keeps the whole secret out.
        sorted_matches.sort_by_key(|m| (m.start_position, std::cmp::Reverse(m.end_position)));

        let mut redacted = String::with_capacity(content.len());
        let mut cursor = 0usize;

        for dlp_match in sorted_matches {
            let start = dlp_match.start_position.min(content.len());
            let end = dlp_match.end_position.min(content.len());
            // Skip matches that are malformed, out of order or already covered.
            if start < cursor
                || start >= end
                || !content.is_char_boundary(start)
                || !content.is_char_boundary(end)
            {
                continue;
            }
            redacted.push_str(&content[cursor..start]);
            redacted.push_str(&replacement(dlp_match));
            cursor = end;
        }

        redacted.push_str(&content[cursor..]);
        redacted
    }

    pub fn redact_content(&self, content: &str, matches: &[DlpMatch]) -> String {
        Self::redact_with(content, matches, |m| m.data_type.redaction_pattern().to_string())
    }

    pub fn partial_redact_content(&self, content: &str, matches: &[DlpMatch]) -> String {
        Self::redact_with(content, matches, DlpMatch::masked_value)
    }

    pub async fn add_policy(&self, policy: DlpPolicy) {
        let mut policies = self.policies.write().await;
        policies.push(policy);
    }

    pub async fn remove_policy(&self, policy_id: Uuid) -> bool {
        let mut policies = self.policies.write().await;
        let initial_len = policies.len();
        policies.retain(|p| p.id != policy_id);
        policies.len() < initial_len
    }

    pub async fn get_policies(&self) -> Vec<DlpPolicy> {
        let policies = self.policies.read().await;
        policies.clone()
    }

    pub async fn add_custom_pattern(
        &self,
        name: &str,
        pattern: &str,
        data_type: SensitiveDataType,
    ) -> Result<()> {
        let regex = crate::re_cache::dynamic(pattern)
            .ok_or_else(|| anyhow!("Invalid regex pattern"))?;

        let mut patterns = self.custom_patterns.write().await;
        patterns.push((name.to_string(), regex, data_type));

        info!("Added custom DLP pattern: {}", name);
        Ok(())
    }

    pub async fn record_violation(
        &self,
        policy_id: Uuid,
        policy_name: &str,
        user_id: Option<Uuid>,
        result: &DlpScanResult,
        content: &str,
    ) {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = hex::encode(hasher.finalize());

        let violation = DlpViolation {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            policy_id,
            policy_name: policy_name.to_string(),
            user_id,
            data_types: result.data_types_found().into_iter().collect(),
            action_taken: result.action,
            severity: result.severity,
            content_hash,
            match_count: result.match_count(),
        };

        let mut violations = self.violations.write().await;
        violations.push(violation);
    }

    pub async fn get_violations(
        &self,
        limit: usize,
        user_id: Option<Uuid>,
    ) -> Vec<DlpViolation> {
        let violations = self.violations.read().await;

        let filtered: Vec<DlpViolation> = if let Some(uid) = user_id {
            violations
                .iter()
                .filter(|v| v.user_id == Some(uid))
                .cloned()
                .collect()
        } else {
            violations.clone()
        };

        filtered.into_iter().rev().take(limit).collect()
    }

    pub fn config(&self) -> &DlpConfig {
        &self.config
    }
}
