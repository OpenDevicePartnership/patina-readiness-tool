use common::serializable_hob::HobSerDe;
use common::serializable_hob::ResourceDescriptorSerDe;
use mu_pi::hob::{EFI_RESOURCE_IO, EFI_RESOURCE_IO_RESERVED};
use r_efi::efi;
use uefi_sdk::base::UEFI_PAGE_SIZE;

use crate::validate::{ValidationApp, ValidationKind};
use crate::ValidationResult;
use common::{DxeReadinessCaptureSerDe, Interval};

use super::HobValidationKind;

fn is_io(resource_type: u32) -> bool {
    resource_type == EFI_RESOURCE_IO || resource_type == EFI_RESOURCE_IO_RESERVED
}

fn check_hob_overlap<T>(resource_list: &[&T]) -> Vec<(T, T)>
where
    T: Interval,
{
    let mut overlaps = Vec::new();
    for i in 0..resource_list.len() {
        for j in (i + 1)..resource_list.len() {
            if resource_list[i].overlaps(resource_list[j]) {
                overlaps.push(((*resource_list[i]).clone(), (*resource_list[j]).clone()));
            }
        }
    }

    overlaps
}

impl ValidationApp {
    pub fn check_memory_overlap(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        let mut overlaps = Vec::new();

        let mut v1_memory_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v2_memory_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v1_io_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v2_io_hobs: Vec<&ResourceDescriptorSerDe> = Vec::new();

        for hob in hob_list {
            match hob {
                HobSerDe::ResourceDescriptor(resource) if !is_io(resource.resource_type) => {
                    v1_memory_hobs.push(resource)
                }
                HobSerDe::ResourceDescriptorV2 { v1: resource, .. } if !is_io(resource.resource_type) => {
                    v2_memory_hobs.push(resource)
                }
                HobSerDe::ResourceDescriptor(resource) if is_io(resource.resource_type) => v1_io_hobs.push(resource),
                HobSerDe::ResourceDescriptorV2 { v1: resource, .. } if is_io(resource.resource_type) => {
                    v2_io_hobs.push(resource)
                }
                _ => (),
            }
        }

        overlaps.extend(check_hob_overlap(&v1_memory_hobs));
        overlaps.extend(check_hob_overlap(&v2_memory_hobs));
        overlaps.extend(check_hob_overlap(&v1_io_hobs));
        overlaps.extend(check_hob_overlap(&v2_io_hobs));

        for (hob1, hob2) in &overlaps {
            self.validation_report.add_violation(
                ValidationKind::Hob(HobValidationKind::OverlappingMemoryRanges),
                &format!("{:?} <-> {:?}", hob1, hob2),
            );
        }

        Ok(())
    }

    // For v1/v2, The requirement is that v2 hobs are a superset of v1 Below is
    // the strategy use:
    // - Check for consistency:
    //  - If any v1 hobs overlap with v2 hobs, make sure they have the same
    //    attributes
    // - Check for superset property:
    //  - Sort and merge all hobs
    //  - For each v1 interval, make sure some combination (merged) of v2 hobs
    //    covers it fully quick proof sketch that merging is safe: if a V1
    //    overlaps with any V2s, those V2s must have the same attributes as it,
    //    so it's safe to merge for the superset check
    // - If v1 and v2 overlap, make sure info is consistent
    fn check_overlapping_v1v2_attributes(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        let mut inconsistent_v1_v2 = Vec::new();

        for hob1 in hob_list {
            for hob2 in hob_list {
                let HobSerDe::ResourceDescriptor(v1) = hob1 else { continue };
                let HobSerDe::ResourceDescriptorV2 { v1: v2, .. } = hob2 else { continue };
                if v1.overlaps(v2) && v1.resource_type != v2.resource_type
                    || v1.resource_attribute != v2.resource_attribute
                    || v1.owner != v2.owner
                {
                    inconsistent_v1_v2.push(((*v1).clone(), (*v2).clone()));
                }
            }
        }

        for (hob1, hob2) in &inconsistent_v1_v2 {
            self.validation_report.add_violation(
                ValidationKind::Hob(HobValidationKind::InconsistentMemoryAttributes),
                &format!("Inconsistent Memory Attribute HOBs: {:?} and {:?}", hob1, hob2),
            );
        }

        Ok(())
    }

    fn check_v1v2_superset(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        let mut v1_resources: Vec<&ResourceDescriptorSerDe> = Vec::new();
        let mut v2_resources: Vec<&ResourceDescriptorSerDe> = Vec::new();

        let mut v1_not_migrated = Vec::new();

        for hob in hob_list {
            if let HobSerDe::ResourceDescriptor(v1) = hob {
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
                v1_not_migrated.push((*v1).clone());
            }
        }

        for v1 in &v1_not_migrated {
            self.validation_report.add_violation(
                ValidationKind::Hob(HobValidationKind::V1MemoryRangeNotContainedInV2),
                &format!("{:?}", v1),
            );
        }

        Ok(())
    }

    fn check_v1v2_consistency(&mut self) -> ValidationResult {
        self.check_overlapping_v1v2_attributes()?;
        self.check_v1v2_superset()?;
        Ok(())
    }

    fn check_page0(&mut self) -> ValidationResult {
        const PAGE_ZERO_END: u64 = UEFI_PAGE_SIZE as u64 - 1;
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        for hob in hob_list {
            if let HobSerDe::MemoryAllocation { alloc_descriptor } = hob {
                if alloc_descriptor.memory_base_address <= PAGE_ZERO_END {
                    self.validation_report.add_violation(
                        ValidationKind::Hob(HobValidationKind::PageZeroMemoryDescribed),
                        &format!("{:?}", alloc_descriptor),
                    );
                }
            }
        }

        Ok(())
    }

    /// Confirm no resource descriptor HOB v2 contains EFI_MEMORY_UCE as the set attribute
    fn check_mem_uce(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        for hob in hob_list {
            if let HobSerDe::ResourceDescriptorV2 { v1, attributes } = hob {
                if attributes & efi::MEMORY_UCE != 0 {
                    self.validation_report.add_violation(
                        ValidationKind::Hob(HobValidationKind::V2ContainsUceAttribute),
                        &format!("{:?}", v1),
                    );
                }
            }
        }
        Ok(())
    }

    /// Confirm resource descriptor HOB v2 contains atleast one valid cacheability attribute set
    fn check_mem_valid_cacheability(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Ok(());
        };

        for hob in hob_list {
            if let HobSerDe::ResourceDescriptorV2 { v1, attributes } = hob {
                const CACHE_ATTRIBUTE_IGNORED_MASK: u64 = !efi::MEMORY_UCE;
                let mask = efi::CACHE_ATTRIBUTE_MASK & CACHE_ATTRIBUTE_IGNORED_MASK;
                if attributes & mask == 0 {
                    self.validation_report.add_violation(
                        ValidationKind::Hob(HobValidationKind::V2MissingValidCacheabilityAttribute),
                        &format!("{:?}", v1),
                    );
                }
            }
        }
        Ok(())
    }

    pub fn validate_hobs(&mut self) -> ValidationResult {
        let Some(DxeReadinessCaptureSerDe { ref hob_list, .. }) = self.data.as_ref() else {
            return Err("HOB list is empty".to_string());
        };

        if hob_list.is_empty() {
            return Err("HOB list is empty".to_string());
        }

        self.check_memory_overlap()?;
        self.check_v1v2_consistency()?;
        self.check_page0()?;
        self.check_mem_uce()?;
        self.check_mem_valid_cacheability()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::serializable_hob::{MemAllocDescriptorSerDe, ResourceDescriptorSerDe};
    use mu_pi::hob::EfiPhysicalAddress;

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
    fn test_check_memory_overlap() {
        // it is OKAY if v1 v2 hobs overlap -- it should not be flagged
        let hob1 = create_v1_hob(100, 50, 3, 0, "owner1");
        let hob2 = create_v2_hob(100, 50, 3, 0, "owner1", 123);
        let hob_list = vec![hob1, hob2];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        assert_eq!(app.check_memory_overlap(), Ok(()));
        assert!(app.validation_report.is_empty());
    }

    #[test]
    fn test_check_v1v2_superset_ok() {
        // V1 hob fully covered by single V2
        let v1_hob = create_v1_hob(200, 30, 3, 0, "owner1");
        let v2_hob = create_v2_hob(100, 200, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        assert_eq!(app.check_v1v2_superset(), Ok(()));
        assert!(app.validation_report.is_empty());
    }

    #[test]
    fn test_check_v1v2_multiple_superset_ok() {
        // V1 hob fully covered by multiple V2's
        // [200, 250] is covered by [100, 220] and [220, 300]
        let v1_hob = create_v1_hob(200, 50, 3, 0, "owner1");
        let v2_hob1 = create_v2_hob(100, 120, 3, 0, "owner1", 123);
        let v2_hob2 = create_v2_hob(220, 80, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob1, v2_hob2];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        assert_eq!(app.check_v1v2_superset(), Ok(()));
        assert!(app.validation_report.is_empty());
    }

    #[test]
    fn test_check_v1v2_superset_fail() {
        // V1 not fully covered (gap)
        let v1_hob = create_v1_hob(200, 100, 3, 0, "owner1");
        let v2_hob1 = create_v2_hob(100, 50, 3, 0, "owner1", 123);
        let v2_hob2 = create_v2_hob(180, 10, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob1, v2_hob2];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_v1v2_superset();
        assert!(res.is_ok());
        assert!(!app.validation_report.is_empty());
    }

    #[test]
    fn test_check_overlapping_v1v2_consistency_ok() {
        // Consistent v1 and v2
        let v1_hob = create_v1_hob(100, 100, 3, 0, "owner1");
        let v2_hob = create_v2_hob(150, 100, 3, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        assert_eq!(app.check_overlapping_v1v2_attributes(), Ok(()));
        assert!(app.validation_report.is_empty());
    }

    #[test]
    fn test_check_overlapping_v1v2_consistency_fail() {
        // Overlapping and inconsistent v1/v2 (diff resource type)
        let v1_hob = create_v1_hob(100, 100, 3, 0, "owner1");
        let v2_hob = create_v2_hob(150, 100, 4, 0, "owner1", 123);
        let hob_list = vec![v1_hob, v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_overlapping_v1v2_attributes();
        assert!(res.is_ok());
        assert!(!app.validation_report.is_empty());
    }

    #[test]
    fn test_pagezero() {
        let page_zero_mem_hob = create_memory_hob("test".to_string(), 0, 0x10, 1);
        let mem_hob = create_memory_hob("test2".to_string(), UEFI_PAGE_SIZE as u64 + 1, 0x100, 1);
        let hob_list = vec![mem_hob.clone()];
        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_page0();
        assert!(res.is_ok());
        assert!(app.validation_report.is_empty());

        let hob_list = vec![mem_hob.clone(), page_zero_mem_hob];
        app = ValidationApp::new_with_data(DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] });
        let res = app.check_page0();
        assert!(res.is_ok());
        assert!(!app.validation_report.is_empty());
    }

    #[test]
    fn test_mem_uce() {
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UCE);
        let hob_list = vec![v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_mem_uce();
        assert!(res.is_ok());
        assert!(!app.validation_report.is_empty());
    }

    #[test]
    fn test_mem_v2_cacheability() {
        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UC);
        let hob_list = vec![v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_mem_valid_cacheability();
        assert!(res.is_ok());
        assert!(app.validation_report.is_empty());

        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_UCE);
        let hob_list = vec![v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_mem_valid_cacheability();
        assert!(res.is_ok());
        assert!(!app.validation_report.is_empty());

        let v2_hob = create_v2_hob(100, 100, 3, 0, "owner1", efi::MEMORY_RO);
        let hob_list = vec![v2_hob];

        let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
        let mut app = ValidationApp::new_with_data(data);
        let res = app.check_mem_valid_cacheability();
        assert!(res.is_ok());
        assert!(!app.validation_report.is_empty());
    }
}
