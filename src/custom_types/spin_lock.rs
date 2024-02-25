use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};
use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct SpinLock<T> {
    locked: AtomicBool, // состояние объекта (свободен или заблокирован)
    value: UnsafeCell<T> // предоставление внутринней изменяемости объекта
}

impl<T> SpinLock<T> {
    // конструктор
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value)
        }
    }

    // блокировка объекта
    pub fn lock(&self) -> Guard<T> {
        if self.is_locked(){
            panic!("SpinLock: double lock")
        }
        while self.locked.swap(true, Acquire) {
            // подсказка циклу вращения (сообщает процессору, что мы вращаемся, ожидая каких-либо изменени)
            // приводит к созданию специальной инструкции для оптимизации поведения блокировки
            core::hint::spin_loop();
        }
        Guard { lock: self }
    }

    // освобождение объекта (предназначена лишь для вызова диструктором)
    unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }

    // проверяем, заблокирован ли объект
    pub fn is_locked(&self) -> bool {
        self.locked.load(Acquire)
    }
}

// указываем, что использование типа между потоками является безопасным
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

// реализация вывода в консоль для отладки
impl<T: fmt::Debug> fmt::Debug for SpinLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinLock")
            .field("locked", &self.locked)
            .field("value", unsafe { &*self.value.get() })
            .finish()
    }
}

// безопасный интерфейс для блокировки
pub struct Guard<'lock, T> {
    lock: &'lock SpinLock<T>,
}

// реализация вывода в консоль для отладки
impl<T: fmt::Debug> fmt::Debug for Guard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Guard")
            .field("value", unsafe { &*self.lock.value.get() })
            .finish()
    }
}

// реализация разыменования
impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

// реализация разыменования с получением изменяемой ссылки
impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

// реализация диструктора (особождение блокировки)
impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        unsafe { self.lock.unlock() }
    }
}

// тесты для проверки работы SpinLock
#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn test_lock() {
        let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
        assert_eq!(array.is_locked(), false);

        let lock_array = array.lock();
        assert!(array.is_locked())
    }

    #[test_case]
    fn test_unlock() {
        let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
        let lock_array = array.lock();
        assert!(array.is_locked());

        drop(lock_array);
        assert_eq!(array.is_locked(), false);
    }

    #[test_case]
    fn test_defer(){
        let num = SpinLock::new(12);
        assert_eq!(12, *num.lock());
    }
}
