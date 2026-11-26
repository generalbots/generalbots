//! OpenBao 2.7 has no `file` storage backend ("unknown storage type file"),
//! so the bundled server keeps its storage in the same `data/vault` folder
//! with OpenBao's integrated (`raft`) storage, as a single node.

/// Raft node id of the single local secrets server.
pub const NODE_ID: &str = "gbo-local";

/// `config.hcl` text with its `storage "file"` block turned into single-node
/// raft storage on the same path. `None` when there is no such block.
#[must_use]
pub fn file_storage_to_raft(text: &str) -> Option<String> {
    let start = text.find("storage \"file\"")?;
    let open = start + text[start..].find('{')?;
    let close = open + text[open..].find('}')?;
    let path_line = text[open + 1..close]
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("path"))?;
    let mut out = String::with_capacity(text.len() + 40);
    out.push_str(&text[..start]);
    out.push_str("storage \"raft\" {\n  ");
    out.push_str(path_line);
    out.push_str("\n  node_id = \"");
    out.push_str(NODE_ID);
    out.push_str("\"\n}");
    out.push_str(&text[close + 1..]);
    Some(out)
}

/// The config already uses the storage OpenBao reads.
#[must_use]
pub fn uses_raft(text: &str) -> bool {
    text.contains("storage \"raft\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DESKTOP: &str = "storage \"file\" { \r\n  path = \"C:/stack/data/vault\" \r\n} \r\n\r\nlistener \"tcp\" { \r\n  address = \"127.0.0.1:8200\" \r\n  tls_disable = true \r\n} \r\ncluster_addr = \"http://127.0.0.1:8201\" \r\nui = true \r\n";

    #[test]
    fn the_desktop_file_storage_becomes_raft_on_the_same_path() {
        let out = file_storage_to_raft(DESKTOP).unwrap_or_default();
        assert!(out.starts_with("storage \"raft\" {\n  path = \"C:/stack/data/vault\"\n  node_id = \"gbo-local\"\n}"), "{out}");
        assert!(!out.contains("\"file\""));
        assert!(out.contains("address = \"127.0.0.1:8200\""));
        assert!(out.contains("cluster_addr"));
        assert!(uses_raft(&out));
    }

    #[test]
    fn a_config_without_file_storage_is_left_alone() {
        assert_eq!(file_storage_to_raft("storage \"raft\" {\n path = \"x\"\n}\n"), None);
        assert_eq!(file_storage_to_raft("storage \"file\" {\n}\n"), None);
    }
}
