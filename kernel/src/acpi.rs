use core::ptr::NonNull;

use acpi::RawPhysicalMapping;
use acpi::platform::AcpiPlatform;
use acpi::{AcpiTables, Handle, Handler, PciAddress, aml::AmlError};

use crate::PHYSICAL_MEMORY_START;

#[derive(Clone)]
pub struct AcpiHandler;

impl AcpiHandler {
    fn new() -> Self {
        Self
    }
}

impl Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> RawPhysicalMapping<T> {
        unsafe {
            RawPhysicalMapping::new(
                physical_address,
                NonNull::new((PHYSICAL_MEMORY_START + physical_address as u64).as_mut_ptr())
                    .unwrap(),
                size,
                size,
            )
        }
    }

    unsafe fn unmap_physical_region<T>(&self, _region: RawPhysicalMapping<T>) {}

    fn read_u8(&self, _address: usize) -> u8 {
        todo!()
    }
    fn read_u16(&self, _address: usize) -> u16 {
        todo!()
    }
    fn read_u32(&self, _address: usize) -> u32 {
        todo!()
    }
    fn read_u64(&self, _address: usize) -> u64 {
        todo!()
    }
    fn write_u8(&self, _address: usize, _value: u8) {
        todo!()
    }
    fn write_u16(&self, _address: usize, _value: u16) {
        todo!()
    }
    fn write_u32(&self, _address: usize, _value: u32) {
        todo!()
    }
    fn write_u64(&self, _address: usize, _value: u64) {
        todo!()
    }
    fn read_io_u8(&self, _port: u16) -> u8 {
        todo!()
    }
    fn read_io_u16(&self, _port: u16) -> u16 {
        todo!()
    }
    fn read_io_u32(&self, _port: u16) -> u32 {
        todo!()
    }
    fn write_io_u8(&self, _port: u16, _value: u8) {
        todo!()
    }
    fn write_io_u16(&self, _port: u16, _value: u16) {
        todo!()
    }
    fn write_io_u32(&self, _port: u16, _value: u32) {
        todo!()
    }
    fn read_pci_u8(&self, _address: PciAddress, _offset: u16) -> u8 {
        todo!()
    }
    fn read_pci_u16(&self, _address: PciAddress, _offset: u16) -> u16 {
        todo!()
    }
    fn read_pci_u32(&self, _address: PciAddress, _offset: u16) -> u32 {
        todo!()
    }
    fn write_pci_u8(&self, _address: PciAddress, _offset: u16, _value: u8) {
        todo!()
    }
    fn write_pci_u16(&self, _address: PciAddress, _offset: u16, _value: u16) {
        todo!()
    }
    fn write_pci_u32(&self, _address: PciAddress, _offset: u16, _value: u32) {
        todo!()
    }
    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }
    fn stall(&self, _microseconds: u64) {
        todo!()
    }
    fn sleep(&self, _milliseconds: u64) {
        todo!()
    }
    fn create_mutex(&self) -> Handle {
        todo!()
    }
    fn acquire(&self, _mutex: Handle, _timeout: u16) -> Result<(), AmlError> {
        todo!()
    }
    fn release(&self, _mutex: Handle) {
        todo!()
    }
}

pub fn init(rsdp_addr: Option<&u64>) -> AcpiPlatform<AcpiHandler> {
    let handler = AcpiHandler::new();
    let tables =
        unsafe { AcpiTables::from_rsdp(handler.clone(), *rsdp_addr.unwrap() as usize).unwrap() };
    log::info!("ACPI tables located");

    AcpiPlatform::new(tables, handler).unwrap()
}
