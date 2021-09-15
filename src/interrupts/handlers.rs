use x86_64::{
    structures::idt::InterruptStackFrame,
    instructions::port::Port,
};
use pc_keyboard::{
    layouts::Us104Key,
    DecodedKey,
    HandleControl,
    Keyboard,
    ScancodeSet1,
};
use spin::Mutex;
use crate::{print,println,cursor_timer};
use super::{
    PICS,
    InterruptID,
};


lazy_static::lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(Us104Key, ScancodeSet1,
            HandleControl::Ignore)
        );
}


pub extern "x86-interrupt" fn breakpoint(stack_frame:InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}",stack_frame);
}
pub extern "x86-interrupt" fn double_fault(stack_frame:InterruptStackFrame,error_code:u64)->! {
    panic!("#DF: {}\n{:#?}",error_code,stack_frame);
}
pub extern "x86-interrupt" fn general_prot(stack_frame:InterruptStackFrame,error_code:u64) {
    println!("#GP: {}\n{:#?}",error_code,stack_frame);
}
pub extern "x86-interrupt" fn timer(_stack_frame:InterruptStackFrame) {
    cursor_timer!();
    unsafe{PICS.lock().notify_end_of_interrupt(InterruptID::Timer.into())};
}
pub extern "x86-interrupt" fn keyboard(_stack_frame:InterruptStackFrame) {
    let mut keyboard=KEYBOARD.lock();
    let mut port=Port::new(0x60);
    let scancode:u8=unsafe{port.read()};
    if let Ok(Some(key_event))=keyboard.add_byte(scancode) {
        if let Some(key)=keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character)=>print!("{}",character),
                DecodedKey::RawKey(key)=>print!("{:?}",key),
            }
        }
    }
    unsafe{PICS.lock().notify_end_of_interrupt(InterruptID::Keyboard.into())};
}
