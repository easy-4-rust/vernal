//! Vernal 各内核共享的错误基础类型。

use std::{error::Error, sync::Arc};

/// 用于类型擦除边界的独占错误对象。
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// 用于不可变诊断对象、可以低成本克隆的共享错误源。
pub type SharedError = Arc<dyn Error + Send + Sync + 'static>;
