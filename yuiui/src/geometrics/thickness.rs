#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Thickness<T = f32> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Thickness<T> {
    pub fn uniform(length: T) -> Self
    where
        T: Copy,
    {
        Self {
            left: length,
            right: length,
            top: length,
            bottom: length,
        }
    }
}
