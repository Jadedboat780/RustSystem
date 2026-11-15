use core::{
    cell::UnsafeCell,
    fmt,
    ops::{Deref, DerefMut},
    sync::atomic::{
        AtomicBool,
        Ordering::{Acquire, Relaxed, Release},
    },
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
        while self.locked.compare_exchange_weak(false, true, Acquire, Relaxed).is_err() {
            while self.locked.load(Relaxed) {
                core::hint::spin_loop();
            }
        }

        Guard { lock: self }
    }

    #[inline]
    pub fn try_lock(&self) -> Option<Guard<'_, T>> {
        self.locked
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_ok()
            .then_some(Guard { lock: self })
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
    pub unsafe fn force_unlock(&self) {
        self.unlock();
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Relaxed)
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
        let mut d = f.debug_struct("SpinLock");
        match self.try_lock() {
            Some(guard) => d.field("value", &*guard),
            None => d.field("value", &format_args!("<locked>")),
        };
        d.finish_non_exhaustive()
    }
}

#[must_use = "if unused the SpinLock will immediately unlock"]
pub struct Guard<'lock, T> {
    lock: &'lock SpinLock<T>,
}

unsafe impl<T: Sync> Sync for Guard<'_, T> {}
impl<T: fmt::Debug> fmt::Debug for Guard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Guard").field("value", &**self).finish()
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
