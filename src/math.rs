use core::ops::{
    Mul,
    Add,
    Sub,
};
use embedded_graphics::geometry::Point as EGPoint;


#[derive(Default,Debug,Copy,Clone,PartialEq,Eq)]
pub struct Point(pub usize,pub usize);
impl Point {
    pub const fn zero()->Point {Point(0,0)}
}
impl Mul<Point> for Point {
    type Output=Point;
    fn mul(self,p:Point)->Point {
        Point(self.0*p.0,self.1*p.1)
    }
}
impl Add<Point> for Point {
    type Output=Point;
    fn add(self,p:Point)->Point {
        Point(self.0+p.0,self.1+p.1)
    }
}
impl Sub<Point> for Point {
    type Output=Point;
    fn sub(self,p:Point)->Point {
        Point(self.0-p.0,self.1-p.1)
    }
}
impl From<Point> for (f32,f32) {
    fn from(p:Point)->(f32,f32) {
        (p.0 as f32,p.1 as f32)
    }
}
impl From<Point> for (i32,i32) {
    fn from(p:Point)->(i32,i32) {
        (p.0 as i32,p.1 as i32)
    }
}
impl From<Point> for EGPoint {
    fn from(p:Point)->EGPoint {
        EGPoint{x:p.0 as i32,y:p.1 as i32}
    }
}
impl From<(usize,usize)> for Point {
    fn from(p:(usize,usize))->Point {
        Point(p.0,p.1)
    }
}
impl From<(i32,i32)> for Point {
    fn from(p:(i32,i32))->Point {
        Point(p.0 as usize,p.1 as usize)
    }
}
#[derive(Default,Debug,Copy,Clone,PartialEq,Eq)]
pub struct Vector(pub u32,pub u32);
impl From<Vector> for (f32,f32) {
    fn from(v:Vector)->(f32,f32) {
        (v.0 as f32,v.1 as f32)
    }
}
