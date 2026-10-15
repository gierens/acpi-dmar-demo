use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::{MutexGuard, Once, RwLock, RwLockReadGuard, RwLockWriteGuard};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
};

use crate::{PHYSICAL_MEMORY_START, allocator::Locked};

pub static MAPPER: Once<RwLock<OffsetPageTable<'static>>> = Once::new();

#[allow(unused)]
pub fn mapper() -> RwLockReadGuard<'static, OffsetPageTable<'static>> {
    MAPPER.get().expect("Failed to get mapper from Once").read()
}

#[allow(unused)]
pub fn mapper_mut() -> RwLockWriteGuard<'static, OffsetPageTable<'static>> {
    MAPPER
        .get()
        .expect("Failed to get mapper from Once")
        .write()
}

/// Initialize a new OffsetPageTable.
pub fn init_mapper() {
    unsafe {
        let level_4_table = active_level_4_table();
        MAPPER
            .call_once(|| RwLock::new(OffsetPageTable::new(level_4_table, PHYSICAL_MEMORY_START)));
    }
}

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = PHYSICAL_MEMORY_START + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub static FRAME_ALLOCATOR: Once<Locked<BootInfoFrameAllocator>> = Once::new();

pub fn frame_allocator() -> MutexGuard<'static, BootInfoFrameAllocator> {
    FRAME_ALLOCATOR
        .get()
        .expect("Failed to get frame allocator from Once")
        .lock()
}

pub fn init_frame_allocator(memory_regions: &'static MemoryRegions) {
    FRAME_ALLOCATOR
        .call_once(|| Locked::new(unsafe { BootInfoFrameAllocator::init(memory_regions) }));
    log::info!("Frame allocator initialized");
    FRAME_ALLOCATOR.get().unwrap().lock().dump_usable_regions();
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
///
/// This is a simple bump allocator without reclamation and without fragmentation prevention.
// TODO: implement a proper allocator with reclamation
pub struct BootInfoFrameAllocator {
    /// Slice of memory regions.
    memory_regions: &'static MemoryRegions,
    /// Index of the region in the `MemoryRegions.ptr` array that will be used for the next
    /// allocation, if it fits.
    next_region: usize,
    /// Address within the `next_region`, that is used for the next allocation, if it fits.
    next_address: u64,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        let mut allocator = BootInfoFrameAllocator {
            memory_regions,
            next_region: 0,
            next_address: memory_regions[0].start,
        };
        allocator.go_to_region(
            allocator
                .next_usable_region()
                .expect("No usable memory regions at all"),
        );
        allocator
    }

    /// Dumps all usable memory regions at log level trace
    pub fn dump_usable_regions(&self) {
        log::trace!("======== BootInfoFrameAllocator Memory Regions ========");
        let mut count = 0;
        for region in self
            .memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
        {
            log::trace!(
                "- start: 0x{:x}, end: 0x{:x}, size: 0x{:x}, frames: 0x{:x}",
                region.start,
                region.end,
                region.end - region.start,
                (region.end - region.start) / 4096
            );
            count += 1;
        }
        log::trace!("Number of regions: {}", count);
        log::trace!("=======================================================");
    }

    fn regions_exhausted(&self) -> bool {
        self.next_region >= self.memory_regions.len()
    }

    fn region_full(&self) -> bool {
        self.regions_exhausted() || self.next_address >= self.memory_regions[self.next_region].end
    }

    fn next_usable_region(&self) -> Option<usize> {
        Some(
            self.next_region
                + self
                    .memory_regions
                    .iter()
                    .skip(self.next_region)
                    .position(|r| r.kind == MemoryRegionKind::Usable)?,
        )
    }

    fn go_to_region(&mut self, index: usize) {
        self.next_region = index;
        self.next_address = self.memory_regions[index].start;
    }

    /// Allocate a single frame or None if this is not possible
    pub fn allocate_single_frame(&mut self) -> Option<PhysAddr> {
        if self.region_full() {
            self.go_to_region(self.next_region + 1);
            self.go_to_region(self.next_usable_region()?);
        }
        let addr = PhysAddr::new(self.next_address);
        self.next_address += 4096;
        Some(addr)
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_single_frame()
            .map(|a| unsafe { PhysFrame::from_start_address_unchecked(a) })
    }
}
