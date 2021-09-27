#[derive(Debug)]
pub enum Lifecycle<T> {
    OnMount,
    OnUpdate(T),
    OnUnmount,
}

impl<T> Lifecycle<T> {
    pub fn map<F, U>(self, f: F) -> Lifecycle<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::OnMount => Lifecycle::OnMount,
            Self::OnUpdate(new_value) => Lifecycle::OnUpdate(f(new_value)),
            Self::OnUnmount => Lifecycle::OnUnmount,
        }
    }
}
