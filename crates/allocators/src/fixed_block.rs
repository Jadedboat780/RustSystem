use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use custom_types::spin_lock::SpinLock;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KB

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl Default for FixedSizeBlockAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

pub struct LockFixedSizeBlockAllocator(pub SpinLock<FixedSizeBlockAllocator>);

unsafe impl GlobalAlloc for LockFixedSizeBlockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.0.lock();
        match list_index(&layout) {
            Some(index) => match allocator.list_heads[index].take() {
                Some(node) => {
                    allocator.list_heads[index] = node.next.take();
                    node as *mut ListNode as *mut u8
                }
                None => {
                    let block_size = BLOCK_SIZES[index];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    allocator.fallback_alloc(layout)
                }
            },
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            let mut allocator = self.0.lock();
            match list_index(&layout) {
                Some(index) => {
                    let new_node = ListNode {
                        next: allocator.list_heads[index].take(),
                    };
                    assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                    assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                    let new_node_ptr = ptr as *mut ListNode;
                    new_node_ptr.write(new_node);
                    allocator.list_heads[index] = Some(&mut *new_node_ptr);
                }
                None => {
                    let ptr = ptr::NonNull::new(ptr).unwrap();
                    allocator.fallback_allocator.deallocate(ptr, layout);
                }
            }
        }
    }
}
