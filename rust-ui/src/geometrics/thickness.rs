#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Thickness<T = f32> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}
