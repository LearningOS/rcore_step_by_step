# buddy-allocator

This crate use struct Vec, so you should **init heap** before using it.

```
// in Cargo.toml

[dependencies]
buddy_system_allocator = "0.1"
```

```
use buddy_system_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init_heap() {
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
    println!("heap init end");
}
```