//! Enumerations and helpers describing different types of validation checks and violations.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use patina::pi::serializable::{
    Interval,
    serializable_fv::{FirmwareFileSerDe, FirmwareSectionSerDe, FirmwareVolumeSerDe},
    serializable_hob::{FvHobSerDe, MemAllocDescriptorSerDe, ResourceDescriptorSerDe},
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HobValidationKind<'a> {
    // HOBs must define consistent memory attributes
    InconsistentMemoryAttributes { hob1: &'a ResourceDescriptorSerDe, hob2: &'a ResourceDescriptorSerDe },

    // HOBs must not define overlapping memory ranges
    OverlappingMemoryRanges { hob1: &'a ResourceDescriptorSerDe, hob2: &'a ResourceDescriptorSerDe },

    // Page zero must not be described in memory HOBs
    PageZeroMemoryDescribed { alloc_desc: &'a MemAllocDescriptorSerDe },

    // All V1 ranges must be covered by V2
    V1MemoryRangeNotContainedInV2 { hob1: &'a ResourceDescriptorSerDe },

    // V2 ranges must not have the UCE attribute
    V2ContainsUceAttribute { hob1: &'a ResourceDescriptorSerDe, attributes: u64 },

    // V2 resource descriptor must have at most one valid Cacheability attribute set
    V2MissingValidCacheabilityAttribute { hob1: &'a ResourceDescriptorSerDe, attributes: u64 },

    // V2 resource descriptor for io must have no cacheability or memory protection attributes set
    V2InvalidIoCacheabilityAttributes { hob1: &'a ResourceDescriptorSerDe, attributes: u64 },

    // More than one Resource Descriptor HOB owned by gEfiMemoryTypeInformationGuid is present
    MemoryTypeInfoMultipleResourceHobs { hob1: &'a ResourceDescriptorSerDe },

    // Memory Type Info Resource Descriptor HOB ResourceLength is smaller than the sum of bin sizes
    MemoryTypeInfoResourceLengthTooSmall { hob1: &'a ResourceDescriptorSerDe, required_bytes: u64, actual_bytes: u64 },

    // Memory allocation HOBs must not overlap
    MemoryAllocationOverlap { hob1: &'a MemAllocDescriptorSerDe, hob2: &'a MemAllocDescriptorSerDe },

    // Memory allocation HOBs must be page aligned
    MemoryAllocationNotPageAligned { hob: &'a MemAllocDescriptorSerDe },

    // Resource descriptor HOBs must be page aligned
    ResourceDescriptorNotPageAligned { hob: &'a ResourceDescriptorSerDe },

    // FV HOBs must fall within memory allocation HOBs
    FvNotWithinMemoryAllocation { fv: &'a FvHobSerDe },
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FvValidationKind<'a> {
    // FV must not contain combined drivers
    CombinedDriversPresent {
        fv: &'a FirmwareVolumeSerDe,
        file: &'a FirmwareFileSerDe,
    },

    // FV must not contain LZMA-compressed sections
    LzmaCompressedSections {
        fv: &'a FirmwareVolumeSerDe,
        file: &'a FirmwareFileSerDe,
        section: &'a FirmwareSectionSerDe,
    },

    // FV must not contain an Apriori file
    ProhibitedAprioriFile {
        fv: &'a FirmwareVolumeSerDe,
        file: &'a FirmwareFileSerDe,
    },

    // FV must not contain traditional SMM drivers
    UsesTraditionalSmm {
        fv: &'a FirmwareVolumeSerDe,
        file: &'a FirmwareFileSerDe,
    },

    // PE images must have page-aligned section alignments
    InvalidSectionAlignment {
        fv: &'a FirmwareVolumeSerDe,
        file: &'a FirmwareFileSerDe,
        section: &'a FirmwareSectionSerDe,
        required_alignment: usize,
    },
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationKind<'a> {
    Hob(HobValidationKind<'a>),
    Fv(FvValidationKind<'a>),
}

impl ValidationKind<'_> {
    pub fn header(&self) -> &str {
        match self {
            ValidationKind::Hob(hob) => match hob {
                HobValidationKind::InconsistentMemoryAttributes { .. } => "HOB: Inconsistent Memory Attributes",
                HobValidationKind::OverlappingMemoryRanges { .. } => "HOB: Overlapping Memory Ranges",
                HobValidationKind::PageZeroMemoryDescribed { .. } => "HOB: Page Zero Memory Described",
                HobValidationKind::V1MemoryRangeNotContainedInV2 { .. } => "HOB: V1 Memory Range Not Contained in V2",
                HobValidationKind::V2ContainsUceAttribute { .. } => "HOB: V2 Range Contains UCE Attribute",
                HobValidationKind::V2MissingValidCacheabilityAttribute { .. } => {
                    "HOB: V2 Missing Valid Cacheability Attribute"
                }
                HobValidationKind::V2InvalidIoCacheabilityAttributes { .. } => {
                    "HOB: V2 Invalid IO Cacheability Attributes"
                }
                HobValidationKind::MemoryTypeInfoMultipleResourceHobs { .. } => {
                    "HOB: Multiple Memory Type Info Resource Descriptor HOBs"
                }
                HobValidationKind::MemoryTypeInfoResourceLengthTooSmall { .. } => {
                    "HOB: Memory Type Info Resource Descriptor HOB Length Too Small"
                }
                HobValidationKind::MemoryAllocationOverlap { .. } => "HOB: Memory Allocation Ranges Overlap",
                HobValidationKind::MemoryAllocationNotPageAligned { .. } => {
                    "HOB: Memory Allocation Range Not Page Aligned"
                }
                HobValidationKind::ResourceDescriptorNotPageAligned { .. } => {
                    "HOB: Resource Descriptor Range Not Page Aligned"
                }
                HobValidationKind::FvNotWithinMemoryAllocation { .. } => {
                    "HOB: Firmware Volume Not Within Memory Allocation"
                }
            },
            ValidationKind::Fv(fv) => match fv {
                FvValidationKind::CombinedDriversPresent { .. } => "FV: Combined Drivers Present",
                FvValidationKind::LzmaCompressedSections { .. } => "FV: LZMA Compressed Sections Present",
                FvValidationKind::ProhibitedAprioriFile { .. } => "FV: Prohibited Apriori File Present",
                FvValidationKind::UsesTraditionalSmm { .. } => "FV: Uses Traditional SMM Driver",
                FvValidationKind::InvalidSectionAlignment { .. } => "FV: PE Image Invalid Section Alignment",
            },
        }
    }

    pub fn guidance(&self) -> &str {
        match self {
            ValidationKind::Hob(hob) => match hob {
                HobValidationKind::InconsistentMemoryAttributes { .. } => "   Platforms must producing V1 and V2 HOBs for describing the same range(s) should have consistent memory attributes.\n   \
                                                                              Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                HobValidationKind::OverlappingMemoryRanges { .. } => "   Platforms must produce non-overlapping HOBs by splitting up overlapping HOBs\n   \
                                                                         into multiple HOBs and eliminating duplicates.\n   \
                                                                         Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                HobValidationKind::PageZeroMemoryDescribed { .. } => "   Platforms must not allocate page 0.\n   \
                                                                         Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                HobValidationKind::V1MemoryRangeNotContainedInV2 { .. } => "   All V1 HOB ranges should be described/covered by corresponding V2 HOBs.",
                HobValidationKind::V2ContainsUceAttribute { .. } => "   V2 HOB contains prohibited EFI_MEMORY_UCE attribute.",
                HobValidationKind::V2MissingValidCacheabilityAttribute { .. } => "   Platforms must produce Resource Descriptor HOB v2s with a single valid\n   \
                                                                                     cacheability attribute set. These can be the existing Resource Descriptor HOB\n   \
                                                                                     fields with the cacheability attribute set as the only additional field in the\n   \
                                                                                     v2 HOB.\n   \
                                                                                     Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                HobValidationKind::V2InvalidIoCacheabilityAttributes { .. } => "   Platforms must produce Resource Descriptor HOB v2s with no cacheability or memory protection\n   \
                                                                                   attributes set for IO resource types.",
                HobValidationKind::MemoryTypeInfoMultipleResourceHobs { .. } => "   Only one Resource Descriptor HOB owned by the Memory Type Information GUID is allowed. ",
                HobValidationKind::MemoryTypeInfoResourceLengthTooSmall { .. } => "   The Memory Type Info Resource Descriptor HOB's ResourceLength must be large enough\n   \
                                                                                    to hold the sum of bin sizes reported in the Memory Type Information GUID HOB.\n   \
                                                                                    Note: the check uses the raw page-count sum. Platforms may need additional space\n   \
                                                                                    for per-bin alignment padding.",
                HobValidationKind::MemoryAllocationOverlap { .. } => "   Memory Allocation HOB ranges must not overlap.",
                HobValidationKind::MemoryAllocationNotPageAligned { .. } => "   Memory Allocation HOB base addresses and lengths must be page aligned.",
                HobValidationKind::ResourceDescriptorNotPageAligned { .. } => "   Resource Descriptor HOB base addresses and lengths must be page aligned.",
                HobValidationKind::FvNotWithinMemoryAllocation { .. } => "   Firmware Volume HOB ranges must be contained within a Memory Allocation HOB range.",
            },
            ValidationKind::Fv(fv) => match fv {
                FvValidationKind::CombinedDriversPresent { .. } => "   Firmware volume contains prohibited combined drivers. \nBelow file types are prohibited\n- COMBINED_MM_DXE(0x0C)\n- COMBINED_PEIM_DRIVER(0x08).\n   \
                                                                       Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                FvValidationKind::LzmaCompressedSections { .. } => "   Temporarily, LZMA compressed sections that will be decompressed in DXE should use Brotli or TianoCompress.\n   \
                                                                       Tracking: https://github.com/OpenDevicePartnership/patina/issues/517\n   \
                                                                       Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                FvValidationKind::ProhibitedAprioriFile { .. } => "   A Priori sections must be removed and proper driver dispatch must be ensured\n   \
                                                                      using depex statements. Drivers may produce empty protocols solely to ensure\n   \
                                                                      that other drivers can use that protocol as a depex statement, if required.\n   \
                                                                      Platforms may also list drivers in FFSes in the order they should be dispatched,\n   \
                                                                      though it is recommended to rely on depex statements.\n   \
                                                                      Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html\n   \
                                                                      Ref: https://github.com/OpenDevicePartnership/patina-qemu/pull/40",
                FvValidationKind::UsesTraditionalSmm { .. } => "   Platforms must transition to Standalone MM (or not use MM at all, as applicable)\n   \
                                                                   using the provided guidance. All combined modules must be dropped in favor of\n   \
                                                                   single phase modules.\n   \
                                                                   Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html",
                FvValidationKind::InvalidSectionAlignment { .. } => "   All PE images must have section alignment that is a multiple of page size. \n   \
                                                                        This is not a PI spec requirement, but is a Patina requirement.\n    \
                                                                        Platforms should drop unaligned images or re-build images to ensure section alignment is page-aligned.    \n
                                                                        Ref: https://opendevicepartnership.github.io/patina/integrate/patina_dxe_core_requirements_checklist.html"
            },
        }
    }

    pub fn name(&self) -> String {
        match self {
            ValidationKind::Hob(hob) => match hob {
                HobValidationKind::InconsistentMemoryAttributes { .. } => "InconsistentMemoryAttributes".to_string(),
                HobValidationKind::OverlappingMemoryRanges { .. } => "OverlappingMemoryRanges".to_string(),
                HobValidationKind::PageZeroMemoryDescribed { .. } => "PageZeroMemoryDescribed".to_string(),
                HobValidationKind::V1MemoryRangeNotContainedInV2 { .. } => "V1MemoryRangeNotContainedInV2".to_string(),
                HobValidationKind::V2ContainsUceAttribute { .. } => "V2ContainsUceAttribute".to_string(),
                HobValidationKind::V2MissingValidCacheabilityAttribute { .. } => {
                    "V2MissingValidCacheabilityAttribute".to_string()
                }
                HobValidationKind::V2InvalidIoCacheabilityAttributes { .. } => {
                    "V2InvalidIoCacheabilityAttributes".to_string()
                }
                HobValidationKind::MemoryTypeInfoMultipleResourceHobs { .. } => {
                    "MemoryTypeInfoMultipleResourceHobs".to_string()
                }
                HobValidationKind::MemoryTypeInfoResourceLengthTooSmall { .. } => {
                    "MemoryTypeInfoResourceLengthTooSmall".to_string()
                }
                HobValidationKind::MemoryAllocationOverlap { .. } => "MemoryAllocationOverlap".to_string(),
                HobValidationKind::MemoryAllocationNotPageAligned { .. } => {
                    "MemoryAllocationNotPageAligned".to_string()
                }
                HobValidationKind::ResourceDescriptorNotPageAligned { .. } => {
                    "ResourceDescriptorNotPageAligned".to_string()
                }
                HobValidationKind::FvNotWithinMemoryAllocation { .. } => "FvNotWithinMemoryAllocation".to_string(),
            },
            ValidationKind::Fv(fv) => match fv {
                FvValidationKind::CombinedDriversPresent { .. } => "CombinedDriversPresent".to_string(),
                FvValidationKind::LzmaCompressedSections { .. } => "LzmaCompressedSections".to_string(),
                FvValidationKind::ProhibitedAprioriFile { .. } => "ProhibitedAprioriFile".to_string(),
                FvValidationKind::UsesTraditionalSmm { .. } => "UsesTraditionalSmm".to_string(),
                FvValidationKind::InvalidSectionAlignment { .. } => "InvalidSectionAlignment".to_string(),
            },
        }
    }
}

pub trait PrettyPrintTable {
    fn table_header(&self) -> Vec<&str>;
    fn table_row(&self, row_num: String) -> Vec<String>;
}

impl PrettyPrintTable for ValidationKind<'_> {
    fn table_header(&self) -> Vec<&str> {
        match self {
            ValidationKind::Hob(hob) => match hob {
                HobValidationKind::InconsistentMemoryAttributes { .. } => {
                    vec!["#", "V1 Hob", "V2 Hob", "Violation/Resolution"]
                }
                HobValidationKind::OverlappingMemoryRanges { .. } => {
                    vec!["#", "Hob 1", "Hob 2", "Violation/Resolution"]
                }
                HobValidationKind::PageZeroMemoryDescribed { .. } => {
                    vec!["#", "Memory Allocation Descriptor", "Violation/Resolution"]
                }
                HobValidationKind::V1MemoryRangeNotContainedInV2 { .. } => vec!["#", "V1 Hob", "Violation/Resolution"],
                HobValidationKind::V2ContainsUceAttribute { .. } => vec!["#", "V2 Hob", "Violation/Resolution"],
                HobValidationKind::V2MissingValidCacheabilityAttribute { .. } => {
                    vec!["#", "V2 Hob", "Violation/Resolution"]
                }
                HobValidationKind::V2InvalidIoCacheabilityAttributes { .. } => {
                    vec!["#", "V2 Hob", "Violation/Resolution"]
                }
                HobValidationKind::MemoryTypeInfoMultipleResourceHobs { .. } => {
                    vec!["#", "Resource Descriptor Hob", "Violation/Resolution"]
                }
                HobValidationKind::MemoryTypeInfoResourceLengthTooSmall { .. } => {
                    vec!["#", "Resource Descriptor Hob", "Violation/Resolution"]
                }
                HobValidationKind::MemoryAllocationOverlap { .. } => {
                    vec!["#", "Memory Allocation HOB 1", "Memory Allocation HOB 2", "Violation/Resolution"]
                }
                HobValidationKind::MemoryAllocationNotPageAligned { .. } => {
                    vec!["#", "Memory Allocation HOB", "Violation/Resolution"]
                }
                HobValidationKind::ResourceDescriptorNotPageAligned { .. } => {
                    vec!["#", "Resource Descriptor HOB", "Violation/Resolution"]
                }
                HobValidationKind::FvNotWithinMemoryAllocation { .. } => {
                    vec!["#", "Firmware Volume HOB", "Violation/Resolution"]
                }
            },
            ValidationKind::Fv(fv) => match fv {
                FvValidationKind::CombinedDriversPresent { .. } => vec!["#", "File", "Violation/Resolution"],
                FvValidationKind::LzmaCompressedSections { .. } => vec!["#", "LZMA Section", "Violation/Resolution"],
                FvValidationKind::ProhibitedAprioriFile { .. } => vec!["#", "A Priori File", "Violation/Resolution"],
                FvValidationKind::UsesTraditionalSmm { .. } => {
                    vec!["#", "Traditional SMM Driver", "Violation/Resolution"]
                }
                FvValidationKind::InvalidSectionAlignment { .. } => {
                    vec!["#", "PE Image Section Alignment", "Violation/Resolution"]
                }
            },
        }
    }

    fn table_row(&self, row_num: String) -> Vec<String> {
        match self {
            ValidationKind::Hob(hob) => match hob {
                HobValidationKind::InconsistentMemoryAttributes { hob1, hob2 } => {
                    let v1_hob_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                    let v2_hob_column =
                        serde_json::to_string_pretty(hob2).unwrap_or("hob 2 serialization failed!".to_string());
                    let resolution = if hob1.resource_attribute != hob2.resource_attribute {
                        format!(
                            "hob 1 resource_attribute({}) do not match with hob 2 resource_attribute({})",
                            hob1.resource_attribute, hob2.resource_attribute
                        )
                    } else if hob1.resource_type != hob2.resource_type {
                        format!(
                            "hob 1 resource_type({}) do not match with hob 2 resource_type({})",
                            hob1.resource_type, hob2.resource_type
                        )
                    } else {
                        "invalid hob 1 and hob 2".to_string()
                    };
                    vec![row_num, v1_hob_column, v2_hob_column, resolution]
                }
                HobValidationKind::OverlappingMemoryRanges { hob1, hob2 } => {
                    let hob1_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                    let hob2_column =
                        serde_json::to_string_pretty(hob2).unwrap_or("hob 2 serialization failed!".to_string());
                    let resolution = format!(
                        "Hob 1 range should not overlap with Hob 2 range\nHob 1 range({:#x}, {:#x}) | Hob 2 range({:#x}, {:#x})",
                        hob1.start(),
                        hob1.end(),
                        hob2.start(),
                        hob2.end()
                    );
                    vec![row_num, hob1_column, hob2_column, resolution]
                }
                HobValidationKind::PageZeroMemoryDescribed { alloc_desc } => {
                    let mem_alloc_desc_column = serde_json::to_string_pretty(alloc_desc)
                        .unwrap_or("Memory Allocation Descriptor\nserialization failed!".to_string());
                    let resolution = format!(
                        "memory_base_address, memory_length\nshould not describe Page 0\nMemory allocation range({:#x}, {:#x})",
                        alloc_desc.start(),
                        alloc_desc.end()
                    );
                    vec![row_num, mem_alloc_desc_column, resolution]
                }
                HobValidationKind::V1MemoryRangeNotContainedInV2 { hob1 } => {
                    let v1_hob_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                    let resolution =
                        "V1 Resource Descriptor Hob should have\ncorresponding V2 Resource Descriptor Hob".to_string();
                    vec![row_num, v1_hob_column, resolution]
                }
                HobValidationKind::V2ContainsUceAttribute { hob1, attributes } => {
                    let hob1_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                    let resolution =
                        format!("Attributes(0x{:X}) should not contain\nMEMORY_UCE(0x10) attribute", attributes);
                    vec![row_num, hob1_column, resolution]
                }
                HobValidationKind::V2MissingValidCacheabilityAttribute { hob1, attributes } => {
                    let hob1_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                    let resolution = format!(
                        "V2 Hob should contain exactly\none valid cacheability attributes(0x{:X})\n - MEMORY_UC(0x1)\n - MEMORY_WC(0x2)\n - MEMORY_WT(0x4)\n - MEMORY_WB(0x8)\n - MEMORY_UCE(0x10)\n - MEMORY_WP(0x1000)",
                        attributes
                    );
                    vec![row_num, hob1_column, resolution]
                }
                HobValidationKind::V2InvalidIoCacheabilityAttributes { hob1, attributes } => {
                    let hob1_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                    let resolution = format!(
                        "V2 Hob should not contain cacheability or memory protection attributes(0x{:X}) for IO ranges",
                        attributes
                    );
                    vec![row_num, hob1_column, resolution]
                }
                HobValidationKind::MemoryTypeInfoMultipleResourceHobs { hob1 } => {
                    let hob_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob serialization failed!".to_string());
                    let resolution =
                        "Only one Resource Descriptor HOB owned by\ngEfiMemoryTypeInformationGuid is allowed"
                            .to_string();
                    vec![row_num, hob_column, resolution]
                }
                HobValidationKind::MemoryTypeInfoResourceLengthTooSmall { hob1, required_bytes, actual_bytes } => {
                    let hob_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("hob serialization failed!".to_string());
                    let resolution = format!(
                        "ResourceLength(0x{:X}) is smaller than the\nraw sum of bin sizes(0x{:X}) reported in\nthe Memory Type Information GUID HOB",
                        actual_bytes, required_bytes
                    );
                    vec![row_num, hob_column, resolution]
                }
                HobValidationKind::MemoryAllocationOverlap { hob1, hob2 } => {
                    let hob1_column =
                        serde_json::to_string_pretty(hob1).unwrap_or("HOB 1 serialization failed!".to_string());
                    let hob2_column =
                        serde_json::to_string_pretty(hob2).unwrap_or("HOB 2 serialization failed!".to_string());
                    let resolution = format!(
                        "Memory allocation ranges must not overlap\nHOB 1 range({:#x}, {:#x}) | HOB 2 range({:#x}, {:#x})",
                        hob1.start(),
                        hob1.end(),
                        hob2.start(),
                        hob2.end()
                    );
                    vec![row_num, hob1_column, hob2_column, resolution]
                }
                HobValidationKind::MemoryAllocationNotPageAligned { hob } => {
                    let hob_column =
                        serde_json::to_string_pretty(hob).unwrap_or("HOB serialization failed!".to_string());
                    let resolution = "Memory allocation base address and length must be page aligned.".to_string();
                    vec![row_num, hob_column, resolution]
                }
                HobValidationKind::ResourceDescriptorNotPageAligned { hob } => {
                    let hob_column =
                        serde_json::to_string_pretty(hob).unwrap_or("HOB serialization failed!".to_string());
                    let resolution = "Resource descriptor base address and length must be page aligned.".to_string();
                    vec![row_num, hob_column, resolution]
                }
                HobValidationKind::FvNotWithinMemoryAllocation { fv } => {
                    let fv_column =
                        serde_json::to_string_pretty(fv).unwrap_or("FV HOB serialization failed!".to_string());
                    let resolution =
                        "Firmware volume range must be contained within a memory allocation HOB range.".to_string();
                    vec![row_num, fv_column, resolution]
                }
            },
            ValidationKind::Fv(fv) => match fv {
                FvValidationKind::CombinedDriversPresent { fv, file } => {
                    let file_column = format!("FV: {}\nFile: {}\nFile Type: {}", fv.fv_name, file.name, file.file_type);
                    let resolution =
                        "File types should not be\n - COMBINED_MM_DXE(0x0C)\n - COMBINED_PEIM_DRIVER(0x08)."
                            .to_string();
                    vec![row_num, file_column, resolution]
                }
                FvValidationKind::LzmaCompressedSections { fv, file, section } => {
                    let section_json =
                        serde_json::to_string_pretty(section).unwrap_or("section serialization failed!".to_string());
                    let section_column = format!("FV: {}\nFile: {}\nSection: {}", fv.fv_name, file.name, section_json);
                    let resolution = "File section should not be compressed with LZMA.".to_string();
                    vec![row_num, section_column, resolution]
                }
                FvValidationKind::ProhibitedAprioriFile { fv, file } => {
                    let file_column = format!("FV: {}\nFile: {}", fv.fv_name, file.name);
                    let resolution =
                        "Following Apriori Files are not supported\n - PeiAprioriFileNameGuid(1b45cc0a-156a-428a-af62-49864da0e6e6)\n - AprioriGuid(fc510ee7-ffdc-11d4-bd41-0080c73c8881)."
                            .to_string();
                    vec![row_num, file_column, resolution]
                }
                FvValidationKind::UsesTraditionalSmm { fv, file } => {
                    let file_column = format!(
                        "FV: {}\nSMM Driver File: {}\nSMM Driver Type: {}",
                        fv.fv_name, file.name, file.file_type
                    );
                    let resolution =
                        "File types should not be\n - COMBINED_MM_DXE(0x0C)\n - COMBINED_PEIM_DRIVER(0x08)\n - MM(0x0A)\n - MM_CORE(0x0D)."
                            .to_string();
                    vec![row_num, file_column, resolution]
                }
                FvValidationKind::InvalidSectionAlignment { fv, file, section, required_alignment } => {
                    let file_column = format!(
                        "FV: {}\nFile: {}\nSection Alignment: {}\nRequired Alignment:{}\n",
                        fv.fv_name,
                        file.name,
                        section.pe_info.unwrap().section_alignment,
                        required_alignment,
                    );
                    let resolution =
                        "PE images must have section alignment that is a positive multiple of UEFI_PAGE_SIZE (4k). \n ARM64 DXE_RUNTIME_DRIVERs must have section alignment that is a positive multiple of 64k."
                            .to_string();
                    vec![row_num, file_column, resolution]
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patina::pi::serializable::serializable_fv::{FirmwareFileSerDe, FirmwareSectionSerDe, PeHeaderInfo};
    use patina::pi::serializable::serializable_hob::{FvHobSerDe, MemAllocDescriptorSerDe, ResourceDescriptorSerDe};

    fn resource(start: u64, length: u64, resource_type: u32, resource_attribute: u32) -> ResourceDescriptorSerDe {
        ResourceDescriptorSerDe {
            owner: "owner".to_string(),
            resource_type,
            resource_attribute,
            physical_start: start,
            resource_length: length,
        }
    }

    fn mem_alloc(start: u64, length: u64) -> MemAllocDescriptorSerDe {
        MemAllocDescriptorSerDe {
            name: "name".to_string(),
            memory_base_address: start,
            memory_length: length,
            memory_type: 1,
        }
    }

    fn section() -> FirmwareSectionSerDe {
        FirmwareSectionSerDe {
            section_type: "Pe32".to_string(),
            length: 256,
            compression_type: "LZMA ".to_string(),
            pe_info: Some(PeHeaderInfo { section_alignment: 0x1000, machine: 0, subsystem: 0 }),
        }
    }

    fn file() -> FirmwareFileSerDe {
        FirmwareFileSerDe {
            name: "File".to_string(),
            file_type: "Driver".to_string(),
            length: 512,
            attributes: 0,
            sections: vec![],
        }
    }

    fn fv() -> FirmwareVolumeSerDe {
        FirmwareVolumeSerDe {
            fv_name: "Fv".to_string(),
            fv_length: 1024,
            fv_base_address: 0x1000,
            fv_attributes: 0,
            files: vec![],
        }
    }

    #[test]
    fn test_all_validation_kinds_render_non_empty() {
        let r1 = resource(0x1000, 0x1000, 0, 0);
        let r_attr = resource(0x1000, 0x1000, 0, 1);
        let r_type = resource(0x1000, 0x1000, 5, 0);
        let ma = mem_alloc(0x2000, 0x1000);
        let ma2 = mem_alloc(0x2800, 0x1000);
        let fv_hob = FvHobSerDe { base_address: 0x8000, length: 0x1000 };
        let fv = fv();
        let file = file();
        let section = section();

        let kinds = vec![
            // Attribute-mismatch branch of the resolution string.
            ValidationKind::Hob(HobValidationKind::InconsistentMemoryAttributes { hob1: &r1, hob2: &r_attr }),
            // Resource-type-mismatch branch of the resolution string.
            ValidationKind::Hob(HobValidationKind::InconsistentMemoryAttributes { hob1: &r1, hob2: &r_type }),
            ValidationKind::Hob(HobValidationKind::OverlappingMemoryRanges { hob1: &r1, hob2: &r_attr }),
            ValidationKind::Hob(HobValidationKind::PageZeroMemoryDescribed { alloc_desc: &ma }),
            ValidationKind::Hob(HobValidationKind::V1MemoryRangeNotContainedInV2 { hob1: &r1 }),
            ValidationKind::Hob(HobValidationKind::V2ContainsUceAttribute { hob1: &r1, attributes: 0x10 }),
            ValidationKind::Hob(HobValidationKind::V2MissingValidCacheabilityAttribute { hob1: &r1, attributes: 0 }),
            ValidationKind::Hob(HobValidationKind::V2InvalidIoCacheabilityAttributes { hob1: &r1, attributes: 0x1 }),
            ValidationKind::Hob(HobValidationKind::MemoryTypeInfoMultipleResourceHobs { hob1: &r1 }),
            ValidationKind::Hob(HobValidationKind::MemoryTypeInfoResourceLengthTooSmall {
                hob1: &r1,
                required_bytes: 0x2000,
                actual_bytes: 0x1000,
            }),
            ValidationKind::Hob(HobValidationKind::MemoryAllocationOverlap { hob1: &ma, hob2: &ma2 }),
            ValidationKind::Hob(HobValidationKind::MemoryAllocationNotPageAligned { hob: &ma }),
            ValidationKind::Hob(HobValidationKind::ResourceDescriptorNotPageAligned { hob: &r1 }),
            ValidationKind::Hob(HobValidationKind::FvNotWithinMemoryAllocation { fv: &fv_hob }),
            ValidationKind::Fv(FvValidationKind::CombinedDriversPresent { fv: &fv, file: &file }),
            ValidationKind::Fv(FvValidationKind::LzmaCompressedSections { fv: &fv, file: &file, section: &section }),
            ValidationKind::Fv(FvValidationKind::ProhibitedAprioriFile { fv: &fv, file: &file }),
            ValidationKind::Fv(FvValidationKind::UsesTraditionalSmm { fv: &fv, file: &file }),
            ValidationKind::Fv(FvValidationKind::InvalidSectionAlignment {
                fv: &fv,
                file: &file,
                section: &section,
                required_alignment: 0x1000,
            }),
        ];

        for kind in &kinds {
            assert!(!kind.header().is_empty(), "header empty for {:?}", kind);
            assert!(!kind.name().is_empty(), "name empty for {:?}", kind);
            assert!(!kind.guidance().is_empty(), "guidance empty for {:?}", kind);
            assert!(!kind.table_header().is_empty(), "table_header empty for {:?}", kind);

            let row = kind.table_row("1".to_string());
            assert_eq!(row[0], "1");
            assert!(row.len() >= 3, "row too short for {:?}", kind);
        }
    }

    #[test]
    fn test_overlapping_memory_ranges_resolution_uses_hex() {
        let hob1 = resource(0x1000, 0x2000, 0, 0);
        let hob2 = resource(0x2000, 0x2000, 0, 0);
        let kind = ValidationKind::Hob(HobValidationKind::OverlappingMemoryRanges { hob1: &hob1, hob2: &hob2 });

        let row = kind.table_row("1".to_string());
        let resolution = row.last().unwrap();
        assert!(resolution.contains("Hob 1 range(0x1000, 0x3000)"), "got: {resolution}");
        assert!(resolution.contains("Hob 2 range(0x2000, 0x4000)"), "got: {resolution}");
    }

    #[test]
    fn test_page_zero_memory_resolution_uses_hex() {
        let alloc = mem_alloc(0x0, 0x1000);
        let kind = ValidationKind::Hob(HobValidationKind::PageZeroMemoryDescribed { alloc_desc: &alloc });

        let row = kind.table_row("1".to_string());
        let resolution = row.last().unwrap();
        assert!(resolution.contains("Memory allocation range(0x0, 0x1000)"), "got: {resolution}");
    }
}
