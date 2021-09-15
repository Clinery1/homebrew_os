use crate::{
    math::Point,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{
        Rgb888,
        RgbColor,
    },
    geometry::{
        Dimensions,
        Point as EGPoint,
        Size,
    },
    primitives::Rectangle,
    Pixel,
};


#[derive(Default)]
pub struct Screen {
    pub w:usize,
    pub h:usize,
    fb:usize,
    s:usize,
}
impl Dimensions for Screen {
    fn bounding_box(&self)->Rectangle {
        Rectangle::new(EGPoint::new(0,0),Size::new(self.w as u32,self.h as u32))
    }
}
impl DrawTarget for Screen {
    type Color=Rgb888;
    type Error=core::convert::Infallible;
    fn draw_iter<I:IntoIterator<Item=Pixel<Self::Color>>>(&mut self,pixels:I)->Result<(),Self::Error> {
        for Pixel(EGPoint{x,y},color) in pixels {
            self.draw_pixel(Point(x as usize,y as usize),color);
        }
        return Ok(());
    }
    fn clear(&mut self,color:Rgb888)->Result<(),Self::Error> {
        for x in 0..self.w {
            for y in 0..self.h {
                self.draw_pixel(Point(x,y),color);
            }
        }
        return Ok(());
    }
}
impl Screen {
    pub fn new(w:usize,h:usize,s:usize,fb:usize)->Screen {
        Screen {w,h,s,fb}
    }
    pub fn draw_pixel(&self,p:Point,color:Rgb888) {
        let pixel_loc=self.fb+(self.s*p.1)+(p.0*4); // 4 bytes/pixel
        let color=((color.r()as u32)<<16)|((color.g()as u32)<<8)|(color.b()as u32);
        unsafe{*(pixel_loc as *mut u32)=color;}
    }
    pub fn get_pixel(&self,p:Point)->Rgb888 {
        let pixel_loc=self.fb+(self.s*p.1)+(p.0*4); // 4 bytes/pixel
        let raw=unsafe{(pixel_loc as *mut u32).read()};
        Rgb888::new(((raw>>16)&0xFF)as u8,((raw>>8)&0xFF)as u8,(raw&0xFF)as u8)
    }
    pub fn move_up(&self,amt:usize,clear_color:Rgb888) {
        for y in amt..self.h {
            for x in 0..self.w {
                self.draw_pixel(Point(x,y-amt),self.get_pixel(Point(x,y)));
            }
        }
        for y in self.h-amt..self.h {
            for x in 0..self.w {
                self.draw_pixel(Point(x,y),clear_color);
            }
        }
    }
}
