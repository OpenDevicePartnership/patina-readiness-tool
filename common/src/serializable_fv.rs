use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use mu_pi::fw_fs::FirmwareVolume;
use mu_pi::hob::EfiPhysicalAddress;
use r_efi::efi;
use serde::{Deserialize, Serialize};

use crate::format_guid;

// This is the serialized version of the FV list.
#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareVolumeSerDe {
    pub fv_name: String,
    pub fv_length: usize,
    pub fv_base_address: EfiPhysicalAddress,
    pub fv_attributes: u32,
    pub files: Vec<FirmwareFileSerDe>,
}

// This is the serialized version of the file list.
#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareFileSerDe {
    pub name: String, // GUID
    pub file_type: String,
    pub length: usize,
    // pub base_address: EfiPhysicalAddress,
    pub attributes: u32,
    pub sections: Vec<FirmwareSectionSerDe>,
}

// This is the serialized version of the section list.
#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareSectionSerDe {
    pub section_type: String,
    pub length: usize,
    // pub attributes: u32,
}

impl From<FirmwareVolume<'_>> for FirmwareVolumeSerDe {
    fn from(fv: FirmwareVolume) -> Self {
        // Get the FV name, length, base address, and attributes
        let fv_name = format_guid(fv.fv_name().unwrap_or(efi::Guid::from_bytes(&[0; 16])));
        let fv_length = fv.size() as usize;
        let fv_attributes = fv.attributes();
        let files = fv
            .file_iter()
            .filter_map(|file| {
                // Iterate over the Files in the FV
                let Ok(file) = file else {
                    return None;
                };
                let file_name = format_guid(file.name());
                let file_length = file.size() as usize;
                let file_attributes = file.attributes_raw() as u32;
                let file_type =
                    file.file_type().map(|ft| format!("{:#x?}", ft)).unwrap_or_else(|| "Invalid".to_string());
                let sections = file
                    .section_iter()
                    .filter_map(|section| {
                        // Iterate over the section in file
                        let Ok(section) = section else {
                            return None;
                        };
                        let section_length = section.section_size();
                        let section_type = section
                            .section_type()
                            .map(|st| format!("{:#x?}", st))
                            .unwrap_or_else(|| "Invalid".to_string());
                        Some(FirmwareSectionSerDe { section_type, length: section_length })
                    })
                    .collect::<Vec<_>>();

                Some(FirmwareFileSerDe {
                    name: file_name,
                    length: file_length,
                    attributes: file_attributes,
                    sections,
                    file_type,
                })
            })
            .collect::<Vec<_>>();

        FirmwareVolumeSerDe { fv_name, fv_length, fv_attributes, files, fv_base_address: 0 /* filed outside */ }
    }
}
