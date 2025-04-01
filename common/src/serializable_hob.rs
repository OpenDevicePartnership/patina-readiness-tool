// basically you can't deserialize references (complex explanation but if this assumption is wrong then we can rework the code)
// but the key point is that we don't actually need to deserialize into mu_pi::hob structs, we can deserialize kinda whatever we want
// so here i cut out some information from mu_pi (ie headers)
// and also get rid of the references so we can easily derive deserialization
// if the assumptions are wrong (assumption #0: deserialization of references not possible) then we can rework this
// but the point is that these structs are local to this tool and there's no need to reference the more complex PI spec structs
// for internal validation purposes

use mu_pi::hob::{Hob, HobList};
use r_efi::efi::Guid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeserializableHobList {
    pub hobs: Vec<DeserializableHob>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeserializableHob {
    MemoryAllocation {
        alloc_descriptor: DeserializableMemAllocDescriptor,
    },
    ResourceDescriptor {
        owner: String, // GUID as a string
        resource_type: u32,
        resource_attribute: u32,
        physical_start: u64,
        resource_length: u64,
    },
    UnknownHob,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeserializableMemAllocDescriptor {
    // Fields you care about from the original alloc_descriptor
    name: String, // e.g., GUID as a string
    memory_base_address: u64,
    memory_length: u64,
    memory_type: u32,
    reserved: [u8; 4],
}

fn format_guid(g: Guid) -> String {
    // we need this because refi::Guid has private fields
    // and we can't make it derive Serialize (can't modify efi::Guid directly)
    let (time_low, time_mid, time_hi_and_version, clk_seq_hi_res, clk_seq_low, node) = g.as_fields();
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        time_low,
        time_mid,
        time_hi_and_version,
        clk_seq_hi_res,
        clk_seq_low,
        node[0],
        node[1],
        node[2],
        node[3],
        node[4],
        node[5]
    )
}

impl From<&HobList<'_>> for DeserializableHobList {
    fn from(hob_list: &HobList) -> Self {
        DeserializableHobList { hobs: hob_list.iter().map(DeserializableHob::from).collect() }
    }
}

// TODO: add more hob types
impl From<&Hob<'_>> for DeserializableHob {
    fn from(hob: &Hob) -> Self {
        match hob {
            Hob::MemoryAllocation(mem_alloc) => Self::MemoryAllocation {
                alloc_descriptor: DeserializableMemAllocDescriptor {
                    name: format!("{}", format_guid(mem_alloc.alloc_descriptor.name)), // Convert GUID to string
                    memory_base_address: mem_alloc.alloc_descriptor.memory_base_address,
                    memory_length: mem_alloc.alloc_descriptor.memory_length,
                    memory_type: mem_alloc.alloc_descriptor.memory_type,
                    reserved: mem_alloc.alloc_descriptor.reserved,
                },
            },
            Hob::ResourceDescriptor(res) => Self::ResourceDescriptor {
                owner: format!("{}", format_guid(res.owner)),
                resource_type: res.resource_type,
                resource_attribute: res.resource_attribute,
                physical_start: res.physical_start,
                resource_length: res.resource_length,
            },
            _ => Self::UnknownHob {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mu_pi::hob;
    use serde_json::{from_str, to_string_pretty};

    // TODO: fix tests to have asserts instead of just printing

    #[test]
    fn test_hoblist_deserialization() {
        let json_data = r#"
        {
            "hobs": [
                {
                    "type": "memory_allocation",
                    "alloc_descriptor": {
                        "name": "123e4567-e89b-12d3-a456-426614174000",
                        "memory_base_address": 0,
                        "memory_length": 12345678,
                        "memory_type": 0,
                        "reserved": [0, 0, 0, 0]
                    }
                },
                {
                    "type": "resource_descriptor",
                    "owner": "123e4567-e89b-12d3-a456-426614174000",
                    "resource_type": 1,
                    "resource_attribute": 2,
                    "physical_start": 4096,
                    "resource_length": 8192
                }
            ]
        }
        "#;

        let owned_hob_list: DeserializableHobList = from_str(json_data).expect("Failed to deserialize");

        println!("{:?}", owned_hob_list.hobs[0]);
        println!("{:?}", owned_hob_list.hobs[1]);
    }

    #[test]
    fn test_hoblist_serialization() {
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

        let mut hob_list = HobList::default();
        hob_list.push(Hob::ResourceDescriptor(&resource_desc_hob));
        hob_list.push(Hob::MemoryAllocation(&memory_alloc_hob));

        let serializable_list = DeserializableHobList::from(&hob_list);
        let json = to_string_pretty(&serializable_list).expect("Serialization failed");

        println!("{}", json);
    }
}
