use std::fmt;

use common::serializable_hob::ResourceDescriptorSerDe;

/// Public result type for the crate.
pub type Result<T> = core::result::Result<T, PlatformError>;

#[derive(Debug, PartialEq, Eq)]
pub enum PlatformError {
    MemoryRangeOverlap { overlaps: Vec<(ResourceDescriptorSerDe, ResourceDescriptorSerDe)> },
    InconsistentMemoryAttributes { conflicting_intervals: Vec<(ResourceDescriptorSerDe, ResourceDescriptorSerDe)> },
    InconsistentRanges { unmatched_v1: Vec<ResourceDescriptorSerDe> },
    MissingMemoryProtections,
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::MemoryRangeOverlap { overlaps } => {
                write!(f, "Memory range overlaps detected for ranges {:?}", overlaps)
            }
            PlatformError::InconsistentMemoryAttributes { conflicting_intervals } => {
                write!(f, "Memory ranges overlap but have different attributes: {:?}", conflicting_intervals)
            }
            PlatformError::InconsistentRanges { unmatched_v1 } => {
                write!(f, "V1 ranges {:?} not matched by V2 range", unmatched_v1)
            }
            PlatformError::MissingMemoryProtections => {
                write!(f, "Memory protection settings HOB is missing or invalid")
            }
        }
    }
}
