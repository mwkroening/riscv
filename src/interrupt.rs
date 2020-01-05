//! Interrupts

use register::mstatus;
use mutex_trait::Mutex;
use core::cell::RefCell;

pub struct RISCVMutex<T> {
    data: T
}

unsafe impl<T> Sync for RISCVMutex<T> {}

impl<T> RISCVMutex<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }

    fn access(&self) -> &T {
        &self.data
    }
}

impl<T> Mutex for RISCVMutex<T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        free(|| {f(&mut self.data)})
    }
}

impl<'a, T> Mutex for &'a RISCVMutex<RefCell<T>> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        free(|| {f(&mut *self.access().borrow_mut())})
    }
}

/// Disables all interrupts
#[inline]
pub unsafe fn disable() {
    match () {
        #[cfg(riscv)]
        () => mstatus::clear_mie(),
        #[cfg(not(riscv))]
        () => unimplemented!(),
    }
}

/// Enables all the interrupts
///
/// # Safety
///
/// - Do not call this function inside an `interrupt::free` critical section
#[inline]
pub unsafe fn enable() {
    match () {
        #[cfg(riscv)]
        () => mstatus::set_mie(),
        #[cfg(not(riscv))]
        () => unimplemented!(),
    }
}

/// Execute closure `f` in an interrupt-free context.
///
/// This as also known as a "critical section".
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let mstatus = mstatus::read();

    // disable interrupts
    unsafe { disable(); }

    let r = f();

    // If the interrupts were active before our `disable` call, then re-enable
    // them. Otherwise, keep them disabled
    if mstatus.mie() {
        unsafe { enable(); }
    }

    r
}
