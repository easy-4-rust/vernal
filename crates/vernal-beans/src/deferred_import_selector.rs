//! DeferredImportSelector — 延迟导入选择器。
use crate::import_selector::ImportSelector;

/// 延迟导入选择器 trait。
pub trait DeferredImportSelector: ImportSelector {
    fn get_import_group(&self) -> Option<String> { None }
}
