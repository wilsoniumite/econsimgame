/// A heap for each thread, so that islands played side by side (sweep, ablate,
/// lab, the atlas) do not queue on the process's one heap (`heaps.rs` says why).
#[global_allocator]
static HEAPS: isostate_core::heaps::ThreadHeaps = isostate_core::heaps::ThreadHeaps;

fn main() {
    isostate_core::cli::main();
}
