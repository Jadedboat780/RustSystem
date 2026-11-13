use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    pub fn lock(&self) -> Guard<'_, T> {
        while self.locked.swap(true, Acquire) {
            core::hint::spin_loop();
        }

        Guard { lock: self }
    }

    #[inline]
    pub fn try_lock(&self) -> Option<Guard<'_, T>> {
        if !self.locked.swap(true, Acquire) {
            Some(Guard { lock: self })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    #[inline]
    fn unlock(&self) {
        self.locked.store(false, Release);
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Acquire)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> From<T> for SpinLock<T> {
    fn from(value: T) -> Self {
        SpinLock::new(value)
    }
}

impl<T: Default> Default for SpinLock<T> {
    fn default() -> Self {
        SpinLock::new(Default::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for SpinLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinLock")
            .field("locked", &self.locked)
            .field("value", unsafe { &*self.value.get() })
            .finish()
    }
}

pub struct Guard<'lock, T> {
    lock: &'lock SpinLock<T>,
}

unsafe impl<T: Sync> Sync for Guard<'_, T> {}
impl<T: fmt::Debug> fmt::Debug for Guard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Guard")
            .field("value", unsafe { &*self.lock.value.get() })
            .finish()
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    // Safety: The very existence of this Guard
    // guarantees we've exclusively locked the lock
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    // Safety: The very existence of this Guard
    // guarantees we've exclusively locked the lock
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test_case]
//     fn test_lock() {
//         let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
//         assert!(!array.is_locked());
//
//         let _lock_array = array.lock();
//         assert!(array.is_locked())
//     }
//
//     #[test_case]
//     fn test_try_lock() {
//         let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
//         let lock_array_1 = array.try_lock();
//         assert!(lock_array_1.is_some());
//
//         let lock_array_2 = array.try_lock();
//         assert!(lock_array_2.is_none())
//     }
//
//     #[test_case]
//     fn test_unlock() {
//         let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
//         let lock_array = array.lock();
//         assert!(array.is_locked());
//
//         drop(lock_array);
//         assert!(!array.is_locked());
//     }
// }
