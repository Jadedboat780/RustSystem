# Минимальное  64-битное ядро для архитектуры x86 на Rust </h1>

Проект является моей попыткой научиться писать разные системные вещи, поэтому какой-либо конечной цели у проекта нет. Актуальная версия проекта: 0.0.1

Запуск происходит с помощью эмулятора QEMU. Перед запуском необходимо выполнить команды:
```commandline
cargo install bootimage
cargo bootimage
```
## Реализовано:
* минимальная работа с консолью
* минимальный vga буффер
* ввод\ввывод с клавиатуры
* работа с динамической памятью
* исключения цп
* тестирование
---
## Структура проекта:
* .cargo - папка с конфигурационными данными 
* src - папка с кодом ядра
  * lib.rs - реализует вспомогательные функции
  * main.rs - сборка проекта
  * allocator.rs - реализует аллокатор, чтобы работать с динамической памятью
  * gdt.rs - содержит отдельный стек двойных ошибок в таблице стека прерываний
  * interrupts.rs - реализация стека прерываний
  * memory.rs - реализует управление физической памятью и создание отображений страниц виртуальной памяти на физическую память
  * serial.rs - реализация Serial Port
  * vga_buffer.rs - реализация минимального VGA буфера
  * spin_lock.rs - реализация простой обёртки для монопольного доступа к данным
* test - папка с тестами
    * basic_boot.rs - тестирование функций
    * heap_allocation.rs - тестирование работы аллокатора
    * should_panic.rs - тестирование вызовов паники
    * stack_overflow.rs - тестирование переполнения стека
* Cargo.toml -  зависимости проекта
* rust-toolchain - указывает версию языка (ночную)
* x86_64-os.json - инструкции для сборки системы
## Для создания проекта использовался материал из:
* [Writing an OS in Rust](https://os.phil-opp.com)
* [Rust Atomics and Locks](https://marabos.nl/atomics/)