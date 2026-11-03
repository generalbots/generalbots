//! #1633 — self-extracting desktop binary.
//!
//! The published executable is the normal `generalbots` app with a compressed
//! stack appended:
//!
//! ```text
//! [ executable ][ payload.tar.zst ][ trailer ]
//! trailer = magic(12) + offset(u64 LE) + length(u64 LE) + sha256(32)
//! ```
//!
//! First run verifies the trailer, extracts the payload into the per-user
//! stack directory and starts the same services the tarball installer uses.
//! Later runs see the `.payload-sha256` marker and skip extraction.
//!
//! `--verify`, `--extract-only` and `--version` are handled by `main` before
//! Tauri starts, so the payload can be inspected offline.

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 12] = b"GB6PAYLOAD!!";
const TRAILER_LEN: u64 = 60;
const MARKER: &str = ".payload-sha256";

#[derive(Debug, Clone)]
pub struct Trailer {
    pub offset: u64,
    pub length: u64,
    pub sha256: [u8; 32],
}

impl Trailer {
    pub fn sha_hex(&self) -> String {
        hex::encode(self.sha256)
    }
}

pub fn stack_dir() -> PathBuf {
    if let Ok(path) = std::env::var("BOTSERVER_STACK_PATH") {
        return PathBuf::from(path);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("generalbots")
        .join("stack")
}

fn current_exe() -> Result<PathBuf> {
    std::env::current_exe().context("cannot locate the running executable")
}

/// Reads the trailer from an executable. `Ok(None)` means the file is a plain
/// development build with no payload appended.
pub fn read_trailer(path: &Path) -> Result<Option<Trailer>> {
    let mut file = File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let len = file.metadata()?.len();
    if len < TRAILER_LEN {
        return Ok(None);
    }
    file.seek(SeekFrom::End(-(TRAILER_LEN as i64)))?;
    let mut trailer = [0u8; TRAILER_LEN as usize];
    file.read_exact(&mut trailer)?;
    if &trailer[..12] != MAGIC {
        return Ok(None);
    }
    let offset = u64::from_le_bytes(
        trailer[12..20]
            .try_into()
            .map_err(|_| anyhow!("invalid payload offset"))?,
    );
    let length = u64::from_le_bytes(
        trailer[20..28]
            .try_into()
            .map_err(|_| anyhow!("invalid payload length"))?,
    );
    let mut sha256 = [0u8; 32];
    sha256.copy_from_slice(&trailer[28..60]);
    if offset
        .checked_add(length)
        .map(|end| end > len - TRAILER_LEN)
        .unwrap_or(true)
    {
        return Err(anyhow!("payload trailer points outside the file"));
    }
    Ok(Some(Trailer {
        offset,
        length,
        sha256,
    }))
}

/// Verifies the trailer digest without extracting anything.
pub fn verify() -> Result<Trailer> {
    let exe = current_exe()?;
    let trailer = read_trailer(&exe)?.ok_or_else(|| {
        anyhow!(
            "{} has no embedded payload (development build)",
            exe.display()
        )
    })?;
    let digest = payload_digest(&exe, &trailer)?;
    if digest != trailer.sha256 {
        return Err(anyhow!(
            "payload checksum mismatch: expected {}, got {}",
            trailer.sha_hex(),
            hex::encode(digest)
        ));
    }
    Ok(trailer)
}

fn payload_digest(path: &Path, trailer: &Trailer) -> Result<[u8; 32]> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(trailer.offset))?;
    let mut remaining = trailer.length;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    while remaining > 0 {
        let want = buffer.len().min(remaining as usize);
        let read = file.read(&mut buffer[..want])?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    if remaining != 0 {
        return Err(anyhow!("payload is truncated"));
    }
    Ok(hasher.finalize().into())
}

/// Extracts the embedded payload to the per-user stack directory. A plain
/// development build (no trailer) is a no-op.
pub fn extract_if_present() -> Result<Option<PathBuf>> {
    let exe = current_exe()?;
    let Some(trailer) = read_trailer(&exe)? else {
        return Ok(None);
    };
    let stack = stack_dir();
    let marker = stack.join(MARKER);
    if marker.exists() {
        if let Ok(previous) = fs::read_to_string(&marker) {
            if previous.trim() == trailer.sha_hex() {
                log::info!(
                    "payload {} already extracted into {}",
                    trailer.sha_hex(),
                    stack.display()
                );
                return Ok(Some(stack));
            }
        }
    }

    log::info!(
        "extracting embedded payload ({} bytes) into {}",
        trailer.length,
        stack.display()
    );
    let digest = payload_digest(&exe, &trailer)?;
    if digest != trailer.sha256 {
        return Err(anyhow!(
            "payload checksum mismatch: expected {}, got {}",
            trailer.sha_hex(),
            hex::encode(digest)
        ));
    }

    fs::create_dir_all(&stack)
        .with_context(|| format!("cannot create {}", stack.display()))?;
    let mut file = File::open(&exe)?;
    file.seek(SeekFrom::Start(trailer.offset))?;
    let reader = file.take(trailer.length);
    let decoder = zstd::stream::read::Decoder::new(reader)
        .map_err(|error| anyhow!("payload is not valid zstd: {error}"))?;
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(&stack)
        .map_err(|error| anyhow!("payload extraction failed: {error}"))?;
    fs::write(&marker, trailer.sha_hex())?;
    log::info!("payload extracted into {}", stack.display());
    Ok(Some(stack))
}

/// Handles the offline inspection flags before Tauri starts. Returns the exit
/// code when one of them was used, `None` for a normal launch.
pub fn handle_cli() -> Option<i32> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("generalbots {}", env!("CARGO_PKG_VERSION"));
        return Some(0);
    }
    if args.iter().any(|arg| arg == "--verify") {
        return Some(match verify() {
            Ok(trailer) => {
                println!(
                    "payload ok: {} bytes at offset {} (sha256 {})",
                    trailer.length,
                    trailer.offset,
                    trailer.sha_hex()
                );
                0
            }
            Err(error) => {
                eprintln!("payload verification failed: {error}");
                1
            }
        });
    }
    if args.iter().any(|arg| arg == "--extract-only") {
        return Some(match extract_if_present() {
            Ok(Some(stack)) => {
                println!("payload extracted into {}", stack.display());
                0
            }
            Ok(None) => {
                eprintln!("this executable has no embedded payload");
                1
            }
            Err(error) => {
                eprintln!("payload extraction failed: {error}");
                1
            }
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{read_trailer, Trailer};

    fn write_payload(path: &std::path::Path, body: &[u8]) -> Trailer {
        use sha2::{Digest, Sha256};
        let mut bytes = b"executable".to_vec();
        let offset = bytes.len() as u64;
        bytes.extend_from_slice(body);
        let length = body.len() as u64;
        bytes.extend_from_slice(b"GB6PAYLOAD!!");
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&length.to_le_bytes());
        let digest: [u8; 32] = Sha256::digest(body).into();
        bytes.extend_from_slice(&digest);
        std::fs::write(path, bytes).expect("write fixture");
        Trailer {
            offset,
            length,
            sha256: digest,
        }
    }

    #[test]
    fn reads_a_valid_trailer() {
        let path = std::env::temp_dir().join(format!("gb6-{}", uuid_like()));
        let expected = write_payload(&path, b"payload-bytes");
        let found = read_trailer(&path).expect("read").expect("trailer present");
        assert_eq!(found.offset, expected.offset);
        assert_eq!(found.length, expected.length);
        assert_eq!(found.sha256, expected.sha256);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn plain_binaries_have_no_trailer() {
        let path = std::env::temp_dir().join(format!("gb6-plain-{}", uuid_like()));
        std::fs::write(&path, b"not a payload").expect("write fixture");
        assert!(read_trailer(&path).expect("read").is_none());
        let _ = std::fs::remove_file(path);
    }

    fn uuid_like() -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{nanos}")
    }
}
