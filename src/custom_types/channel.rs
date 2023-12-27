use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Relaxed, Release}
};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

//структура, представляющая канал
pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,    //хранит сообщение
    ready: AtomicBool,                      //отвечает за отслеживание сообщения
}

//указываем, что использование типа между потоками является безопасным
unsafe impl<T> Sync for Channel<T> where T: Send {}

//структуры, представляющая отправителя(в одином канале можно отправить лишь одно сообщение)
pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

//структуры, представляющая получателя
pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Channel<T> {
    //создание экземпляра объекта
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    //cоздание нового экземпляра канала и возвращение пары из отправителя и получателя
    pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
        *self = Self::new();
        (Sender { channel: self }, Receiver { channel: self })
    }
}

impl<T> Sender<'_, T> {
    //запись сообщения в канал
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Release);
    }
}

impl<T> Receiver<'_, T> {
    //проверка, готово ли сообщение для чтения
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    //cчитывание сообщения из канала
    pub fn receive(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            panic!("no message available!");
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

//реализация диструктора
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}