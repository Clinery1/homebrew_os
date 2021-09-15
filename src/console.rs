use crate::{
    math::Point,
    screen::{
        Screen,
    },
    bootboot::{
        BootBootUnpacked,
        BOOTBOOT_INFO,
        BOOTBOOT_FB,
        BOOTBOOT,
    },
};
use spin::Mutex;
use core::{
    fmt::{
        self,
        Write
    },
};
use embedded_graphics::{
    text::{
        Text,
    },
    primitives::{
        line::Line,
        PrimitiveStyle,
        Primitive,
    },
    geometry::{
        Point as EGPoint,
    },
    pixelcolor::{
        Rgb888,
    },
    Drawable,
};
use bitmap_font::{
    tamzen::FONT_8x15,
    TextStyle,
};


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! move_up {
    () => ($crate::console::_move_up());
}
#[macro_export]
macro_rules! cursor_timer {
    () => ($crate::console::_cursor());
}


lazy_static::lazy_static! {
    pub static ref CONSOLE:Mutex<Console>={
        let bootboot:BootBootUnpacked=unsafe{*(BOOTBOOT_INFO as *const BOOTBOOT)}.into(); // convert raw data into not-so-raw data and an aligned rust struct
        let screen=Screen::new(bootboot.fb.width as usize,bootboot.fb.height as usize,bootboot.fb.scanline as usize,BOOTBOOT_FB as usize);
        Mutex::new(Console::new(screen))
    };
}


#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(||{
        CONSOLE.lock().write_fmt(args).unwrap();
    });
}
#[doc(hidden)]
pub fn _move_up() {
    x86_64::instructions::interrupts::without_interrupts(||{
        CONSOLE.lock().move_up();
    });
}
#[doc(hidden)]
pub fn _cursor() {
    x86_64::instructions::interrupts::without_interrupts(||{
        CONSOLE.lock().cursor_tick();
    });
}


pub struct Console {
    screen:Screen,
    w:usize,
    h:usize,
    fg:Rgb888,
    bg:Rgb888,
    cursor:Point,
    cursor_delay:u8,
    cursor_toggle:bool,
}
impl Write for Console {
    fn write_str(&mut self,string:&str)->fmt::Result {
        if let Err(_)=self.print_str_format(string,self.fg,Some(self.bg),false) {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}
#[allow(dead_code)]
impl Console {
    pub const FONT_WIDTH:usize=8;
    pub const FONT_HEIGHT:usize=15;
    pub const CURSOR_ON_DELAY:u8=50;
    pub const CURSOR_OFF_DELAY:u8=20;
    pub fn new(screen:Screen)->Console {
        Console {
            w:(screen.w/Self::FONT_WIDTH),
            h:(screen.h/(Self::FONT_HEIGHT)),
            screen,
            fg:Rgb888::new(175,175,175),
            bg:Rgb888::new(0,0,0),
            cursor:Point::zero(),
            cursor_delay:Self::CURSOR_ON_DELAY,
            cursor_toggle:true,
        }
    }
    pub fn new_colors(screen:Screen,fg:Rgb888,bg:Rgb888)->Console {
        Console {
            w:(screen.w/Self::FONT_WIDTH),
            h:(screen.h/(Self::FONT_HEIGHT)),
            screen,
            fg,
            bg,
            cursor:Point::zero(),
            cursor_delay:Self::CURSOR_ON_DELAY,
            cursor_toggle:true,
        }
    }
    pub fn draw_cursor(&mut self,color:Rgb888) {
        let start_coords=self.cursor*Point(Self::FONT_WIDTH,Self::FONT_HEIGHT);
        Line::new(start_coords.into(),(start_coords+Point(0,Self::FONT_HEIGHT-1)).into())
            .into_styled(PrimitiveStyle::with_stroke(color,1))
            .draw(&mut self.screen).unwrap();
    }
    pub fn cursor_tick(&mut self) {
        if self.cursor_delay==0 {
            if self.cursor_toggle {
                self.draw_cursor(self.fg);
                self.cursor_delay=Self::CURSOR_ON_DELAY;
            } else {
                self.draw_cursor(self.bg);
                self.cursor_delay=Self::CURSOR_OFF_DELAY;
            }
            self.cursor_toggle=!self.cursor_toggle;
        } else {
            self.cursor_delay-=1;
        }

    }
    pub fn println(&mut self,string:&str)->Result<(),usize> {
        if let Err(e)=self.print(string) {return Err(e)}
        self.cursor.0=0;
        self.cursor.1+=1;
        return Ok(());
    }
    pub fn print(&mut self,string:&str)->Result<(),usize> {
        self.print_str_format(string,self.fg,Some(self.bg),false)
    }
    pub fn println_str_format(&mut self,string:&str,fg:Rgb888,bg:Option<Rgb888>,underline:bool)->Result<(),usize> {
        if let Err(i)=self.print_str_format(string,fg,bg,underline) {
            return Err(i);
        }
        self.cursor.0=0;
        self.cursor.1+=1;
        return Ok(());
    }
    pub fn move_up(&mut self) {
        self.screen.move_up(Self::FONT_HEIGHT,self.bg);
    }
    pub fn print_str_format(&mut self,string:&str,fg:Rgb888,bg:Option<Rgb888>,underline:bool)->Result<(),usize> {  // index of the non-ascii char is returned
        /*  Disabled is_ascii check
        for (i,c) in string.chars().enumerate() {
            if !c.is_ascii() {return Err(i)}
        }
        */
        let mut newline=None;
        let mut carrige_return=None;
        for (i,c) in string.chars().enumerate() {
            if c=='\n' {
                newline=Some(i);
                break;
            } else if c=='\r' {
                carrige_return=Some(i);
                break;
            }
        }
        if self.cursor.0>=self.w {
            self.cursor.0-=self.w;
            self.cursor.1+=1;
        }
        if self.cursor.1>=self.h {
            self.cursor.0=0;
            self.cursor.1=self.h-1;
            self.screen.move_up(Self::FONT_HEIGHT,self.bg);
        }
        let start_coords=self.cursor*Point(Self::FONT_WIDTH,Self::FONT_HEIGHT);
        let style;
        if let Some(bg)=bg {
            style=TextStyle::new_bg(&FONT_8x15,Rgb888::new(255,255,255),bg);
        } else {
            style=TextStyle::new(&FONT_8x15,Rgb888::new(255,255,255));
        }
        if let Some(loc)=newline {
            self.draw_cursor(self.bg);
            if let Err(e)=self.println_str_format(&string[..loc],fg,bg,underline) {return Err(e)}
            if let Err(e)=self.print_str_format(&string[loc+1..],fg,bg,underline) {return Err(e)}
        } else if let Some(loc)=carrige_return {
            if let Err(e)=self.print_str_format(&string[..loc],fg,bg,underline) {return Err(e)}
            self.cursor.0=0;
            if let Err(e)=self.print_str_format(&string[loc+1..],fg,bg,underline) {return Err(e)}
        } else if self.cursor.0+string.len()>self.w {
            let split_loc=(self.cursor.0+string.len())-self.w;
            if let Err(e)=self.println_str_format(&string[..split_loc],fg,bg,underline) {return Err(e)}
            if let Err(e)=self.print_str_format(&string[split_loc..],fg,bg,underline) {return Err(e)}
        } else {
            let end=Text::new(string,start_coords.into(),style).draw(&mut self.screen).unwrap();
            self.cursor.0+=string.len();
            if underline {
                Line::new((start_coords+Point(0,Self::FONT_HEIGHT)).into(),EGPoint{x:end.x-1,y:end.y+Self::FONT_HEIGHT as i32})
                    .into_styled(PrimitiveStyle::with_stroke(fg,1))
                    .draw(&mut self.screen).unwrap();
            }
        }
        return Ok(());
    }
}
