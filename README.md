# Dxe Readiness Capture/Validation Tool

## Build Tool

`cargo make build`

|      | X64                         | AArch64                     |
| ---- | --------------------------- | --------------------------- |
| UEFI | dxe_readiness_capture.efi   | dxe_readiness_capture.efi   |
| Std  | dxe_readiness_validater.exe | dxe_readiness_validater.exe |


## Launch QEMU

python .\build_and_run_dxe_readiness_validation_tool.py --uefi-rust-repo C:\r\UefiRust2 --qemu-rust-bin-repo  C:\r\dxe_readiness_validation_tool --fw-patch-repo C:\r\fw_rust_patcher
