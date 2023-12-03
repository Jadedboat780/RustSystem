use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};
use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};


pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,   //предоставление изменяемости объекта
}

impl<T> SpinLock<T> {
    //создание экземпляра объекта
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),      //необходимо для внутренней изменивости
        }
    }

    //блокировка объекта
    pub fn lock(&self) -> Guard<T> {
        while self.locked.swap(true, Acquire) {
            core::hint::spin_loop();
        }
        Guard { lock: self }
    }

    //разблокировка объекта
    unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }

    //проверяем, заблокирован ли объект
    pub fn is_locked(&self) -> &AtomicBool {
        &self.locked
    }
}

//указываем, что использование типа между потоками является безопасным
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

//реализация вывода в консоль для отладки
impl<T: fmt::Debug> fmt::Debug for SpinLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinLock")
            .field("locked", &self.locked)
            .field("value", unsafe { &*self.value.get() })
            .finish()
    }
}

//безопасный интерфейс
pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

//реализация вывода в консоль для отладки
impl<T: fmt::Debug> fmt::Debug for Guard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Guard")
            .field("value", unsafe { &*self.lock.value.get() })
            .finish()
    }
}

//реализация разыменования указателя
impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

//реализация разыменования указателя с получением изменяемой ссылки
impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

//реализация диструктора
impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}
