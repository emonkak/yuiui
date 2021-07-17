mod iter;
mod waker;

#[cfg(test)]
mod tests;

use std::cell::UnsafeCell;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct Generator<'a, Yield, Resume, Return> {
    coroutine: Coroutine<Yield, Resume>,
    future: Pin<Box<dyn Future<Output = Return> + 'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum GeneratorState<Yield, Return> {
    Yielded(Yield),
    Complete(Return),
}

#[derive(Debug)]
pub struct Coroutine<Yield, Resume> {
    state: Rc<UnsafeCell<CoroutineState<Yield, Resume>>>,
}

#[derive(Debug, Eq, PartialEq)]
enum CoroutineState<Yield, Resume> {
    Empty,
    Suspend(Yield),
    Resume(Resume),
    Done,
}

impl<'a, Yield, Resume, Return> Generator<'a, Yield, Resume, Return> {
    pub fn new<F>(producer: impl FnOnce(Coroutine<Yield, Resume>) -> F + 'a) -> Self
    where
        F: Future<Output = Return> + 'a,
    {
        let coroutine = Coroutine::new();
        let future = Box::pin(producer(coroutine.clone()));
        Self { coroutine, future }
    }

    pub fn start(&mut self) -> GeneratorState<Yield, Return> {
        debug_assert_eq!(self.coroutine.peek_state(), CoroutineState::Empty);

        self.coroutine.advance(self.future.as_mut())
    }

    pub fn resume(&mut self, argument: Resume) -> GeneratorState<Yield, Return> {
        debug_assert_eq!(self.coroutine.peek_state(), CoroutineState::Empty);

        self.coroutine
            .replace_state(CoroutineState::Resume(argument));
        self.coroutine.advance(self.future.as_mut())
    }

    pub fn is_complete(&self) -> bool {
        self.coroutine.peek_state() == CoroutineState::Done
    }
}

impl<'a, Yield, Return> IntoIterator for Generator<'a, Yield, (), Return> {
    type Item = Yield;
    type IntoIter = iter::IntoIter<'a, Yield, Return>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { generator: self }
    }
}

impl<Yield, Return> GeneratorState<Yield, Return> {
    pub fn yielded(self) -> Option<Yield> {
        match self {
            GeneratorState::Yielded(value) => Some(value),
            _ => None,
        }
    }

    pub fn complete(self) -> Option<Return> {
        match self {
            GeneratorState::Complete(value) => Some(value),
            _ => None,
        }
    }
}

impl<Yield, Resume> Coroutine<Yield, Resume> {
    fn new() -> Self {
        Self {
            state: Rc::new(UnsafeCell::new(CoroutineState::Empty)),
        }
    }

    pub fn suspend(&self, value: Yield) -> impl Future<Output = Resume> {
        match self.peek_state() {
            CoroutineState::Empty | CoroutineState::Resume(_) => {}
            state @ _ => panic!("Invalid state: {:?}", state),
        }

        self.replace_state(CoroutineState::Suspend(value));
        self.clone()
    }

    fn peek_state(&self) -> CoroutineState<(), ()> {
        unsafe { self.state.get().as_ref().unwrap().without_values() }
    }

    fn replace_state(
        &self,
        next_state: CoroutineState<Yield, Resume>,
    ) -> CoroutineState<Yield, Resume> {
        unsafe { ptr::replace(self.state.get(), next_state) }
    }

    fn advance<Return>(
        &self,
        future: Pin<&mut dyn Future<Output = Return>>,
    ) -> GeneratorState<Yield, Return> {
        let waker = waker::create();
        let mut context = Context::from_waker(&waker);

        match future.poll(&mut context) {
            Poll::Pending => match self.replace_state(CoroutineState::Empty) {
                CoroutineState::Suspend(value) => GeneratorState::Yielded(value),
                state @ _ => panic!("Invalid state: {:?}", state.without_values()),
            },
            Poll::Ready(value) => {
                self.replace_state(CoroutineState::Done);
                GeneratorState::Complete(value)
            }
        }
    }

    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<Yield, Resume> Future for Coroutine<Yield, Resume> {
    type Output = Resume;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match self.peek_state() {
            CoroutineState::Suspend(_) => Poll::Pending,
            CoroutineState::Resume(_) => match self.replace_state(CoroutineState::Empty) {
                CoroutineState::Resume(argument) => Poll::Ready(argument),
                state @ _ => panic!("Invalid state: {:?}", state.without_values()),
            },
            state @ _ => panic!("Invalid state: {:?}", state),
        }
    }
}

impl<Yield, Resume> CoroutineState<Yield, Resume> {
    fn without_values(&self) -> CoroutineState<(), ()> {
        match self {
            Self::Empty => CoroutineState::Empty,
            Self::Suspend(_) => CoroutineState::Suspend(()),
            Self::Resume(_) => CoroutineState::Resume(()),
            Self::Done => CoroutineState::Done,
        }
    }
}
