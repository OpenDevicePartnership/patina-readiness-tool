use log::LevelFilter;
use uefi_sdk::log::SerialLogger;

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
        use uefi_sdk::{log::Format, serial::uart::UartPl011};
        static LOGGER: AdvancedLogger<UartPl011> = AdvancedLogger::new(
            Format::Standard,
            &[],
            log::LevelFilter::Trace,
            UartPl011::new(0x6000_0000),
        );
    } else {
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
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
}
