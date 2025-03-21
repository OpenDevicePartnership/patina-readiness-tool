use log::{Level, LevelFilter, Metadata, Record};
use spin::Once;
use uefi_sdk::serial::SerialIO;

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
        use uefi_sdk::serial::UartPl011;
        struct SimpleLogger {
            uart: UartPl011,
        }
    } else {
        use uefi_sdk::serial::Uart16550;
        struct SimpleLogger {
            uart: Uart16550,
        }
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.uart.write(record.args().as_str().expect("No Log!").as_bytes());
        }
    }

    fn flush(&self) {}
}

static LOGGER: Once<SimpleLogger> = Once::new();

pub fn init_logger() {
    let logger = LOGGER.call_once(|| {
        cfg_if::cfg_if! {
            if #[cfg(all(target_os = "uefi", target_arch = "aarch64"))] {
                let uart = UartPl011::new(0x6000_0000);
                uart.init();
                SimpleLogger { uart }
            } else {
                let uart = Uart16550::new(uefi_sdk::serial::Interface::Io(0x402));
                uart.init();
                SimpleLogger { uart }
            }
        }
    });
    let _ = log::set_logger(logger).map(|()| log::set_max_level(LevelFilter::Info));
}
