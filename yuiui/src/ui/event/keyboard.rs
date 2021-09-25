#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[rustfmt::skip]
#[repr(u8)]
pub enum Modifier {
    None    = 0,
    Control = 1 << 0,
    Shift   = 1 << 1,
    Alt     = 1 << 2,
    Super   = 1 << 3,
}

impl Into<usize> for Modifier {
    fn into(self) -> usize {
        self as usize
    }
}
