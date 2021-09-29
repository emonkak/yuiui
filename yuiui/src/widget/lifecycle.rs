#[derive(Debug)]
pub enum Lifecycle<T> {
    Mounted,
    Updated(T),
    Unmounted,
}

impl<T> Lifecycle<T> {
    pub fn map<F, U>(self, f: F) -> Lifecycle<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Mounted => Lifecycle::Mounted,
            Self::Updated(new_value) => Lifecycle::Updated(f(new_value)),
            Self::Unmounted => Lifecycle::Unmounted,
        }
    }
}
