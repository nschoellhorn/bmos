use bootloader::boot_info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::page_table::PageTableFlags;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::{
    page::{Page, Size4KiB},
    PhysFrame,
};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};
use x86_64::{structures::paging::mapper::Mapper, PhysAddr};

use crate::debug;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static HEAP_START: u64 = 0x_0000_1337_1337;
static HEAP_SIZE: u64 = 8192 * 1024; // 8 Megabytes of heap memory for the kernel

pub fn init(memory: &MemoryRegions, memory_offset: u64) {
    let usable_memory = (&**memory)
        .iter()
        .find(|region| region.kind == MemoryRegionKind::Usable)
        .unwrap();

    let mut frame_alloc = PhysicalFrameAllocator::new(usable_memory);

    debug!(
        "Usable memory region being used for kernel heap: {:?}",
        &usable_memory
    );

    debug!("Setting up page table...");
    let (phys_frame_l4, _) = Cr3::read();
    let mut page_table = unsafe {
        let l4_ptr = (phys_frame_l4.start_address().as_u64() + memory_offset) as *mut PageTable;
        let l4_page_table = &mut *l4_ptr;

        OffsetPageTable::new(l4_page_table, VirtAddr::new(memory_offset))
    };

    let heap_start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(HEAP_START));
    let heap_end_page: Page<Size4KiB> =
        Page::containing_address(VirtAddr::new(HEAP_START + HEAP_SIZE - 1));

    let heap_range = Page::range_inclusive(heap_start_page, heap_end_page);

    unsafe {
        heap_range.for_each(|page| {
            let phys_frame = match frame_alloc.allocate_frame() {
                Some(frame) => frame,
                None => panic!("Out of memory, no physical frames left."),
            };

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            match page_table.map_to(page, phys_frame, flags, &mut frame_alloc) {
                Ok(tlb) => tlb.flush(),
                Err(error) => panic!("Failed to map heap page: {:?}", error),
            }
        });
    }

    debug!("Heap mapped successfully.");
    debug!("Initializing global allocator.");

    let heap_start = HEAP_START as usize;
    let heap_size = HEAP_SIZE as usize;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

struct PhysicalFrameAllocator<'a> {
    usable_region: &'a MemoryRegion,
    last_frame: usize,
}

impl<'a> PhysicalFrameAllocator<'a> {
    pub fn new(usable_region: &'a MemoryRegion) -> Self {
        Self {
            usable_region,
            last_frame: 0,
        }
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for PhysicalFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let mut phys_iter = (self.usable_region.start..self.usable_region.end)
            .step_by(4096)
            .skip(self.last_frame);
        let frame_start = phys_iter.next();

        self.last_frame += 1;

        match frame_start {
            Some(addr) => Some(PhysFrame::containing_address(PhysAddr::new(addr))),
            None => None,
        }
    }
}
