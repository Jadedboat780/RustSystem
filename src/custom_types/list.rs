use alloc::boxed::Box;

type Link<T> = Option<Box<Node<T>>>;

// представление односвязного списка
#[derive(Debug)]
pub struct List<T> {
    head: Link<T>,
}

// представление узла
#[derive(Debug)]
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    // инициализация списка
    pub fn new() -> Self {
        List { head: None }
    }

    // добавление элемента в начало списка
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    // удаляет элемент из начала списка и возвращает его
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    // возвращает ссылку на первый элемент
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }

    // возвращает мутабельную ссылку на первый элемент
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }
}

// реализация диструктора
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}
