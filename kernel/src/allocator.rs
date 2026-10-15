use core::{
    alloc::{GlobalAlloc, Layout},
    ops::DerefMut,
};

use crate::{
    HEAP_SIZE, HEAP_START,
    memory::{frame_allocator, mapper_mut},
};
use linked_list_allocator::LockedHeap;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags},
};

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    #[allow(mismatched_lifetime_syntaxes)]
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

#[allow(unused)]
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator()
            .allocate_frame()
            .expect("Failed to allocate frame for heap");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper_mut()
                .map_to(page, frame, flags, frame_allocator().deref_mut())
                .expect("Failed to map frame for heap")
                .flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
}

#[allow(unused)]
pub fn kmalloc(layout: Layout) -> *mut u8 {
    unsafe { ALLOCATOR.alloc(layout) }
}

#[allow(unused)]
pub fn kfree(ptr: *mut u8, layout: Layout) {
    unsafe { ALLOCATOR.dealloc(ptr, layout) }
}
