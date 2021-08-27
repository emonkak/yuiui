use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

#[derive(Debug, Clone)]
pub struct StateCell<T> {
    state: Arc<State>,
    _type: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct WeakStateCell<T> {
    state: Weak<State>,
    _type: PhantomData<T>,
}

#[derive(Debug)]
pub struct StateRef<'a, T> {
    guard: MutexGuard<'a, Box<dyn Any + Send>>,
    _type: PhantomData<T>,
}

pub type State = Mutex<Box<dyn Any + Send>>;

impl<T> StateCell<T> {
    pub fn new(state: Arc<State>) -> Self {
        Self {
            state,
            _type: PhantomData,
        }
    }

    pub fn borrow(&self) -> StateRef<'_, T> {
        StateRef {
            guard: self.state.lock().unwrap(),
            _type: self._type,
        }
    }

    pub fn downgrade(&self) -> WeakStateCell<T> {
        WeakStateCell {
            state: Arc::downgrade(&self.state),
            _type: self._type,
        }
    }
}

impl<T> WeakStateCell<T> {
    pub fn upgrade(&self) -> Option<StateCell<T>> {
        if let Some(state) = self.state.upgrade() {
            Some(StateCell {
                state,
                _type: self._type,
            })
        } else {
            None
        }
    }
}

impl<T: 'static> Deref for StateRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        (*self.guard).downcast_ref().unwrap()
    }
}

impl<T: 'static> DerefMut for StateRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        (*self.guard).downcast_mut().unwrap()
    }
}
