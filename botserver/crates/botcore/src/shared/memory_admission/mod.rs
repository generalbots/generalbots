//! Per-request memory admission control for expensive document work (#2530).
//!
//! The Office apps parse OOXML archives synchronously on the request path. A
//! small upload can expand into hundreds of megabytes of heap before any cap
//! is checked again, and an unbounded number of such requests can kill the
//! whole process. This module is the shared, framework-agnostic guard those
//! paths can adopt:
//!
//! - [`CgroupMemory`] reads the effective cgroup v2/v1 ceiling from
//!   `/sys/fs/cgroup` with a conservative fallback when it is absent,
//!   unlimited or unreadable.
//! - [`MemoryPolicy`] is the central upload/JSON class table: a fixed upload
//!   cap per class plus a peak-heap multiplier, with the request budget derived
//!   from the cgroup ceiling rather than from a constant.
//! - [`MemoryAdmission`] hands out RAII permits from a bounded per-class
//!   semaphore and reports the queue depth for observability.
//! - [`ImportBudget`] is the shared **pure** pre-materialisation predicate:
//!   byte/row/page limits resolved once from the environment, checked before
//!   any importer builds its in-memory model, with a typed [`BudgetError`].
//! - [`AdmissionError`] maps refusals to 413/507/429 with a stable code and an
//!   i18n key, and emits a structured event containing no document content.
//!
//! No background task is started: detection is a one-shot read and each permit
//! is released by `Drop`, so there is no daemon loop to shut down.

mod budget;
mod cgroup;
mod error;
mod gate;
mod policy;

pub use budget::*;
pub use cgroup::*;
pub use error::*;
pub use gate::*;
pub use policy::*;
