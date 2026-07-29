use crate::{lifecycle::Lifecycle, phased::Phased};
pub trait SmartLifecycle: Lifecycle + Phased {
    fn is_auto_startup(&self) -> bool {
        true
    }
}
