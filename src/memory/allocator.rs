//! This is the temporary home of the memory allocator kernel module. Once kernel modules are
//! actually implemented, this will be moved out of the kernel to reduce the size.


use alloc::alloc::{
    GlobalAlloc,
    Layout,
};
use x86_64::addr::VirtAddr;
use core::ptr::null_mut;
use super::{
    frame::FRAME_ALLOCATOR,
};


#[global_allocator]
pub static GLOBAL_ALLOCATOR:GlobalAllocator=GlobalAllocator;


#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout:Layout)->! {
    panic!("Memory allocate error. Layout: {:?}",layout);
}


pub struct GlobalAllocator;
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self,layout:Layout)->*mut u8 {
        if layout.size()>0 {
            if let Ok(ptr)=FRAME_ALLOCATOR.lock().allocate(layout.size()) {
                return ptr.as_u64() as *mut u8;
            }
        }
        null_mut()
    }
    unsafe fn dealloc(&self,ptr:*mut u8,layout:Layout) {
        if layout.size()>0 {
            FRAME_ALLOCATOR.lock().deallocate(VirtAddr::new(ptr as u64),layout.size()).unwrap();
        }
    }
}
