mod array;
mod either;
mod hlist;
mod option;
mod tuple;
mod vec;

use bitflags::bitflags;

bitflags! {
    pub struct RenderFlags: u32 {
        const NONE = 1 << 0;
        const COMMITED = 1 << 1;
        const UPDATED = 1 << 2;
        const SWAPPED = 1 << 3;
    }
}
