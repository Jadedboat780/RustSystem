use core::cell::UnsafeCell;
use core::fmt::{self, Debug};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};

/// Примитив синхронизации для коротковременных блокировок
pub struct SpinLock<T> {
    locked: AtomicBool,   // состояние объекта (свободен/заблокирован)
    value: UnsafeCell<T>, // предоставление внутринней изменяемости объекта
}

impl<T> SpinLock<T> {
    /// Конструктор
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Блокировка объекта
    #[inline]
    pub fn lock(&self) -> Guard<T> {
        while self.locked.swap(true, Acquire) {
            // подсказка циклу вращения (сообщает процессору, что мы вращаемся, ожидая каких-либо изменени)
            // приводит к созданию специальной инструкции для оптимизации поведения блокировки
            core::hint::spin_loop();
        }

        Guard { lock: self }
    }

    /// Попытка блокировки объекта
    #[inline]
    pub fn try_lock(&self) -> Option<Guard<T>> {
        if !self.locked.swap(true, Acquire) {
            Some(Guard { lock: self })
        } else {
            None
        }
    }

    /// Возвращает мутабельную ссылку на объект
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    /// Разблокировка объекта (предназначена лишь для вызова диструктором)
    #[inline]
    fn unlock(&self) {
        self.locked.store(false, Release);
    }

    /// Проверка, заблокирован ли объект
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Acquire)
    }

    /// Извлечение значения из SpinLock
    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

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

unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T: Debug> Debug for SpinLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinLock")
            .field("locked", &self.locked)
            .field("value", unsafe { &*self.value.get() })
            .finish()
    }
}

/// Интерфейс для заблокированного объекта
pub struct Guard<'lock, T> {
    lock: &'lock SpinLock<T>,
}

impl<T: Debug> Debug for Guard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Guard")
            .field("value", unsafe { &*self.lock.value.get() })
            .finish()
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

/// Тесты
#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn test_lock() {
        let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
        assert!(!array.is_locked());

        let _lock_array = array.lock();
        assert!(array.is_locked())
    }

    #[test_case]
    fn test_try_lock() {
        let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
        let lock_array_1 = array.try_lock();
        assert!(lock_array_1.is_some());

        let lock_array_2 = array.try_lock();
        assert!(lock_array_2.is_none())
    }

    #[test_case]
    fn test_unlock() {
        let array: SpinLock<[i32; 5]> = SpinLock::new([1, 2, 3, 4, 5]);
        let lock_array = array.lock();
        assert!(array.is_locked());

        drop(lock_array);
        assert!(!array.is_locked());
    }
}
