use log::LevelFilter;
use uefi_sdk::log::SerialLogger;

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
        use uefi_sdk::{log::Format, serial::uart::UartPl011};
        static LOGGER: SerialLogger<UartPl011> = SerialLogger::new(
            Format::Standard,
            &[],
            log::LevelFilter::Trace,
            UartPl011::new(0x6000_0000),
        );
    } else if #[cfg(all(target_os = "uefi", target_arch = "x86_64"))] {
        use uefi_sdk::{log::Format, serial::uart::Uart16550};
        static LOGGER: SerialLogger<Uart16550> = SerialLogger::new(
            Format::Standard,
            &[],
            log::LevelFilter::Trace,
            Uart16550::Io { base: 0x402 },
        );
    }
}

pub fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(not(test))] {
            let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
        }
    }
}
