# Contributing to DXE Readiness Capture/Validation Tool

## Getting Started

To build and run the project locally, you'll need:

- Latest stable Rust version
- [`cargo make`](https://crates.io/crates/cargo-make/0.3.54)
- Python 3 (for patching and running in QEMU)

To build and run with QEMU locally, you will also need to clone [UefiRust](https://dev.azure.com/microsoft/MsUEFI/_git/UefiRust).

To build for your specific architecture, see the `Makefile.toml` for specific build options.

## Project Outline

The DXE Readiness tool is split into three main crates:

- `common`: Common functionality between the capture and validation phases.
  These include shared structs for serialization and utility functions.
- `dxe_readiness_capture`: Code to serialize pre-DXE structs for validation.
  This crate runs in a `no_std` environment with a custom logger and allocator.
- `dxe_readiness_validator`: Code to deserialize previously serialized pre-DXE
  structs and to validate that they meet platform requirements. This crate runs
  in standard Rust.

### `common`

This crate is mainly for shared serialization structs between the capture and
validation phases. There are two main structs, `SerializableHob` (representing
entries in the HOB list) and `SerializableFV` (representing FV sections). These
structs mirror their PI spec representations, which can be found in
[`mu_rust_pi`](https://github.com/microsoft/mu_rust_pi).

Any changes to these serializable structs should be validated for compatibility
for both capture and validation.

### `dxe_readiness_capture`

This crate builds `libdxe_readiness_capture-xxx.rlib` library, while the
`src/bin/*.rs` files produce the corresponding `.efi` binaries that run in a
`no_std` environment. Each `src/bin/*.rs` file includes the platform specific
logger configuration.

Any new structs defined here must implement `Serialize` and `Deserialize` to be
present during the validation phase. An example output can be viewed in
[q35_capture.json](dxe_readiness_validator/src/tests/data/q35_capture.json).

### `dxe_readiness_validator`

This crate validates the HOBs and FVs and provides a user CLI to run
validations. Unlike the capture crate, this crate runs in standard Rust and
produces a `.exe`.

Validations are based on on agreed-upon
[requirements](https://github.com/OpenDevicePartnership/uefi-dxe-core/issues/222).
Before contributing any new validations, make sure to document and get approval
for your new requirement.

## Code Style

- Use the provided `rustfmt` file for general formatting guidelines.
- Run `cargo make clippy` to catch common errors before submitting a PR.

## Testing

All contributions should include unit tests.
Before submitting code:

- Run all tests locally:

  ```bash
  cargo make test
  ```

- Run the tool on QEMU
- Optionally, test on physical hardware

### HOB Validator Example

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
    let res = app.check_page0();
    assert!(res.is_ok());
    assert!(app.validation_report.is_empty());

    let hob_list = vec![mem_hob.clone(), page_zero_mem_hob];
    app = ValidationApp::new_with_data(DxeReadinessCaptureSerDe { hob_list, fv_list: vec![] });
    let res = app.check_page0();
    assert!(res.is_ok());
    assert!(!app.validation_report.is_empty());
}
```

## Submitting Changes

1. Create a branch using the format: `personal/<your-alias>/<description-of-pr>`.
2. Push your changes and create a Pull Request (PR).
3. At least one reviewer must approve the final PR for it to be merged.
4. Merge into `main` after approval.

### Validation Requirements

If your contribution involves new validation requirements, follow these steps:

1. Review the current list of [validation
   requirements](https://github.com/OpenDevicePartnership/uefi-dxe-core/issues/222)
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
