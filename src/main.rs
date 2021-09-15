#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]


/*!
TODO:
    Allocator,
    Screen as an init module,
    ?Mouse driver,
    ?USB driver,
    ?PCI(E) driver

Notes on OSDEV:
    one IDT/processor,
    one GDT/OS (dont need more than one, but multiple possible)
*/


extern crate alloc;


use raw_cpuid::{
    CpuId,
    TopologyType,
};
use x86_64::{
    addr::VirtAddr,
    structures::paging::{
        FrameAllocator as FrameAllocatorTrait,
        FrameDeallocator as FrameDeallocatorTrait,
    },
};
use spin::Mutex;
use bootboot::*;
use memory::frame::{
    FrameAllocator,
    print_mmap,
};


/// TODO: make this a module that is loaded in initrd
mod screen;
mod bootboot;
mod math;
mod console;
mod initrd;
mod interrupts;
mod gdt;
mod memory;


static mut CPUS:Mutex<usize>=Mutex::new(0);


#[no_mangle]
fn _start()->! {  // This runs on all cores at once
    x86_64::instructions::interrupts::disable();
    let cpuid=CpuId::new();
    let bootboot:BootBootUnpacked=unsafe{*(BOOTBOOT_INFO as *const BOOTBOOT)}.into(); // convert raw data into not-so-raw data and an aligned rust struct
    let mut core=None;
    let mut cores=0;
    let mut threads=0;
    for level in cpuid.get_extended_topology_info().unwrap() {
        match level.level_type() {
            TopologyType::SMT=>{
                threads+=level.processors();
                if let None=core {
                    core=Some(level.x2apic_id());
                }
            },
            TopologyType::Core=>{
                cores+=level.processors();
                if let None=core {
                    core=Some(level.x2apic_id());
                }
            },
            _=>{}
        }
    }
    if core==Some(0) {  // if we are on core0
        let core=core.unwrap()as usize;
        gdt::init();
        interrupts::init(core).unwrap();    // we are core 0, so this will never panic
        iter_delay(10000);
        unsafe{*CPUS.lock()+=1;}
        println!("{} logical cores detected\n{} threads/physical core\n{} logical cores checked in",cores,threads,unsafe{CPUS.lock()});
        println!("Screen resolution: {}x{}",bootboot.fb.width,bootboot.fb.height);
        println!("Success!");
        print_mmap(&bootboot);
        let mut frame_allocator=FrameAllocator::new_cr3(VirtAddr::new(0),&bootboot);
        loop {  // do nothing
            x86_64::instructions::hlt();
        }
    } else {    // other cores
        // TODO: load IDT/core and start the cores on the work-fetching.
        // Core 0 is the scheduling core, for now, so we can schedule work easily.
        unsafe{*CPUS.lock()+=1;}
        loop {
            // this is a great idea to halt instead of spin. we use much less power this way.
            x86_64::instructions::hlt();
            // work fetching function
        }
    }
}
fn iter_delay(amt:usize) {
    for _ in 0..amt {}
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}",info);
    loop {
        x86_64::instructions::hlt();
    }
}
