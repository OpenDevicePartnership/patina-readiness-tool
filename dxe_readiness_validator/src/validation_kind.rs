//! Enumerations and helpers describing different types of validation checks and violations.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use common::serializable_fv::{FirmwareFileSerDe, FirmwareSectionSerDe, FirmwareVolumeSerDe};
use mu_pi::serializable::{
    serializable_hob::{MemAllocDescriptorSerDe, ResourceDescriptorSerDe},
    Interval,
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
                                                                              Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                HobValidationKind::OverlappingMemoryRanges { .. } => "   Platforms must produce non-overlapping HOBs by splitting up overlapping HOBs\n   \
                                                                         into multiple HOBs and eliminating duplicates.\n   \
                                                                         Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                HobValidationKind::PageZeroMemoryDescribed { .. } => "   Platforms must not allocate page 0.\n   \
                                                                         Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                HobValidationKind::V1MemoryRangeNotContainedInV2 { .. } => "   All V1 HOB ranges should be described/covered by corresponding V2 HOBs.",
                HobValidationKind::V2ContainsUceAttribute { .. } => "   V2 HOB contains prohibited EFI_MEMORY_UCE attribute.",
                HobValidationKind::V2MissingValidCacheabilityAttribute { .. } => "   Platforms must produce Resource Descriptor HOB v2s with a single valid\n   \
                                                                                     cacheability attribute set. These can be the existing Resource Descriptor HOB\n   \
                                                                                     fields with the cacheability attribute set as the only additional field in the\n   \
                                                                                     v2 HOB.\n   \
                                                                                     Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                HobValidationKind::V2InvalidIoCacheabilityAttributes { .. } => "   Platforms must produce Resource Descriptor HOB v2s with no cacheability or memory protection\n   \
                                                                                   attributes set for IO resource types.",
            },
            ValidationKind::Fv(fv) => match fv {
                FvValidationKind::CombinedDriversPresent { .. } => "   Firmware volume contains prohibited combined drivers. \nBelow file types are prohibited\n- COMBINED_MM_DXE(0x0C)\n- COMBINED_PEIM_DRIVER(0x08).\n   \
                                                                       Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                FvValidationKind::LzmaCompressedSections { .. } => "   Temporarily, LZMA compressed sections that will be decompressed in DXE should use Brotli or TianoCompress.\n   \
                                                                       Tracking: https://github.com/OpenDevicePartnership/patina/issues/517\n   \
                                                                       Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                FvValidationKind::ProhibitedAprioriFile { .. } => "   A Priori sections must be removed and proper driver dispatch must be ensured\n   \
                                                                      using depex statements. Drivers may produce empty protocols solely to ensure\n   \
                                                                      that other drivers can use that protocol as a depex statement, if required.\n   \
                                                                      Platforms may also list drivers in FFSes in the order they should be dispatched,\n   \
                                                                      though it is recommended to rely on depex statements.\n   \
                                                                      Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md\n   \
                                                                      Ref: https://github.com/OpenDevicePartnership/patina-qemu/pull/40",
                FvValidationKind::UsesTraditionalSmm { .. } => "   Platforms must transition to Standalone MM (or not use MM at all, as applicable)\n   \
                                                                   using the provided guidance. All combined modules must be dropped in favor of\n   \
                                                                   single phase modules.\n   \
                                                                   Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md",
                FvValidationKind::InvalidSectionAlignment { .. } => "   All PE images must have section alignment that is a multiple of page size. \n   \
                                                                        This is not a PI spec requirement, but is a Patina requirement.\n    \
                                                                        Platforms should drop unaligned images or re-build images to ensure section alignment is page-aligned.    \n
                                                                        Ref: https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/integrate/patina_requirements.md"
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
            ValidationKind::Hob(hob) => {
                match hob {
                    HobValidationKind::InconsistentMemoryAttributes { hob1, hob2 } => {
                        let v1_hob_column =
                            serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                        let v2_hob_column =
                            serde_json::to_string_pretty(hob2).unwrap_or("hob 2 serialization failed!".to_string());
                        let resolution = if hob1.owner != hob2.owner {
                            format!("hob 1 owner({}) do not match with hob 2 owner({})", hob1.owner, hob2.owner)
                        } else if hob1.resource_attribute != hob2.resource_attribute {
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
                        let resolution =
                            format!("Hob 1 range should not overlap with Hob 2 range\nHob 1 range({}, {}) | Hob 2 range({}, {})",
                            hob1.start(), hob1.start(), hob2.start(), hob2.end());
                        vec![row_num, hob1_column, hob2_column, resolution]
                    }
                    HobValidationKind::PageZeroMemoryDescribed { alloc_desc } => {
                        let mem_alloc_desc_column = serde_json::to_string_pretty(alloc_desc)
                            .unwrap_or("Memory Allocation Descriptor\nserialization failed!".to_string());
                        let resolution =
                            format!("memory_base_address, memory_length\nshould not describe Page 0\nMemory allocation range({}, {})",
                            alloc_desc.start(), alloc_desc.end());
                        vec![row_num, mem_alloc_desc_column, resolution]
                    }
                    HobValidationKind::V1MemoryRangeNotContainedInV2 { hob1 } => {
                        let v1_hob_column =
                            serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                        let resolution =
                            "V1 Resource Descriptor Hob should have\ncorresponding V2 Resource Descriptor Hob"
                                .to_string();
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
                        let resolution =
                        format!("V2 Hob should contain exactly\none valid cacheability attributes(0x{:X})\n - MEMORY_UC(0x1)\n - MEMORY_WC(0x2)\n - MEMORY_WT(0x4)\n - MEMORY_WB(0x8)\n - MEMORY_UCE(0x10)\n - MEMORY_WP(0x1000)", attributes);
                        vec![row_num, hob1_column, resolution]
                    }
                    HobValidationKind::V2InvalidIoCacheabilityAttributes { hob1, attributes } => {
                        let hob1_column =
                            serde_json::to_string_pretty(hob1).unwrap_or("hob 1 serialization failed!".to_string());
                        let resolution =
                            format!("V2 Hob should not contain cacheability or memory protection attributes(0x{:X}) for IO ranges", attributes);
                        vec![row_num, hob1_column, resolution]
                    }
                }
            }
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
