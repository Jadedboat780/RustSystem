use core::{
    cell::UnsafeCell,
    fmt,
    hint::spin_loop,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::atomic::{
        AtomicU8,
        Ordering::{Acquire, Relaxed, Release},
    },
};
use x86_64::instructions::interrupts;

const UNINIT: u8 = 0;
const RUNNING: u8 = 1;
const READY: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitInProgress;

impl fmt::Display for InitInProgress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("lazy value is being initialized")
    }
}

union Data<T, F> {
    f: ManuallyDrop<F>,
    value: ManuallyDrop<T>,
}

pub struct Lazy<T, F = fn() -> T> {
    state: AtomicU8,
    data: UnsafeCell<Data<T, F>>,
}

impl<T, F> Lazy<T, F> {
    #[inline]
    pub const fn new(f: F) -> Self {
        Self {
            state: AtomicU8::new(UNINIT),
            data: UnsafeCell::new(Data { f: ManuallyDrop::new(f) }),
        }
    }

    #[inline]
    pub fn get(this: &Self) -> Option<&T> {
        if this.state.load(Acquire) == READY {
            Some(unsafe { &(*this.data.get()).value })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if *this.state.get_mut() == READY {
            Some(unsafe { &mut this.data.get_mut().value })
        } else {
            None
        }
    }

    pub fn into_inner(this: Self) -> Result<T, F> {
        let mut this = ManuallyDrop::new(this);
        let this = &mut *this;
        let data = this.data.get_mut();
        unsafe {
            match *this.state.get_mut() {
                READY => Ok(ManuallyDrop::take(&mut data.value)),
                UNINIT => Err(ManuallyDrop::take(&mut data.f)),
                _ => unreachable!("Lazy: initialization did not finish"),
            }
        }
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    #[inline]
    pub fn force(this: &Self) -> &T {
        if this.state.load(Acquire) != READY {
            this.init_slow();
        }
        unsafe { &(*this.data.get()).value }
    }

    #[inline]
    pub fn force_mut(this: &mut Self) -> &mut T {
        if *this.state.get_mut() != READY {
            let data = this.data.get_mut();
            unsafe {
                let f = ManuallyDrop::take(&mut data.f);
                data.value = ManuallyDrop::new(f());
            }
            *this.state.get_mut() = READY;
        }
        unsafe { &mut this.data.get_mut().value }
    }

    #[inline]
    pub fn try_force(this: &Self) -> Result<&T, InitInProgress> {
        match this.state.load(Acquire) {
            READY => {}
            RUNNING => return Err(InitInProgress),
            _ => this.init_slow(),
        }
        Ok(unsafe { &(*this.data.get()).value })
    }

    #[cold]
    #[inline(never)]
    fn init_slow(&self) {
        interrupts::without_interrupts(|| {
            loop {
                match self.state.compare_exchange(UNINIT, RUNNING, Relaxed, Acquire) {
                    Ok(_) => {
                        let data = self.data.get();
                        let f = unsafe { ManuallyDrop::take(&mut (*data).f) };
                        let value = f();
                        unsafe { (*data).value = ManuallyDrop::new(value) };
                        self.state.store(READY, Release);
                        return;
                    }
                    Err(READY) => return,
                    Err(_) => spin_loop(),
                }
            }
        });
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        Lazy::force(self)
    }
}

impl<T, F: FnOnce() -> T> DerefMut for Lazy<T, F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        Lazy::force_mut(self)
    }
}

impl<T: Default> Default for Lazy<T> {
    fn default() -> Self {
        Lazy::new(T::default)
    }
}

impl<T: fmt::Debug, F> fmt::Debug for Lazy<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("Lazy");
        match Lazy::get(self) {
            Some(value) => d.field(value),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

unsafe impl<T: Sync + Send, F: Send> Sync for Lazy<T, F> {}

impl<T, F> Drop for Lazy<T, F> {
    fn drop(&mut self) {
        let data = self.data.get_mut();
        unsafe {
            match *self.state.get_mut() {
                READY => ManuallyDrop::drop(&mut data.value),
                UNINIT => ManuallyDrop::drop(&mut data.f),
                _ => {}
            }
        }
    }
}
