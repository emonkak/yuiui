mod array;
mod either;
mod hlist;
mod option;
mod tuple;
mod vec;

use bitflags::bitflags;

bitflags! {
    pub struct RenderFlags: u32 {
        const NONE     = 0b000;
        const COMMITED = 0b001;
        const UPDATED  = 0b010;
        const SWAPPED  = 0b100;
    }
}
