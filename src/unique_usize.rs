use std::sync::atomic::{AtomicUsize, Ordering};

pub fn get_unique_usize() -> usize {
    static VALUE: AtomicUsize = AtomicUsize::new(0);
    VALUE.fetch_add(1, Ordering::Relaxed)
}
