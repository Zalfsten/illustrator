//! Test-only counting global allocator (per-thread, opt-in).
//!
//! Compiled into the library's unit-test binary (`#[cfg(test)]` in `lib.rs`)
//! and into `tests/alloc_free.rs` via `#[path]`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct CountingAlloc;

thread_local! {
    /// Allocations on this thread while counting is enabled (`None` = off).
    static COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = COUNT.try_with(|c| c.set(c.get().map(|n| n + 1)));
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = COUNT.try_with(|c| c.set(c.get().map(|n| n + 1)));
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

/// Number of heap allocations `f` performs on this thread.
pub fn allocations(f: impl FnOnce()) -> usize {
    COUNT.with(|c| c.set(Some(0)));
    f();
    COUNT.with(|c| c.replace(None)).unwrap_or(0)
}

