//! Trait 与具体组件之间的不可变绑定对象。

use std::{any::Any, fmt, sync::Arc};

use crate::{ComponentKey, Qualifier, TraitKey, factory::parsing::component_definition::ErasedComponent};

pub(crate) type ErasedTraitComponent = Arc<dyn Any + Send + Sync>;
type TraitUpcast =
    dyn Fn(ErasedComponent) -> Result<ErasedTraitComponent, ComponentKey> + Send + Sync + 'static;

/// 描述一个具体组件如何作为指定 Trait Object 被解析。
///
/// 转换函数在注册时由 Rust 类型系统检查，例如
/// `|service: Arc<EmailService>| service as Arc<dyn NotificationService>`。
/// 运行时只根据 `TypeId` 与 qualifier 选择绑定，不使用字符串反射，也不构造
/// 第二份业务对象。Singleton/Transient 语义仍由目标组件定义负责。
pub struct TraitBinding {
    key: TraitKey,
    target: ComponentKey,
    primary: bool,
    upcast: Arc<TraitUpcast>,
}

impl TraitBinding {
    /// 创建从具体组件 `C` 到 Trait Object `T` 的绑定。
    ///
    /// `upcast` 接收容器已经解析出的同一个 `Arc<C>`。调用方通常直接依赖 Rust
    /// 自动解引用强制转换返回 `Arc<T>`，不会复制组件内部状态。
    pub fn new<T, C, F>(upcast: F) -> Self
    where
        T: ?Sized + Send + Sync + 'static,
        C: Any + Send + Sync,
        F: Fn(Arc<C>) -> Arc<T> + Send + Sync + 'static,
    {
        let target = ComponentKey::of::<C>();
        let mismatch_target = target.clone();

        Self {
            key: TraitKey::of::<T>(),
            target,
            primary: false,
            upcast: Arc::new(move |component| {
                let concrete =
                    Arc::downcast::<C>(component).map_err(|_| mismatch_target.clone())?;
                let trait_object = upcast(concrete);

                // `Arc<T>` 自身是 Sized 值，可以安全放入 Any；解析时恢复的是这层
                // Arc，而不是尝试对 unsized Trait Object 本体做 Any downcast。
                Ok(Arc::new(trait_object) as ErasedTraitComponent)
            }),
        }
    }

    /// 为 Trait 绑定设置命名限定符。
    #[must_use]
    pub fn qualified(mut self, qualifier: Qualifier) -> Self {
        self.key = self.key.with_qualifier(qualifier);
        self
    }

    /// 指定绑定目标组件本身的限定符。
    ///
    /// 该方法用于同一具体 Rust 类型注册多个组件定义的场景；Trait qualifier 与
    /// 目标组件 qualifier 相互独立，前者负责选择绑定，后者负责定位实例。
    #[must_use]
    pub fn target_qualified(mut self, qualifier: Qualifier) -> Self {
        self.target = self.target.with_qualifier(qualifier);
        self
    }

    /// 将该实现标记为无限定符单值注入时的首选实现。
    ///
    /// 同一 Trait 最多允许一个 Primary；命名解析始终优先使用精确 qualifier，
    /// 不受 Primary 影响。
    #[must_use]
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// 返回 Trait 选择键。
    #[must_use]
    pub fn key(&self) -> &TraitKey {
        &self.key
    }

    /// 返回目标具体组件标识。
    #[must_use]
    pub fn target(&self) -> &ComponentKey {
        &self.target
    }

    /// 返回该绑定是否为首选实现。
    #[must_use]
    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// 将已解析具体组件转换为擦除后的 `Arc<dyn Trait>` 包装值。
    pub(crate) fn upcast(
        &self,
        component: ErasedComponent,
    ) -> Result<ErasedTraitComponent, ComponentKey> {
        (self.upcast)(component)
    }
}

impl fmt::Debug for TraitBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TraitBinding")
            .field("key", &self.key)
            .field("target", &self.target)
            .field("primary", &self.primary)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for TraitBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.key, self.target)?;
        if self.primary {
            formatter.write_str(" [primary]")?;
        }
        Ok(())
    }
}
