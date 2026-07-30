//! JakartaAnnotationsRuntimeHints — Jakarta 注解运行时提示。

use std::sync::atomic::{AtomicU32, Ordering};

pub struct JakartaAnnotationsRuntimeHints {
    registered: AtomicU32,
}

impl JakartaAnnotationsRuntimeHints {
    pub fn new() -> Self { Self { registered: AtomicU32::new(0) } }
    pub fn register(&self) { self.registered.fetch_add(1, Ordering::SeqCst); }
    pub fn registered_count(&self) -> u32 { self.registered.load(Ordering::SeqCst) }
    pub fn reset(&self) { self.registered.store(0, Ordering::SeqCst); }
}
impl Default for JakartaAnnotationsRuntimeHints { fn default() -> Self { Self::new() } }
