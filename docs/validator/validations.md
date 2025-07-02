# Validations

This serves as a living document to track and evolve the set of validations implemented in the DXE readiness tool.

<!-- markdownlint-disable MD013 : Disable line limit.-->
## HOB Validations

| Validation Kind                              | Description                                                                                              |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Inconsistent Memory Attributes**           | Identifies V1 and V2 HOBs describing the same range(s) with inconsistent memory attributes (prohibited). |
| **Overlapping Memory Ranges**                | Identifies HOBs with overlapping memory ranges (prohibited).                                             |
| **Page Zero Memory Allocated**               | Identifies HOBs that describe page zero memory allocation (prohibited).                                  |
| **V1 Memory Range Not Contained In V2**      | Identifies V1 HOBs with memory ranges not covered by V2 (prohibited).                                    |
| **V2 Contains UCE Attribute**                | Identifies V2 HOBs that use the prohibited `EFI_MEMORY_UCE` cacheability attribute.                      |
| **V2 Missing Valid Cacheability Attributes** | Identifies V2 HOBs have valid cacheability attribute set(at most one).                                   |
| **V2 Invalid IO Cacheability Attributes**    | Identifies V2 HOBs for IO resource types with non-zero attributes. Zero is expected at this time.        |

## Firmware Volume (FV) Validations

| Validation Kind               | Description                                                                                         |
| ----------------------------- | --------------------------------------------------------------------------------------------------- |
| **Combined Drivers Present**  | Firmware volumes must not contain combined drivers (prohibited).                                    |
| **Lzma Compressed Sections**  | Firmware volumes must not contain LZMA-compressed sections (prohibited).                            |
| **Prohibited Apriori File**   | Firmware volumes must not contain an A Priori file (prohibited).                                    |
| **Uses Traditional Smm**      | Firmware volumes must not contain traditional SMM (prohibited).                                     |
| **Invalid Section Alignment** | PE images in firmware volumes must have section alignment that is a positive multiple of page size. |
