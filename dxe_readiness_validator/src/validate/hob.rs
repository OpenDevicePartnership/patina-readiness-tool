//! Validation logic for HOB (Hand-Off Block) structures.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use patina::{
    OwnedGuid,
    base::UEFI_PAGE_SIZE,
    pi::{
        hob::{EFI_RESOURCE_IO, EFI_RESOURCE_IO_RESERVED, MEMORY_TYPE_INFO_HOB_GUID},
        serializable::{
            Interval,
            serializable_hob::{HobSerDe, MemoryTypeInfoEntrySerDe, ResourceDescriptorSerDe},
        },
    },
};
use r_efi::efi;

use crate::{
    ValidationAppError,
    validation_kind::{HobValidationKind, ValidationKind},
    validator::Validator,
};

use super::{ValidationReport, ValidationResult};

/// Performs validation on a list of hobs to check for violations of Patina
/// requirements.
pub struct HobValidator<'a> {
    hob_list: &'a Vec<HobSerDe>,
}

impl<'a> HobValidator<'a> {
    pub fn new(hob_list: &'a Vec<HobSerDe>) -> Self {
        HobValidator { hob_list }
    }

    fn is_io(resource_type: u32) -> bool {
        resource_type == EFI_RESOURCE_IO || resource_type == EFI_RESOURCE_IO_RESERVED
    }

    /// Returns true when a resource descriptor HOB is owned by the Memory Type Information GUID
    /// and therefore describes the PEI memory bin ranges.
    fn is_memory_type_info(resource: &ResourceDescriptorSerDe) -> bool {
        OwnedGuid::try_from_string(&resource.owner).is_ok_and(|owner| owner == MEMORY_TYPE_INFO_HOB_GUID)
    }

    fn check_hob_overlap<'b, T>(resource_list: &[&'b T]) -> Vec<(&'b T, &'b T)>
    where
        T: Interval,
    {
        let mut overlaps = Vec::new();
        for i in 0..resource_list.len() {
            for j in (i + 1)..resource_list.len() {
                if resource_list[i].overlaps(resource_list[j]) {
                    overlaps.push((resource_list[i], resource_list[j]));
                }
            }
        }

        overlaps
    }

    /// Checks for overlapping address ranges in memory and I/O resource
    /// descriptor HOBs. Reports each overlapping pair as a validation
    /// violation.
    fn validate_memory_overlap(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        let mut overlaps = Vec::new();
        let mut v1_memory_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v2_memory_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v1_io_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v2_io_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();

        for hob in self.hob_list {
            match hob {
                HobSerDe::ResourceDescriptor(resource) if !Self::is_io(resource.resource_type) => {
                    v1_memory_hobs.push(resource)
                }
                HobSerDe::ResourceDescriptorV2 { v1: resource, .. } if !Self::is_io(resource.resource_type) => {
                    v2_memory_hobs.push(resource)
                }
                HobSerDe::ResourceDescriptor(resource) if Self::is_io(resource.resource_type) => {
                    v1_io_hobs.push(resource)
                }
                HobSerDe::ResourceDescriptorV2 { v1: resource, .. } if Self::is_io(resource.resource_type) => {
                    v2_io_hobs.push(resource)
                }
                _ => (),
            }
        }

        overlaps.extend(Self::check_hob_overlap(&v1_memory_hobs));
        overlaps.extend(Self::check_hob_overlap(&v2_memory_hobs));
        overlaps.extend(Self::check_hob_overlap(&v1_io_hobs));
        overlaps.extend(Self::check_hob_overlap(&v2_io_hobs));

        for (hob1, hob2) in &overlaps {
            validation_report
                .add_violation(ValidationKind::Hob(HobValidationKind::OverlappingMemoryRanges { hob1, hob2 }));
        }

        Ok(validation_report)
    }

    /// Checks for inconsistencies between overlapping V1 and V2 resource
    /// descriptor HOBs. Reports violations when `resource_type` or
    /// `resource_attribute` differ between V1 and V2 descriptors that cover
    /// overlapping ranges.
    ///
    /// Resource descriptors whose `owner` is `MEMORY_TYPE_INFO_HOB_GUID` are
    /// skipped because the PEI memory bin HOB is expected to overlap with the
    /// resource descriptors describing the system memory backing those bins.
    ///
    /// For v1/v2, The requirement is that v2 hobs are a superset of v1 Below is
    /// the strategy use:
    /// - Check for consistency:
    ///  - If any v1 hobs overlap with v2 hobs, make sure they have the same
    ///    attributes
    /// - Check for superset property:
    ///  - Sort and merge all hobs
    ///  - For each v1 interval, make sure some combination (merged) of v2 hobs
    ///    covers it fully quick proof sketch that merging is safe: if a V1
    ///    overlaps with any V2s, those V2s must have the same attributes as it,
    ///    so it's safe to merge for the superset check
    /// - If v1 and v2 overlap, make sure info is consistent
    fn validate_overlapping_v1v2_attributes(&self) -> ValidationResult<'_> {
        let mut inconsistent_v1_v2 = Vec::new();
        let mut validation_report = ValidationReport::new();
        for hob1 in self.hob_list {
            for hob2 in self.hob_list {
                let HobSerDe::ResourceDescriptor(v1) = hob1 else { continue };
                let HobSerDe::ResourceDescriptorV2 { v1: v2, .. } = hob2 else { continue };
                if Self::is_memory_type_info(v1) || Self::is_memory_type_info(v2) {
                    continue;
                }
                if v1.overlaps(v2)
                    && (v1.resource_type != v2.resource_type || v1.resource_attribute != v2.resource_attribute)
                {
                    inconsistent_v1_v2.push((v1, v2));
                }
            }
        }

        for (hob1, hob2) in inconsistent_v1_v2 {
            validation_report
                .add_violation(ValidationKind::Hob(HobValidationKind::InconsistentMemoryAttributes { hob1, hob2 }));
        }

        Ok(validation_report)
    }

    /// Checks that all V1 resource descriptors are covered by V2 descriptors,
    /// reporting any V1 ranges not migrated to V2.
    ///
    /// Resource descriptors whose `owner` is `MEMORY_TYPE_INFO_HOB_GUID` are
    /// skipped as the HOB describes PEI memory bins overlaying system memory.
    fn validate_v1v2_superset(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        let mut v1_resources: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v2_resources: Vec<&ResourceDescriptorSerDe> = Vec::new();

        let mut v1_not_migrated = Vec::new();

        for hob in self.hob_list {
            if let HobSerDe::ResourceDescriptor(v1) = hob {
                if Self::is_memory_type_info(v1) {
                    continue;
                }
                v1_resources.push(v1);
            } else if let HobSerDe::ResourceDescriptorV2 { v1: v2, .. } = hob {
                v2_resources.push(v2);
            }
        }

        // if no v1, that's okay
        // if no v2, is that okay? it means they haven't migrated over to the new resource descriptor format

        let merged_v2 = Interval::merge_intervals(&v2_resources);

        for v1 in v1_resources {
            let mut is_v1_migrated = false;
            for v2 in &merged_v2 {
                if v2.contains(v1) {
                    is_v1_migrated = true;
                    break;
                }
            }

            if !is_v1_migrated {
                v1_not_migrated.push(v1);
            }
        }

        for hob1 in &v1_not_migrated {
            validation_report
                .add_violation(ValidationKind::Hob(HobValidationKind::V1MemoryRangeNotContainedInV2 { hob1 }));
        }

        Ok(validation_report)
    }

    /// Validates that no memory allocations describe page zero address range
    /// (below UEFI_PAGE_SIZE). Reports a violation for each allocation
    /// overlapping this restricted range.
    fn validate_page0_memory_allocation(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        const PAGE_ZERO_END: u64 = UEFI_PAGE_SIZE as u64 - 1;
        for hob in self.hob_list {
            if let HobSerDe::MemoryAllocation { alloc_descriptor } = hob
                && alloc_descriptor.memory_base_address <= PAGE_ZERO_END
            {
                validation_report.add_violation(ValidationKind::Hob(HobValidationKind::PageZeroMemoryDescribed {
                    alloc_desc: alloc_descriptor,
                }));
            }
        }

        Ok(validation_report)
    }

    /// Checks for presence of the MEMORY_UCE attribute in V2 resource
    /// descriptors and reports violations if found.
    fn validate_memory_uce_attribute(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        for hob in self.hob_list {
            if let HobSerDe::ResourceDescriptorV2 { v1, attributes } = hob
                && attributes & efi::MEMORY_UCE != 0
            {
                validation_report.add_violation(ValidationKind::Hob(HobValidationKind::V2ContainsUceAttribute {
                    hob1: v1,
                    attributes: *attributes,
                }));
            }
        }
        Ok(validation_report)
    }

    /// Validates that each V2 resource descriptor has exactly one valid
    /// cacheability attribute set, reporting violations if none or multiple
    /// cache bits are present.
    fn validate_memory_cacheability_attribute(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        for hob in self.hob_list {
            if let HobSerDe::ResourceDescriptorV2 { v1, attributes } = hob {
                const CACHE_ATTRIBUTE_IGNORED_MASK: u64 = !efi::MEMORY_UCE;
                let mask = efi::CACHE_ATTRIBUTE_MASK & CACHE_ATTRIBUTE_IGNORED_MASK;
                // Ensure exactly one cache attribute is set:
                // 1. Check if none of the cache bits are set
                // 2. Check if more than one bit is set by checking if it is not a power of 2
                if (v1.resource_type != EFI_RESOURCE_IO && v1.resource_type != EFI_RESOURCE_IO_RESERVED)
                    && (attributes & mask == 0 || ((attributes & mask) & (attributes - 1)) != 0)
                {
                    validation_report.add_violation(ValidationKind::Hob(
                        HobValidationKind::V2MissingValidCacheabilityAttribute { hob1: v1, attributes: *attributes },
                    ));
                }
            }
        }
        Ok(validation_report)
    }

    /// Validates that each V2 resource descriptor with an IO resource type has
    /// no attributes set.
    fn validate_memory_cacheability_attribute_io_resource_hob(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        for hob in self.hob_list {
            if let HobSerDe::ResourceDescriptorV2 { v1, attributes } = hob
                && (v1.resource_type == EFI_RESOURCE_IO || v1.resource_type == EFI_RESOURCE_IO_RESERVED)
                && *attributes != 0
            {
                validation_report.add_violation(ValidationKind::Hob(
                    HobValidationKind::V2InvalidIoCacheabilityAttributes { hob1: v1, attributes: *attributes },
                ));
            }
        }
        Ok(validation_report)
    }

    /// Returns all Resource Descriptor HOBs whose owner is `MEMORY_TYPE_INFO_HOB_GUID`.
    fn memory_type_info_resource_hobs(&self) -> Vec<&ResourceDescriptorSerDe> {
        self.hob_list
            .iter()
            .filter_map(|hob| match hob {
                HobSerDe::ResourceDescriptor(resource) | HobSerDe::ResourceDescriptorV2 { v1: resource, .. }
                    if Self::is_memory_type_info(resource) =>
                {
                    Some(resource)
                }
                _ => None,
            })
            .collect()
    }

    /// Returns the parsed bin entries from the Memory Type Information GUID HOB, if present.
    fn memory_type_info_entries(&self) -> Option<&[MemoryTypeInfoEntrySerDe]> {
        self.hob_list.iter().find_map(|hob| match hob {
            HobSerDe::MemoryTypeInformation { entries } => Some(entries.as_slice()),
            _ => None,
        })
    }

    /// Validates that at most one Resource Descriptor HOB owned by `MEMORY_TYPE_INFO_HOB_GUID`
    /// exists. The DXE core rejects all such HOBs when multiple are present to avoid an
    /// ambiguous bin-region selection. One violation is reported per discovered HOB.
    fn validate_memory_type_info_single_resource_hob(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        let hobs = self.memory_type_info_resource_hobs();
        if hobs.len() > 1 {
            for hob1 in hobs {
                validation_report
                    .add_violation(ValidationKind::Hob(HobValidationKind::MemoryTypeInfoMultipleResourceHobs { hob1 }));
            }
        }
        Ok(validation_report)
    }

    /// Validates that the `ResourceLength` of the Memory Type Info Resource Descriptor HOB is
    /// large enough to hold all bins reported in the Memory Type Information GUID HOB.
    ///
    /// The check only runs when exactly one Memory Type Info Resource Descriptor HOB is present
    /// and a Memory Type Information GUID HOB has been captured.
    fn validate_memory_type_info_resource_length(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        let hobs = self.memory_type_info_resource_hobs();
        let [resource] = hobs[..] else {
            return Ok(validation_report);
        };
        let Some(entries) = self.memory_type_info_entries() else {
            return Ok(validation_report);
        };

        let required_bytes: u64 =
            entries.iter().map(|entry| entry.number_of_pages as u64 * UEFI_PAGE_SIZE as u64).sum();

        if resource.resource_length < required_bytes {
            validation_report.add_violation(ValidationKind::Hob(
                HobValidationKind::MemoryTypeInfoResourceLengthTooSmall {
                    hob1: resource,
                    required_bytes,
                    actual_bytes: resource.resource_length,
                },
            ));
        }
        Ok(validation_report)
    }
}

impl Validator for HobValidator<'_> {
    fn validate(&self) -> ValidationResult<'_> {
        let mut validation_report = ValidationReport::new();
        if self.hob_list.is_empty() {
            return Err(ValidationAppError::EmptyHobList);
        }

        validation_report.append_report(self.validate_memory_overlap()?);
        validation_report.append_report(self.validate_overlapping_v1v2_attributes()?);
        validation_report.append_report(self.validate_v1v2_superset()?);
        validation_report.append_report(self.validate_page0_memory_allocation()?);
        validation_report.append_report(self.validate_memory_uce_attribute()?);
        validation_report.append_report(self.validate_memory_cacheability_attribute()?);
        validation_report.append_report(self.validate_memory_cacheability_attribute_io_resource_hob()?);
        validation_report.append_report(self.validate_memory_type_info_single_resource_hob()?);
        validation_report.append_report(self.validate_memory_type_info_resource_length()?);
        Ok(validation_report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patina::pi::{
        hob::{EFI_RESOURCE_IO, EFI_RESOURCE_IO_RESERVED, EfiPhysicalAddress},
        serializable::serializable_hob::{MemAllocDescriptorSerDe, ResourceDescriptorSerDe},
    };

    fn create_v1_hob(
        start: EfiPhysicalAddress,
        length: u64,
        resource_type: u32,
        resource_attribute: u32,
        owner: &str,
    ) -> HobSerDe {
        HobSerDe::ResourceDescriptor(ResourceDescriptorSerDe {
            physical_start: start,
            resource_length: length,
            resource_type,
            resource_attribute,
            owner: owner.to_string(),
        })
    }

    fn create_v2_hob(
        start: EfiPhysicalAddress,
        length: u64,
        resource_type: u32,
        resource_attribute: u32,
        owner: &str,
        attributes: u64,
    ) -> HobSerDe {
        HobSerDe::ResourceDescriptorV2 {
            v1: ResourceDescriptorSerDe {
                physical_start: start,
                resource_length: length,
                resource_type,
                resource_attribute,
                owner: owner.to_string(),
            },
            attributes,
        }
    }

    fn create_memory_hob(name: String, memory_base_address: u64, memory_length: u64, memory_type: u32) -> HobSerDe {
        HobSerDe::MemoryAllocation {
            alloc_descriptor: MemAllocDescriptorSerDe { name, memory_base_address, memory_length, memory_type },
        }
    }

    #[test]
    fn test_validate_memory_overlap() {
        // it is OKAY if v1 v2 hobs overlap -- it should not be flagged
        let hob1 = create_v1_hob(100, 50, 3, 0, "owner1");
        let hob2 = create_v2_hob(100, 50, 3, 0, "owner1", 123);
        let hob_list = vec![hob1, hob2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_overlap();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_validate_v1v2_superset_ok() {
        // V1 hob fully covered by single V2
        let v1_hob = create_v1_hob(200, 30, 3, 0, "owner1");
        let v2_hob = create_v2_hob(100, 200, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_v1v2_superset();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_check_v1v2_multiple_superset_ok() {
        // V1 hob fully covered by multiple V2's
        // [200, 250] is covered by [100, 220] and [220, 300]
        let v1_hob = create_v1_hob(200, 50, 3, 0, "owner1");
        let v2_hob1 = create_v2_hob(100, 120, 3, 0, "owner1", 123);
        let v2_hob2 = create_v2_hob(220, 80, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob1, v2_hob2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_v1v2_superset();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_validate_v1v2_superset_fail() {
        // V1 not fully covered (gap)
        let v1_hob = create_v1_hob(200, 100, 3, 0, "owner1");
        let v2_hob1 = create_v2_hob(100, 50, 3, 0, "owner1", 123);
        let v2_hob2 = create_v2_hob(180, 10, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob1, v2_hob2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_v1v2_superset();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_check_overlapping_v1v2_consistency_ok() {
        // Consistent v1 and v2
        let v1_hob = create_v1_hob(100, 100, 3, 0, "owner1");
        let v2_hob = create_v2_hob(150, 100, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_check_non_overlapping_v1v2_different_attributes_no_violation() {
        // Non-overlapping v1 and v2 with different resource attributes and owners should not produce a violation
        let v1_hob = create_v1_hob(100, 50, 3, 1, "owner1");
        let v2_hob = create_v2_hob(200, 50, 4, 2, "owner2", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_check_overlapping_v1v2_consistency_fail() {
        // Overlapping and inconsistent v1/v2 (diff resource type)
        let v1_hob = create_v1_hob(100, 100, 3, 0, "owner1");
        let v2_hob = create_v2_hob(150, 100, 4, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_page0_memory_allocation() {
        let page_zero_mem_hob = create_memory_hob("test".to_string(), 0, 0x10, 1);
        let mem_hob = create_memory_hob("test2".to_string(), UEFI_PAGE_SIZE as u64 + 1, 0x100, 1);
        let hob_list = vec![mem_hob.clone()];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_page0_memory_allocation();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);

        let hob_list = vec![mem_hob.clone(), page_zero_mem_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_page0_memory_allocation();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_memory_uce_attribute() {
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UCE);
        let hob_list = vec![v2_hob];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_uce_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_memory_v2_cacheability_attributes() {
        // +ve test - valid cacheability attribute specified
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UC);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);

        // -ve test - supported cacheability attribute specified
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UCE);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        // -ve test - invalid cacheability attribute specified
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_RO);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        // -ve test - multiple cacheability attributes specified
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_WT | efi::MEMORY_WC);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        // +ve test - valid cacheability attributes specified
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_WC);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);

        // -ve test - invalid cacheability attributes value(0) specified
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", 0);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_memory_v2_access_protection_attributes() {
        // +ve test - valid cacheability attribute specified with a single access protection attribute
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UC | efi::MEMORY_RO);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);

        // +ve test - valid cacheability attribute specified with multiple access protection attributes
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UC | efi::MEMORY_RO | efi::MEMORY_XP);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_eq!(validation_report.violation_count(), 0);
    }

    #[test]
    fn test_memory_v2_io_cacheability_attributes() {
        // -ve test - an io resource descriptor should not have any cacheability attributes
        let v2_hob = create_v2_hob(100, 100, EFI_RESOURCE_IO, 0, "owner1", efi::MEMORY_UC);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute_io_resource_hob();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);

        // -ve test - an io reserved resource descriptor should not have any cacheability attributes
        let v2_hob = create_v2_hob(100, 100, EFI_RESOURCE_IO_RESERVED, 0, "owner1", efi::MEMORY_UC);
        let hob_list = vec![v2_hob];
        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_cacheability_attribute_io_resource_hob();
        assert!(result.is_ok());
        let validation_report = result.unwrap();
        assert_ne!(validation_report.violation_count(), 0);
    }

    /// String form of `MEMORY_TYPE_INFO_HOB_GUID` used for tests.
    fn mem_info_owner() -> String {
        MEMORY_TYPE_INFO_HOB_GUID.as_guid().to_string()
    }

    /// String form of `OwnedGuid::ZERO` used for tests.
    fn zero_owner() -> String {
        OwnedGuid::ZERO.to_string()
    }

    /// Test that a V1 resource descriptor HOB owned by `MEMORY_TYPE_INFO_HOB_GUID`
    /// that overlaps a V2 system memory range with different `resource_attribute`
    /// does not get flagged.
    #[test]
    fn test_memory_type_info_v1_overlap_with_v2_is_not_flagged() {
        let v1 = create_v1_hob(0x7e233000, 0xdbb000, 0, 0x7, &mem_info_owner());
        let v2 = create_v2_hob(0x100000, 0x7ef00000, 0, 0x3c07, &zero_owner(), 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// Test that the `MEMORY_TYPE_INFO_HOB_GUID` skip is case-insensitive. A
    /// captured owner string in uppercase (or mixed case) should match the same
    /// well-known GUID and be skipped.
    ///
    /// Note: This test deliberately uses a hard-coded uppercase string (rather than
    /// the SDK constant).
    #[test]
    fn test_memory_type_info_skip_is_case_insensitive() {
        const MEM_INFO_OWNER_UPPER: &str = "4C19049F-4137-4DD3-9C10-8B97A83FFDFA";
        let v1 = create_v1_hob(0x7e233000, 0xdbb000, 0, 0x7, MEM_INFO_OWNER_UPPER);
        let v2 = create_v2_hob(0x100000, 0x7ef00000, 0, 0x3c07, &zero_owner(), 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// Test that the skip relies on `OwnedGuid::try_from_string`, which tolerates
    /// the no-dashes hex form. The same captured GUID without dashes should still
    /// be recognized as `MEMORY_TYPE_INFO_HOB_GUID`.
    #[test]
    fn test_memory_type_info_skip_accepts_no_dashes() {
        const MEM_INFO_OWNER_NO_DASHES: &str = "4c19049f41374dd39c108b97a83ffdfa";
        let v1 = create_v1_hob(0x7e233000, 0xdbb000, 0, 0x7, MEM_INFO_OWNER_NO_DASHES);
        let v2 = create_v2_hob(0x100000, 0x7ef00000, 0, 0x3c07, &zero_owner(), 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// Test that a malformed owner string does not silently match `MEMORY_TYPE_INFO_HOB_GUID`.
    #[test]
    fn test_malformed_owner_does_not_match_memory_type_info() {
        let v1 = create_v1_hob(0x100000, 0x10000, 0, 0x7, "not-a-guid");
        let v2 = create_v2_hob(0x100000, 0x10000, 0, 0x3c07, &zero_owner(), 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 1);
    }

    /// An owner-only mismatch between overlapping V1 and V2 descriptors (with
    /// identical `resource_type` and `resource_attribute`) is not a memory
    /// attribute inconsistency and should not produce a violation.
    #[test]
    fn test_owner_only_mismatch_is_not_flagged() {
        let v1 = create_v1_hob(0x100000, 0x10000, 0, 0x3c07, "owner-a");
        let v2 = create_v2_hob(0x100000, 0x10000, 0, 0x3c07, "owner-b", 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// Test that the V1/V2 attribute consistency check detects mismatches for
    /// different attributes with a non-`MEMORY_TYPE_INFO_HOB_GUID` owner.
    #[test]
    fn test_non_memory_type_info_attribute_mismatch_is_still_flagged() {
        // Same range, same owner, mismatched resource_attribute.
        let v1 = create_v1_hob(0x100000, 0x10000, 0, 0x7, "owner-x");
        let v2 = create_v2_hob(0x100000, 0x10000, 0, 0x3c07, "owner-x", 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_overlapping_v1v2_attributes();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 1);
    }

    fn mem_type_info_hob(entries: Vec<MemoryTypeInfoEntrySerDe>) -> HobSerDe {
        HobSerDe::MemoryTypeInformation { entries }
    }

    /// Exactly one Resource Descriptor HOB owned by `MEMORY_TYPE_INFO_HOB_GUID`
    /// is the expected configuration and must not be flagged.
    #[test]
    fn test_single_memory_type_info_resource_hob_is_ok() {
        let v1 = create_v1_hob(0x7e233000, 0xdbb000, 0, 0x7, &mem_info_owner());
        let hob_list = vec![v1];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_single_resource_hob();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// A Memory Type Info HOB reported as a V2 Resource Descriptor HOB should be accepted
    /// the same as a V1 Resource Descriptor HOB.
    #[test]
    fn test_single_memory_type_info_v2_resource_hob_is_ok() {
        let v2 = create_v2_hob(0x7e233000, 0xdbb000, 0, 0x7, &mem_info_owner(), 0);
        let hob_list = vec![v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_single_resource_hob();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// Zero Resource Descriptor HOBs owned by `MEMORY_TYPE_INFO_HOB_GUID` is
    /// out of scope for this check.
    #[test]
    fn test_zero_memory_type_info_resource_hobs_is_ok() {
        let v2 = create_v2_hob(0x100000, 0x1000, 0, 0x3c07, &zero_owner(), 0);
        let hob_list = vec![v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_single_resource_hob();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// Multiple Resource Descriptor HOBs owned by `MEMORY_TYPE_INFO_HOB_GUID`
    /// must each be reported as a violation.
    #[test]
    fn test_multiple_memory_type_info_resource_hobs_are_flagged() {
        let v1a = create_v1_hob(0x7e233000, 0xdbb000, 0, 0x7, &mem_info_owner());
        let v1b = create_v1_hob(0x90000000, 0x10000, 0, 0x7, &mem_info_owner());
        let v1c = create_v1_hob(0xa0000000, 0x10000, 0, 0x7, &mem_info_owner());
        let hob_list = vec![v1a, v1b, v1c];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_single_resource_hob();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 3);
    }

    /// Multiple Memory Type Info Resource Descriptor HOBs are flagged regardless
    /// of whether they are reported as V1 or V2 Resource Descriptor HOBs. A mix of the two
    /// will report one violation per HOB.
    #[test]
    fn test_multiple_memory_type_info_mixed_v1_v2_resource_hobs_are_flagged() {
        let v1 = create_v1_hob(0x7e233000, 0xdbb000, 0, 0x7, &mem_info_owner());
        let v2 = create_v2_hob(0x90000000, 0x10000, 0, 0x7, &mem_info_owner(), 0);
        let hob_list = vec![v1, v2];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_single_resource_hob();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 2);
    }

    /// `ResourceLength` greater than the raw bin total must not be flagged.
    #[test]
    fn test_memory_type_info_resource_length_sufficient_is_ok() {
        let v1 = create_v1_hob(0x7e000000, 0x100000, 0, 0x7, &mem_info_owner());
        let mti = mem_type_info_hob(vec![
            MemoryTypeInfoEntrySerDe { memory_type: 6, number_of_pages: 2 },
            MemoryTypeInfoEntrySerDe { memory_type: 5, number_of_pages: 2 },
        ]);
        let hob_list = vec![v1, mti];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_resource_length();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// `ResourceLength` exactly equal to the raw bin total must not be flagged.
    #[test]
    fn test_memory_type_info_resource_length_exact_is_ok() {
        let required = 4 * UEFI_PAGE_SIZE as u64;
        let v1 = create_v1_hob(0x7e000000, required, 0, 0x7, &mem_info_owner());
        let mti = mem_type_info_hob(vec![
            MemoryTypeInfoEntrySerDe { memory_type: 6, number_of_pages: 2 },
            MemoryTypeInfoEntrySerDe { memory_type: 5, number_of_pages: 2 },
        ]);
        let hob_list = vec![v1, mti];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_resource_length();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// `ResourceLength` smaller than the raw bin total must be flagged.
    #[test]
    fn test_memory_type_info_resource_length_too_small_is_flagged() {
        let v1 = create_v1_hob(0x7e000000, 0x1000, 0, 0x7, &mem_info_owner());
        let mti = mem_type_info_hob(vec![
            MemoryTypeInfoEntrySerDe { memory_type: 6, number_of_pages: 5 },
            MemoryTypeInfoEntrySerDe { memory_type: 5, number_of_pages: 5 },
        ]);
        let hob_list = vec![v1, mti];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_resource_length();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 1);
    }

    /// The `ResourceLength` check applies to a Memory Type Info HOB reported as a
    /// V2 Resource Descriptor HOB as well.
    #[test]
    fn test_memory_type_info_v2_resource_length_too_small_is_flagged() {
        let v2 = create_v2_hob(0x7e000000, 0x1000, 0, 0x7, &mem_info_owner(), 0);
        let mti = mem_type_info_hob(vec![
            MemoryTypeInfoEntrySerDe { memory_type: 6, number_of_pages: 5 },
            MemoryTypeInfoEntrySerDe { memory_type: 5, number_of_pages: 5 },
        ]);
        let hob_list = vec![v2, mti];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_resource_length();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 1);
    }

    /// When no Memory Type Information GUID HOB is captured, the length check
    /// is  skipped (since the required size cannot be determined).
    #[test]
    fn test_memory_type_info_resource_length_without_guid_hob_is_skipped() {
        let v1 = create_v1_hob(0x7e000000, 0x1000, 0, 0x7, &mem_info_owner());
        let hob_list = vec![v1];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_resource_length();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }

    /// When more than one Memory Type Info Resource Descriptor HOB is present,
    /// the length check defers to the single instance check so a second violation
    /// is not reported.
    #[test]
    fn test_memory_type_info_resource_length_with_multiple_hobs_is_skipped() {
        let v1a = create_v1_hob(0x7e000000, 0x1000, 0, 0x7, &mem_info_owner());
        let v1b = create_v1_hob(0x90000000, 0x1000, 0, 0x7, &mem_info_owner());
        let mti = mem_type_info_hob(vec![MemoryTypeInfoEntrySerDe { memory_type: 6, number_of_pages: 100 }]);
        let hob_list = vec![v1a, v1b, mti];

        let validator = HobValidator::new(&hob_list);
        let result = validator.validate_memory_type_info_resource_length();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().violation_count(), 0);
    }
}
