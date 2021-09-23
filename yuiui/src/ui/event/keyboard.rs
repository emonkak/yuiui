#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[rustfmt::skip]
#[repr(u8)]
pub enum Modifier {
    None    = 0b0000,
    Control = 0b0001,
    Shift   = 0b0010,
    Alt     = 0b0100,
    Super   = 0b1000,
}

impl Into<usize> for Modifier {
    fn into(self) -> usize {
        self as usize
    }
}
