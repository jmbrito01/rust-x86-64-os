use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr, structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate}};
use bootloader::{BootInfo, bootinfo::{MemoryMap, MemoryRegionType}};
use lazy_static::lazy_static;

pub mod allocator;

lazy_static! {
  static ref PHYS_MEMORY_OFFSET: Mutex<u64> = Mutex::new(0);
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(boot_info: &'static BootInfo) -> OffsetPageTable<'static> {
  let physical_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  let level_4_table = active_level_4_table(physical_mem_offset);
  let mut mapper = OffsetPageTable::new(level_4_table, physical_mem_offset);

  let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);
  allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("Heap Allocation failed");

  // Saves phyisical mem offset for later
  let mut offset = PHYS_MEMORY_OFFSET.lock();
  *offset = boot_info.physical_memory_offset;

  mapper
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
  VirtAddr::new(addr.as_u64() + *PHYS_MEMORY_OFFSET.lock())
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
  -> &'static mut PageTable
{
  use x86_64::registers::control::Cr3;

  let (level_4_table_frame, _) = Cr3::read();

  let phys = level_4_table_frame.start_address();
  let virt = physical_memory_offset + phys.as_u64();
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

  &mut *page_table_ptr // unsafe
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
}

impl BootInfoFrameAllocator {
  /// Returns an iterator over the usable frames specified in the memory map.
  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
    // get usable regions from memory map
    let regions = self.memory_map.iter();
    let usable_regions = regions
      .filter(|r| r.region_type == MemoryRegionType::Usable);
    // map each region to its address range
    let addr_ranges = usable_regions
      .map(|r| r.range.start_addr()..r.range.end_addr());
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