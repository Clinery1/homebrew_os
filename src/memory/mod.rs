pub mod frame;
pub mod allocator;


pub const FREE_MARKER:u64=0x1C31C3BABEEEEEEE;   // LOL
pub const PAGE_SIZE:u64=4096;


/// These denotate the start of a free page
pub struct LinkedListNode {
    magic:u64,
    size:u64,
    next:Option<*mut LinkedListNode>,
}
impl LinkedListNode {
    pub fn new(next:Option<*mut LinkedListNode>)->LinkedListNode {
        LinkedListNode {
            magic:FREE_MARKER,
            size:PAGE_SIZE,
            next,
        }
    }
    pub fn new_size(next:Option<*mut LinkedListNode>,size:u64)->LinkedListNode {
        let size=size&!(PAGE_SIZE-1);
        LinkedListNode {
            magic:FREE_MARKER,
            size,
            next,
        }
    }
    pub fn set_next(&mut self,next:*mut LinkedListNode) {
        self.next=Some(next);
    }
    pub fn clear_next(&mut self) {
        self.next=None;
    }
    pub fn verify(&self)->bool {
        self.magic==FREE_MARKER
    }
}
