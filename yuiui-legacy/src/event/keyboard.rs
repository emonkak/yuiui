use bit_flags::IntoBits;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[rustfmt::skip]
#[repr(u32)]
pub enum Modifier {
    None    = 0,
    Control = 1 << 0,
    Shift   = 1 << 1,
    Alt     = 1 << 2,
    Super   = 1 << 3,
}

impl IntoBits for Modifier {
    type Bits = u32;

    #[inline]
    fn into_bits(self) -> Self::Bits {
        self as u32
    }
}
