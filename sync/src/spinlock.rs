use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::{cell::UnsafeCell, sync::atomic::AtomicBool};

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    pub fn new(value: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Lock and return a Gaurd which allows access via referece and mutable reference.
    pub fn lock(&self) -> Guard<T> {
        // If locked keep spinning until we can set to true.
        // Acquire means we see all operations before a Release on another thread.
        while self.locked.swap(true, Acquire) {
            // Provide a hint that we're in a spin oop
            std::hint::spin_loop()
        }

        Guard { lock: self }
    }
}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

// Allow references to the SpinLock between threads
unsafe impl<T> Sync for SpinLock<T> where T: Sync {}

// Ensure that Guard<T> is only sync/send if T is Send.
// Without this the compiler assumes it is sync if UnsafeCell<T> is Sync
unsafe impl<T> Send for Guard<'_, T> where T: Send {}
unsafe impl<T> Sync for Guard<'_, T> where T: Sync {}

#[cfg(test)]
mod tests {
    use crate::SpinLock;

    #[test]
    fn spin_lock() {
        let x = SpinLock::new(Vec::new());
        std::thread::scope(|s| {
            s.spawn(|| x.lock().push(1));
            s.spawn(|| {
                let mut g = x.lock();
                g.push(2);
                g.push(3);
            });
        });
        let g = x.lock();
        assert!(g.as_slice() == [1, 2, 3] || g.as_slice() == [3, 2, 1]);
    }
}
