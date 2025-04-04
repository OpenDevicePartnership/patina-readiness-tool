// basically you can't deserialize references (complex explanation but if this assumption is wrong then we can rework the code)
// but the key point is that we don't actually need to deserialize into mu_pi::hob structs, we can deserialize kinda whatever we want
// so here i cut out some information from mu_pi (ie headers)
// and also get rid of the references so we can easily derive deserialization
// if the assumptions are wrong (assumption #0: deserialization of references not possible) then we can rework this
// but the point is that these structs are local to this tool and there's no need to reference the more complex PI spec structs
// for internal validation purposes

use mu_pi::hob::{EfiPhysicalAddress, Hob, HobList};
use serde::{Deserialize, Serialize};

use alloc::string::String;
use alloc::vec::Vec;

use crate::format_guid;

// This is the serialized version of the HOB list.
#[derive(Serialize, Deserialize, Debug)]
pub struct HobListSerDe {
    pub hobs: Vec<HobSerDe>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HobSerDe {
    Handoff {
        version: u32,
        memory_top: EfiPhysicalAddress,
        memory_bottom: EfiPhysicalAddress,
        free_memory_top: EfiPhysicalAddress,
        free_memory_bottom: EfiPhysicalAddress,
        end_of_hob_list: EfiPhysicalAddress,
    },
    MemoryAllocation {
        alloc_descriptor: MemAllocDescriptorSerDe,
    },
    ResourceDescriptor(ResourceDescriptorSerDe),
    ResourceDescriptorV2 {
        v1: ResourceDescriptorSerDe,
        attributes: u64,
    },
    GuidExtension {
        name: String,
    },
    FirmwareVolume {
        base_address: EfiPhysicalAddress,
        length: u64,
    },
    Cpu {
        size_of_memory_space: u8,
        size_of_io_space: u8,
    },
    UnknownHob,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemAllocDescriptorSerDe {
    name: String, // GUID as a string
    memory_base_address: u64,
    memory_length: u64,
    memory_type: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceDescriptorSerDe {
    owner: String, // GUID as a string
    resource_type: u32,
    resource_attribute: u32,
    physical_start: u64,
    resource_length: u64,
}

impl From<&HobList<'_>> for HobListSerDe {
    fn from(hob_list: &HobList) -> Self {
        HobListSerDe { hobs: hob_list.iter().map(HobSerDe::from).collect() }
    }
}

impl From<&Hob<'_>> for HobSerDe {
    fn from(hob: &Hob) -> Self {
        match hob {
            Hob::Handoff(handoff) => Self::Handoff {
                version: handoff.version,
                memory_top: handoff.memory_top,
                memory_bottom: handoff.memory_bottom,
                free_memory_top: handoff.free_memory_top,
                free_memory_bottom: handoff.free_memory_bottom,
                end_of_hob_list: handoff.end_of_hob_list,
            },
            Hob::MemoryAllocation(mem_alloc) => Self::MemoryAllocation {
                alloc_descriptor: MemAllocDescriptorSerDe {
                    name: format_guid(mem_alloc.alloc_descriptor.name),
                    memory_base_address: mem_alloc.alloc_descriptor.memory_base_address,
                    memory_length: mem_alloc.alloc_descriptor.memory_length,
                    memory_type: mem_alloc.alloc_descriptor.memory_type,
                },
            },
            Hob::ResourceDescriptor(resource_desc) => Self::ResourceDescriptor(ResourceDescriptorSerDe {
                owner: format_guid(resource_desc.owner),
                resource_type: resource_desc.resource_type,
                resource_attribute: resource_desc.resource_attribute,
                physical_start: resource_desc.physical_start,
                resource_length: resource_desc.resource_length,
            }),
            Hob::ResourceDescriptorV2(resource_desc2) => Self::ResourceDescriptorV2 {
                v1: ResourceDescriptorSerDe {
                    owner: format_guid(resource_desc2.v1.owner),
                    resource_type: resource_desc2.v1.resource_type,
                    resource_attribute: resource_desc2.v1.resource_attribute,
                    physical_start: resource_desc2.v1.physical_start,
                    resource_length: resource_desc2.v1.resource_length,
                },
                attributes: resource_desc2.attributes,
            },
            Hob::GuidHob(guid_ext, _) => {
                Self::GuidExtension { name: format_guid(guid_ext.name) /* data: data.to_vec() */ }
            }
            Hob::FirmwareVolume(fv) => Self::FirmwareVolume { base_address: fv.base_address, length: fv.length },
            Hob::Cpu(cpu) => {
                Self::Cpu { size_of_memory_space: cpu.size_of_memory_space, size_of_io_space: cpu.size_of_io_space }
            }
            _ => Self::UnknownHob {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mu_pi::{hob, BootMode};
    use serde_json::{from_str, to_string_pretty};

    #[test]
    fn test_hoblist_deserialization() {
        let json_data = r#"
        {
            "hobs": [
                {
                    "type": "handoff",
                    "version": 1,
                    "memory_top": 3735928559,
                    "memory_bottom": 3735932206,
                    "free_memory_top": 1048576,
                    "free_memory_bottom": 65536,
                    "end_of_hob_list": 4277009102
                },
                {
                    "type": "memory_allocation",
                    "alloc_descriptor": {
                    "name": "123e4567-e89b-12d3-a456-426614174000",
                    "memory_base_address": 4096,
                    "memory_length": 12345678,
                    "memory_type": 0
                    }
                },
                {
                    "type": "resource_descriptor",
                    "owner": "123e4567-e89b-12d3-a456-426614174000",
                    "resource_type": 1,
                    "resource_attribute": 2,
                    "physical_start": 8192,
                    "resource_length": 16384
                },
                {
                    "type": "resource_descriptor_v2",
                    "v1": {
                    "owner": "123e4567-e89b-12d3-a456-426614174000",
                    "resource_type": 1,
                    "resource_attribute": 2,
                    "physical_start": 8192,
                    "resource_length": 16384
                    },
                    "attributes": 42
                },
                {
                    "type": "guid_extension",
                    "name": "123e4567-e89b-12d3-a456-426614174000"
                },
                {
                    "type": "firmware_volume",
                    "base_address": 65536,
                    "length": 987654321
                },
                {
                    "type": "cpu",
                    "size_of_memory_space": 48,
                    "size_of_io_space": 16
                },
                {
                    "type": "unknown_hob"
                }
            ]
        }
        "#;

        let deserialized_hob_list: HobListSerDe = from_str(json_data).expect("Failed to deserialize");

        assert_eq!(deserialized_hob_list.hobs.len(), 8);
        if let HobSerDe::Handoff {
            version,
            memory_top,
            memory_bottom,
            free_memory_top,
            free_memory_bottom,
            end_of_hob_list,
        } = &deserialized_hob_list.hobs[0]
        {
            assert_eq!(*version, 1);
            assert_eq!(*memory_top, 3735928559);
            assert_eq!(*memory_bottom, 3735932206);
            assert_eq!(*free_memory_top, 1048576);
            assert_eq!(*free_memory_bottom, 65536);
            assert_eq!(*end_of_hob_list, 4277009102);
        } else {
            panic!("First element is not a Handoff HOB");
        }

        if let HobSerDe::MemoryAllocation { alloc_descriptor } = &deserialized_hob_list.hobs[1] {
            assert_eq!(alloc_descriptor.name, "123e4567-e89b-12d3-a456-426614174000");
            assert_eq!(alloc_descriptor.memory_base_address, 4096);
            assert_eq!(alloc_descriptor.memory_length, 12345678);
            assert_eq!(alloc_descriptor.memory_type, 0);
        } else {
            panic!("Second element is not a MemoryAllocation HOB");
        }

        if let HobSerDe::ResourceDescriptor(resource_desc) = &deserialized_hob_list.hobs[2] {
            assert_eq!(resource_desc.owner, "123e4567-e89b-12d3-a456-426614174000");
            assert_eq!(resource_desc.resource_type, 1);
            assert_eq!(resource_desc.resource_attribute, 2);
            assert_eq!(resource_desc.physical_start, 8192);
            assert_eq!(resource_desc.resource_length, 16384);
        } else {
            panic!("Third element is not a ResourceDescriptor HOB");
        }

        if let HobSerDe::ResourceDescriptorV2 { v1, attributes } = &deserialized_hob_list.hobs[3] {
            assert_eq!(v1.owner, "123e4567-e89b-12d3-a456-426614174000");
            assert_eq!(v1.resource_type, 1);
            assert_eq!(v1.resource_attribute, 2);
            assert_eq!(v1.physical_start, 8192);
            assert_eq!(v1.resource_length, 16384);
            assert_eq!(*attributes, 42);
        } else {
            panic!("Fourth element is not a ResourceDescriptorV2 HOB");
        }

        if let HobSerDe::GuidExtension { name } = &deserialized_hob_list.hobs[4] {
            assert_eq!(name, "123e4567-e89b-12d3-a456-426614174000");
        } else {
            panic!("Fifth element is not a GuidExtension HOB");
        }

        if let HobSerDe::FirmwareVolume { base_address, length } = &deserialized_hob_list.hobs[5] {
            assert_eq!(*base_address, 65536);
            assert_eq!(*length, 987654321);
        } else {
            panic!("Sixth element is not a FirmwareVolume HOB");
        }

        if let HobSerDe::Cpu { size_of_memory_space, size_of_io_space } = &deserialized_hob_list.hobs[6] {
            assert_eq!(*size_of_memory_space, 48);
            assert_eq!(*size_of_io_space, 16);
        } else {
            panic!("Seventh element is not a CPU HOB");
        }
    }

    #[test]
    fn test_hoblist_serialization() {
        let header = hob::header::Hob {
            r#type: hob::HANDOFF,
            length: size_of::<hob::PhaseHandoffInformationTable>() as u16,
            reserved: 0,
        };
        let handoff_hob = hob::PhaseHandoffInformationTable {
            header,
            version: 0x00010000,
            boot_mode: BootMode::BootWithFullConfiguration,
            memory_top: 0xdeadbeef,
            memory_bottom: 0xdeadc0de,
            free_memory_top: 104,
            free_memory_bottom: 255,
            end_of_hob_list: 0xdeaddeadc0dec0de,
        };

        let header = hob::header::Hob {
            r#type: hob::MEMORY_ALLOCATION,
            length: size_of::<hob::MemoryAllocation>() as u16,
            reserved: 0,
        };
        let alloc_descriptor = hob::header::MemoryAllocation {
            name: r_efi::efi::Guid::from_fields(1, 2, 3, 4, 5, &[6, 7, 8, 9, 10, 11]),
            memory_base_address: 0,
            memory_length: 0x0123456789abcdef,
            memory_type: 0,
            reserved: [0; 4],
        };
        let memory_alloc_hob = hob::MemoryAllocation { header, alloc_descriptor };

        let header = hob::header::Hob {
            r#type: hob::RESOURCE_DESCRIPTOR,
            length: size_of::<hob::ResourceDescriptor>() as u16,
            reserved: 0,
        };
        let resource_desc_hob = hob::ResourceDescriptor {
            header,
            owner: r_efi::efi::Guid::from_fields(1, 2, 3, 4, 5, &[6, 7, 8, 9, 10, 11]),
            resource_type: hob::EFI_RESOURCE_SYSTEM_MEMORY,
            resource_attribute: hob::EFI_RESOURCE_ATTRIBUTE_PRESENT,
            physical_start: 0,
            resource_length: 0x0123456789abcdef,
        };

        let mut v1 = hob::ResourceDescriptor {
            header,
            owner: r_efi::efi::Guid::from_fields(1, 2, 3, 4, 5, &[6, 7, 8, 9, 10, 11]),
            resource_type: hob::EFI_RESOURCE_SYSTEM_MEMORY,
            resource_attribute: hob::EFI_RESOURCE_ATTRIBUTE_PRESENT,
            physical_start: 0,
            resource_length: 0x0123456789abcdef,
        };
        v1.header.r#type = hob::RESOURCE_DESCRIPTOR2;
        v1.header.length = size_of::<hob::ResourceDescriptorV2>() as u16;
        let resource_desc2_hob = hob::ResourceDescriptorV2 { v1, attributes: 8 };

        let data = [1_u8, 2, 3, 4, 5, 6, 7, 8];
        let guid_hob = (
            hob::GuidHob {
                header: hob::header::Hob {
                    r#type: hob::GUID_EXTENSION,
                    length: (size_of::<hob::GuidHob>() + data.len()) as u16,
                    reserved: 0,
                },
                name: r_efi::efi::Guid::from_fields(1, 2, 3, 4, 5, &[6, 7, 8, 9, 10, 11]),
            },
            data,
        );

        let header = hob::header::Hob { r#type: hob::FV, length: size_of::<hob::FirmwareVolume>() as u16, reserved: 0 };
        let fv_hob = hob::FirmwareVolume { header, base_address: 0, length: 0x0123456789abcdef };

        let header = hob::header::Hob { r#type: hob::CPU, length: size_of::<hob::Cpu>() as u16, reserved: 0 };
        let cpu_hob = hob::Cpu { header, size_of_memory_space: 0, size_of_io_space: 0, reserved: [0; 6] };

        let mut hob_list = HobList::default();
        hob_list.push(Hob::Handoff(&handoff_hob));
        hob_list.push(Hob::ResourceDescriptor(&resource_desc_hob));
        hob_list.push(Hob::MemoryAllocation(&memory_alloc_hob));
        hob_list.push(Hob::ResourceDescriptor(&resource_desc_hob));
        hob_list.push(Hob::ResourceDescriptorV2(&resource_desc2_hob));
        hob_list.push(Hob::GuidHob(&guid_hob.0, &data));
        hob_list.push(Hob::FirmwareVolume(&fv_hob));
        hob_list.push(Hob::Cpu(&cpu_hob));

        let serializable_list = HobListSerDe::from(&hob_list);
        let json = to_string_pretty(&serializable_list).expect("Serialization failed");

        assert!(json.contains(r#""type": "handoff""#), "Handoff HOB missing");
        assert!(json.contains(r#""memory_top": 3735928559"#), "Memory top value incorrect");
        assert!(json.contains(r#""memory_bottom": 3735929054"#), "Memory bottom value incorrect");

        assert!(json.contains(r#""type": "memory_allocation""#), "Memory Allocation HOB missing");
        assert!(json.contains(r#""memory_length": 81985529216486895"#), "Memory length incorrect");

        assert!(json.contains(r#""type": "resource_descriptor""#), "Resource Descriptor HOB missing");
        assert!(json.contains(r#""physical_start": 0"#), "Physical start missing");

        assert!(json.contains(r#""type": "resource_descriptor_v2""#), "Resource Descriptor V2 missing");
        assert!(json.contains(r#""attributes": 8"#), "Resource Descriptor V2 attributes incorrect");

        assert!(json.contains(r#""type": "guid_extension""#), "GUID Extension HOB missing");

        assert!(json.contains(r#""type": "firmware_volume""#), "Firmware Volume HOB missing");
        assert!(json.contains(r#""length": 81985529216486895"#), "Firmware Volume length incorrect");

        assert!(json.contains(r#""type": "cpu""#), "CPU HOB missing");
        assert!(json.contains(r#""size_of_memory_space": 0"#), "CPU memory space size incorrect");
        assert!(json.contains(r#""size_of_io_space": 0"#), "CPU IO space size incorrect");
    }
}
