pub enum Lifecycle<T> {
    WillMount,
    WillUpdate(T),
    WillUnmount,
    DidMount,
    DidUpdate(T),
    DidUnmount,
}

pub struct LifecycleContext;

impl<T> Lifecycle<T> {
    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Lifecycle<U> {
        match self {
            Lifecycle::WillMount => Lifecycle::WillMount,
            Lifecycle::WillUpdate(widget) => Lifecycle::WillUpdate(f(widget)),
            Lifecycle::WillUnmount => Lifecycle::WillUnmount,
            Lifecycle::DidMount => Lifecycle::DidMount,
            Lifecycle::DidUpdate(widget) => Lifecycle::DidUpdate(f(widget)),
            Lifecycle::DidUnmount => Lifecycle::DidUnmount,
        }
    }
}
