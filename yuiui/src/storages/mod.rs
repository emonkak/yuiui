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

fn binary_search_by<T, F>(xs: &[T], mut f: F) -> Result<usize, usize>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_by() {
        let xs: Vec<Option<u32>> = vec![None, None, None, None, None, None, None, None, None, None];
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&0))), Err(0));

        let xs: Vec<Option<u32>> = vec![
            Some(1), // 0
            None,    // 1
            None,    // 2
            None,    // 3
            None,    // 4
            None,    // 5
            None,    // 6
            None,    // 7
            None,    // 8
            Some(2), // 9
        ];
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&0))), Err(0));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&1))), Ok(0));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&2))), Ok(9));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&3))), Err(10));

        let xs: Vec<Option<u32>> = vec![
            None,    // 0
            Some(1), // 1
            None,    // 2
            Some(3), // 3
            Some(4), // 4
            Some(5), // 5
            Some(6), // 6
            None,    // 7
            Some(8), // 8
            None,    // 9
        ];
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&0))), Err(0));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&1))), Ok(1));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&2))), Err(2));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&3))), Ok(3));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&4))), Ok(4));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&5))), Ok(5));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&6))), Ok(6));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&7))), Err(8));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&8))), Ok(8));
        assert_eq!(binary_search_by(&xs, |x| x.map(|x| x.cmp(&9))), Err(9));
    }
}
