use core::{alloc::Layout, ptr::NonNull};

use axalloc::global_allocator;
use axhal::mem::virt_to_phys;
use driver_net::e1000::KernelFunc;


pub struct E1000HalImpl;

impl KernelFunc for E1000HalImpl {
    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize) {
        let layout = Layout::from_size_align(pages, 8).unwrap();

        let vaddr = if let Ok(vaddr) = global_allocator().alloc(layout) {
            vaddr.as_ptr() as usize
        } else {
            return (0, 0);
        };

        let paddr = virt_to_phys(vaddr.into());

        (vaddr, paddr.as_usize())
    }

    fn dma_free_coherent(&mut self, vaddr: usize, pages: usize) {
        let layout = Layout::from_size_align(pages, 8).unwrap();
        global_allocator().dealloc(NonNull::new(vaddr as *mut u8).unwrap(), layout);
    }
}