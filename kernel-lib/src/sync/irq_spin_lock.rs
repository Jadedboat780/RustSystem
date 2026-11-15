use super::spin_lock::{Guard, SpinLock};
use core::{
    fmt,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};
use x86_64::instructions::interrupts;

pub struct IrqSpinLock<T> {
    inner: SpinLock<T>,
}

impl<T> IrqSpinLock<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            inner: SpinLock::new(value),
        }
    }

    #[inline]
    pub fn lock(&self) -> IrqGuard<'_, T> {
        let were_enabled = interrupts::are_enabled();
        interrupts::disable();
        IrqGuard::new(self.inner.lock(), were_enabled)
    }

    #[inline]
    pub fn try_lock(&self) -> Option<IrqGuard<'_, T>> {
        let were_enabled = interrupts::are_enabled();
        interrupts::disable();
        match self.inner.try_lock() {
            Some(guard) => Some(IrqGuard::new(guard, were_enabled)),
            None => {
                if were_enabled {
                    interrupts::enable();
                }
                None
            }
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    #[inline]
    pub unsafe fn force_unlock(&self) {
        unsafe { self.inner.force_unlock() }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T> From<T> for IrqSpinLock<T> {
    fn from(value: T) -> Self {
        IrqSpinLock::new(value)
    }
}

impl<T: Default> Default for IrqSpinLock<T> {
    fn default() -> Self {
        IrqSpinLock::new(Default::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for IrqSpinLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("IrqSpinLock");
        match self.try_lock() {
            Some(guard) => d.field("value", &*guard),
            None => d.field("value", &format_args!("<locked>")),
        };
        d.finish_non_exhaustive()
    }
}

#[must_use = "if unused the IrqSpinLock will immediately unlock"]
pub struct IrqGuard<'lock, T> {
    guard: ManuallyDrop<Guard<'lock, T>>,
    were_enabled: bool,
    _not_send: PhantomData<*const ()>,
}

impl<'lock, T> IrqGuard<'lock, T> {
    #[inline]
    fn new(guard: Guard<'lock, T>, were_enabled: bool) -> Self {
        Self {
            guard: ManuallyDrop::new(guard),
            were_enabled,
            _not_send: PhantomData,
        }
    }
}

unsafe impl<T: Sync> Sync for IrqGuard<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for IrqGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IrqGuard").field("value", &**self).finish()
    }
}

impl<T> Deref for IrqGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> DerefMut for IrqGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

impl<T> Drop for IrqGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.guard) };
        if self.were_enabled {
            interrupts::enable();
        }
    }
}
