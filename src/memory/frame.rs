//! This module contains [`PageAllocator`] and [`FrameAllocator`]
//!
//! `PageAllocator` contains functions to allocate physical memory as contiguous virtual memory
//! which may/may not be contiguous in physical memory.


use x86_64::{
    addr::{
        VirtAddr,
        PhysAddr,
    },
    registers::control::Cr3,
    structures::paging::{
        PageTable,
        PageTableFlags,
        FrameAllocator as FrameAllocatorTrait,
        FrameDeallocator as FrameDeallocatorTrait,
        mapper::{
            OffsetPageTable,
            Mapper,
            MapperFlush,
        },
        page::{
            Page,
            Size4KiB,
        },
        frame::{
            PhysFrame,
        },
    },
};
use spin::Mutex;
use core::ops::{
    Deref,
    DerefMut,
};
use crate::{
    bootboot::{
        BootBootUnpacked,
        BOOTBOOT_INFO,
        BOOTBOOT,
        MMapType,
    },
    println,
};
use super::{
    LinkedListNode,
    PAGE_SIZE,
};


pub const STARTING_MEM_OFFSET:u64=1099511627776;  // 1TB


lazy_static::lazy_static! {
    pub static ref FRAME_ALLOCATOR:Mutex<PageAllocator>={   // this kinda has to be lazy since it relies on RT things
        let bb:BootBootUnpacked=unsafe{*(BOOTBOOT_INFO as *const BOOTBOOT)}.into();
        Mutex::new(PageAllocator::new(&bb))
    };
}


pub enum FrameAllocateError {
    Oom,
}


pub struct PageAllocator {
    frame:FrameAllocator,
    page:OffsetPageTable<'static>,
}
impl PageAllocator {
    pub fn new(bb:&BootBootUnpacked)->PageAllocator {
        let (cr3,_)=Cr3::read();
        let page_directory=unsafe{(cr3.start_address().as_u64() as *mut u8 as *mut PageTable).as_mut().unwrap()};
        PageAllocator {
            frame:FrameAllocator::new(&bb),
            page:unsafe{OffsetPageTable::new(page_directory,VirtAddr::new(0))}
        }
    }
    pub fn min_frames_from_size(size:usize)->usize {
        let mut frames=size/4096;
        if size%4096>0 {frames+=1}
        return frames;
    }
    /// This is continuous (virtual) memory. Physical memory may/may not be contiguous
    pub fn allocate(&mut self,size:usize)->Result<VirtAddr,FrameAllocateError> {
        let frames=Self::min_frames_from_size(size);
        if let Some(frame)=self.allocate_frame() {
            if let Ok((addr,flush))=unsafe{self.map_frame(frame,None)} {
                for frame_idx in 1..frames {
                    let addr_offset=addr+(PAGE_SIZE as usize*frame_idx);
                    if let Some(frame)=self.allocate_frame() {
                        if let Err(_)=unsafe{self.map_frame(frame,Some(addr_offset))} {
                            // if we don't succeed at mapping it, then deallocate it so we don't
                            // fill the RAM with unused allocations
                            unsafe {
                                self.frame.deallocate_frame(frame);
                                self.deallocate(addr,(frame_idx-1)*PAGE_SIZE as usize).unwrap();
                            }
                            return Err(FrameAllocateError::Oom);
                        }
                    } else {
                        // same here, except we didn't even allocate a frame this time.
                        unsafe {
                            self.deallocate(addr,(frame_idx-1)*PAGE_SIZE as usize).unwrap();
                        }
                        return Err(FrameAllocateError::Oom);
                    }
                }
                flush.flush();
                return Ok(addr);
            } else {
                // same here, except we don't even get past the first mapping!
                unsafe {
                    self.frame.deallocate_frame(frame);
                }
            }
        }
        return Err(FrameAllocateError::Oom);
    }
    /// Caller must make sure `virt` is a valid, unmapped page address. If `virt` is unaligned, we
    /// return `Err(())`
    pub unsafe fn map_frame(&mut self,frame:PhysFrame<Size4KiB>,virt:Option<VirtAddr>)->Result<(VirtAddr,MapperFlush<Size4KiB>),()> {
        let virt=virt.unwrap_or({
            let addr=self.memory_allocate_offset;
            self.memory_allocate_offset+=PAGE_SIZE;
            VirtAddr::new(addr)
        });
        if let Ok(page)=Page::<Size4KiB>::from_start_address(virt) {
            return Ok((virt,self.page.map_to(page,frame,PageTableFlags::PRESENT,&mut self.frame).unwrap()));
        }
        return Err(());
    }
    /// Accepts pointers to continuous (virtual) memory. Physical memory may/may not be contiguous
    pub unsafe fn deallocate(&mut self,ptr:VirtAddr,size:usize)->Result<(),()> {
        let frames=Self::min_frames_from_size(size);
        let ptr=ptr;
        for frame in 0..frames {
            let addr=ptr+(PAGE_SIZE as usize*frame);
            let page=Page::<Size4KiB>::from_start_address(addr).unwrap();
            if let Ok((frame,flush))=self.page.unmap(page) {
                self.deallocate_frame(frame);
                flush.flush();
            } else {
                return Err(());
            }
        }
        return Ok(());
    }
}
impl Deref for PageAllocator {
    type Target=FrameAllocator;
    fn deref(&self)->&FrameAllocator {
        &self.frame
    }
}
impl DerefMut for PageAllocator {
    fn deref_mut(&mut self)->&mut FrameAllocator {
        &mut self.frame
    }
}
pub struct FrameAllocator {
    first_free_node:Option<u64>,
    memory_allocate_offset:u64,
}
impl FrameAllocator {
    pub fn new(bb:&BootBootUnpacked)->FrameAllocator {
        FrameAllocator {
            first_free_node:Some(Self::mark_free(bb)),
            memory_allocate_offset:STARTING_MEM_OFFSET,
        }
    }
    #[allow(dead_code)]
    pub fn new_addr(bb:&BootBootUnpacked)->FrameAllocator {
        FrameAllocator {
            first_free_node:Some(Self::mark_free(bb)),
            memory_allocate_offset:STARTING_MEM_OFFSET,
        }
    }
    pub fn mark_free(bb:&BootBootUnpacked)->u64 {
        let mut first_node:Option<*mut LinkedListNode>=None;
        let mut last_node:Option<*mut LinkedListNode>=None;
        let mut list_entries=0;
        let mut not_aligned=0;
        for entry in (0..bb.mmio_count).map(|i|{bb.mmio_entry(i)}) {
            if let Some(entry)=entry {
                let mut size=entry.size();
                let mut start=PhysAddr::new(entry.ptr());
                if !start.is_aligned(PAGE_SIZE) {
                    not_aligned+=1;
                    let new_ptr=start.align_up(PAGE_SIZE);
                    let offset=new_ptr-start;
                    size-=offset;
                }
                if start==PhysAddr::new(0) {
                    start+=PAGE_SIZE;
                    size-=PAGE_SIZE;
                }
                if size%PAGE_SIZE==0&&size>PAGE_SIZE {
                    let ptr=start.as_u64() as *mut LinkedListNode;
                    unsafe{ptr.write(LinkedListNode::new_size(None,size));}
                    if let None=first_node {    // only executes once
                        first_node=Some(ptr);
                    }
                    if let Some(node)=last_node {
                        unsafe{node.as_mut().unwrap().set_next(ptr);}
                    } else {
                        last_node=Some(ptr);
                    }
                    list_entries+=1;
                }
            }
        }
        return first_node.expect("BootBoot did not specify any free memory!") as u64;
    }
}
unsafe impl FrameAllocatorTrait<Size4KiB> for FrameAllocator {
    fn allocate_frame(&mut self)->Option<PhysFrame<Size4KiB>> { // unsafe things happen in here. its kinda scary
        if let Some(node_ptr)=self.first_free_node {
            let node=unsafe{(node_ptr as *mut u8 as *mut LinkedListNode).read()};
            if !node.verify() {return None} // if this activates, we are in trouble
            if node.size>PAGE_SIZE {
                let node_addr=node_ptr as u64;
                let next_node_addr=node_addr+PAGE_SIZE;
                let next_node_addr_ptr=next_node_addr as *mut u8 as *mut LinkedListNode;
                unsafe{next_node_addr_ptr.write(LinkedListNode::new_size(node.next,node.size-PAGE_SIZE));}   // replace this node with an altered one in the next frame
                self.first_free_node=Some(next_node_addr);
                let node_ptr=node_ptr as *mut [u64;3];
                unsafe{node_ptr.write([0;3]);}  // clear the old node
                return Some(PhysFrame::from_start_address(PhysAddr::new(node_addr)).unwrap());
            } else {
                let addr=node_ptr as u64;
                let node_ptr=node_ptr as *mut [u64;3];
                unsafe{node_ptr.write([0;3]);}
                if let Some(next_ptr)=node.next {   // rewriting to avoid `unwrap()`s
                    self.first_free_node=Some(next_ptr as u64);
                } else {
                    self.first_free_node=None;
                }
                return Some(PhysFrame::from_start_address(PhysAddr::new(addr)).unwrap());
            }
        } else {
            return None;
        }
    }
}
impl FrameDeallocatorTrait<Size4KiB> for FrameAllocator {
    unsafe fn deallocate_frame(&mut self,frame:PhysFrame<Size4KiB>) {
        let ptr=frame.start_address().as_u64() as *mut u64 as *mut LinkedListNode;
        if let Some(addr)=self.first_free_node {
            ptr.write(LinkedListNode::new(Some(addr as *mut u8 as *mut LinkedListNode)));
        } else {
            ptr.write(LinkedListNode::new(None));
        }
        self.first_free_node=Some(ptr as u64);
    }
}


pub fn print_mmap(bb:&BootBootUnpacked) {
    println!("{} MMAP entries",bb.mmio_count);
    let mut ram=0;
    let mut last_ptr=0;
    for (i,entry) in (0..bb.mmio_count).map(|i|{bb.mmio_entry(i)}).enumerate() {
        if let Some(entry)=entry {
            let diff=entry.ptr()-last_ptr;
            println!("{} at {:#x} of size {}: {:?}, diff: {}",i,entry.ptr(),entry.size(),entry.entry_type(),diff);
            ram+=entry.size();
            if entry.entry_type()==MMapType::Used||entry.entry_type()==MMapType::Free {
                ram+=diff;
            }
            last_ptr=entry.ptr()+entry.size();
        } else {break}
    }
    println!("Total RAM: {}MB",(ram/1024)/1024);
}
#[allow(dead_code)]
unsafe fn access_bootboot_struct() {    // proof I know what I am talking about. this works only because the physical memory is allocated at its physical location in virtual memory
    let (cr3,_cr3_flags)=Cr3::read();
    let page_directory=(cr3.start_address().as_u64() as *const u8 as *const PageTable).read();
    let addr=BOOTBOOT_INFO;

    let level2_page_table_index=(addr>>39)&511;
    let level2_page_table_ptr=&page_directory[level2_page_table_index as usize].addr().as_u64();
    let level2_page_table=(*level2_page_table_ptr as *const u8 as *const PageTable).read();

    let level3_page_table_index=(addr>>30)&511;
    let level3_page_table_ptr=level2_page_table[level3_page_table_index as usize].addr().as_u64();
    let level3_page_table=(level3_page_table_ptr as *const u8 as *const PageTable).read();

    let level4_page_table_index=(addr>>21)&511;
    let level4_page_table_ptr=level3_page_table[level4_page_table_index as usize].addr().as_u64();
    let level4_page_table=(level4_page_table_ptr as *const u8 as *const PageTable).read();

    let page_index=(addr>>12)&511;
    let page_ptr=level4_page_table[page_index as usize].addr().as_u64()|(addr&0xfff);

    let bb_packed=(page_ptr as *const u8 as *const BOOTBOOT).read();
    let bb_original=*(BOOTBOOT_INFO as *const BOOTBOOT);
    assert!(bb_packed==bb_original);
}
