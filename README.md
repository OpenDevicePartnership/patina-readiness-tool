<!-- markdownlint-disable MD013 : Disable line limit.-->
# **DXE Readiness Capture/Validation Tool**

The workspace consists of two packages:

1. **DXE Readiness Capture** – An EFI application
2. **DXE Readiness Validator** – A standard Rust binary

## **Quick Start (Prebuilt Binaries)**

Every release ships prebuilt binaries.

### **1. Download the Release**

1. Go to the [Releases page](https://github.com/OpenDevicePartnership/patina-readiness-tool/releases).
2. Download `patina_readiness_tool.zip` from the latest release.
3. Extract it. The archive is laid out as follows:

```txt
capture/
├── x86_64/
│   ├── intel_dxe_readiness_capture.efi      # DXE-phase capture (Intel LNL/PTL)
│   ├── qemu_dxe_readiness_capture.efi       # DXE-phase capture (QEMU Q35)
│   └── uefishell_dxe_readiness_capture.efi  # UEFI Shell capture (any platform)
└── aarch64/
    ├── qemu_dxe_readiness_capture.efi       # DXE-phase capture (QEMU)
    ├── arm_virt_dxe_readiness_capture.efi   # DXE-phase capture (ARM virt)
    └── uefishell_dxe_readiness_capture.efi  # UEFI Shell capture (any platform)
validator/
├── windows/
│   ├── dxe_readiness_validator_x86_64.exe
│   └── dxe_readiness_validator_aarch64.exe
└── linux/
    └── dxe_readiness_validator_x86_64
```

### **2. Capture the Platform State**

Pick **one** of the two capture methods below. Both produce the same JSON
schema, but they observe the platform at different points in the boot flow.

**Option A is strongly preferred.** It captures the system state exactly as
Patina would receive it at DXE Core handoff, whereas that state can change
drastically by the time the UEFI Shell is reached. Use Option B only when
modifying the firmware image is not practical.

#### **Option A: DXE-Phase Capture (Preferred)**

The `*_dxe_readiness_capture.efi` binaries replace the DXE Core in the firmware
image, so PEI hands the HOB list directly to the tool.

1. Choose the binary matching your platform and architecture from
   `capture/<arch>/`.
2. Replace the DXE Core in your firmware volume with this binary and flash/boot
   the resulting image. For QEMU, see [Launching QEMU](#launching-qemu).
3. Capture the serial output of the boot. The tool prints a banner
   (`Dxe Readiness Capture Tool`) followed by the JSON capture, then dead loops.
4. Save the JSON portion of the serial log to a file, e.g. `capture.json`
   (strip the log prefixes and any non-JSON lines - the file must start with `{`
   and end with `}`).

If your platform is not listed, it only needs a small platform-specific binary
that configures a logger. See the [Platform Onboarding
Guide](docs/capture/platform_onboarding_guide.md).

#### **Option B: UEFI Shell Capture**

`uefishell_dxe_readiness_capture.efi` requires no firmware modification.

1. Copy `capture/<arch>/uefishell_dxe_readiness_capture.efi` to a FAT-formatted
   USB drive or the EFI System Partition.
2. Boot the target platform to the UEFI Shell.
3. Select the volume and run the application, redirecting the output to a file:

   ```sh
   fs0:
   uefishell_dxe_readiness_capture.efi capture.json
   ```

4. Copy `capture.json` back to your development machine and remove any non-JSON
   lines (banner/log lines) from the top and bottom of the file.

### **3. Validate the Capture**

Run the validator for your host OS against the captured JSON:

```sh
# Windows
dxe_readiness_validator_x86_64.exe -f capture.json

# Linux
chmod +x dxe_readiness_validator_x86_64
./dxe_readiness_validator_x86_64 -f capture.json
```

The validator prints a report of every requirement that passed or was violated.
See [Sample Validation Report](#sample-validation-report) below and the
[validation list](docs/validator/validations.md) for what is checked.

## **Building the Packages**

Running `cargo make build` compiles both packages for all supported
architectures and targets.

| Target               | x86_64                                                               | AArch64                                                               |
| -------------------- | -------------------------------------------------------------------- | --------------------------------------------------------------------- |
| **UEFI Dxe Phase**   | target\x86_64-unknown-uefi\debug\qemu_dxe_readiness_capture.efi      | target\aarch64-unknown-uefi\debug\qemu_dxe_readiness_capture.efi      |
| **UEFI Shell Phase** | target\x86_64-unknown-uefi\debug\uefishell_dxe_readiness_capture.efi | target\aarch64-unknown-uefi\debug\uefishell_dxe_readiness_capture.efi |
| **Windows**          | target\x86_64-pc-windows-msvc\debug\dxe_readiness_validater.exe      | target\aarch64-pc-windows-msvc\debug\dxe_readiness_validater.exe      |
| **Linux**            | target\x86_64-unknown-linux-gnu\debug\dxe_readiness_validater        | target\aarch64-unknown-linux-gnu\debug\dxe_readiness_validater        |

### **Supported Hardware Platforms**

The `cargo make build` command, along with the QEMU-based UEFI
`qemu_dxe_readiness_capture.efi`, also builds the following hardware
platform-specific binaries:

| Platform                            | Binary                                                             |
| ----------------------------------- | ------------------------------------------------------------------ |
| **Intel Lunar Lake & Panther Lake** | `target\x86_64-unknown-uefi\debug\intel_dxe_readiness_capture.efi` |

## **Running Tests**

Executing `cargo make test` builds and runs the test binaries for both packages,
matching the host architecture(x86_64-pc-windows-msvc|aarch64-pc-windows-msvc).

## **Launching QEMU**

To launch the application in QEMU, navigate to:

```sh
C:\r\patina-qemu
```

Then, run the following command:

```sh
python .\build_and_run_rust_binary.py --fw-patch-repo C:\r\fw_rust_patcher --custom-efi C:\r\platform_handoff_validation_tool\target\x86_64-unknown-uefi\debug\qemu_dxe_readiness_capture.efi
```

## **Running Validator**

To run the validator application with the appropriate JSON file captured during
the capture phase, use the command below:

```sh
cargo make run -- -f dxe_readiness_validator\src\tests\data\q35_capture.json
or
target\debug\dxe_readiness_validator.exe -f dxe_readiness_validator\src\tests\data\q35_capture.json
```

### Sample Validation Report

![Validation Report](docs/images/validation_report.png)

## Contributing to DXE Readiness Capture/Validation Tool

### Getting Started

To build and run the project locally, you'll need:

- Latest stable Rust version
- [`cargo make`](https://crates.io/crates/cargo-make)
  - If you would like to patch and run the tool in QEMU: Python 3

To build and run with QEMU locally, you will also need to clone
[patina-qemu](https://github.com/OpenDevicePartnership/patina-qemu/).

To build for your specific architecture, see the `Makefile.toml` for specific build options.

### Project Outline

The DXE Readiness tool is split into two main crates:

- `dxe_readiness_capture`: Code to serialize pre-DXE structs for validation.
  This crate runs in a `no_std` environment with a custom logger and allocator.
- `dxe_readiness_validator`: Code to deserialize previously serialized pre-DXE
  structs and to validate that they meet platform requirements. This crate runs
  in standard Rust.

#### `dxe_readiness_capture`

This crate builds `libdxe_readiness_capture-xxx.rlib` library, while the
`src/bin/*.rs` files produce the corresponding `.efi` binaries that run in a
`no_std` environment. Each `src/bin/*.rs` file includes the platform specific
logger configuration.

Any new structs defined here must implement `Serialize` and `Deserialize` to be
present during the validation phase. An example output can be viewed in
[q35_capture.json](dxe_readiness_validator/src/tests/data/q35_capture.json).

To onboard a new hardware platform, see the [Platform Onboarding
Guide](docs/capture/platform_onboarding_guide.md).

#### `dxe_readiness_validator`

This crate validates the HOBs and FVs and provides a user CLI to run
validations. Unlike the capture crate, this crate runs in standard Rust and
produces a `.exe`.

Validations are based on on agreed-upon
[requirements](https://github.com/OpenDevicePartnership/patina/issues/222).
Before contributing any new validations, make sure to document and get approval
for your new requirement.

### Code Style

- Use the provided `rustfmt` file for general formatting guidelines.
- Run `cargo make clippy` to catch common errors before submitting a PR.

### Testing

All contributions should include unit tests.
Before submitting code:

- Run all tests locally:

  ```bash
  cargo make test
  ```

- Run the tool on QEMU
- Optionally, test on physical hardware

#### HOB Validator Example

Below is an example unit test for the HOB validator. Tests should validate both
error and success scenarios. Also note the use of common constructors, such as
`create_memory_hob`.

```rust
#[test]
fn test_pagezero() {
    let page_zero_mem_hob = create_memory_hob("test".to_string(), 0, 0x10, 1);
    let mem_hob = create_memory_hob("test2".to_string(), UEFI_PAGE_SIZE as u64 + 1, 0x100, 1);
    let hob_list = vec![mem_hob.clone()];
    let data = DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] };
    let mut app = ValidationApp::new_with_data(data);
    let mut validation_report = ValidationReport::new();
    let res = app.validate_page0_memory_allocation(&mut validation_report);
    assert!(res.is_ok());
    assert_eq!(validation_report.violation_count(), 0);

    let hob_list = vec![mem_hob.clone(), page_zero_mem_hob];
    app = ValidationApp::new_with_data(DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] });
    let mut validation_report = ValidationReport::new();
    let res = app.validate_page0_memory_allocation(&mut validation_report);
    assert!(res.is_ok());
    assert_ne!(validation_report.violation_count(), 0);
}
```

#### Validation Requirements

If your contribution involves new validation requirements, follow these steps:

1. Review the current list of [validation
   requirements](https://github.com/OpenDevicePartnership/patina/issues/222)
   on Github.
2. If your requirement is not listed, add it to the discussion in the Github
   issue. Include justification on why the new requirement is necessary.
3. Implement the validation logic and corresponding unit tests.
4. Document the requirement (see below).

For an example of how to add a new requirement in code, view [this
PR](https://dev.azure.com/microsoft/MsUEFI/_git/platform_handoff_validation_tool/pullrequest/12866229),
which adds a new requirement to HOB validation.

For non-requirement work (e.g., JSON capture tooling), you can directly raise a
PR.
