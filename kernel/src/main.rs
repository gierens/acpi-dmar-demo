#![no_std]
#![no_main]

extern crate alloc;

mod acpi;
mod allocator;
mod exit;
mod logger;
mod memory;

use ::acpi::sdt::dmar::{Dmar, DmarEntry};
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use x86_64::VirtAddr;

pub const KERNEL_BASE: u64 = 0x0000_0000_f000_0000;
pub const DYNAMIC_RANGE_START: u64 = 0xffff_8000_0000_0000;
pub const PHYSICAL_MEMORY_START: VirtAddr = VirtAddr::new(0xffff_c000_0000_0000);
pub const HEAP_START: usize = 0xffff_a000_0000_0000;
pub const HEAP_SIZE: usize = 10 * 1024 * 1024; // 10 MiB

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.kernel_base = Mapping::FixedAddress(KERNEL_BASE);
    config.mappings.dynamic_range_start = Some(DYNAMIC_RANGE_START);
    config.mappings.physical_memory = Some(Mapping::FixedAddress(PHYSICAL_MEMORY_START.as_u64()));
    config
};

entry_point!(bsp_main, config = &BOOTLOADER_CONFIG);

fn bsp_main(boot_info: &'static mut BootInfo) -> ! {
    logger::init(&mut boot_info.framebuffer);

    memory::init_mapper();
    memory::init_frame_allocator(&boot_info.memory_regions);

    allocator::init();

    let platform = acpi::init(boot_info.rsdp_addr.as_ref());

    let dmar = platform.tables.find_table::<Dmar>().unwrap();
    let dmar = dmar.get();
    log::info!("DMAR: {dmar:?}");
    for entry in dmar.entries() {
        log::info!("DMAR entry: {entry:?}");
        if let DmarEntry::Drhd(drhd) = entry {
            for scope in drhd.device_scopes() {
                log::info!("Device scope: {scope:?}, path: {:?}", scope.path());
            }
        }
    }

    exit::success();
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{info}");
    exit::failure()
}
