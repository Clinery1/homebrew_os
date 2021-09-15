use alloc::alloc::{
    GlobalAlloc,
    Layout,
};
use core::ptr::null_mut;
use super::{
    frame::FRAME_ALLOCATOR,
};


#[global_allocator]
pub static GLOBAL_ALLOCATOR:GlobalAllocator=GlobalAllocator;


#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(_:Layout)->! {
    panic!();
}


pub struct GlobalAllocator;
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self,layout:Layout) -> *mut u8 {
        //FRAME_ALLOCATOR.lock().allocate();
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
