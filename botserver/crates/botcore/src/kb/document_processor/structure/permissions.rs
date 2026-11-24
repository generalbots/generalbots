//! Permission-aware retrieval over chunk metadata.
//!
//! Filtering happens before any chunk content is returned: an unauthorized
//! chunk is simply absent from the result, so the caller gets neither the text
//! nor an existence hint. Tenant mismatch is always denied.

use super::types::{ChunkPermissions, StructuredChunk, Visibility};

/// Identity of the principal requesting retrieval.
#[derive(Debug, Clone, Copy)]
pub struct AccessContext<'a> {
    pub tenant_id: &'a str,
    pub user_id: &'a str,
    pub groups: &'a [String],
}

/// Whether a single chunk is readable by the principal.
pub fn can_read(chunk: &StructuredChunk, context: &AccessContext<'_>) -> bool {
    is_allowed(&chunk.permissions, context)
}

/// Returns only the chunks the principal may read, preserving input order.
pub fn filter_for_read(
    chunks: &[StructuredChunk],
    context: &AccessContext<'_>,
) -> Vec<StructuredChunk> {
    chunks
        .iter()
        .filter(|chunk| can_read(chunk, context))
        .cloned()
        .collect()
}

fn is_allowed(acl: &ChunkPermissions, context: &AccessContext<'_>) -> bool {
    if acl.tenant_id != context.tenant_id {
        return false;
    }
    match acl.visibility {
        Visibility::Public => true,
        Visibility::Workspace => !context.user_id.is_empty(),
        Visibility::Private => {
            let by_user = !context.user_id.is_empty()
                && acl.allowed_users.iter().any(|user| user == context.user_id);
            let by_group = context
                .groups
                .iter()
                .any(|group| acl.allowed_groups.contains(group));
            by_user || by_group
        }
    }
}
