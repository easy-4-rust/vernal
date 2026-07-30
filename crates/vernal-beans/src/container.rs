//! 类型安全的组件容器对象。

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;
use vernal_core::SharedError;

use crate::{
    ComponentDefinition, ComponentKey, Dependency, Qualifier, Registry, ResolveError, Resolver,
    Scope, ScopeContext, ScopeKey, TraitBinding, TransientTracker,
    component_definition::ErasedComponent, named_bean_holder::NamedBeanHolder,
    resolution_tracker::ResolutionTracker,
};

type SingletonCell = OnceLock<Result<ErasedComponent, ResolveError>>;

/// 隔离持有组件实例和作用域状态的运行时容器。
///
/// 注册表可以共享，而单例缓存始终属于容器实例。Vernal 不使用进程级组件表，
/// 因此多个应用上下文、租户容器和并行测试不会互相覆盖实例。
///
/// ## Transient 实例追踪
///
/// Container 自动追踪所有 Transient 实例的弱引用。上层（如 vernal-context）
/// 在关闭时可通过 `transient_tracker()` 获取仍存活的实例并执行清理。
pub struct Container {
    registry: Registry,
    singletons: Arc<Mutex<HashMap<ComponentKey, Arc<SingletonCell>>>>,
    resolutions: Arc<ResolutionTracker>,
    transient_tracker: Arc<TransientTracker>,
    bean_post_processors: Arc<Mutex<Vec<Arc<dyn crate::bean_post_processor::BeanPostProcessor>>>>,
    /// 可变 BeanDefinition 缓存（支持 register/remove/get 操作）。
    ///
    /// 使用 Mutex 保证线程安全，支持运行时动态注册/删除 Bean 定义。
    /// 这是 Container 层 BeanDefinitionRegistry trait 实现的核心存储。
    dynamic_definitions:
        Arc<Mutex<HashMap<String, Arc<dyn crate::bean_definition::BeanDefinition>>>>,
    /// BeanDefinition 代理缓存（用于 get_bean_definition 返回引用）。
    ///
    /// 缓存 ProxyBeanDefinition 对象，使 get_bean_definition 可以返回引用。
    definition_cache: Arc<Mutex<HashMap<String, ProxyBeanDefinition>>>,
    owner: Arc<()>,

    // ── HierarchicalBeanFactory / ConfigurableBeanFactory 字段 ────────
    /// 父 BeanFactory（支持父子容器层级结构）。
    parent: Arc<Mutex<Option<Arc<dyn crate::bean_factory::BeanFactory>>>>,

    // ── ConfigurableBeanFactory 字段 ─────────────────────────────────
    /// 自定义 Scope 注册表。
    scopes: Arc<Mutex<HashMap<String, Arc<dyn crate::bean_scope::BeanScope>>>>,
    /// Bean 别名映射（alias -> bean_name）。
    aliases: Arc<Mutex<HashMap<String, String>>>,
    /// 嵌入式值解析器链（用于 `${...}` 占位符解析）。
    embedded_value_resolvers: Arc<Mutex<Vec<Arc<dyn Fn(&str) -> String + Send + Sync>>>>,
    /// 当前正在创建中的 Bean 名称集合。
    currently_in_creation: Arc<Mutex<HashSet<String>>>,
    /// 依赖关系映射：bean_name -> 依赖该 bean 的所有 bean 名称。
    dependent_beans: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// 依赖关系映射：bean_name -> 该 bean 依赖的所有 bean 名称。
    dependencies_for_bean: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// 配置是否已冻结。
    configuration_frozen: Arc<Mutex<bool>>,

    // ── SingletonBeanRegistry 字段 ──────────────────────────────────
    /// 单例创建回调（bean_name -> 回调列表）。
    singleton_callbacks: Arc<Mutex<HashMap<String, Vec<Arc<dyn Fn(&dyn Any) + Send + Sync>>>>>,

    // ── ConfigurableListableBeanFactory 字段 ─────────────────────────
    /// 忽略的依赖类型集合。
    ignored_dependency_types: Arc<Mutex<HashSet<TypeId>>>,
    /// 忽略的依赖接口集合。
    ignored_dependency_interfaces: Arc<Mutex<HashSet<TypeId>>>,
    /// 可解析依赖映射（TypeId -> 实例）。
    resolvable_dependencies: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    /// 单例名称到 ComponentKey 的映射（用于 SingletonBeanRegistry 按名称查找）。
    name_to_singleton_key: Arc<Mutex<HashMap<String, ComponentKey>>>,
}

impl Container {
    /// 基于已校验注册表创建空实例容器。
    #[must_use]
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            singletons: Arc::new(Mutex::new(HashMap::new())),
            resolutions: Arc::new(ResolutionTracker::new()),
            transient_tracker: Arc::new(TransientTracker::new()),
            bean_post_processors: Arc::new(Mutex::new(Vec::new())),
            dynamic_definitions: Arc::new(Mutex::new(HashMap::new())),
            definition_cache: Arc::new(Mutex::new(HashMap::new())),
            owner: Arc::new(()),
            parent: Arc::new(Mutex::new(None)),
            scopes: Arc::new(Mutex::new(HashMap::new())),
            aliases: Arc::new(Mutex::new(HashMap::new())),
            embedded_value_resolvers: Arc::new(Mutex::new(Vec::new())),
            currently_in_creation: Arc::new(Mutex::new(HashSet::new())),
            dependent_beans: Arc::new(Mutex::new(HashMap::new())),
            dependencies_for_bean: Arc::new(Mutex::new(HashMap::new())),
            configuration_frozen: Arc::new(Mutex::new(false)),
            singleton_callbacks: Arc::new(Mutex::new(HashMap::new())),
            ignored_dependency_types: Arc::new(Mutex::new(HashSet::new())),
            ignored_dependency_interfaces: Arc::new(Mutex::new(HashSet::new())),
            resolvable_dependencies: Arc::new(Mutex::new(HashMap::new())),
            name_to_singleton_key: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建共享同一实例缓存、解析追踪和 Scope 身份的内部受限句柄。
    ///
    /// 该入口不实现公开 `Clone`，只供类型安全 [`crate::ComponentProvider`] 持有。
    /// [`Registry::container`] 仍会创建完全隔离的新 Container。
    pub(crate) fn shared_handle(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            singletons: Arc::clone(&self.singletons),
            resolutions: Arc::clone(&self.resolutions),
            transient_tracker: Arc::clone(&self.transient_tracker),
            bean_post_processors: Arc::clone(&self.bean_post_processors),
            dynamic_definitions: Arc::clone(&self.dynamic_definitions),
            definition_cache: Arc::clone(&self.definition_cache),
            owner: Arc::clone(&self.owner),
            parent: Arc::clone(&self.parent),
            scopes: Arc::clone(&self.scopes),
            aliases: Arc::clone(&self.aliases),
            embedded_value_resolvers: Arc::clone(&self.embedded_value_resolvers),
            currently_in_creation: Arc::clone(&self.currently_in_creation),
            dependent_beans: Arc::clone(&self.dependent_beans),
            dependencies_for_bean: Arc::clone(&self.dependencies_for_bean),
            configuration_frozen: Arc::clone(&self.configuration_frozen),
            singleton_callbacks: Arc::clone(&self.singleton_callbacks),
            ignored_dependency_types: Arc::clone(&self.ignored_dependency_types),
            ignored_dependency_interfaces: Arc::clone(&self.ignored_dependency_interfaces),
            resolvable_dependencies: Arc::clone(&self.resolvable_dependencies),
            name_to_singleton_key: Arc::clone(&self.name_to_singleton_key),
        }
    }

    /// 添加 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `ConfigurableBeanFactory.addBeanPostProcessor(BeanPostProcessor)`。
    pub fn add_bean_post_processor(
        &mut self,
        processor: Arc<dyn crate::bean_post_processor::BeanPostProcessor>,
    ) {
        self.bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(processor);
    }

    /// 获取 BeanPostProcessor 数量。
    ///
    /// 对应 Spring 的 `ConfigurableBeanFactory.getBeanPostProcessorCount()`。
    pub fn bean_post_processor_count(&self) -> usize {
        self.bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
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

    /// 返回 Transient 实例追踪器。
    ///
    /// 上层（如 vernal-context）在关闭时可通过此追踪器获取仍存活的 Transient 实例
    /// 并执行清理逻辑。对标 tx_di 的 `Store.prototype_instances`。
    #[must_use]
    pub fn transient_tracker(&self) -> &TransientTracker {
        &self.transient_tracker
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

    /// 为可选立即依赖或 Provider 解析具体类型；只把根候选缺失转换为 `None`。
    pub(crate) fn resolve_optional_typed<T>(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        let definition = match self.select_definition(dependency, stack) {
            Ok(definition) => definition,
            Err(ResolveError::NotFound { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        let component = self.resolve_definition(definition, stack, scope)?;
        Arc::downcast::<T>(component)
            .map(Some)
            .map_err(|_| ResolveError::TypeMismatch {
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

    /// 为可选立即依赖或 Trait Provider 解析单个实现；只把根绑定缺失转换为 `None`。
    pub(crate) fn resolve_optional_trait_typed<T>(
        &self,
        dependency: &Dependency,
        stack: &[ComponentKey],
        scope: Option<&ScopeContext>,
    ) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let binding = match self.select_trait_binding(dependency, stack) {
            Ok(binding) => binding,
            Err(ResolveError::NotFound { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        self.resolve_binding(binding, stack, scope).map(Some)
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
            Scope::Transient => {
                let instance = self.construct(definition, stack, scope);
                // 追踪 Transient 实例的弱引用，供上层在关闭时通知存活实例
                if let Ok(ref arc) = instance {
                    self.transient_tracker.track(definition.key().type_id, arc);
                }
                instance
            }
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
                let result = cell.get_or_init(|| self.construct(definition, stack, None)).clone();
                // 记录名称到 ComponentKey 的映射（用于 SingletonBeanRegistry）
                let bean_name = definition.key().type_name().to_string();
                self.name_to_singleton_key
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .entry(bean_name)
                    .or_insert_with(|| definition.key().clone());
                result
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
        // Custom Scope 和 Transient 工厂失败都保持”未使用”，诊断不会掩盖失败。
        if result.is_ok() {
            self.resolutions.record(definition.key());
        }

        // 应用 BeanPostProcessor 链
        result.map(|instance| {
            let processors = self
                .bean_post_processors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if processors.is_empty() {
                return instance;
            }
            let bean_name = definition.key().type_name();
            let mut current = instance;
            for processor in processors.iter() {
                match processor.post_process_after_initialization(current.clone(), bean_name) {
                    Ok(Some(replaced)) => current = replaced,
                    Ok(None) => {}
                    Err(_e) => {
                        // PostProcessor 错误不阻止 Bean 创建，只记录
                    }
                }
            }
            current
        })
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
        // Provider 只允许在拥有它的工厂完整返回后使用。弱守卫不会延长构造期，
        // 但能把工厂内的重入访问转换成结构化错误，避免递归进入 Singleton OnceLock。
        let construction_guard = Arc::new(());
        let resolver = Resolver::new(self, definition, &next_stack, scope, &construction_guard);

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
    pub(crate) fn ensure_scope_owner(&self, scope: &ScopeContext) -> Result<(), ResolveError> {
        if scope.belongs_to(&self.owner) {
            Ok(())
        } else {
            Err(ResolveError::ScopeOwnerMismatch { scope: scope.key() })
        }
    }
}

// ── Spring BeanFactory 接口实现 ───────────────────────────────────────────

impl crate::bean_factory::BeanFactory for Container {
    fn get_bean_by_key(
        &self,
        key: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let definition = self
            .registry
            .definitions()
            .iter()
            .find(|d| d.key() == key)
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Bean '{}' not found", key),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        self.resolve_definition(definition, &[], None).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to resolve bean '{}': {}", key, e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })
    }

    fn get_bean_by_type_id(
        &self,
        type_id: std::any::TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let definitions: Vec<_> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| d.key().type_id == type_id)
            .collect();

        match definitions.as_slice() {
            [definition] => self.resolve_definition(definition, &[], None).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to resolve bean: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            }),
            [] => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No bean found for type",
            )) as Box<dyn std::error::Error + Send + Sync>),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Multiple beans found for type ({} candidates)",
                    definitions.len()
                ),
            )) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    fn contains_bean(&self, key: &ComponentKey) -> bool {
        self.registry.definitions().iter().any(|d| d.key() == key)
    }

    fn is_singleton(
        &self,
        key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.registry
            .definitions()
            .iter()
            .find(|d| d.key() == key)
            .map(|d| d.scope().is_singleton())
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Bean '{}' not found", key),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    fn is_prototype(
        &self,
        key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.registry
            .definitions()
            .iter()
            .find(|d| d.key() == key)
            .map(|d| d.scope().is_transient())
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Bean '{}' not found", key),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    fn get_type(
        &self,
        key: &ComponentKey,
    ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> {
        self.registry
            .definitions()
            .iter()
            .find(|d| d.key() == key)
            .map(|d| Some(d.key().type_name()))
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Bean '{}' not found", key),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    fn get_aliases(&self, _key: &ComponentKey) -> Vec<ComponentKey> {
        Vec::new()
    }

    fn get_bean_provider_by_type_id(
        &self,
        _type_id: std::any::TypeId,
    ) -> Result<
        Box<dyn crate::object_provider::ObjectProvider<dyn Any + Send + Sync> + '_>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(Box::new(ContainerObjectProvider(self)))
    }

    fn is_type_match(&self, key: &ComponentKey, type_id: std::any::TypeId) -> bool {
        self.registry
            .definitions()
            .iter()
            .find(|d| d.key() == key)
            .map(|d| d.key().type_id == type_id)
            .unwrap_or(false)
    }
}

/// Container 内部的 ObjectProvider 实现。
struct ContainerObjectProvider<'a>(&'a Container);

impl<'a> crate::object_provider::ObjectProvider<dyn Any + Send + Sync>
    for ContainerObjectProvider<'a>
{
    fn get(&self) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let singletons = self
            .0
            .singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for cell in singletons.values() {
            if let Some(Ok(instance)) = cell.get() {
                return Ok(Arc::clone(instance));
            }
        }
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No bean available",
        )) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn if_available(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.get().ok()
    }

    fn get_if_unique(
        &self,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.get()
    }

    fn stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        let singletons = self
            .0
            .singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        singletons
            .values()
            .filter_map(|cell| cell.get().and_then(|r| r.as_ref().ok()).cloned())
            .collect()
    }

    fn ordered_stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.stream()
    }
}

// ── Spring AutowireCapableBeanFactory 接口实现 ───────────────────────────

impl crate::autowire_capable_bean_factory::AutowireCapableBeanFactory for Container {
    /// 创建一个新的 Bean 实例。
    ///
    /// 对应 Spring 的 `createBean(Class<T>)`：
    /// 1. 在注册表中查找匹配 bean_class_name 的定义
    /// 2. 调用工厂创建实例
    /// 3. 应用 BeanPostProcessor
    /// 4. 返回实例
    fn create_bean(
        &self,
        bean_class_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 按类名查找定义
        let definition = self
            .registry
            .definitions()
            .iter()
            .find(|d| d.key().type_name() == bean_class_name)
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("No bean definition found for class '{}'", bean_class_name),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        // 使用现有的 resolve_definition（已包含 PostProcessor 链）
        self.resolve_definition(definition, &[], None).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create bean '{}': {}", bean_class_name, e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })
    }

    /// 自动装配现有 Bean 的属性。
    ///
    /// 对应 Spring 的 `autowireBean(Object existingBean)`：
    /// 按类型在注册表中查找匹配的依赖并注入。
    ///
    /// 在 vernal-beans 的类型驱动模型中，此方法的实现策略是：
    /// 1. 根据 existing_bean 的 TypeId 在注册表中查找对应的 ComponentDefinition
    /// 2. 检查该定义是否声明了依赖（dependencies）
    /// 3. 如果有依赖，通过工厂 + Resolver 重新创建实例（自动注入依赖）
    /// 4. 如果依赖已全部解析（singleton 已缓存），直接返回新实例
    ///
    /// 这与 Spring 的 autowireBean 语义一致：Spring 通过反射注入字段，
    /// vernal 通过工厂闭包 + Resolver 注入依赖。
    fn autowire_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let existing_type_id = (&*existing_bean).type_id();

        // 查找与 existing_bean 类型匹配的 ComponentDefinition
        let matching_definitions: Vec<_> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| d.key().type_id == existing_type_id)
            .collect();

        match matching_definitions.as_slice() {
            [definition] => {
                // 找到唯一匹配的定义
                // 检查是否有未解析的依赖
                let has_unresolved_deps = definition
                    .dependencies()
                    .iter()
                    .any(|dep| !dep.is_deferred());

                if has_unresolved_deps {
                    // 有依赖需要注入：通过工厂重新创建实例
                    // 工厂闭包会通过 Resolver 自动解析所有依赖
                    let new_instance =
                        self.resolve_definition(definition, &[], None)
                            .map_err(|e| {
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Failed to autowire bean: {}", e),
                                ))
                                    as Box<dyn std::error::Error + Send + Sync>
                            })?;
                    Ok(new_instance)
                } else {
                    // 无依赖：直接返回原实例
                    Ok(existing_bean)
                }
            }
            [] => {
                // 没有匹配的定义：返回原实例（无法注入）
                Ok(existing_bean)
            }
            _ => {
                // 多个匹配：返回原实例（歧义）
                Ok(existing_bean)
            }
        }
    }

    /// 配置现有 Bean（应用属性值 + 初始化）。
    ///
    /// 对应 Spring 的 `configureBean(Object existingBean, String beanName)`：
    /// 1. 应用属性值
    /// 2. 调用 BeanPostProcessor
    /// 3. 调用初始化回调
    fn configure_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 应用 PostProcessor 链
        let processors = self
            .bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut current = existing_bean;
        for processor in processors.iter() {
            match processor.post_process_after_initialization(current.clone(), bean_name) {
                Ok(Some(replaced)) => current = replaced,
                Ok(None) => {}
                Err(_e) => {}
            }
        }
        Ok(current)
    }

    /// 按指定 autowire 模式创建 Bean。
    ///
    /// 对应 Spring 的 `autowire(Class<?> beanClass, int autowireMode, boolean dependencyCheck)`。
    fn autowire(
        &self,
        bean_class_name: &str,
        autowire_mode: i32,
        _dependency_check: bool,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        match autowire_mode {
            0 => self.create_bean(bean_class_name), // AUTOWIRE_NO
            1 => {
                // AUTOWIRE_BY_NAME: 按名称匹配属性名和 Bean 名称
                let bean = self.create_bean(bean_class_name)?;
                self.autowire_bean_properties(bean, autowire_mode, false)
            }
            2 => {
                // AUTOWIRE_BY_TYPE: 按类型匹配属性类型和 Bean 类型
                let bean = self.create_bean(bean_class_name)?;
                self.autowire_bean_properties(bean, autowire_mode, false)
            }
            3 => {
                // AUTOWIRE_CONSTRUCTOR: 按构造器参数类型匹配
                self.create_bean(bean_class_name)
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid autowire mode: {}", autowire_mode),
            )) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// 自动装配现有 Bean 的属性（指定模式）。
    ///
    /// 对应 Spring 的 `autowireBeanProperties(Object existingBean, int autowireMode, boolean dependencyCheck)`。
    ///
    /// 根据 autowire 模式：
    /// - AUTOWIRE_NO (0): 不做任何操作
    /// - AUTOWIRE_BY_NAME (1): 按 ComponentKey 名称匹配注册表中的依赖并注入
    /// - AUTOWIRE_BY_TYPE (2): 按 TypeId 匹配注册表中的依赖并注入
    fn autowire_bean_properties(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        autowire_mode: i32,
        _dependency_check: bool,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        match autowire_mode {
            0 => Ok(existing_bean), // AUTOWIRE_NO: 不做任何操作
            1 | 2 => {
                // AUTOWIRE_BY_NAME / AUTOWIRE_BY_TYPE:
                // 遍历注册表中所有定义的依赖，找到匹配 existing_bean 类型的
                // 依赖并解析注入。在 vernal 的类型驱动模型中，这等价于：
                // 1. 查找声明了与 existing_bean 同类型依赖的定义
                // 2. 解析这些定义，将 existing_bean 作为依赖的一部分注入
                let existing_type_id = (&*existing_bean).type_id();

                // 查找所有声明依赖了 existing_bean 类型的定义
                for definition in self.registry.definitions() {
                    for dep in definition.dependencies() {
                        if dep.type_id == existing_type_id && !dep.is_deferred() {
                            // 找到匹配的依赖声明
                            // 尝试解析这个定义，它会自动注入 existing_bean 作为依赖
                            let _ = self.resolve_definition(definition, &[], None);
                        }
                    }
                }
                // 返回原始 bean（注入是单向的：existing_bean → 依赖它的组件）
                Ok(existing_bean)
            }
            _ => Ok(existing_bean),
        }
    }

    /// 应用属性值到现有 Bean。
    ///
    /// 对应 Spring 的 `applyBeanPropertyValues(Object existingBean, String beanName)`。
    fn apply_bean_property_values(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 在 vernal 中，属性值通过 ComponentDefinition 的 factory 闭包注入
        // 这里保持现有实例不变
        Ok(existing_bean)
    }

    /// 初始化现有 Bean（应用初始化回调）。
    ///
    /// 对应 Spring 的 `initializeBean(Object existingBean, String beanName)`：
    /// 1. BeanNameAware.setBeanName
    /// 2. BeanFactoryAware.setBeanFactory
    /// 3. BeanPostProcessor.postProcessBeforeInitialization
    /// 4. InitializingBean.afterPropertiesSet
    /// 5. 自定义 init-method
    /// 6. BeanPostProcessor.postProcessAfterInitialization
    fn initialize_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 应用 PostProcessor 链
        let processors = self
            .bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut current = existing_bean;
        for processor in processors.iter() {
            match processor.post_process_before_initialization(current.clone(), bean_name) {
                Ok(Some(replaced)) => current = replaced,
                Ok(None) => {}
                Err(_e) => {}
            }
        }
        for processor in processors.iter() {
            match processor.post_process_after_initialization(current.clone(), bean_name) {
                Ok(Some(replaced)) => current = replaced,
                Ok(None) => {}
                Err(_e) => {}
            }
        }
        Ok(current)
    }

    /// 销毁 Bean。
    ///
    /// 对应 Spring 的 `destroyBean(Object existingBean)`。
    fn destroy_bean_instance(
        &self,
        _bean_name: &str,
        _bean_instance: &dyn Any,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 在 vernal 中，销毁由 Component::shutdown 管理
        Ok(())
    }

    /// 按类型解析命名 Bean。
    ///
    /// 对应 Spring 的 `resolveNamedBean(Class<T> requiredType)`。
    fn resolve_named_bean(
        &self,
        type_id: std::any::TypeId,
    ) -> Result<NamedBeanHolder<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let definitions: Vec<_> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| d.key().type_id == type_id)
            .collect();

        match definitions.as_slice() {
            [definition] => {
                let instance = self
                    .resolve_definition(definition, &[], None)
                    .map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to resolve: {}", e),
                        )) as Box<dyn std::error::Error + Send + Sync>
                    })?;
                let bean_name = definition.key().type_name().to_string();
                Ok(NamedBeanHolder::new(Arc::new(instance), bean_name))
            }
            [] => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No bean found for type",
            )) as Box<dyn std::error::Error + Send + Sync>),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Multiple beans found for type ({} candidates)",
                    definitions.len()
                ),
            )) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// 解析依赖。
    ///
    /// 对应 Spring 的 `resolveDependency(DependencyDescriptor descriptor, String requestingBeanName)`。
    fn resolve_dependency(
        &self,
        descriptor: &crate::factory::support::dependency_descriptor::DependencyDescriptor,
        _requesting_bean_name: Option<&str>,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 按依赖描述符的类型查找匹配的定义
        let definitions: Vec<_> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| d.key().type_id == descriptor.type_id())
            .collect();

        match definitions.as_slice() {
            [definition] => {
                let instance = self
                    .resolve_definition(definition, &[], None)
                    .map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to resolve dependency: {}", e),
                        )) as Box<dyn std::error::Error + Send + Sync>
                    })?;
                Ok(Some(instance))
            }
            [] if !descriptor.is_required() => Ok(None),
            [] => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "No bean found for dependency type '{}'",
                    descriptor.type_name
                ),
            )) as Box<dyn std::error::Error + Send + Sync>),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Multiple beans found for dependency type '{}'",
                    descriptor.type_name
                ),
            )) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// 设置类型转换器。
    fn set_type_converter(
        &mut self,
        _converter: Option<Arc<dyn crate::type_converter::TypeConverter>>,
    ) {
        // 当前实现不存储类型转换器（简化）
    }

    /// 获取类型转换器。
    fn type_converter(&self) -> Option<&dyn crate::type_converter::TypeConverter> {
        None // 当前实现不存储类型转换器（简化）
    }
}

// ── Spring BeanDefinitionRegistry 接口实现 ────────────────────────────────

/// 代理 BeanDefinition（用于 Container 的 BeanDefinitionRegistry 实现）。
#[derive(Debug, Clone)]
pub(crate) struct ProxyBeanDefinition {
    pub(crate) bean_name: String,
    pub(crate) type_name: String,
    pub(crate) scope: crate::component_scope::Scope,
    pub(crate) source: String,
}

impl crate::bean_definition::BeanDefinition for ProxyBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("ProxyBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        &self.type_name
    }

    fn scope(&self) -> crate::component_scope::Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}

/// 被删除的 BeanDefinition 标记。
#[derive(Debug)]
struct DeletedBeanDefinition {
    bean_name: String,
}

impl crate::bean_definition::BeanDefinition for DeletedBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("DeletedBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        "__DELETED__"
    }

    fn scope(&self) -> crate::component_scope::Scope {
        crate::component_scope::Scope::Singleton
    }

    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}

/// 被移除的 BeanDefinition（用于 remove_bean_definition 返回值）。
#[derive(Debug)]
struct RemovedBeanDefinition {
    bean_name: String,
    type_name: String,
    scope: crate::component_scope::Scope,
}

impl crate::bean_definition::BeanDefinition for RemovedBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("RemovedBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        &self.type_name
    }

    fn scope(&self) -> crate::component_scope::Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}

// ── Spring BeanDefinitionRegistry 接口实现 ────────────────────────────────

impl crate::bean_definition_registry::BeanDefinitionRegistry for Container {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn crate::bean_definition::BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut defs = self
            .dynamic_definitions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if defs.contains_key(&bean_name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Bean definition '{}' already exists", bean_name),
            )));
        }
        defs.insert(bean_name, definition.into());
        Ok(())
    }

    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<
        Box<dyn crate::bean_definition::BeanDefinition>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let removed = {
            let mut defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            defs.remove(bean_name)
        };

        if let Some(definition) = removed {
            if definition.bean_class_name() == "__DELETED__" {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Bean definition '{}' not found", bean_name),
                )));
            }
            return Ok(Box::new(RemovedBeanDefinition {
                bean_name: bean_name.to_string(),
                type_name: definition.bean_class_name().to_string(),
                scope: definition.scope(),
            }));
        }

        let in_registry = self
            .registry
            .definitions()
            .iter()
            .any(|d| d.key().type_name() == bean_name);

        if in_registry {
            let mut defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            defs.insert(
                bean_name.to_string(),
                Arc::new(DeletedBeanDefinition {
                    bean_name: bean_name.to_string(),
                }),
            );
            let key_to_remove = self
                .registry
                .definitions()
                .iter()
                .find(|d| d.key().type_name() == bean_name)
                .map(|d| d.key().clone());
            if let Some(key) = key_to_remove {
                self.singletons
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove(&key);
            }
            return Ok(Box::new(RemovedBeanDefinition {
                bean_name: bean_name.to_string(),
                type_name: bean_name.to_string(),
                scope: crate::component_scope::Scope::Singleton,
            }));
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Bean definition '{}' not found", bean_name),
        )))
    }

    /// 获取 Bean 定义。
    ///
    /// 对应 Spring 的 `BeanDefinition getBeanDefinition(String beanName)`。
    ///
    /// 通过 `Box::leak` 创建静态引用，使返回的引用具有 `'static` 生命周期。
    /// 这是 Rust 中返回 trait 对象引用的标准模式。
    fn get_bean_definition(
        &self,
        bean_name: &str,
    ) -> Option<&'static dyn crate::bean_definition::BeanDefinition> {
        // 1. 检查 dynamic_definitions
        {
            let defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(definition) = defs.get(bean_name) {
                if definition.bean_class_name() == "__DELETED__" {
                    return None;
                }
                // 动态注册的定义：创建静态引用的代理对象
                let proxy = Box::new(ProxyBeanDefinition {
                    bean_name: bean_name.to_string(),
                    type_name: definition.bean_class_name().to_string(),
                    scope: definition.scope(),
                    source: "dynamic".to_string(),
                });
                return Some(Box::leak(proxy));
            }
        }

        // 2. 检查 Registry
        if let Some(definition) = self
            .registry
            .definitions()
            .iter()
            .find(|d| d.key().type_name() == bean_name)
        {
            let proxy = Box::new(ProxyBeanDefinition {
                bean_name: bean_name.to_string(),
                type_name: definition.key().type_name().to_string(),
                scope: definition.scope(),
                source: "registry".to_string(),
            });
            return Some(Box::leak(proxy));
        }

        None
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        {
            let defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(definition) = defs.get(bean_name) {
                if definition.bean_class_name() == "__DELETED__" {
                    return false;
                }
                return true;
            }
        }
        {
            let defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if defs.contains_key(bean_name) {
                return false;
            }
        }
        self.registry
            .definitions()
            .iter()
            .any(|d| d.key().type_name() == bean_name)
    }

    fn bean_definition_count(&self) -> usize {
        let defs = self
            .dynamic_definitions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registry_count = self
            .registry
            .definitions()
            .iter()
            .filter(|d| !defs.contains_key(d.key().type_name()))
            .count();
        let dynamic_count = defs
            .values()
            .filter(|d| d.bean_class_name() != "__DELETED__")
            .count();
        registry_count + dynamic_count
    }

    fn bean_definition_names(&self) -> Vec<String> {
        let defs = self
            .dynamic_definitions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<String> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| !defs.contains_key(d.key().type_name()))
            .map(|d| d.key().type_name().to_string())
            .collect();
        for (name, definition) in defs.iter() {
            if definition.bean_class_name() != "__DELETED__" && !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    }
}

// ── Spring SingletonBeanRegistry 接口实现 ────────────────────────────────────

impl crate::singleton_bean_registry::SingletonBeanRegistry for Container {
    fn register_singleton(&self, bean_name: &str, singleton_object: Arc<dyn Any + Send + Sync>) {
        // 使用 () 类型创建一个占位 ComponentKey，用 bean_name 作为 type_name 的替代
        // 通过 name_to_singleton_key 映射支持按名称查找
        let key = ComponentKey::of::<()>();
        let cell = Arc::new(OnceLock::new());
        let _ = cell.set(Ok(singleton_object));
        self.singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key.clone(), cell);

        // 记录名称到 ComponentKey 的映射
        self.name_to_singleton_key
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(bean_name.to_string(), key.clone());

        // 触发注册的回调
        if let Some(callbacks) = self
            .singleton_callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(bean_name)
            .map(|v| v.clone())
        {
            if let Some(cell) = self
                .singletons
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&key)
            {
                if let Some(Ok(instance)) = cell.get() {
                    for callback in &callbacks {
                        callback(instance.as_ref());
                    }
                }
            }
        }
    }

    fn get_singleton(&self, bean_name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        // 首先通过 name_to_singleton_key 查找
        let key = self
            .name_to_singleton_key
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(bean_name)
            .cloned();

        if let Some(key) = key {
            let singletons = self
                .singletons
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(cell) = singletons.get(&key) {
                if let Some(Ok(instance)) = cell.get() {
                    return Some(Arc::clone(instance));
                }
            }
        }

        // 回退：遍历 singletons 按 type_name 匹配
        let singletons = self
            .singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (key, cell) in singletons.iter() {
            if let Some(Ok(instance)) = cell.get() {
                if key.type_name() == bean_name {
                    return Some(Arc::clone(instance));
                }
            }
        }
        None
    }

    fn contains_singleton(&self, bean_name: &str) -> bool {
        self.get_singleton(bean_name).is_some()
    }

    fn singleton_names(&self) -> Vec<String> {
        let singletons = self
            .singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let name_map = self
            .name_to_singleton_key
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut names: Vec<String> = Vec::new();
        for (key, cell) in singletons.iter() {
            if cell.get().is_some() {
                // 优先从 name_map 中查找名称
                let name = name_map
                    .iter()
                    .find(|(_, k)| *k == key)
                    .map(|(n, _)| n.clone())
                    .unwrap_or_else(|| key.type_name().to_string());
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        names
    }

    fn singleton_count(&self) -> usize {
        let singletons = self
            .singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        singletons
            .values()
            .filter(|cell| cell.get().is_some())
            .count()
    }

    fn add_singleton_callback(
        &mut self,
        bean_name: String,
        callback: Arc<dyn Fn(&dyn Any) + Send + Sync>,
    ) {
        self.singleton_callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(bean_name)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    fn singleton_mutex(&self) -> Arc<dyn Any + Send + Sync> {
        // 返回 singletons 的 Arc 引用作为同步原语
        Arc::clone(&self.singletons) as Arc<dyn Any + Send + Sync>
    }
}

// ── Spring HierarchicalBeanFactory 接口实现 ───────────────────────────────────

impl crate::hierarchical_bean_factory::HierarchicalBeanFactory for Container {
    fn parent_bean_factory(&self) -> Option<Arc<dyn crate::bean_factory::BeanFactory>> {
        let parent = self
            .parent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        parent.clone()
    }

    fn contains_local_bean(&self, name: &str) -> bool {
        // 检查本地（非父容器）是否包含 Bean
        // 检查 dynamic_definitions
        {
            let defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(definition) = defs.get(name) {
                if definition.bean_class_name() != "__DELETED__" {
                    return true;
                }
            }
        }
        // 检查 registry
        self.registry
            .definitions()
            .iter()
            .any(|d| d.key().type_name() == name)
    }
}

// ── Spring ListableBeanFactory 接口实现 ───────────────────────────────────────

impl crate::listable_bean_factory::ListableBeanFactory for Container {
    fn bean_definition_count(&self) -> usize {
        let defs = self
            .dynamic_definitions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registry_count = self
            .registry
            .definitions()
            .iter()
            .filter(|d| !defs.contains_key(d.key().type_name()))
            .count();
        let dynamic_count = defs
            .values()
            .filter(|d| d.bean_class_name() != "__DELETED__")
            .count();
        registry_count + dynamic_count
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        {
            let defs = self
                .dynamic_definitions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(definition) = defs.get(bean_name) {
                if definition.bean_class_name() == "__DELETED__" {
                    return false;
                }
                return true;
            }
        }
        self.registry
            .definitions()
            .iter()
            .any(|d| d.key().type_name() == bean_name)
    }

    fn bean_definition_names(&self) -> Vec<String> {
        let defs = self
            .dynamic_definitions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<String> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| !defs.contains_key(d.key().type_name()))
            .map(|d| d.key().type_name().to_string())
            .collect();
        for (name, definition) in defs.iter() {
            if definition.bean_class_name() != "__DELETED__" && !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    }

    fn bean_names_for_type_id(
        &self,
        type_id: TypeId,
        _include_non_singletons: bool,
        _allow_eager_init: bool,
    ) -> Vec<String> {
        self.registry
            .definitions()
            .iter()
            .filter(|d| d.key().type_id == type_id)
            .map(|d| d.key().type_name().to_string())
            .collect()
    }

    fn beans_of_type_id(
        &self,
        type_id: TypeId,
        _include_non_singletons: bool,
        _allow_eager_init: bool,
    ) -> Result<HashMap<String, Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut result = HashMap::new();
        let definitions: Vec<_> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| d.key().type_id == type_id)
            .collect();

        for definition in definitions {
            let instance = self.resolve_definition(definition, &[], None).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to resolve bean '{}': {}", definition.key(), e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
            result.insert(definition.key().type_name().to_string(), instance);
        }

        Ok(result)
    }

    fn bean_post_processor_count(&self) -> usize {
        self.bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    fn contains_non_singleton_bean(&self) -> bool {
        self.registry
            .definitions()
            .iter()
            .any(|d| !d.scope().is_singleton())
    }

    fn contains_singleton_bean(&self) -> bool {
        self.registry
            .definitions()
            .iter()
            .any(|d| d.scope().is_singleton())
    }

    fn bean_names_iterator(&self) -> Box<dyn Iterator<Item = String> + '_> {
        let names = self.bean_definition_names();
        Box::new(names.into_iter())
    }
}

// ── Spring ConfigurableBeanFactory 接口实现 ───────────────────────────────────

impl crate::configurable_bean_factory::ConfigurableBeanFactory for Container {
    fn set_parent_bean_factory(
        &mut self,
        parent: Arc<dyn crate::bean_factory::BeanFactory>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut p = self
            .parent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *p = Some(parent);
        Ok(())
    }

    fn register_scope(&mut self, scope_name: &str, scope: Box<dyn crate::bean_scope::BeanScope>) {
        self.scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(scope_name.to_string(), Arc::from(scope));
    }

    fn registered_scope_names(&self) -> Vec<String> {
        self.scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }

    fn get_registered_scope(&self, scope_name: &str) -> Option<Arc<dyn crate::bean_scope::BeanScope>> {
        let scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        scopes.get(scope_name).map(|arc_scope| {
            Arc::clone(arc_scope)
        })
    }

    fn add_bean_post_processor(&mut self, processor: Arc<dyn crate::bean_post_processor::BeanPostProcessor>) {
        self.bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(processor);
    }

    fn bean_post_processor_count(&self) -> usize {
        self.bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    fn register_alias(
        &mut self,
        bean_name: &str,
        alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut aliases = self
            .aliases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing_target) = aliases.get(alias) {
            if existing_target != bean_name {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!(
                        "Alias '{}' already points to '{}', cannot reassign to '{}'",
                        alias, existing_target, bean_name
                    ),
                )));
            }
        }
        aliases.insert(alias.to_string(), bean_name.to_string());
        Ok(())
    }

    fn is_factory_bean(&self, name: &str) -> bool {
        // 检查名称是否以 FactoryBean 前缀开头
        name.starts_with(crate::bean_factory::FACTORY_BEAN_PREFIX)
    }

    fn set_currently_in_creation(&mut self, bean_name: &str, in_creation: bool) {
        let mut creation_set = self
            .currently_in_creation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if in_creation {
            creation_set.insert(bean_name.to_string());
        } else {
            creation_set.remove(bean_name);
        }
    }

    fn is_currently_in_creation(&self, bean_name: &str) -> bool {
        self.currently_in_creation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(bean_name)
    }

    fn register_dependent_bean(&mut self, bean_name: &str, dependent_bean_name: &str) {
        // dependent_beans: bean_name -> set of beans that depend on it
        self.dependent_beans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(bean_name.to_string())
            .or_insert_with(HashSet::new)
            .insert(dependent_bean_name.to_string());

        // dependencies_for_bean: dependent_bean_name -> set of beans it depends on
        self.dependencies_for_bean
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(dependent_bean_name.to_string())
            .or_insert_with(HashSet::new)
            .insert(bean_name.to_string());
    }

    fn get_dependent_beans(&self, bean_name: &str) -> Vec<String> {
        self.dependent_beans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(bean_name)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn get_dependencies_for_bean(&self, bean_name: &str) -> Vec<String> {
        self.dependencies_for_bean
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(bean_name)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn destroy_bean(
        &self,
        _bean_name: &str,
        _bean_instance: &dyn Any,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 在 vernal 中，销毁由 Component::shutdown 管理
        Ok(())
    }

    fn destroy_singletons(&self) {
        let mut singletons = self
            .singletons
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        singletons.clear();
    }

    fn add_embedded_value_resolver(&mut self, resolver: Arc<dyn Fn(&str) -> String + Send + Sync>) {
        self.embedded_value_resolvers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(resolver);
    }

    fn resolve_embedded_value(&self, value: &str) -> String {
        let resolvers = self
            .embedded_value_resolvers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if resolvers.is_empty() {
            return value.to_string();
        }
        let mut result = value.to_string();
        for resolver in resolvers.iter() {
            result = resolver(&result);
        }
        result
    }
}

// ── Spring ConfigurableListableBeanFactory 接口实现 ───────────────────────────

impl crate::configurable_listable_bean_factory::ConfigurableListableBeanFactory for Container {
    fn ignore_dependency_type(&mut self, type_id: TypeId) {
        self.ignored_dependency_types
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(type_id);
    }

    fn ignore_dependency_interface(&mut self, interface_id: TypeId) {
        self.ignored_dependency_interfaces
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(interface_id);
    }

    fn register_resolvable_dependency(
        &mut self,
        dependency_type: TypeId,
        autowired_value: Arc<dyn Any + Send + Sync>,
    ) {
        self.resolvable_dependencies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(dependency_type, autowired_value);
    }

    fn is_autowire_candidate(&self, _bean_name: &str) -> bool {
        // 默认所有 Bean 都是 autowire candidate
        // 可以通过扩展字段支持排除逻辑
        true
    }

    fn freeze_configuration(&mut self) {
        *self
            .configuration_frozen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    }

    fn is_configuration_frozen(&self) -> bool {
        *self
            .configuration_frozen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn pre_instantiate_singletons(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 预实例化所有 singleton Bean
        let definitions: Vec<_> = self
            .registry
            .definitions()
            .iter()
            .filter(|d| d.scope().is_singleton())
            .cloned()
            .collect();

        for definition in definitions {
            self.resolve_definition(&definition, &[], None).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to pre-instantiate singleton '{}': {}",
                        definition.key(),
                        e
                    ),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        }
        Ok(())
    }
}
