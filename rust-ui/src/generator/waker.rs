use std::task::{RawWaker, RawWakerVTable, Waker};

const RAW_WAKER: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);
const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

pub fn create() -> Waker {
    unsafe { Waker::from_raw(RAW_WAKER) }
}

unsafe fn clone(_: *const ()) -> RawWaker {
    RAW_WAKER
}
unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}
