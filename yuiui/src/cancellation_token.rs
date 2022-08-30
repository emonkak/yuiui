use std::cell::UnsafeCell;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const WAITING: usize = 0b00;
const REGISTERING: usize = 0b01;
const CANCELING: usize = 0b10;

#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<Inner>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: AtomicUsize::new(WAITING),
                token: UnsafeCell::new(None),
            }),
        }
    }

    pub fn register(&self, token: RawToken) {
        match self
            .inner
            .state
            .compare_exchange(WAITING, REGISTERING, Ordering::Acquire, Ordering::Acquire)
            .unwrap_or_else(|x| x)
        {
            WAITING => unsafe {
                *self.inner.token.get() = Some(token);

                match self.inner.state.compare_exchange(
                    REGISTERING,
                    WAITING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {}
                    Err(actual) => {
                        debug_assert_eq!(actual, REGISTERING | CANCELING);
                        let token = (*self.inner.token.get()).take().unwrap();
                        self.inner.state.swap(WAITING, Ordering::AcqRel);
                        token.cancel();
                    }
                }
            },
            CANCELING => {
                token.cancel();
            }
            state => {
                debug_assert!(state == REGISTERING || state == REGISTERING | CANCELING);
            }
        }
    }

    pub fn cancel(&self) {
        if let Some(token) = self.take() {
            token.cancel();
        }
    }

    fn take(&self) -> Option<RawToken> {
        match self.inner.state.fetch_or(CANCELING, Ordering::AcqRel) {
            WAITING => {
                let token = unsafe { (*self.inner.token.get()).take() };
                self.inner.state.fetch_and(!CANCELING, Ordering::Release);
                token
            }
            state => {
                debug_assert!(
                    state == REGISTERING || state == REGISTERING | CANCELING || state == CANCELING
                );
                None
            }
        }
    }
}

impl fmt::Debug for CancellationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CancellationToken")
    }
}

unsafe impl Send for CancellationToken {}

unsafe impl Sync for CancellationToken {}

struct Inner {
    state: AtomicUsize,
    token: UnsafeCell<Option<RawToken>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(token) = unsafe { (*self.token.get()).take() } {
            token.drop();
        }
    }
}

pub struct RawToken {
    data: *const (),
    vtable: &'static RawTokenVTable,
}

impl RawToken {
    pub fn new(data: *const (), vtable: &'static RawTokenVTable) -> Self {
        Self { data, vtable }
    }

    fn cancel(self) {
        unsafe { (self.vtable.cancel)(self.data) }
    }

    fn drop(self) {
        unsafe { (self.vtable.drop)(self.data) }
    }
}

pub struct RawTokenVTable {
    cancel: unsafe fn(*const ()),
    drop: unsafe fn(*const ()),
}

impl RawTokenVTable {
    pub const fn new(cancel: unsafe fn(*const ()), drop: unsafe fn(*const ())) -> Self {
        Self { cancel, drop }
    }
}
