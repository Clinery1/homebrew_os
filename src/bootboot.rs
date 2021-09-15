#![allow(dead_code)]
pub const BOOTBOOT_MAGIC: &'static [u8; 5usize] = b"BOOT\0";
pub const PROTOCOL_MINIMAL: u32 = 0;
pub const PROTOCOL_STATIC: u32 = 1;
pub const PROTOCOL_DYNAMIC: u32 = 2;
pub const PROTOCOL_BIGENDIAN: u32 = 128;
pub const LOADER_BIOS: u32 = 0;
pub const LOADER_UEFI: u32 = 4;
pub const LOADER_RPI: u32 = 8;
pub const FB_ARGB: u32 = 0;
pub const FB_RGBA: u32 = 1;
pub const FB_ABGR: u32 = 2;
pub const FB_BGRA: u32 = 3;
pub const INITRD_MAXSIZE: u32 = 16;
pub const BOOTBOOT_MMIO: u64 = 0xfffffffff8000000;  /* memory mapped IO virtual address */
pub const BOOTBOOT_FB: u64 = 0xfffffffffc000000;  /* frame buffer virtual address */
pub const BOOTBOOT_INFO: u64 = 0xffffffffffe00000;  /* bootboot struct virtual address */
pub const BOOTBOOT_ENV: u64 = 0xffffffffffe01000;  /* environment string virtual address */
pub const BOOTBOOT_CORE: u64 = 0xffffffffffe02000;  /* core loadable segment start */


#[repr(u8)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum MMapType {
    Used=0,
    Free,
    Acpi,
    Mmio,
}


#[repr(C, packed)]
#[derive(Debug,Copy,Clone)]
pub struct MMapEnt {
    ptr: u64,
    size: u64,
}
impl MMapEnt {
    pub fn entry_type(&self)->MMapType {
        use MMapType::*;
        match self.size&0xf {
            0=>Used,
            1=>Free,
            2=>Acpi,
            3=>Mmio,
            _=>unreachable!(),
        }
    }
    pub fn size(&self)->u64{
        (self.size&(!0xf))as u64
    }
    pub fn ptr(&self)->u64 {
        self.ptr
    }
}
#[repr(C, packed)]
#[derive(Copy,Clone,PartialEq)]
pub struct BOOTBOOT {
    pub magic: [u8; 4usize],
    pub size: u32,
    pub protocol: u8,
    pub fb_type: u8,
    pub numcores: u16,
    pub bspid: u16,
    pub timezone: i16,
    pub datetime: [u8; 8usize],
    pub initrd_ptr: u64,
    pub initrd_size: u64,
    pub fb_ptr: *mut u8,
    pub fb_size: u32,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_scanline: u32,
    pub arch: arch_union,
}
#[derive(Copy,Clone)]
pub struct FrameBuffer {
    pub ptr:*mut u8,
    pub size:u32,
    pub width:u32,
    pub height:u32,
    pub scanline:u32,
    pub fb_type:u8,
}
#[derive(Copy,Clone)]
pub struct BootBootUnpacked {
    pub magic:[u8;4],
    pub size:u32,
    pub protocol:u8,
    pub numcores:u16,
    pub bspid:u16,
    pub timezone:i16,
    pub datetime:[u8;8],
    pub initrd_ptr:usize,
    pub initrd_size:usize,
    pub arch:arch_union,
    pub fb:FrameBuffer,
    pub mmio_count:usize,
    mmio_entry_ptr:*mut MMapEnt,    // has to be access through the impl for safety reasons
}
impl BootBootUnpacked {
    pub fn mmio_entry(&self,idx:usize)->Option<MMapEnt> {
        if idx>=self.mmio_count {return None}   // this makes accessing the (not really) unknown length array safe enough
        return Some(unsafe{self.mmio_entry_ptr.offset(idx as isize).read()});
    }
}
impl From<BOOTBOOT> for BootBootUnpacked {
    fn from(b:BOOTBOOT)->BootBootUnpacked {
        let fb=FrameBuffer {
            ptr:b.fb_ptr,
            size:b.fb_size,
            width:b.fb_width,
            height:b.fb_height,
            scanline:b.fb_scanline,
            fb_type:b.fb_type,
        };
        let mmio_count=(b.size as usize-128)/16;
        let mmio_entry_ptr=(BOOTBOOT_INFO+128) as *mut MMapEnt;
        BootBootUnpacked {
            magic:          b.magic,
            size:           b.size,
            protocol:       b.protocol,
            numcores:       b.numcores,
            bspid:          b.bspid,
            timezone:       b.timezone,
            datetime:       b.datetime,
            initrd_ptr:     b.initrd_ptr as usize,
            initrd_size:    b.initrd_size as usize,
            arch:           b.arch,
            fb,
            mmio_count,
            mmio_entry_ptr,
        }
    }
}
#[repr(C)]
#[derive(Copy,Clone)]
pub union arch_union {
    pub x86_64: arch_x86,
    pub aarch64: arch_aarch64,
    _bindgen_union_align: [u64; 8usize],
}
impl PartialEq for arch_union {
    fn eq(&self,other:&arch_union)->bool {
        unsafe{self.x86_64==other.x86_64}
    }
}
#[repr(C)]
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct arch_x86 {
    pub acpi_ptr: u64,
    pub smbi_ptr: u64,
    pub efi_ptr: u64,
    pub mp_ptr: u64,
    pub unused0: u64,
    pub unused1: u64,
    pub unused2: u64,
    pub unused3: u64,
}
#[repr(C)]
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct arch_aarch64 {
    pub acpi_ptr: u64,
    pub mmio_ptr: u64,
    pub efi_ptr: u64,
    pub unused0: u64,
    pub unused1: u64,
    pub unused2: u64,
    pub unused3: u64,
    pub unused4: u64,
}
