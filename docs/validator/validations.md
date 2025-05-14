# Validations

This serves as a living document to track and evolve the set of validations implemented in the platform validation tool.

## HOB Validations

| Validation Area | Description |
|----------------|-------------|
| **HOB Internal Consistency** | Validate that V2 Resource Descriptor HOBs describe all ranges covered by V1 HOBs. |
| **No Overlapping Memory Ranges** | Ensure HOB memory ranges described in Resource Descriptor HOBs do not overlap. |
| **Memory Protection HOB** | The GUID extension HOB marked by `gDxeMemoryProtectionSettingsGuid` must exist. |
| **Page 0 Allocation** | Page 0 should not be covered by any memory allocation HOB. |
| **EFI_MEMORY_UCE in RD HOB v2** | `EFI_MEMORY_UCE` should not be set as a cacheability attribute in V2 Resource Descriptor HOBs.|

## Firmware Volume (FV) Validations

| Validation Area | Description |
|----------------|-------------|
| **Unsupported FFS File Types** | Combined drivers are not supported and should be flagged. |
| **LZMA Compression** | Do not use LZMA-compressed sections. |
| **SMM Mode** | Use Standalone MM, not Traditional SMM. |
| **A Priori Driver Usage** | A Priori driver execution should not be used. |
