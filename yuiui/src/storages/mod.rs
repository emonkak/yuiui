mod array;
mod either;
mod hlist;
mod option;
mod tuple;
mod vec;

use bitflags::bitflags;
use std::cmp::Ordering;

bitflags! {
    pub struct RenderFlags: u32 {
        const NONE     = 0b000;
        const COMMITED = 0b001;
        const UPDATED  = 0b010;
        const SWAPPED  = 0b100;
    }
}

pub fn binary_search_by<T, F>(xs: &[T], mut f: F) -> Result<usize, usize>
where
    F: FnMut(&T) -> Option<Ordering>,
{
    let mut size = xs.len();
    let mut left = 0;
    let mut right = size;

    while left < right {
        let mut middle = left + size / 2;
        let mut start = middle;
        let mut end = middle + 1;

        loop {
            match f(&xs[middle]) {
                Some(Ordering::Less) => {
                    left = end;
                    break;
                }
                Some(Ordering::Greater) => {
                    right = start;
                    break;
                }
                Some(Ordering::Equal) => {
                    return Ok(middle);
                }
                None => {
                    if end < right {
                        middle = end;
                        end += 1;
                    } else if start > left {
                        start -= 1;
                        middle = start;
                    } else {
                        return Err(left);
                    }
                }
            }
        }

        size = right - left;
    }

    Err(left)
}
