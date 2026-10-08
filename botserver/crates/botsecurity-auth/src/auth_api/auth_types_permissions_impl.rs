use super::Permission;

impl Permission {
    pub fn from_alias(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        Self::from_alias_chunk1(&s_lower)
            .or_else(|| Self::from_alias_chunk2(&s_lower))
            .or_else(|| Self::from_alias_chunk3(&s_lower))
            .or_else(|| Self::from_alias_chunk4(&s_lower))
            .or_else(|| Self::from_alias_chunk5(&s_lower))
            .or_else(|| Self::from_alias_chunk6(&s_lower))
            .or_else(|| Self::from_alias_chunk7(&s_lower))
            .or_else(|| Self::from_alias_chunk8(&s_lower))
    }
}

