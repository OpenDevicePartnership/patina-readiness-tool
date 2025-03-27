# **DXE Readiness Capture/Validation Tool**

The workspace consists of two packages:

1. **DXE Readiness Capture** – An EFI application
2. **DXE Readiness Validator** – A standard Rust binary

## **Building the Packages**

Running `cargo make build` compiles both packages for all supported
architectures and targets.

| Target   | x86_64                                                          | AArch64                                                          |
| -------- | --------------------------------------------------------------- | ---------------------------------------------------------------- |
| **UEFI** | target\x86_64-unknown-uefi\debug\dxe_readiness_capture.efi      | target\aarch64-unknown-uefi\debug\dxe_readiness_capture.efi      |
| **Std**  | target\x86_64-pc-windows-msvc\debug\dxe_readiness_validater.exe | target\aarch64-pc-windows-msvc\debug\dxe_readiness_validater.exe |

## **Running Tests**

Executing `cargo make test` builds and runs the test binaries for both packages,
matching the host architecture(x86_64-pc-windows-msvc|aarch64-pc-windows-msvc).

| Target  | x86_64                                                         | AArch64                                                                |
| ------- | -------------------------------------------------------------- | ---------------------------------------------------------------------- |
| **Std** | target\debug\deps\dxe_readiness_capture-d1a2f334330e0b78.exe   | dxe_readiness_capture-d1a2f334330e0b78.exe (aarch64-pc-windows-msvc)   |
| **Std** | target\debug\deps\dxe_readiness_validater-0217a28b86858ac9.exe | dxe_readiness_validator-d1a2f334330e0b78.exe (aarch64-pc-windows-msvc) |

## **Launching QEMU**

To launch the application in QEMU, navigate to:

```sh
C:\r\UefiRust
```

Then, run the following command:

```sh
python .\build_and_run_rust_binary.py --fw-patch-repo C:\r\fw_rust_patcher --custom-efi C:\r\platform_handoff_validation_tool\target\x86_64-unknown-uefi\debug\dxe_readiness_capture.efi
```
