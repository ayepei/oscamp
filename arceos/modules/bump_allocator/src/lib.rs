#![no_std]

use core::ptr::NonNull;

use allocator::{ AllocError, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize>{
    start:usize,
    byte_pos:usize,
    end:usize,
    page_pos:usize,
    count:usize,
    count_page:usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            byte_pos: 0,
            end: 0,
            page_pos: 0,
            count:0,
            count_page:0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());
        self.start=start;
        self.byte_pos=start;
        self.end=start+size;
        self.page_pos=start+size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        todo!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: core::alloc::Layout) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        if self.byte_pos+layout.size()>self.page_pos{
           return Err(AllocError::NoMemory);
        }
        let left=self.byte_pos;
        self.byte_pos=self.byte_pos+layout.size();
        self.count+=1;
        Ok(unsafe { NonNull::new_unchecked(left as *mut u8) })
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.count-=1;
        if self.count==0{
            self.byte_pos=self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.page_pos-self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte_pos-self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_pos-self.byte_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize= PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, _align_pow2: usize) -> allocator::AllocResult<usize> {
        if self.page_pos-num_pages*PAGE_SIZE<self.byte_pos{
            return Err(AllocError::NoMemory);
         }
         self.page_pos=self.page_pos-num_pages*PAGE_SIZE;
         self.count_page+=num_pages;
         Ok(num_pages)
    }

    fn dealloc_pages(&mut self, _pos: usize, num_pages: usize) {
        self.count_page-=num_pages;
        if self.count_page==0{
            self.page_pos=self.end;
        }
    }

    fn total_pages(&self) -> usize {
        (self.end-self.byte_pos)/PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end-self.page_pos)/PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.page_pos-self.byte_pos)/PAGE_SIZE
    }
}
