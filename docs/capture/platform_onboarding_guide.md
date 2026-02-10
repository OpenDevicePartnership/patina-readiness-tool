# Platform Onboarding Guide - DXE Readiness Capture Tool

## Overview

The DXE Readiness Capture Tool captures Hand-Off Block (HOB) data and Firmware
Volume (FV) information from a UEFI platform during boot. Each supported
platform is represented as a separate **binary target** under
`dxe_readiness_capture/src/bin/`. This guide provides step-by-step instructions
for onboarding a new platform.

---

## Architecture at a Glance

```txt
dxe_readiness_capture/
├── Cargo.toml                  # Package manifest (binary targets declared here)
├── src/
│   ├── lib.rs                  # Shared library - exports `core_start()`
│   ├── allocator.rs            # Heap allocator for non-UEFI-shell binaries
│   ├── capture.rs              # Core capture logic (HOBs & FVs)
│   ├── capture/
│   │   ├── fv.rs               # Firmware Volume capture
│   │   └── hob.rs              # HOB capture
│   └── bin/
│       ├── intel_dxe_readiness_capture.rs      # Intel (Lunar Lake / Panther Lake)
│       ├── qemu_dxe_readiness_capture.rs       # QEMU virtual platform
│       └── uefishell_dxe_readiness_capture.rs  # Generic UEFI Shell binary
```

Every platform binary follows the same contract:

1. **Initialize a logger** (serial or provided outside of patina libraries).
2. **Call `core_start(physical_hob_list)`** from the shared library.
3. **Enter a dead loop** (or return success for UEFI Shell apps).

All heavy lifting (HOB discovery, FV parsing, JSON serialization) is handled by
the shared library - a platform binary only needs to supply **logging** and the
**entry point**.

---

## Step-by-Step: Adding a New Platform Binary

### 1. Create the Binary Source File

Add a new file under `dxe_readiness_capture/src/bin/` with the naming convention:

```cmd
<platform>_dxe_readiness_capture.rs
```

For example: `sample_dxe_readiness_capture.rs`.

### 2. Follow the Required File Structure

Every platform binary **must** follow this skeleton:

```rust
//! Dxe Readiness Capture Tool - <Platform Description>
//!

#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "<ARCH>"))] {
        // --- Platform-specific imports ---
        use log::LevelFilter;
        use core::ffi::c_void;
        use dxe_readiness_capture::core_start;
        // ... logger / serial imports ...

        // --- Logger setup ---
        fn init_logger() {
            // Initialize the platform's serial logger or any other non-serial
            // logger that implements Rust's `log` crate `Log` trait.
        }

        // --- EFI entry point ---
        #[unsafe(export_name = "efi_main")]
        pub extern "efiapi" fn _start(physical_hob_list: *const c_void) -> ! {
            init_logger();
            core_start(physical_hob_list);
            log::info!("Dead Loop");
            loop {}
        }
    } else {
        fn main() {}
    }
}
```

> **Key points:**
>
> - The `#![cfg_attr(...)]` attributes ensure `no_std` / `no_main` only apply when compiling for UEFI targets.
> - Wrap all UEFI-specific code inside `cfg_if::cfg_if!` gated on `target_os = "uefi"` (and optionally `target_arch`).
> - The `else` branch **must** contain a trivial `fn main() {}` so the file
>   compiles as a normal binary during `cargo test` and other host-side
>   operations.
> - Your entry point receives `physical_hob_list` directly as a parameter since
>   it is replacing the DXE core which would be given the HOB list.

### 3. Implement Platform-Specific Logger

The logger is the primary platform-specific component. Use one of the serial
devices from the `patina` crate or implement your own.

**Common UART options from `patina`:**

| UART Type          | Crate Path                        | Typical Use                                    |
|--------------------|-----------------------------------|------------------------------------------------|
| `Uart16550` (MMIO) | `patina::serial::uart::Uart16550` | x86_64 platforms with MMIO-mapped 16550 UART   |
| `Uart16550` (IO)   | `patina::serial::uart::Uart16550` | x86_64 platforms with IO-port-based 16550 UART |
| `UartPl011`        | `patina::serial::uart::UartPl011` | AArch64 platforms (ARM PL011 UART)             |

**Logger initialization pattern:**

```rust
use patina::log::SerialLogger;
use patina::{log::Format, serial::uart::Uart16550}; // or UartPl011

// Option A: Static const logger (preferred when address is known at compile time)
static LOGGER: SerialLogger<Uart16550> = SerialLogger::new(
    Format::Standard,
    &[],
    log::LevelFilter::Trace,
    Uart16550::Io { base: 0x3F8 },
);

fn init_logger() {
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
}

// Option B: Runtime-initialized logger (when detection logic is needed)
static mut LOGGER: Option<SerialLogger<Uart16550>> = None;

#[allow(static_mut_refs)]
fn init_logger() {
    let logger_ref: &'static SerialLogger<'static, Uart16550> = unsafe {
        LOGGER = Some(/* construct logger with runtime detection */);
        LOGGER.as_ref().unwrap()
    };
    let _ = log::set_logger(logger_ref).map(|()| log::set_max_level(LevelFilter::Info));
}
```

> **Important:** Set the max log level to `Info`. Higher levels (e.g., `Trace`,
> `Debug`) may trigger verbose output from dependency crates like `goblin`.

### 4. Handle Multi-Architecture Support

If you intend to build readiness tool supports multiple architectures, use
`cfg_if` branching (see `qemu_dxe_readiness_capture.rs` for an example):

```rust
cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
        // AArch64-specific logger and entry point
    } else if #[cfg(all(target_os = "uefi", target_arch = "x86_64"))] {
        // x86_64-specific logger and entry point
    } else {
        fn main() {}
    }
}
```

### 5. Register the Binary in `Cargo.toml`

No explicit `[[bin]]` entry is required - Cargo auto-discovers files in
`src/bin/`. However, if you add a new **feature** for your platform, declare it
in [dxe_readiness_capture/Cargo.toml](../dxe_readiness_capture/Cargo.toml):

```toml
[features]
default = []
x64 = []
aarch64 = []
uefishell = []
sample = []          # <-- Add your platform feature if needed
```

### 6. Add Build Tasks in `Makefile.toml`

Add a build task for your platform in the root [Makefile.toml](../Makefile.toml). Follow the existing patterns:

**For a DXE driver binary (no_std):**

```toml
[tasks.build-sample]
description = "Builds the sample DXE Readiness Capture UEFI binary."
env = { RUSTFLAGS = "-C force-unwind-tables -C link-arg=/base:0x0 -C link-arg=/subsystem:efi_boot_service_driver -C link-arg=/PDBALTPATH:sample_dxe_readiness_capture.pdb" }
command = "cargo"
args = ["build", "@@split(CAPTURE_BIN_FLAGS, )", "@@split(X86_64_UEFI_TARGET, )", "--bin", "sample_dxe_readiness_capture", "${@}"]
```

Then add the new task to the `[tasks.build]` dependencies list:

```toml
[tasks.build]
dependencies = [
    # ... existing tasks ...
    "build-sample",       # <-- Add here
]
```

### 7. Key `RUSTFLAGS` Reference

| Flag                                             | Purpose                                               |
|--------------------------------------------------|-------------------------------------------------------|
| `-C force-unwind-tables`                         | Required for stack trace support in UEFI              |
| `-C link-arg=/base:0x0`                          | Sets the binary base address to 0 (standard for UEFI) |
| `-C link-arg=/subsystem:efi_boot_service_driver` | Marks binary as a DXE/boot service driver             |
| `-C link-arg=/subsystem:efi_application`         | Marks binary as a UEFI Shell application              |
| `-C link-arg=/PDBALTPATH:<name>.pdb`             | Sets the PDB debug symbol file path                   |

### 8. Build Target Reference

| Target Triple             | Architecture | Use                                       |
|---------------------------|--------------|-------------------------------------------|
| `x86_64-unknown-uefi`     | x86_64       | UEFI binaries (DXE drivers or shell apps) |
| `aarch64-unknown-uefi`    | AArch64      | UEFI binaries (DXE drivers or shell apps) |
| `x86_64-pc-windows-msvc`  | x86_64       | Host validation binary (Windows)          |
| `aarch64-pc-windows-msvc` | AArch64      | Host validation binary (Windows)          |

---

## Checklist

Use this checklist when onboarding a new platform:

- [ ] Create `dxe_readiness_capture/src/bin/<platform>_dxe_readiness_capture.rs`
- [ ] Add `#![cfg_attr(target_os = "uefi", no_std)]` and `#![cfg_attr(target_os = "uefi", no_main)]`
- [ ] Wrap all UEFI code in `cfg_if::cfg_if!` with proper target gates
- [ ] Provide a fallback `fn main() {}` in the `else` branch
- [ ] Implement `init_logger()` with platform-specific serial/UART configuration
- [ ] Call `core_start(physical_hob_list)` from the entry point
- [ ] Add a build task in `Makefile.toml` with the correct `RUSTFLAGS` and target
- [ ] Add the new task to `[tasks.build]` dependencies
- [ ] Add any new features to `dxe_readiness_capture/Cargo.toml` if needed
- [ ] Verify the binary builds: `cargo make build-<platform>`
- [ ] Verify tests still pass: `cargo make test`
- [ ] Run `cargo make clippy` and `cargo make fmt` to ensure code quality
- [ ] Update `cspell.yml` if new platform-specific terms are introduced

---

## Existing Platform Reference

| Platform        | File                                 | Arch             | Type       | Logger                                       |
|-----------------|--------------------------------------|------------------|------------|----------------------------------------------|
| Intel (LNL/PTL) | `intel_dxe_readiness_capture.rs`     | x86_64           | DXE Driver | `Uart16550` (MMIO/IO with runtime detection) |
| QEMU (x86_64)   | `qemu_dxe_readiness_capture.rs`      | x86_64           | DXE Driver | `Uart16550` IO at `0x402`                    |
| QEMU (AArch64)  | `qemu_dxe_readiness_capture.rs`      | AArch64          | DXE Driver | `UartPl011` at `0x6000_0000`                 |
| UEFI Shell      | `uefishell_dxe_readiness_capture.rs` | x86_64 / AArch64 | Shell App  | `uefi` crate built-in logger                 |

---

## FAQ

**Q: Do I need to modify the shared library code (`lib.rs`, `capture.rs`) for my platform?**

A: No. The shared library is platform-agnostic. Your binary only provides a logger and entry point.

**Q: What if my platform uses a UART not supported by `patina`?**

A: Implement the `patina` serial trait for your device. The logger just needs a
type that implements the appropriate serial trait(`SerialIO`).

**Q: What does `NO_STD_FLAGS` in `Makefile.toml` do?**

A: It tells Cargo to build `core`, `compiler_builtins`, and `alloc` from source
since `no_std` UEFI targets don't have a pre-built standard library. This is
only needed for DXE driver binaries, not UEFI Shell apps.
