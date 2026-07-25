//! 类型安全的组件容器对象。

use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;
use vernal_core::SharedError;

use crate::{
    ComponentDefinition, ComponentKey, Dependency, Qualifier, Registry, ResolveError, Resolver,
    Scope, ScopeContext, ScopeKey, TraitBinding, component_definition::ErasedComponent,
    resolution_tracker::ResolutionTracker,
};

type SingletonCell = OnceLock<Result<ErasedComponent, ResolveError>>;

/// 隔离持有组件实例和作用域状态的运行时容器。
///
/// 注册表可以共享，而单例缓存始终属于容器实例。Vernal 不使用进程级组件表，
/// 因此多个应用上下文、租户容器和并行测试不会互相覆盖实例。
pub struct Container {
    registry: Registry,
    singletons: Mutex<HashMap<ComponentKey, Arc<SingletonCell>>>,
    resolutions: ResolutionTracker,
    owner: Arc<()>,
}

impl Container {
    /// 基于已校验注册表创建空实例容器。
    #[must_use]
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            singletons: Mutex::new(HashMap::new()),
            resolutions: ResolutionTracker::new(),
            owner: Arc::new(()),
        }
    }

    /// 进入由标记类型 `S` 识别的根自定义作用域。
    ///
    /// 返回的 Context 绑定当前 Container，不能交给另一个 Container 解析。作用域
    /// 所有者必须在生命周期结束时显式调用 [`ScopeContext::close`]。
    #[must_use]
    pub fn open_scope<S>(&self) -> Arc<ScopeContext>
    where
        S: 'static,
    {
        self.open_scope_with_cancellation::<S>(CancellationToken::new())
    }

    /// 使用调用方提供的取消令牌进入根自定义作用域。
    ///
    /// Adapter 可以传入请求、任务或租户生命周期已有的令牌；关闭 Scope 会取消
    /// 该令牌，外部取消也会立即阻止新的作用域组件解析。
    #[must_use]
    pub fn open_scope_with_cancellation<S>(
        &self,
        cancellation: CancellationToken,
    ) -> Arc<ScopeContext>
    where
        S: 'static,
    {
        ScopeContext::root(Arc::clone(&self.owner), ScopeKey::of::<S>(), cancellation)
    }

    /// 解析唯一注册的 `T` 类型组件。
    ///
    /// # Errors
    ///
    /// 组件缺失、存在歧义、工厂失败或结果类型错误时返回 [`ResolveError`]。
    pub fn resolve<T>(&self) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.resolve_typed(&Dependency::of::<T>(), &[], None)
    }

    /// 在显式自定义作用域链中解析唯一注册的 `T` 类型组件。
    ///
    /// Singleton 构造不会继承传入 Scope，避免长生命周期对象捕获 Request/Task
    /// 等短生命周期组件；Transient 会传播当前 Scope；Custom 定义按类型身份
    /// 选择当前或父 Scope。
    ///
    /// # Errors
    ///
    /// Scope 来自其他 Container、目标作用域未激活/不可用，或普通组件解析失败时
    /// 返回 [`ResolveError`]。
    pub fn resolve_in<T>(&self, scope: &ScopeContext) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.ensure_scope_owner(scope)?;
        self.resolve_typed(&Dependency::of::<T>(), &[], Some(scope))
    }

    /// 解析具有指定限定符的 `T` 类型组件。
    ///
    /// # Errors
    ///
    /// 组件缺失、工厂失败或结果类型错误时返回 [`ResolveError`]。
    pub fn resolve_qualified<T>(&self, qualifier: &Qualifier) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.resolve_typed(&Dependency::qualified::<T>(qualifier.clone()), &[], None)
    }

    /// 在显式自定义作用域链中解析带限定符组件。
    ///
    /// # Errors
    ///
    /// Scope 身份或组件解析失败时返回 [`ResolveError`]。
    pub fn resolve_qualified_in<T>(
        &self,
        qualifier: &Qualifier,
        scope: &ScopeContext,
    ) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.ensure_scope_owner(scope)?;
        self.resolve_typed(
            &Dependency::qualified::<T>(qualifier.clone()),
            &[],
            Some(scope),
        )
    }

    /// 解析指定 Trait Object 的唯一或 Primary 实现。
    ///
    /// # Errors
    ///
    /// 没有绑定、存在歧义、目标构造失败或绑定转换结果错误时返回
    /// [`ResolveError`]。
    pub fn resolve_trait<T>(&self) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_trait_typed(&Dependency::trait_of::<T>(), &[], None)
    }

    /// 在显式自定义作用域链中解析 Trait Object 的唯一或 Primary 实现。
    ///
    /// # Errors
    ///
    /// Scope 身份、Trait 选择或目标组件解析失败时返回 [`ResolveError`]。
    pub fn resolve_trait_in<T>(&self, scope: &ScopeContext) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_scope_owner(scope)?;
        self.resolve_trait_typed(&Dependency::trait_of::<T>(), &[], Some(scope))
    }

    /// 按限定符解析指定 Trait Object 实现。
    ///
    /// # Errors
    ///
    /// 没有精确命名绑定、目标构造失败或绑定转换结果错误时返回
    /// [`ResolveError`]。
    pub fn resolve_qualified_trait<T>(&self, qualifier: &Qualifier) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_trait_typed(
            &Dependency::trait_qualified::<T>(qualifier.clone()),
            &[],
            None,
        )
    }

    /// 在显式自定义作用域链中按限定符解析 Trait Object 实现。
    ///
    /// # Errors
    ///
    /// Scope 身份、Trait 选择或目标组件解析失败时返回 [`ResolveError`]。
    pub fn resolve_qualified_trait_in<T>(
        &self,
        qualifier: &Qualifier,
        scope: &ScopeContext,
    ) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_scope_owner(scope)?;
        self.resolve_trait_typed(
            &Dependency::trait_qualified::<T>(qualifier.clone()),
            &[],
            Some(scope),
        )
    }

    /// 按注册顺序解析指定 Trait Object 的全部实现。
    ///
    /// 没有绑定时返回空集合，而不是错误；任一已存在绑定的目标构造或转换失败时
    /// 返回 [`ResolveError`]。
    ///
    /// # Errors
    ///
    /// 任一绑定目标构造失败或绑定转换结果与 Trait 类型不一致时返回
    /// [`ResolveError`]。
    pub fn resolve_all_traits<T>(&self) -> Result<Vec<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_all_traits_typed(&Dependency::all_traits_of::<T>(), &[], None)
    }

    /// 在显式自定义作用域链中按注册顺序解析 Trait Object 的全部实现。
    ///
    /// # Errors
    ///
    /// Scope 身份或任一绑定目标解析失败时返回 [`ResolveError`]。
    pub fn resolve_all_traits_in<T>(
        &self,
        scope: &ScopeContext,
    ) -> Result<Vec<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_scope_owner(scope)?;
        self.resolve_all_traits_typed(&Dependency::all_traits_of::<T>(), &[], Some(scope))
    }

    /// 按依赖优先顺序构造全部单例。
    ///
    /// 普通 `IoC` 使用可以保持惰性；应用上下文会在 refresh 阶段调用此方法，
    /// 从而在接收流量前暴露组件构造错误。
    ///
    /// # Errors
    ///
    /// 返回首个无法完成构造的 [`ResolveError`]。
    pub fn warm_up(&self) -> Result<(), ResolveError> {
        for index in self.registry.ordered_indices().iter().copied() {
            let definition = &self.registry.definitions()[index];
            if definition.scope() == Scope::Singleton {
                self.resolve_definition(definition, &[], None)?;
            }
        }
        Ok(())
    }

    /// 返回容器使用的不可变注册表。
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// 返回尚未在当前 Container 中成功解析过的组件定义。
    ///
    /// 结果严格按 Registry 已验证的依赖优先构建顺序排列。应用上下文的
    /// [`Self::warm_up`] 会成功解析全部 Singleton，因此它们不再属于未使用定义；
    /// 尚未被请求的 Transient 和 Custom Scope 定义会一直保留，直到某次真实解析
    /// 成功。失败的候选选择、构造或 Scope 解析不会把定义错误标记为已使用。
    ///
    /// 该诊断是每 Container 隔离的历史快照，不使用进程级计数器，也不会把
    /// “依赖图没有入边”误判成未使用。
    #[must_use]
    pub fn unused_definitions(&self) -> Vec<String> {
        let resolved = self.resolutions.snapshot();
        self.registry
            .ordered_indices()
            .iter()
            .map(|index| &self.registry.definitions()[*index])
            .filter(|definition| !resolved.contains(definition.key()))
            .map(|definition| definition.key().to_string())
            .collect()
    }

    /// 完成候选选择、构造和类型恢复。
    pub(crate) fn resolve_typed<T>(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        let definition = self.select_definition(dependency, stack)?;
        let component = self.resolve_definition(definition, stack, scope)?;
        Arc::downcast::<T>(component).map_err(|_| ResolveError::TypeMismatch {
            component: definition.key().clone(),
        })
    }

    /// 为受限 Resolver 执行 Trait 单值解析。
    pub(crate) fn resolve_trait_typed<T>(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let binding = self.select_trait_binding(dependency, stack)?;
        self.resolve_binding(binding, stack, scope)
    }

    /// 为受限 Resolver 执行 Trait 全实现解析。
    pub(crate) fn resolve_all_traits_typed<T>(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<Vec<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.registry
            .bindings()
            .iter()
            .filter(|binding| binding.key().type_id == dependency.type_id)
            .map(|binding| self.resolve_binding(binding, stack, scope))
            .collect()
    }

    /// 按类型与限定符选择唯一组件定义。
    fn select_definition(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
    ) -> Result<&Arc<ComponentDefinition>, ResolveError> {
        let matches: Vec<&Arc<ComponentDefinition>> = self
            .registry
            .definitions()
            .iter()
            .filter(|definition| definition.key().type_id == dependency.type_id)
            .filter(|definition| {
                dependency.qualifier().is_none()
                    || definition.key().qualifier() == dependency.qualifier()
            })
            .collect();

        match matches.as_slice() {
            [definition] => Ok(*definition),
            [] => Err(ResolveError::NotFound {
                component: dependency.to_string(),
                path: Self::display_path(stack, Some(dependency.to_string())),
            }),
            _ => Err(ResolveError::Ambiguous {
                component: dependency.to_string(),
                candidates: matches
                    .iter()
                    .map(|definition| definition.key().to_string())
                    .collect(),
                path: Self::display_path(stack, Some(dependency.to_string())),
            }),
        }
    }

    /// 按 Trait 类型、限定符和 Primary 规则选择唯一绑定。
    fn select_trait_binding(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
    ) -> Result<&Arc<TraitBinding>, ResolveError> {
        let matches: Vec<&Arc<TraitBinding>> = self
            .registry
            .bindings()
            .iter()
            .filter(|binding| binding.key().type_id == dependency.type_id)
            .filter(|binding| {
                dependency.qualifier().is_none()
                    || binding.key().qualifier() == dependency.qualifier()
            })
            .collect();

        match matches.as_slice() {
            [binding] => Ok(*binding),
            [] => Err(ResolveError::NotFound {
                component: dependency.to_string(),
                path: Self::display_path(stack, Some(dependency.to_string())),
            }),
            _ if dependency.qualifier().is_none() => {
                let primary: Vec<&Arc<TraitBinding>> = matches
                    .iter()
                    .copied()
                    .filter(|binding| binding.is_primary())
                    .collect();
                match primary.as_slice() {
                    [binding] => Ok(*binding),
                    _ => Err(ResolveError::Ambiguous {
                        component: dependency.to_string(),
                        candidates: matches.iter().map(ToString::to_string).collect(),
                        path: Self::display_path(stack, Some(dependency.to_string())),
                    }),
                }
            }
            _ => Err(ResolveError::Ambiguous {
                component: dependency.to_string(),
                candidates: matches.iter().map(ToString::to_string).collect(),
                path: Self::display_path(stack, Some(dependency.to_string())),
            }),
        }
    }

    /// 解析绑定目标并恢复其 `Arc<dyn Trait>` 类型。
    fn resolve_binding<T>(
        &self,
        binding: &TraitBinding,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let definition = self
            .registry
            .definitions()
            .iter()
            .find(|definition| definition.key() == binding.target())
            .ok_or_else(|| ResolveError::NotFound {
                component: binding.target().to_string(),
                path: Self::display_path(stack, Some(binding.to_string())),
            })?;
        let component = self.resolve_definition(definition, stack, scope)?;
        let erased_trait =
            binding
                .upcast(component)
                .map_err(|target| ResolveError::TraitBindingTypeMismatch {
                    binding: binding.key().clone(),
                    target,
                })?;

        erased_trait
            .downcast_ref::<Arc<T>>()
            .cloned()
            .ok_or_else(|| ResolveError::TraitBindingTypeMismatch {
                binding: binding.key().clone(),
                target: binding.target().clone(),
            })
    }

    /// 根据作用域解析定义；单例使用每容器 `OnceLock` 保证并发只构造一次。
    fn resolve_definition(
        &self,
        definition: &Arc<ComponentDefinition>,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<ErasedComponent, ResolveError> {
        if let Some(position) = stack.iter().position(|key| key == definition.key()) {
            let mut path: Vec<String> = stack[position..].iter().map(ToString::to_string).collect();
            path.push(definition.key().to_string());
            return Err(ResolveError::CircularRuntime { path });
        }

        let result = match definition.scope() {
            Scope::Transient => self.construct(definition, stack, scope),
            Scope::Singleton => {
                let cell = {
                    let mut singletons = self
                        .singletons
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    singletons
                        .entry(definition.key().clone())
                        .or_insert_with(|| Arc::new(OnceLock::new()))
                        .clone()
                };
                // Singleton 的依赖解析故意不传播调用方 Scope。否则第一次恰好在
                // Request/Tenant 内解析的单例会永久捕获短生命周期对象。
                cell.get_or_init(|| self.construct(definition, stack, None))
                    .clone()
            }
            Scope::Custom(scope_key) => {
                let active_scope =
                    scope
                        .and_then(|scope| scope.find(scope_key))
                        .ok_or_else(|| ResolveError::ScopeNotActive {
                            component: definition.key().clone(),
                            scope: scope_key,
                        })?;
                active_scope.resolve_component(definition.key(), || {
                    // 只把匹配节点而不是最内层叶节点交给工厂：父 Scope 组件可以
                    // 依赖更长生命周期的祖先，但不能捕获更短生命周期的子组件。
                    self.construct(definition, stack, Some(active_scope))
                })
            }
        };

        // 只在完整作用域解析成功后写入追踪器：Singleton 缓存的构造错误、未激活
        // Custom Scope 和 Transient 工厂失败都保持“未使用”，诊断不会掩盖失败。
        if result.is_ok() {
            self.resolutions.record(definition.key());
        }
        result
    }

    /// 创建受限解析器并调用组件工厂。
    fn construct(
        &self,
        definition: &Arc<ComponentDefinition>,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<ErasedComponent, ResolveError> {
        let mut next_stack = stack.to_vec();
        next_stack.push(definition.key().clone());
        let resolver = Resolver::new(self, definition, &next_stack, scope);

        definition
            .create(&resolver)
            .map_err(|source| ResolveError::Construction {
                component: definition.key().clone(),
                source: SharedError::from(source),
            })
    }

    /// 生成包含当前解析栈和可选叶节点的诊断路径。
    fn display_path(stack: &[ComponentKey], leaf: Option<String>) -> Vec<String> {
        let mut path: Vec<String> = stack.iter().map(ToString::to_string).collect();
        if let Some(leaf) = leaf {
            path.push(leaf);
        }
        path
    }

    /// 拒绝把另一个 Container 创建的 `ScopeContext` 用作当前实例缓存。
    fn ensure_scope_owner(&self, scope: &ScopeContext) -> Result<(), ResolveError> {
        if scope.belongs_to(&self.owner) {
            Ok(())
        } else {
            Err(ResolveError::ScopeOwnerMismatch { scope: scope.key() })
        }
    }
}
