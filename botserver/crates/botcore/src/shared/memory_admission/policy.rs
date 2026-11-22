//! Central memory policy table for upload/JSON classes (#2530, Phase 1).
//!
//! The fixed caps mirror the existing endpoint upload caps (25 MB Docs/Sheets,
//! 50 MB Slides, 100 MB Drive). They bound the upload only. The separate
//! cgroup-derived budget bounds the estimated peak heap, which is what actually
//! kills the process. Keeping the two apart is the point of this table.

use super::cgroup::CgroupMemory;
use serde::Serialize;

pub const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum WorkClass {
    DocsImport,
    SheetsImport,
    SlidesImport,
    DriveUpload,
    OfficeJsonSave,
    OfficeExport,
}

impl WorkClass {
    pub const ALL: [Self; 6] = [
        Self::DocsImport,
        Self::SheetsImport,
        Self::SlidesImport,
        Self::DriveUpload,
        Self::OfficeJsonSave,
        Self::OfficeExport,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::DocsImport => 0,
            Self::SheetsImport => 1,
            Self::SlidesImport => 2,
            Self::DriveUpload => 3,
            Self::OfficeJsonSave => 4,
            Self::OfficeExport => 5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DocsImport => "docs_import",
            Self::SheetsImport => "sheets_import",
            Self::SlidesImport => "slides_import",
            Self::DriveUpload => "drive_upload",
            Self::OfficeJsonSave => "office_json_save",
            Self::OfficeExport => "office_export",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassPolicy {
    pub class: WorkClass,
    /// Existing endpoint upload cap: reject above it with HTTP 413.
    pub max_input_bytes: u64,
    /// Conservative expansion factor from compressed archive bytes to peak
    /// heap when parsed, recalculated and duplicated for passthrough.
    pub peak_multiplier: u32,
    /// Bounded concurrency for this class. Heavy imports/exports are one at a
    /// time; the rest queue behind the semaphore.
    pub max_concurrency: usize,
}

impl ClassPolicy {
    pub fn estimated_peak_bytes(&self, input_bytes: u64) -> u64 {
        input_bytes.saturating_mul(u64::from(self.peak_multiplier))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryPolicy {
    pub classes: [ClassPolicy; 6],
    /// Peak-heap budget for a single request, derived from the cgroup ceiling.
    pub budget_bytes: u64,
    pub ceiling: CgroupMemory,
}

impl MemoryPolicy {
    /// Builds the table. Caps stay per-endpoint; the budget follows the
    /// detected ceiling so a small container refuses earlier than a laptop.
    pub fn from_ceiling(ceiling: CgroupMemory) -> Self {
        let entry = |class: WorkClass,
                     cap_mib: u64,
                     peak_multiplier: u32,
                     max_concurrency: usize| ClassPolicy {
            class,
            max_input_bytes: cap_mib * MIB,
            peak_multiplier,
            max_concurrency,
        };

        Self {
            classes: [
                entry(WorkClass::DocsImport, 25, 12, 1),
                entry(WorkClass::SheetsImport, 25, 24, 1),
                entry(WorkClass::SlidesImport, 50, 12, 1),
                entry(WorkClass::DriveUpload, 100, 4, 2),
                entry(WorkClass::OfficeJsonSave, 8, 8, 2),
                entry(WorkClass::OfficeExport, 50, 16, 1),
            ],
            budget_bytes: ceiling.per_request_budget(),
            ceiling,
        }
    }

    pub fn for_class(&self, class: WorkClass) -> ClassPolicy {
        self.classes[class.index()]
    }

    pub fn classes(&self) -> &[ClassPolicy; 6] {
        &self.classes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::memory_admission::CgroupSource;

    fn ceiling(limit_bytes: u64) -> CgroupMemory {
        CgroupMemory {
            limit_bytes,
            current_bytes: None,
            source: CgroupSource::V2,
        }
    }

    #[test]
    fn caps_match_the_endpoint_upload_limits() {
        let policy = MemoryPolicy::from_ceiling(ceiling(4 * 1024 * MIB));
        assert_eq!(
            policy.for_class(WorkClass::DocsImport).max_input_bytes,
            25 * MIB
        );
        assert_eq!(
            policy.for_class(WorkClass::SheetsImport).max_input_bytes,
            25 * MIB
        );
        assert_eq!(
            policy.for_class(WorkClass::SlidesImport).max_input_bytes,
            50 * MIB
        );
        assert_eq!(
            policy.for_class(WorkClass::DriveUpload).max_input_bytes,
            100 * MIB
        );
        assert_eq!(
            policy.for_class(WorkClass::OfficeJsonSave).max_input_bytes,
            8 * MIB
        );
        assert_eq!(
            policy.for_class(WorkClass::OfficeExport).max_input_bytes,
            50 * MIB
        );
    }

    #[test]
    fn every_class_has_a_concurrency_of_at_least_one() {
        let policy = MemoryPolicy::from_ceiling(ceiling(4 * 1024 * MIB));
        for class in WorkClass::ALL {
            assert!(policy.for_class(class).max_concurrency > 0);
        }
    }

    #[test]
    fn peak_estimate_is_saturating() {
        let entry = ClassPolicy {
            class: WorkClass::DocsImport,
            max_input_bytes: 10,
            peak_multiplier: 24,
            max_concurrency: 1,
        };
        assert_eq!(entry.estimated_peak_bytes(10), 240);
        assert_eq!(entry.estimated_peak_bytes(u64::MAX), u64::MAX);
    }

    #[test]
    fn lookup_matches_the_requested_class() {
        let policy = MemoryPolicy::from_ceiling(ceiling(4 * 1024 * MIB));
        for class in WorkClass::ALL {
            assert_eq!(policy.for_class(class).class, class);
        }
    }

    #[test]
    fn budget_follows_the_detected_ceiling() {
        let large = MemoryPolicy::from_ceiling(ceiling(4 * 1024 * MIB));
        let small = MemoryPolicy::from_ceiling(ceiling(1024 * MIB));
        assert!(large.budget_bytes > small.budget_bytes);
        assert_eq!(large.budget_bytes, 1536 * MIB);
        assert_eq!(small.budget_bytes, 384 * MIB);
    }
}
