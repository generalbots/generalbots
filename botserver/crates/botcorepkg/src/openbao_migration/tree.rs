//! KV mounts as plain data: walking a store into memory, writing it back and
//! comparing two copies. No I/O here beyond the [`KvStore`] calls, so the
//! walk is tested against an in-memory store.

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Fields of one secret.
pub type Fields = Map<String, Value>;

/// Upper bound on the secrets read from one mount, so a store that keeps
/// answering with new folders cannot hold the start forever.
const MAX_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvVersion {
    V1,
    V2,
}

/// A KV secrets engine mount, `path` ending in `/` (e.g. `secret/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub path: String,
    pub version: KvVersion,
}

impl Mount {
    /// URL path segments (after `v1`) that list folder `dir` (`""` or `a/b/`).
    #[must_use]
    pub fn list_segments(&self, dir: &str) -> Vec<String> {
        let mut out = self.base_segments(Some("metadata"));
        out.extend(split(dir));
        out.push(String::new());
        out
    }

    /// URL path segments (after `v1`) of secret `path`.
    #[must_use]
    pub fn data_segments(&self, path: &str) -> Vec<String> {
        let mut out = self.base_segments(Some("data"));
        out.extend(split(path));
        out
    }

    fn base_segments(&self, v2_part: Option<&str>) -> Vec<String> {
        let mut out = split(&self.path);
        if self.version == KvVersion::V2 {
            out.extend(v2_part.map(str::to_string));
        }
        out
    }

    /// Request body that writes `fields`.
    #[must_use]
    pub fn write_body(&self, fields: &Fields) -> Value {
        match self.version {
            KvVersion::V1 => Value::Object(fields.clone()),
            KvVersion::V2 => serde_json::json!({ "data": fields }),
        }
    }

    /// Fields in a read response; `None` for a deleted latest version.
    #[must_use]
    pub fn fields_of(&self, response: &Value) -> Option<Fields> {
        let data = response.get("data")?;
        let fields = match self.version {
            KvVersion::V1 => data,
            KvVersion::V2 => data.get("data")?,
        };
        fields.as_object().cloned()
    }
}

fn split(path: &str) -> Vec<String> {
    path.split('/').filter(|s| !s.is_empty()).map(str::to_string).collect()
}

/// Reads and writes of one KV store, implemented over HTTP and in tests.
pub(crate) trait KvStore {
    async fn list(&self, mount: &Mount, dir: &str) -> Result<Vec<String>>;
    async fn read(&self, mount: &Mount, path: &str) -> Result<Option<Fields>>;
    async fn write(&self, mount: &Mount, path: &str, fields: &Fields) -> Result<()>;
}

/// Every secret of one mount, by path.
#[derive(Debug, Default)]
pub struct MountDump {
    pub secrets: BTreeMap<String, Fields>,
}

/// The secrets of several mounts, held in memory only and wiped on drop.
#[derive(Default)]
pub struct Snapshot {
    pub mounts: Vec<(Mount, MountDump)>,
}

impl Snapshot {
    /// Secrets per mount path, for reports and checks.
    #[must_use]
    pub fn counts(&self) -> Vec<(String, usize)> {
        self.mounts.iter().map(|(m, d)| (m.path.clone(), d.secrets.len())).collect()
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        for (_, dump) in &mut self.mounts {
            for fields in dump.secrets.values_mut() {
                fields.values_mut().for_each(wipe_value);
            }
        }
    }
}

/// Overwrites the string contents of `value` before it is freed.
pub fn wipe_value(value: &mut Value) {
    match value {
        Value::String(s) => wipe_string(s),
        Value::Array(items) => items.iter_mut().for_each(wipe_value),
        Value::Object(map) => map.values_mut().for_each(wipe_value),
        _ => {}
    }
}

/// Overwrites `s` with zeros before it is freed.
pub fn wipe_string(s: &mut String) {
    let mut bytes = std::mem::take(s).into_bytes();
    bytes.fill(0);
    std::hint::black_box(&bytes);
}

/// Reads every secret of `mounts` from `store`.
pub(crate) async fn export<S: KvStore>(store: &S, mounts: &[Mount]) -> Result<Snapshot> {
    let mut snapshot = Snapshot::default();
    for mount in mounts {
        let mut dump = MountDump::default();
        let mut pending = vec![String::new()];
        while let Some(dir) = pending.pop() {
            for key in store.list(mount, &dir).await? {
                let path = format!("{dir}{key}");
                if key.ends_with('/') {
                    pending.push(path);
                } else if let Some(fields) = store.read(mount, &path).await? {
                    dump.secrets.insert(path, fields);
                }
                if dump.secrets.len() + pending.len() > MAX_ENTRIES {
                    return Err(anyhow!("mount {} holds more than {MAX_ENTRIES} entries", mount.path));
                }
            }
        }
        snapshot.mounts.push((mount.clone(), dump));
    }
    Ok(snapshot)
}

/// Writes every secret of `snapshot` into `store`; `after_write` is told how
/// many were written so far.
pub(crate) async fn import<S: KvStore>(
    store: &S,
    snapshot: &Snapshot,
    mut after_write: impl FnMut(usize),
) -> Result<usize> {
    let mut written = 0;
    for (mount, dump) in &snapshot.mounts {
        for (path, fields) in &dump.secrets {
            store.write(mount, path, fields).await?;
            written += 1;
            after_write(written);
        }
    }
    Ok(written)
}

/// Checks that `actual` holds the same secrets as `expected`, mount by
/// mount. Errors name mounts and paths, never values.
pub fn verify(expected: &Snapshot, actual: &Snapshot) -> Result<()> {
    for (mount, want) in &expected.mounts {
        let got = actual
            .mounts
            .iter()
            .find(|(m, _)| m.path == mount.path)
            .map(|(_, d)| d)
            .ok_or_else(|| anyhow!("mount {} is missing after the move", mount.path))?;
        if got.secrets.len() != want.secrets.len() {
            return Err(anyhow!(
                "mount {} has {} secrets after the move, {} before",
                mount.path,
                got.secrets.len(),
                want.secrets.len()
            ));
        }
        for (path, fields) in &want.secrets {
            if got.secrets.get(path) != Some(fields) {
                return Err(anyhow!("secret {}{path} differs after the move", mount.path));
            }
        }
    }
    Ok(())
}
