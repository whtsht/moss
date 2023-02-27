use crate::error::{ErrorKind, Result};
use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;
use spin::once::Once;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub fn make_identity_mapping(
    mapper: &mut OffsetPageTable,
    allocator: &mut BootInfoFrameAllocator,
    base_addr: u64,
    num_pages: usize,
) -> Result<()> {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let base_page = Page::from_start_address(VirtAddr::new(base_addr))?;
    let base_frame = PhysFrame::from_start_address(PhysAddr::new(base_addr))?;
    let flags = Flags::PRESENT | Flags::WRITABLE;
    for i in 0..num_pages {
        let page = base_page + i as u64;
        let frame = base_frame + i as u64;
        unsafe { mapper.map_to(page, frame, flags, &mut *allocator) }?.flush();
    }
    Ok(())
}

static OFFSET_PAGE_TABLE: Once<Mutex<OffsetPageTable<'static>>> = Once::new();

/// Initialize a new OffsetPageTable.
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) {
    OFFSET_PAGE_TABLE.call_once(|| {
        let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };

        Mutex::new(unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) })
    });
}

pub fn get_mapper() -> Result<MutexGuard<'static, OffsetPageTable<'static>>> {
    Ok(OFFSET_PAGE_TABLE
        .get()
        .ok_or_else(|| ErrorKind::NotImplemented)?
        .lock())
}
/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
