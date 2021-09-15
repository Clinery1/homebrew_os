//! TODO: Refactor GDT stuff into src/gdt.rs
use x86_64::{
    structures::{
        idt::{
            InterruptDescriptorTable,
        },
    },
};
use pic8259::ChainedPics;
use spin::Mutex;
use crate::{
    gdt::DOUBLE_FAULT_IST_INDEX,
};


pub mod handlers;


pub const PIC1_OFFSET:u8=32;
pub const PIC2_OFFSET:u8=PIC1_OFFSET+8;


lazy_static::lazy_static! {
    pub static ref CORE0_IDT:InterruptDescriptorTable={
        let mut idt=InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(handlers::breakpoint);
        idt.general_protection_fault.set_handler_fn(handlers::general_prot);
        unsafe{idt.double_fault.set_handler_fn(handlers::double_fault).set_stack_index(DOUBLE_FAULT_IST_INDEX);}
        idt[InterruptID::Timer.into()].set_handler_fn(handlers::timer);
        idt[InterruptID::Keyboard.into()].set_handler_fn(handlers::keyboard);
        idt
    };
}
pub static PICS:Mutex<ChainedPics>=Mutex::new(unsafe{ChainedPics::new(PIC1_OFFSET,PIC2_OFFSET)});


#[repr(u8)]
#[derive(Debug,Clone,Copy)]
pub enum InterruptID {
    Timer=PIC1_OFFSET,
    Keyboard,
}
impl From<InterruptID> for usize {fn from(id:InterruptID)->usize {id as u8 as usize}}
impl From<InterruptID> for u8 {fn from(id:InterruptID)->u8 {id as u8}}


pub fn init(core:usize)->Result<(),()> {
    match core {
        0=>{
            CORE0_IDT.load();
        },
        _=>return Err(()),
    }
    unsafe {
        PICS.lock().initialize();
        PICS.lock().write_masks(!3,0xff);
    }
    x86_64::instructions::interrupts::enable();
    return Ok(());
}
