//! A heap for each thread, for the commands that play many islands at once in
//! one process (`sweep`, `ablate`, `lab`, `atlas`) and for the test programs
//! that do the same.
//!
//! Why. A run makes some 1.75 million small allocations a second (a month
//! builds its tables afresh), and on Windows every thread of a process takes
//! them from the process's one heap, which serves one thread at a time. With
//! twelve islands on twelve threads a run took 133 to 143 seconds on the clock
//! where alone it takes 66; with 24 it took 474 to 524, and about five of the
//! machine's 48 processors were busy, the rest of the workers waiting their
//! turn at the heap (the second step back of 20 November 2026, its probes P1
//! and P2). Twelve processes of one island each took 70 seconds a run.
//!
//! What. [`ThreadHeaps`] is a global allocator that gives each thread a heap of
//! its own (`HeapCreate`), so that threads no longer queue on one. Each block
//! carries, in sixteen bytes before it, the heap it came from, so a block freed
//! or grown on another thread (a run's result handed back to the main thread)
//! goes back to its own heap, which takes it safely: a heap made this way still
//! serves any thread, it is only that, kept to one thread, it never makes one
//! wait. There are at most [`HEAPS`] heaps, handed out in turn to threads as
//! they first allocate and never destroyed, so that test programs, which start
//! and end a thread for every test, do not leave a heap behind for each; past
//! [`HEAPS`] threads alive at once, two threads share a heap, which is no worse
//! than one heap for all. A block aligned to more than sixteen bytes (rare:
//! nothing in the core asks for it) goes to the system's allocator as before.
//!
//! It changes where memory comes from, not what is computed: no result moves by
//! a byte (both golden tests, and a sweep compared with main's, say so). The
//! library does not install it; a program does, with
//!
//! ```text
//! #[global_allocator]
//! static HEAPS: isostate_core::heaps::ThreadHeaps = isostate_core::heaps::ThreadHeaps;
//! ```
//!
//! as the core's command does (`main.rs`). The viewer does not: its threads are
//! few and its simulation is one island. Elsewhere than Windows it is the
//! system's allocator unchanged.

use std::alloc::{GlobalAlloc, Layout, System};

/// How many heaps there may be: more than the threads any command runs at once
/// on a machine of 48 processors.
pub const HEAPS: usize = 64;

/// The global allocator: a heap for each thread on Windows, the system's
/// allocator elsewhere.
pub struct ThreadHeaps;

#[cfg(windows)]
mod imp {
    use super::HEAPS;
    use std::cell::Cell;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn HeapCreate(options: u32, initial: usize, maximum: usize) -> *mut c_void;
        fn HeapDestroy(heap: *mut c_void) -> i32;
        fn HeapAlloc(heap: *mut c_void, flags: u32, bytes: usize) -> *mut c_void;
        fn HeapReAlloc(heap: *mut c_void, flags: u32, block: *mut c_void, bytes: usize) -> *mut c_void;
        fn HeapFree(heap: *mut c_void, flags: u32, block: *mut c_void) -> i32;
        fn GetProcessHeap() -> *mut c_void;
    }

    /// `HeapAlloc`'s flag for memory that starts at nought.
    const ZERO: u32 = 0x8;

    /// The bytes before each block that name its heap. A heap's blocks are
    /// sixteen-byte aligned on 64-bit Windows, so the block after them is too.
    pub const HEADER: usize = 16;

    /// The heaps, made as they are first wanted.
    static TABLE: [AtomicPtr<c_void>; HEAPS] = [const { AtomicPtr::new(null_mut()) }; HEAPS];
    /// The next heap to hand to a thread.
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    thread_local! {
        /// This thread's heap, once it has one. A plain cell with nothing to drop,
        /// since an allocator may not use a thread-local that needs dropping.
        static MINE: Cell<*mut c_void> = const { Cell::new(null_mut()) };
    }

    /// The heap of the thread that asks; the process's heap while the thread is
    /// being taken down, or if no heap could be made.
    fn mine() -> *mut c_void {
        MINE.try_with(|m| {
            let h = m.get();
            if !h.is_null() {
                return h;
            }
            let k = NEXT.fetch_add(1, Ordering::Relaxed) % HEAPS;
            let mut h = TABLE[k].load(Ordering::Acquire);
            if h.is_null() {
                // SAFETY: a growable heap with the default options; it is kept for the
                // life of the process
                let made = unsafe { HeapCreate(0, 0, 0) };
                if made.is_null() {
                    // SAFETY: the process's heap always exists
                    return unsafe { GetProcessHeap() };
                }
                h = match TABLE[k].compare_exchange(null_mut(), made, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => made,
                    Err(other) => {
                        // another thread made this one first: use theirs
                        // SAFETY: `made` is ours and holds no block
                        unsafe { HeapDestroy(made) };
                        other
                    }
                };
            }
            m.set(h);
            h
        })
        // SAFETY: the process's heap always exists
        .unwrap_or_else(|_| unsafe { GetProcessHeap() })
    }

    /// A block of `size` bytes from this thread's heap, its heap written before it.
    ///
    /// SAFETY: the caller frees it with `free` or grows it with `grow`.
    pub unsafe fn take(size: usize, flags: u32) -> *mut u8 {
        let Some(bytes) = size.checked_add(HEADER) else { return null_mut() };
        let heap = mine();
        // SAFETY: `heap` is a live heap
        let b = unsafe { HeapAlloc(heap, flags, bytes) } as *mut u8;
        if b.is_null() {
            return b;
        }
        // SAFETY: the block holds HEADER bytes and more, and is aligned for a pointer
        unsafe {
            (b as *mut *mut c_void).write(heap);
            b.add(HEADER)
        }
    }

    pub unsafe fn take_zeroed(size: usize) -> *mut u8 {
        // SAFETY: as `take`
        unsafe { take(size, ZERO) }
    }

    pub unsafe fn take_plain(size: usize) -> *mut u8 {
        // SAFETY: as `take`
        unsafe { take(size, 0) }
    }

    /// Give back a block `take` gave, to the heap it came from.
    ///
    /// SAFETY: `p` came from `take` and is not used again.
    pub unsafe fn free(p: *mut u8) {
        // SAFETY: the header before `p` names the heap the block came from
        unsafe {
            let b = p.sub(HEADER);
            let heap = (b as *mut *mut c_void).read();
            HeapFree(heap, 0, b as *mut c_void);
        }
    }

    /// Grow or shrink a block `take` gave, within the heap it came from.
    ///
    /// SAFETY: `p` came from `take`; after this only the block returned is used.
    pub unsafe fn grow(p: *mut u8, size: usize) -> *mut u8 {
        let Some(bytes) = size.checked_add(HEADER) else { return null_mut() };
        // SAFETY: the header before `p` names the heap the block came from, and a
        // heap moves the header with the block
        unsafe {
            let b = p.sub(HEADER);
            let heap = (b as *mut *mut c_void).read();
            let nb = HeapReAlloc(heap, 0, b as *mut c_void, bytes) as *mut u8;
            if nb.is_null() {
                return nb;
            }
            nb.add(HEADER)
        }
    }
}

#[cfg(windows)]
// SAFETY: every block of sixteen-byte alignment or less comes from `imp::take`
// and goes back through `imp::free` or `imp::grow`, which find its heap in its
// header; every other block is the system allocator's from end to end, since a
// block is given back with the layout it was taken with
unsafe impl GlobalAlloc for ThreadHeaps {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if l.align() > imp::HEADER {
            return unsafe { System.alloc(l) };
        }
        unsafe { imp::take_plain(l.size()) }
    }

    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        if l.align() > imp::HEADER {
            return unsafe { System.alloc_zeroed(l) };
        }
        unsafe { imp::take_zeroed(l.size()) }
    }

    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        if l.align() > imp::HEADER {
            return unsafe { System.dealloc(p, l) };
        }
        unsafe { imp::free(p) }
    }

    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        if l.align() > imp::HEADER {
            return unsafe { System.realloc(p, l, new) };
        }
        unsafe { imp::grow(p, new) }
    }
}

#[cfg(not(windows))]
// SAFETY: the system's allocator, unchanged
unsafe impl GlobalAlloc for ThreadHeaps {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        unsafe { System.realloc(p, l, new) }
    }
}
