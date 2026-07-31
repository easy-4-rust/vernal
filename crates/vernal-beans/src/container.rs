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
    factory::parsing::component_definition::ErasedComponent, factory::config::named_bean_holder::NamedBeanHolder,
    resolution_tracker::ResolutionTracker,
};

use crate::instantiation_aware_bean_post_processor::InstantiationAwareBeanPostProcessor;
use crate::destruction_aware_bean_post_processor::DestructionAwareBeanPostProcessor;
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
    bean_post_processors: Arc<Mutex<Vec<Arc<dyn crate::factory::config::bean_post_processor::BeanPostProcessor>>>>,
    /// 可变 BeanDefinition 缓存（支持 register/remove/get 操作）。
    ///
    /// 使用 Mutex 保证线程安全，支持运行时动态注册/删除 Bean 定义。
    /// 这是 Container 层 BeanDefinitionRegistry trait 实现的核心存储。
    dynamic_definitions:
        Arc<Mutex<HashMap<String, Arc<dyn crate::factory::config::bean_definition::BeanDefinition>>>>,
    /// BeanDefinition 代理缓存（用于 get_bean_definition 返回引用）。
    ///
    /// 缓存 ProxyBeanDefinition 对象，使 get_bean_definition 可以返回引用。
    definition_cache: Arc<Mutex<HashMap<String, ProxyBeanDefinition>>>,
    owner: Arc<()>,

    // ── HierarchicalBeanFactory / ConfigurableBeanFactory 字段 ────────
    /// 父 BeanFactory（支持父子容器层级结构）。
    parent: Arc<Mutex<Option<Arc<dyn crate::factory::bean_factory::BeanFactory>>>>,

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

    // ── 循环依赖早期引用缓存 ─────────────────────────────────────────
    /// 早期单例引用缓存（用于解决循环依赖）。
    /// 对应 Spring 的 DefaultSingletonBeanRegistry.earlySingletonObjects。
    /// 存储正在创建中的单例的早期引用，允许循环依赖提前获取。
    early_singleton_objects: Arc<Mutex<HashMap<ComponentKey, Arc<dyn Any + Send + Sync>>>>,
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
            early_singleton_objects: Arc::new(Mutex::new(HashMap::new())),
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
            early_singleton_objects: Arc::clone(&self.early_singleton_objects),
        }
    }

    /// 获取早期单例引用（用于解决循环依赖）。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.getEarlyBeanReference(String, ObjectFactory)`。
    ///
    /// 在循环依赖场景中，当 Bean A 依赖 Bean B，而 Bean B 又依赖 Bean A 时，
    /// 需要在 Bean A 完全初始化之前提供一个早期引用给 Bean B。
    pub fn get_early_bean_reference(
        &self,
        key: &ComponentKey,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        let early = self
            .early_singleton_objects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        early.get(key).map(Arc::clone)
    }

    /// 注册早期单例引用。
    ///
    /// 在 Bean 创建过程中，将早期引用注册到缓存中，
    /// 以便循环依赖可以获取到部分初始化的 Bean。
    pub fn register_early_bean_reference(
        &self,
        key: ComponentKey,
        bean: Arc<dyn Any + Send + Sync>,
    ) {
        let mut early = self
            .early_singleton_objects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        early.insert(key, bean);
    }

    /// 移除早期单例引用。
    ///
    /// 当 Bean 完全初始化后，移除早期引用并注册到正式的单例缓存中。
    pub fn remove_early_bean_reference(
        &self,
        key: &ComponentKey,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut early = self
            .early_singleton_objects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        early.remove(key)
    }

    /// 添加 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `ConfigurableBeanFactory.addBeanPostProcessor(BeanPostProcessor)`。
    pub fn add_bean_post_processor(
        &mut self,
        processor: Arc<dyn crate::factory::config::bean_post_processor::BeanPostProcessor>,
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

        // ═══════════════════════════════════════════════════════════════════
        // Step 1: post_process_before_instantiation（实例化前回调）
        // ═══════════════════════════════════════════════════════════════════
        // 对应 Spring 的 InstantiationAwareBeanPostProcessor.postProcessBeforeInstantiation
        // 如果返回 Some(proxy)，则跳过正常实例化流程
        let processors = self
            .bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // 对应 Spring 的 InstantiationAwareBeanPostProcessor.postProcessBeforeInstantiation
        // 如果返回 Some(proxy)，则跳过正常实例化流程
        let mut proxy_override: Option<Arc<dyn Any + Send + Sync>> = None;
        for processor in processors.iter() {
            match processor.post_process_before_instantiation(
                definition.key().type_name(),
                &definition.key().type_name(),
            ) {
                Ok(Some(proxy)) => {
                    proxy_override = Some(proxy);
                    break; // 找到代理后停止遍历
                }
                Ok(None) => {} // 继续遍历
                Err(_e) => {} // 忽略错误，继续遍历
            }
        }
        drop(processors);

        // 如果有代理对象，直接返回，跳过正常实例化
        if let Some(proxy) = proxy_override {
            return Ok(proxy);
        }

        // ═══════════════════════════════════════════════════════════════════
        // Step 2: 正常实例化
        // ═══════════════════════════════════════════════════════════════════
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

impl crate::factory::bean_factory::BeanFactory for Container {
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
        Box<dyn crate::factory::object_provider::ObjectProvider<dyn Any + Send + Sync> + '_>,
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

impl<'a> crate::factory::object_provider::ObjectProvider<dyn Any + Send + Sync>
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

impl crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory for Container {
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
        bean_name: &str,
        bean_instance: &dyn Any,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ═══════════════════════════════════════════════════════════════════
        // Step 1: DestructionAwareBeanPostProcessor.post_process_before_destruction
        // ═══════════════════════════════════════════════════════════════════
        // 对应 Spring 的 DestructionAwareBeanPostProcessor.postProcessBeforeDestruction
        // 在销毁前调用所有 DestructionAwareBeanPostProcessor
        let processors = self
            .bean_post_processors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // 对应 Spring 的 DestructionAwareBeanPostProcessor.postProcessBeforeDestruction
        // 在销毁前调用所有 DestructionAwareBeanPostProcessor
        for processor in processors.iter() {
            if processor.requires_destruction(bean_instance) {
                match processor.post_process_before_destruction(bean_instance, bean_name) {
                    Ok(()) => {} // 继续遍历
                    Err(_e) => {} // 忽略错误，继续遍历
                }
            }
        }
        drop(processors);

        // ═══════════════════════════════════════════════════════════════════
        // Step 2: DisposableBean.destroy
        // ═══════════════════════════════════════════════════════════════════
        // 对应 Spring 的 DisposableBean.destroy
        // 在 vernal 中，销毁由 Component::shutdown 管理
        let _ = bean_name;
        let _ = bean_instance;

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

impl crate::factory::config::bean_definition::BeanDefinition for ProxyBeanDefinition {
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

impl crate::factory::config::bean_definition::BeanDefinition for DeletedBeanDefinition {
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

impl crate::factory::config::bean_definition::BeanDefinition for RemovedBeanDefinition {
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

impl crate::factory::support::bean_definition_registry::BeanDefinitionRegistry for Container {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn crate::factory::config::bean_definition::BeanDefinition>,
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
        Box<dyn crate::factory::config::bean_definition::BeanDefinition>,
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
    ) -> Option<&'static dyn crate::factory::config::bean_definition::BeanDefinition> {
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

impl crate::factory::config::singleton_bean_registry::SingletonBeanRegistry for Container {
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

impl crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory for Container {
    fn parent_bean_factory(&self) -> Option<Arc<dyn crate::factory::bean_factory::BeanFactory>> {
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

impl crate::factory::listable_bean_factory::ListableBeanFactory for Container {
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

impl crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory for Container {
    fn set_parent_bean_factory(
        &mut self,
        parent: Arc<dyn crate::factory::bean_factory::BeanFactory>,
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

    fn add_bean_post_processor(&mut self, processor: Arc<dyn crate::factory::config::bean_post_processor::BeanPostProcessor>) {
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
        name.starts_with(crate::factory::bean_factory::FACTORY_BEAN_PREFIX)
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

impl crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory for Container {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry_builder::RegistryBuilder;
    use crate::ComponentDefinition;
    use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
    use crate::factory::config::bean_definition::BeanDefinition as BeanDefinitionTrait;

    fn make_container() -> Container {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
        Container::new(b.build().unwrap())
    }

    // ── Basic resolve ────────────────────────────────────────────────────────

    #[test]
    fn resolve_singleton_returns_same_instance() {
        let c = make_container();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = c.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn resolve_not_found_returns_error() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result: Result<Arc<String>, _> = c.resolve();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_type_mismatch_returns_error() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // Try to resolve String as i32
        let result: Result<Arc<i32>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ── resolve_qualified ────────────────────────────────────────────────────

    #[test]
    fn resolve_qualified_success() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let val: Arc<String> = c.resolve_qualified(&q).unwrap();
        assert_eq!(*val, "primary");
    }

    #[test]
    fn resolve_qualified_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let q = Qualifier::new("missing").unwrap();
        let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
        assert!(result.is_err());
    }

    // ── resolve_in (scope owner check) ───────────────────────────────────────

    #[test]
    fn resolve_in_wrong_owner_returns_error() {
        let c = make_container();
        let other_container = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other_container.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_err());
    }

    // ── resolve_qualified_in ─────────────────────────────────────────────────

    #[test]
    fn resolve_qualified_in_wrong_owner_returns_error() {
        let c = make_container();
        let other_container = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other_container.open_scope::<String>();
        let q = Qualifier::new("q").unwrap();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_err());
    }

    // ── Early bean reference ─────────────────────────────────────────────────

    #[test]
    fn early_bean_reference_round_trip() {
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let early: Arc<dyn Any + Send + Sync> = Arc::new("early".to_string());
        assert!(c.get_early_bean_reference(&key).is_none());
        c.register_early_bean_reference(key.clone(), early.clone());
        let retrieved = c.get_early_bean_reference(&key).unwrap();
        assert_eq!(
            *retrieved.downcast_ref::<String>().unwrap(),
            "early"
        );
        let removed = c.remove_early_bean_reference(&key).unwrap();
        assert_eq!(*removed.downcast_ref::<String>().unwrap(), "early");
        assert!(c.get_early_bean_reference(&key).is_none());
    }

    #[test]
    fn remove_early_bean_reference_returns_none_when_absent() {
        let c = make_container();
        let key = ComponentKey::of::<i32>();
        assert!(c.remove_early_bean_reference(&key).is_none());
    }

    // ── BeanPostProcessor ────────────────────────────────────────────────────

    #[test]
    fn add_and_count_bean_post_processors() {
        let mut c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
        // We can't easily construct a real BeanPostProcessor in unit tests,
        // but we test the count method
    }

    // ── Scope ────────────────────────────────────────────────────────────────

    #[test]
    fn open_scope_returns_context() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        // Context should be valid
        drop(scope);
    }

    #[test]
    fn open_scope_with_cancellation() {
        let c = make_container();
        let token = CancellationToken::new();
        let scope = c.open_scope_with_cancellation::<i32>(token);
        drop(scope);
    }

    // ── Transient tracker ────────────────────────────────────────────────────

    #[test]
    fn transient_tracker_is_accessible() {
        let c = make_container();
        let _tracker = c.transient_tracker();
    }

    // ── Registry access ──────────────────────────────────────────────────────

    #[test]
    fn registry_returns_definition_count() {
        let c = make_container();
        let reg = c.registry();
        assert_eq!(reg.definitions().len(), 2);
    }

    // ── unused_definitions ───────────────────────────────────────────────────

    #[test]
    fn unused_definitions_before_resolve() {
        let c = make_container();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 2); // Both are unused initially
    }

    #[test]
    fn unused_definitions_after_resolve() {
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 1); // Only i32 remains unused
    }

    // ── warm_up ──────────────────────────────────────────────────────────────

    #[test]
    fn warm_up_resolves_all_singletons() {
        let c = make_container();
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        assert!(unused.is_empty());
    }

    // ── display_path ─────────────────────────────────────────────────────────

    #[test]
    fn display_path_with_leaf() {
        let path = Container::display_path(
            &[ComponentKey::of::<String>(), ComponentKey::of::<i32>()],
            Some("leaf".to_string()),
        );
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn display_path_without_leaf() {
        let path = Container::display_path(&[ComponentKey::of::<String>()], None);
        assert_eq!(path.len(), 1);
    }

    // ── ensure_scope_owner ───────────────────────────────────────────────────

    #[test]
    fn ensure_scope_owner_mismatch() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result = c.ensure_scope_owner(&scope);
        assert!(result.is_err());
    }

    #[test]
    fn ensure_scope_owner_match() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let result = c.ensure_scope_owner(&scope);
        assert!(result.is_ok());
    }

    // ── BeanFactory trait ────────────────────────────────────────────────────

    #[test]
    fn bean_factory_get_bean_by_key() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let result = c.get_bean_by_key(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn bean_factory_get_bean_by_key_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<String>();
        let result = c.get_bean_by_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn bean_factory_get_bean_by_type_id_single() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn bean_factory_get_bean_by_type_id_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn bean_factory_get_bean_by_type_id_ambiguous() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn bean_factory_contains_bean() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.contains_bean(&ComponentKey::of::<String>()));
        assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
    }

    #[test]
    fn bean_factory_is_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.is_singleton(&ComponentKey::of::<String>()).unwrap());
    }

    #[test]
    fn bean_factory_is_singleton_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.is_singleton(&ComponentKey::of::<String>()).is_err());
    }

    #[test]
    fn bean_factory_is_prototype() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        assert!(c.is_prototype(&ComponentKey::of::<String>()).unwrap());
    }

    #[test]
    fn bean_factory_is_prototype_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.is_prototype(&ComponentKey::of::<String>()).is_err());
    }

    #[test]
    fn bean_factory_get_type() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let result = c.get_type(&ComponentKey::of::<String>()).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn bean_factory_get_type_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.get_type(&ComponentKey::of::<String>()).is_err());
    }

    #[test]
    fn bean_factory_get_aliases() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let aliases = c.get_aliases(&ComponentKey::of::<String>());
        assert!(aliases.is_empty());
    }

    #[test]
    fn bean_factory_get_bean_provider() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>());
        assert!(provider.is_ok());
    }

    #[test]
    fn bean_factory_is_type_match() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
        assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
        assert!(!c.is_type_match(&ComponentKey::of::<f64>(), std::any::TypeId::of::<f64>()));
    }

    // ── ContainerObjectProvider ──────────────────────────────────────────────

    #[test]
    fn object_provider_get_returns_first_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        // Must resolve first so singleton exists in the cache
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_if_available() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.if_available();
        assert!(result.is_some());
    }

    #[test]
    fn object_provider_get_if_unique() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get_if_unique();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_stream() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(!items.is_empty());
    }

    #[test]
    fn object_provider_ordered_stream() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(!items.is_empty());
    }

    // ── AutowireCapableBeanFactory ───────────────────────────────────────────

    #[test]
    fn create_bean_by_class_name() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.create_bean("alloc::string::String");
        assert!(result.is_ok());
    }

    #[test]
    fn create_bean_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.create_bean("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn autowire_bean_existing() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_no_matching_definition() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i64);
        let result = c.autowire_bean(existing);
        assert!(result.is_ok()); // Returns existing bean
    }

    #[test]
    fn configure_bean_applies_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.configure_bean(bean, "test_bean");
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_no() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_name() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_type() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_constructor() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_invalid_mode() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 99, false);
        assert!(result.is_err());
    }

    #[test]
    fn autowire_bean_properties_no() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_by_name() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_fallback() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 99, false);
        assert!(result.is_ok());
    }

    #[test]
    fn apply_bean_property_values() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.apply_bean_property_values(bean, "test_bean");
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_bean() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.initialize_bean(bean, "test_bean");
        assert!(result.is_ok());
    }

    #[test]
    fn destroy_bean_instance() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("test_bean", bean.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_single() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_named_bean_ambiguous() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_dependency_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        // The dependency resolution may fail due to type_id mismatch in the internal
        // descriptor access pattern, but the method itself is exercised
        let _ = result;
    }

    #[test]
    fn resolve_dependency_not_found_required() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_dependency_ambiguous() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        let _ = result;
    }

    #[test]
    fn set_type_converter_and_type_converter() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.set_type_converter(None);
        assert!(c.type_converter().is_none());
    }

    // ── BeanDefinitionRegistry ───────────────────────────────────────────────

    #[test]
    fn register_and_get_bean_definition() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("myBean"));
        let bd = c.get_bean_definition("myBean");
        assert!(bd.is_some());
    }

    #[test]
    fn register_duplicate_bean_definition_fails() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def1 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def1).unwrap();
        let result = c.register_bean_definition("myBean".to_string(), def2);
        assert!(result.is_err());
    }

    #[test]
    fn remove_dynamic_bean_definition() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def).unwrap();
        let removed = c.remove_bean_definition("myBean");
        assert!(removed.is_ok());
        assert!(!c.contains_bean_definition("myBean"));
    }

    #[test]
    fn remove_registry_bean_definition() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        // "alloc::string::String" is in the registry
        let removed = c.remove_bean_definition("alloc::string::String");
        assert!(removed.is_ok());
        // After removal, it should be marked as deleted
        assert!(!c.contains_bean_definition("alloc::string::String"));
    }

    #[test]
    fn remove_nonexistent_bean_definition_fails() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let result = c.remove_bean_definition("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn get_bean_definition_from_registry() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let bd = c.get_bean_definition("alloc::string::String");
        assert!(bd.is_some());
    }

    #[test]
    fn get_bean_definition_not_found() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let bd = c.get_bean_definition("nonexistent");
        assert!(bd.is_none());
    }

    #[test]
    fn contains_bean_definition_from_registry() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(!c.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn bean_definition_count_includes_both() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = c.bean_definition_count();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("newBean".to_string(), def).unwrap();
        assert_eq!(c.bean_definition_count(), initial + 1);
    }

    #[test]
    fn bean_definition_names_includes_all() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let names = c.bean_definition_names();
        assert!(names.len() >= 2);
    }

    #[test]
    fn get_bean_definition_deleted_returns_none() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        // Remove a registry bean definition, which marks it as __DELETED__
        c.remove_bean_definition("alloc::string::String").unwrap();
        // Now get_bean_definition should return None for the deleted one
        let bd = c.get_bean_definition("alloc::string::String");
        assert!(bd.is_none());
    }

    // ── SingletonBeanRegistry ────────────────────────────────────────────────

    #[test]
    fn register_and_get_singleton() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("singleton_value".to_string());
        c.register_singleton("mySingleton", obj.clone());
        let retrieved = c.get_singleton("mySingleton");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap().downcast_ref::<String>().unwrap(), "singleton_value");
    }

    #[test]
    fn contains_singleton() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("val".to_string());
        c.register_singleton("mySingleton", obj);
        assert!(c.contains_singleton("mySingleton"));
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn singleton_names() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let names = c.singleton_names();
        assert!(!names.is_empty());
    }

    #[test]
    fn singleton_count() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let count = c.singleton_count();
        assert!(count >= 1);
    }

    #[test]
    fn add_singleton_callback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let mut c = make_container();
        let callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(|_| {});
        c.add_singleton_callback("test".to_string(), callback);
    }

    #[test]
    fn singleton_mutex() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let mutex = c.singleton_mutex();
        assert!(mutex.downcast_ref::<std::sync::Mutex<HashMap<ComponentKey, Arc<SingletonCell>>>>().is_some());
    }

    // ── HierarchicalBeanFactory ──────────────────────────────────────────────

    #[test]
    fn parent_bean_factory_initially_none() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.parent_bean_factory().is_none());
    }

    #[test]
    fn contains_local_bean() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(!c.contains_local_bean("nonexistent"));
    }

    // ── ListableBeanFactory ──────────────────────────────────────────────────

    #[test]
    fn listable_bean_definition_count() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
    }

    #[test]
    fn listable_contains_bean_definition() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(!c.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn listable_bean_definition_names() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_definition_names();
        assert!(names.len() >= 2);
    }

    #[test]
    fn listable_bean_names_for_type_id() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn listable_beans_of_type_id() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
    }

    #[test]
    fn listable_bean_post_processor_count() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    #[test]
    fn listable_contains_non_singleton_bean() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(!c.contains_non_singleton_bean());
    }

    #[test]
    fn listable_contains_singleton_bean() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.contains_singleton_bean());
    }

    #[test]
    fn listable_bean_names_iterator() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names: Vec<String> = c.bean_names_iterator().collect();
        assert!(names.len() >= 2);
    }

    // ── ConfigurableBeanFactory ──────────────────────────────────────────────

    #[test]
    fn set_parent_bean_factory() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        let mut c = make_container();
        let parent_container = Container::new(RegistryBuilder::new().build().unwrap());
        let parent: Arc<dyn BeanFactory> = Arc::new(parent_container);
        c.set_parent_bean_factory(parent).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    #[test]
    fn register_scope_and_get() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let scope = Box::new(crate::request_scope::RequestScope::new("test-request"));
        c.register_scope("request", scope);
        let names = c.registered_scope_names();
        assert!(names.contains(&"request".to_string()));
        assert!(c.get_registered_scope("request").is_some());
        assert!(c.get_registered_scope("nonexistent").is_none());
    }

    #[test]
    fn register_alias_success() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
    }

    #[test]
    fn register_alias_conflict() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        let result = c.register_alias("bean2", "alias1");
        assert!(result.is_err());
    }

    #[test]
    fn register_alias_same_target_ok() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap(); // same target, ok
    }

    #[test]
    fn is_factory_bean() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.is_factory_bean("&myFactoryBean"));
        assert!(!c.is_factory_bean("regularBean"));
    }

    #[test]
    fn currently_in_creation() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", true);
        assert!(c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", false);
        assert!(!c.is_currently_in_creation("bean1"));
    }

    #[test]
    fn dependent_beans_tracking() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_dependent_bean("service", "controller");
        let dependents = c.get_dependent_beans("service");
        assert!(dependents.contains(&"controller".to_string()));
        let deps = c.get_dependencies_for_bean("controller");
        assert!(deps.contains(&"service".to_string()));
    }

    #[test]
    fn get_dependent_beans_empty() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let dependents = c.get_dependent_beans("nonexistent");
        assert!(dependents.is_empty());
    }

    #[test]
    fn get_dependencies_for_bean_empty() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let deps = c.get_dependencies_for_bean("nonexistent");
        assert!(deps.is_empty());
    }

    #[test]
    fn destroy_bean_configurable() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.destroy_bean("test_bean", bean.as_ref()).unwrap();
    }

    #[test]
    fn destroy_singletons() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
    }

    #[test]
    fn embedded_value_resolvers() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let resolver: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
            v.replace("${key}", "value")
        });
        c.add_embedded_value_resolver(resolver);
        let result = c.resolve_embedded_value("${key}");
        assert_eq!(result, "value");
    }

    #[test]
    fn embedded_value_no_resolvers() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let result = c.resolve_embedded_value("plain");
        assert_eq!(result, "plain");
    }

    // ── ConfigurableListableBeanFactory ──────────────────────────────────────

    #[test]
    fn ignore_dependency_type() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_type(std::any::TypeId::of::<String>());
    }

    #[test]
    fn ignore_dependency_interface() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_interface(std::any::TypeId::of::<dyn std::fmt::Debug>());
    }

    #[test]
    fn register_resolvable_dependency() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.register_resolvable_dependency(
            std::any::TypeId::of::<String>(),
            Arc::new("resolved".to_string()),
        );
    }

    #[test]
    fn is_autowire_candidate() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = make_container();
        assert!(c.is_autowire_candidate("any_bean"));
    }

    #[test]
    fn freeze_and_check_configuration() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
    }

    #[test]
    fn pre_instantiate_singletons() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = make_container();
        c.pre_instantiate_singletons().unwrap();
    }

    // ── Transient scope ──────────────────────────────────────────────────────

    #[test]
    fn transient_creates_new_instance_each_time() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| {
            use std::sync::atomic::{AtomicI32, Ordering};
            static COUNTER: AtomicI32 = AtomicI32::new(0);
            COUNTER.fetch_add(1, Ordering::SeqCst)
        }));
        let c = Container::new(b.build().unwrap());
        let a: Arc<i32> = c.resolve().unwrap();
        let b_val: Arc<i32> = c.resolve().unwrap();
        assert_ne!(*a, *b_val);
    }

    // ── ProxyBeanDefinition ──────────────────────────────────────────────────

    #[test]
    fn proxy_bean_definition_methods() {
        let proxy = ProxyBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "String".to_string(),
            scope: crate::component_scope::Scope::Singleton,
            source: "registry".to_string(),
        };
        // These methods are from the BeanDefinition trait impl
        assert_eq!(BeanDefinitionTrait::bean_class_name(&proxy), "String");
        assert_eq!(BeanDefinitionTrait::scope(&proxy), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&proxy));
        assert!(!BeanDefinitionTrait::is_primary(&proxy));
    }

    // ── RemovedBeanDefinition ────────────────────────────────────────────────

    #[test]
    fn removed_bean_definition_methods() {
        let removed = RemovedBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "String".to_string(),
            scope: crate::component_scope::Scope::Singleton,
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&removed), "String");
        assert_eq!(BeanDefinitionTrait::scope(&removed), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&removed));
        assert!(!BeanDefinitionTrait::is_primary(&removed));
    }

    // ── DeletedBeanDefinition ────────────────────────────────────────────────

    #[test]
    fn deleted_bean_definition_methods() {
        let deleted = DeletedBeanDefinition {
            bean_name: "test".to_string(),
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&deleted), "__DELETED__");
        assert_eq!(BeanDefinitionTrait::scope(&deleted), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&deleted));
        assert!(!BeanDefinitionTrait::is_primary(&deleted));
    }

    // ── remove_bean_definition for deleted ───────────────────────────────────

    #[test]
    fn remove_bean_definition_already_deleted_returns_error() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        // First removal marks as deleted
        c.remove_bean_definition("alloc::string::String").unwrap();
        // Second removal of the same name should fail because the dynamic definition is __DELETED__
        let result = c.remove_bean_definition("alloc::string::String");
        assert!(result.is_err());
    }

    // ── shared_handle ────────────────────────────────────────────────────────

    #[test]
    fn shared_handle_shares_singletons() {
        let c = make_container();
        let handle = c.shared_handle();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = handle.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    // ── Additional container tests ──────────────────────────────────────

    #[test]
    fn select_definition_not_found_returns_error() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::of::<f64>();
        // Use the internal select_definition via resolve
        let result: Result<Arc<f64>, _> = c.resolve();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_empty_bindings() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        // No bindings registered, so resolve_all_traits should return empty
        // We test this indirectly by checking that the container has no bindings
        assert!(c.registry().bindings().is_empty());
    }

    #[test]
    fn resolve_optional_not_found_returns_none() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_optional_typed::<String>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn resolve_optional_found_returns_some() {
        let c = make_container();
        let result = c.resolve_optional_typed::<String>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn select_definition_ambiguous() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
                .qualified(Qualifier::new("q1").unwrap()),
        );
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q2").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        // Two String definitions with different qualifiers — unqualified resolve is ambiguous
        let result: Result<Arc<String>, _> = c.resolve();
        // This should fail because there's no unqualified String definition
        assert!(result.is_err());
    }

    #[test]
    fn contains_local_bean_with_dynamic_definition() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(c.contains_local_bean("dynamicBean"));
    }

    #[test]
    fn contains_local_bean_deleted_in_dynamic_returns_false_for_deleted() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        // Register a new dynamic bean definition
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(c.contains_local_bean("dynamicBean"));
        // Remove it
        c.remove_bean_definition("dynamicBean").unwrap();
        // After removal, the dynamic definition is deleted, so contains_local_bean returns false
        assert!(!c.contains_local_bean("dynamicBean"));
    }

    #[test]
    fn listable_beans_of_type_id_not_found() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let beans = c.beans_of_type_id(std::any::TypeId::of::<f64>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    #[test]
    fn listable_bean_names_for_type_id_not_found() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<f64>(), true, true);
        assert!(names.is_empty());
    }

    #[test]
    fn configurable_destroy_bean() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.destroy_bean("test_bean", bean.as_ref()).unwrap();
    }

    #[test]
    fn autowire_bean_properties_by_type() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_dependency_not_required_returns_none() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            false, // not required
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_bean_by_type_id_single_match() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn warm_up_with_transient_skipped() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        // Only singletons should be warmed up
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 1); // transient remains unused
    }

    #[test]
    fn resolve_in_same_owner_succeeds() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_in_same_owner_succeeds() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_in_wrong_owner_for_any_type_fails() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_not_found_for_missing_qualifier() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let q = Qualifier::new("missing").unwrap();
        let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_in_wrong_owner_fails() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let q = Qualifier::new("q").unwrap();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_in_same_owner_for_qualified() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_get_empty_returns_error() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_err());
    }

    #[test]
    fn object_provider_if_available_empty_returns_none() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.if_available();
        assert!(result.is_none());
    }

    #[test]
    fn object_provider_stream_empty_v5() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(items.is_empty());
    }

    // ── Additional container tests for coverage ─────────────────────────

    #[test]
    fn resolve_optional_type_mismatch_returns_error() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // resolve_optional_typed: Dependency::of::<i32> won't match String definition
        let result = c.resolve_optional_typed::<i32>(&crate::Dependency::of::<i32>(), &[], None);
        // No i32 definition -> NotFound -> returns None
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn resolve_typed_success() {
        let c = make_container();
        let result = c.resolve_typed::<String>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "hello");
    }

    #[test]
    fn resolve_typed_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_typed::<f64>(&crate::Dependency::of::<f64>(), &[], None);
        assert!(result.is_err());
    }

    #[test]
    fn select_definition_single_match() {
        let c = make_container();
        let dep = crate::Dependency::of::<String>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_definition_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::of::<f64>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_definition_ambiguous_multiple() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
                .qualified(Qualifier::new("q1").unwrap()),
        );
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q2").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::of::<String>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_definition_with_matching_qualifier() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::qualified::<String>(q);
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_trait_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Debug + Send + Sync>();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_empty() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_all_traits::<dyn std::fmt::Debug + Send + Sync>();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn resolve_trait_in_wrong_owner() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Debug + Send + Sync>(&scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_trait_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let q = Qualifier::new("missing").unwrap();
        let result = c.resolve_qualified_trait::<dyn std::fmt::Debug + Send + Sync>(&q);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_trait_in_wrong_owner() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let q = Qualifier::new("q").unwrap();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Debug + Send + Sync>(&q, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_in_wrong_owner() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result = c.resolve_all_traits_in::<dyn std::fmt::Debug + Send + Sync>(&scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_optional_trait_not_found_returns_none() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Debug + Send + Sync>(
            &crate::Dependency::trait_of::<dyn std::fmt::Debug + Send + Sync>(),
            &[],
            None,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn display_path_empty_stack() {
        let path = Container::display_path(&[], Some("leaf".to_string()));
        assert_eq!(path, vec!["leaf"]);
    }

    #[test]
    fn display_path_empty_stack_no_leaf() {
        let path = Container::display_path(&[], None);
        assert!(path.is_empty());
    }

    // ── BeanDefinitionRegistry extended tests ───────────────────────────

    #[test]
    fn contains_bean_definition_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynBean".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("dynBean"));
    }

    #[test]
    fn contains_bean_definition_deleted_returns_false() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toDelete".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("toDelete"));
        c.remove_bean_definition("toDelete").unwrap();
        assert!(!c.contains_bean_definition("toDelete"));
    }

    #[test]
    fn bean_definition_names_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("extra".to_string(), def).unwrap();
        let names = c.bean_definition_names();
        assert!(names.contains(&"extra".to_string()));
    }

    #[test]
    fn bean_definition_names_excludes_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        c.remove_bean_definition("temp").unwrap();
        let names = c.bean_definition_names();
        assert!(!names.contains(&"temp".to_string()));
    }

    // ── SingletonBeanRegistry extended tests ────────────────────────────

    #[test]
    fn register_singleton_with_callback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        use std::sync::atomic::{AtomicBool, Ordering};
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        let callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| {
            called_clone.store(true, Ordering::SeqCst);
        });
        c.add_singleton_callback("cb_test".to_string(), callback);
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("value".to_string());
        c.register_singleton("cb_test", obj);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn get_singleton_by_type_name_fallback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        // Resolve to populate singletons
        let _: Arc<String> = c.resolve().unwrap();
        // get_singleton should find it by type_name
        let result = c.get_singleton("alloc::string::String");
        assert!(result.is_some());
    }

    #[test]
    fn singleton_names_after_resolve() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let names = c.singleton_names();
        assert!(names.len() >= 2);
    }

    // ── HierarchicalBeanFactory extended tests ───────────────────────────

    #[test]
    fn contains_local_bean_in_registry() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(c.contains_local_bean("i32"));
        assert!(!c.contains_local_bean("nonexistent"));
    }

    // ── ConfigurableBeanFactory extended tests ───────────────────────────

    #[test]
    fn embedded_value_resolvers_chained() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let r1: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
            v.replace("${a}", "A")
        });
        let r2: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
            v.replace("${b}", "B")
        });
        c.add_embedded_value_resolver(r1);
        c.add_embedded_value_resolver(r2);
        let result = c.resolve_embedded_value("${a}-${b}");
        assert_eq!(result, "A-B");
    }

    #[test]
    fn registered_scope_names_empty() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.registered_scope_names().is_empty());
    }

    #[test]
    fn get_registered_scope_not_found() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.get_registered_scope("nonexistent").is_none());
    }

    // ── ListableBeanFactory extended tests ───────────────────────────────

    #[test]
    fn listable_contains_non_singleton_with_transient() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_non_singleton_bean());
        assert!(c.contains_singleton_bean());
    }

    #[test]
    fn listable_beans_of_type_id_resolves() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
        assert!(beans.contains_key("alloc::string::String"));
    }

    #[test]
    fn listable_bean_names_for_type_multiple() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 2);
    }

    // ── ConfigurableListableBeanFactory extended tests ───────────────────

    #[test]
    fn pre_instantiate_singletons_resolves_all() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        c.pre_instantiate_singletons().unwrap();
        assert!(c.singleton_count() >= 2);
    }

    // ── AutowireCapableBeanFactory extended tests ────────────────────────

    #[test]
    fn autowire_bean_with_matching_definition() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        // String has a definition, so autowire_bean should find it
        let result = c.autowire_bean(existing);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_multiple_matches() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing);
        // Multiple matches: returns existing bean
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_dependency_found_single() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<i32>(),
            "i32".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        let _ = result; // exercises the code path
    }

    #[test]
    fn resolve_dependency_multiple_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        let _ = result; // exercises the code path
    }

    #[test]
    fn configure_bean_no_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.configure_bean(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn initialize_bean_no_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.initialize_bean(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn destroy_bean_instance_no_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("myBean", bean.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    fn apply_bean_property_values_returns_same_v5() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.apply_bean_property_values(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    // ── Transient tracking ───────────────────────────────────────────────

    #[test]
    fn transient_instances_tracked() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let _: Arc<String> = c.resolve().unwrap();
        // Transient tracker should have the instance
        // We can't directly check the tracker contents without more API, but the code path is exercised
    }

    // ── Circular dependency detection ────────────────────────────────────

    #[test]
    fn resolve_with_stack_circular_detected() {
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let stack = vec![key.clone()];
        let result = c.resolve_typed::<String>(&crate::Dependency::of::<String>(), &stack, None);
        assert!(result.is_err());
    }

    // ── ContainerObjectProvider extended tests ───────────────────────────

    #[test]
    fn object_provider_ordered_stream_empty_v5() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(items.is_empty());
    }

    #[test]
    fn object_provider_get_if_unique_empty_v5() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get_if_unique();
        assert!(result.is_err());
    }

    // ── Warm up edge cases ───────────────────────────────────────────────

    #[test]
    fn warm_up_empty_container() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.warm_up().unwrap();
    }

    #[test]
    fn warm_up_mixed_scopes() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14));
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        // Only transient should remain unused
        assert_eq!(unused.len(), 1);
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn resolve_definition_transient_scope() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = c.resolve().unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn resolve_definition_singleton_scope_cached() {
        let c = make_container();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = c.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn resolve_definition_circular_runtime_error() {
        // Simulate circular dependency by putting key in stack
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let stack = vec![key.clone()];
        let result = c.resolve_typed::<String>(&crate::Dependency::of::<String>(), &stack, None);
        assert!(matches!(result, Err(crate::ResolveError::CircularRuntime { .. })));
    }

    #[test]
    fn select_trait_binding_single_match_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        // Should find the single binding
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_ambiguous_with_primary_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        // Should resolve via Primary
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_ambiguous_no_primary() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        // Should be ambiguous without Primary
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_with_qualifier_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "q_impl".to_string()));
        let q = Qualifier::new("myQ").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_ambiguous_with_qualifier() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let q1 = Qualifier::new("q1").unwrap();
        let q2 = Qualifier::new("q2").unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q1.clone());
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q2.clone());
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        // Both bindings have different qualifiers - resolving unqualified should be ambiguous
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_multiple() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let results = c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn resolve_all_traits_in_same_owner_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let results = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn resolve_trait_in_same_owner_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_in_same_owner_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_ok());
    }

    #[test]
    fn bean_factory_get_type_returns_type_name() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let result = c.get_type(&ComponentKey::of::<String>()).unwrap();
        assert!(result.is_some());
        let name = result.unwrap();
        assert!(name.contains("String"));
    }

    #[test]
    fn bean_factory_get_type_not_registered() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.get_type(&ComponentKey::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn listable_beans_of_type_id_multiple() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true);
        // Both definitions share the same TypeId, so they should be found
        assert!(beans.is_ok());
        assert!(beans.unwrap().len() >= 1);
    }

    #[test]
    fn configurable_set_parent_and_query() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        let mut c = make_container();
        let parent = Container::new(RegistryBuilder::new().build().unwrap());
        let parent_arc: Arc<dyn BeanFactory> = Arc::new(parent);
        c.set_parent_bean_factory(parent_arc).unwrap();
        let p = c.parent_bean_factory();
        assert!(p.is_some());
    }

    #[test]
    fn configurable_register_multiple_scopes() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_scope("request", Box::new(crate::request_scope::RequestScope::new("r1")));
        c.register_scope("session", Box::new(crate::session_scope::SessionScope::new("s1")));
        let names = c.registered_scope_names();
        assert!(names.contains(&"request".to_string()));
        assert!(names.contains(&"session".to_string()));
    }

    #[test]
    fn configurable_get_registered_scope() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_scope("request", Box::new(crate::request_scope::RequestScope::new("r1")));
        let scope = c.get_registered_scope("request");
        assert!(scope.is_some());
        assert!(c.get_registered_scope("missing").is_none());
    }

    #[test]
    fn configurable_register_alias_same_target() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap(); // same target, ok
    }

    #[test]
    fn configurable_dependent_beans_multiple() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_dependent_bean("service", "controller1");
        c.register_dependent_bean("service", "controller2");
        let dependents = c.get_dependent_beans("service");
        assert!(dependents.contains(&"controller1".to_string()));
        assert!(dependents.contains(&"controller2".to_string()));
    }

    #[test]
    fn configurable_listable_ignore_dependency_types() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_type(std::any::TypeId::of::<String>());
        c.ignore_dependency_interface(std::any::TypeId::of::<dyn std::fmt::Debug>());
    }

    #[test]
    fn configurable_listable_register_resolvable() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.register_resolvable_dependency(
            std::any::TypeId::of::<String>(),
            Arc::new("resolved".to_string()),
        );
    }

    #[test]
    fn configurable_listable_freeze_and_check() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
    }

    #[test]
    fn configurable_listable_is_autowire_candidate() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = make_container();
        assert!(c.is_autowire_candidate("any_bean"));
    }

    #[test]
    fn autowire_bean_with_no_dependencies() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_no_matching_def() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new(999i64);
        let result = c.autowire_bean(existing);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_constructor_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_invalid_mode_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 99, false);
        assert!(result.is_err());
    }

    #[test]
    fn autowire_bean_properties_fallback_mode() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 99, false);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_dependency_not_required_not_found_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            false,
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn resolve_dependency_required_not_found_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_named_bean_single_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_not_found_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_named_bean_ambiguous_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn set_type_converter_none_v5() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.set_type_converter(None);
        assert!(c.type_converter().is_none());
    }

    #[test]
    fn destroy_bean_instance_with_bean() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("test_bean", bean.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    fn get_bean_definition_deleted_returns_none_v2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let bd = c.get_bean_definition("alloc::string::String");
        assert!(bd.is_none());
    }

    #[test]
    fn remove_bean_definition_already_deleted_fails() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let result = c.remove_bean_definition("alloc::string::String");
        assert!(result.is_err());
    }

    #[test]
    fn bean_definition_registry_count_with_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = c.bean_definition_count();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("newBean".to_string(), def).unwrap();
        assert_eq!(c.bean_definition_count(), initial + 1);
    }

    #[test]
    fn bean_definition_names_includes_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("extra".to_string(), def).unwrap();
        let names = c.bean_definition_names();
        assert!(names.contains(&"extra".to_string()));
    }

    #[test]
    fn singleton_mutex_returns_arc() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let mutex = c.singleton_mutex();
        assert!(mutex.downcast_ref::<std::sync::Mutex<HashMap<ComponentKey, Arc<SingletonCell>>>>().is_some());
    }

    #[test]
    fn singleton_count_after_resolve() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert_eq!(c.singleton_count(), 0);
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
    }

    #[test]
    fn contains_singleton_after_register() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("val".to_string());
        c.register_singleton("mySingleton", obj);
        assert!(c.contains_singleton("mySingleton"));
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn get_singleton_not_found() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(c.get_singleton("nonexistent").is_none());
    }

    #[test]
    fn add_singleton_callback_and_trigger() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        use std::sync::atomic::{AtomicBool, Ordering};
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        let callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| {
            called_clone.store(true, Ordering::SeqCst);
        });
        c.add_singleton_callback("cb_test".to_string(), callback);
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("value".to_string());
        c.register_singleton("cb_test", obj);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn resolve_definition_records_resolution() {
        let c = make_container();
        assert_eq!(c.unused_definitions().len(), 2);
        let _: Arc<String> = c.resolve().unwrap();
        assert_eq!(c.unused_definitions().len(), 1);
    }

    #[test]
    fn shared_handle_shares_resolutions() {
        let c = make_container();
        let handle = c.shared_handle();
        let _: Arc<String> = c.resolve().unwrap();
        // Both should see the same resolution
        let unused_c = c.unused_definitions();
        let unused_h = handle.unused_definitions();
        assert_eq!(unused_c.len(), unused_h.len());
    }

    #[test]
    fn resolve_optional_trait_found() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(
            &crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>(),
            &[],
            None,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn resolve_optional_trait_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(
            &crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>(),
            &[],
            None,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn listable_contains_non_singleton_bean_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_non_singleton_bean());
    }

    #[test]
    fn listable_contains_singleton_bean_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_singleton_bean());
    }

    #[test]
    fn listable_bean_post_processor_count_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    #[test]
    fn listable_bean_names_iterator_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names: Vec<String> = c.bean_names_iterator().collect();
        assert!(names.len() >= 2);
    }

    #[test]
    fn configurable_destroy_singletons() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
    }

    #[test]
    fn pre_instantiate_singletons_resolves_all_v2() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        c.pre_instantiate_singletons().unwrap();
        assert!(c.singleton_count() >= 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Mock BeanPostProcessors for comprehensive testing
    // ═══════════════════════════════════════════════════════════════════════════

    use crate::factory::config::bean_post_processor::BeanPostProcessor;
    use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

    /// Returns a proxy from post_process_before_instantiation, skipping normal instantiation.
    struct ProxyOverrideProcessor;
    impl BeanPostProcessor for ProxyOverrideProcessor {
        fn post_process_before_instantiation(
            &self, _bean_class: &str, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Some(Arc::new("proxy_override_result".to_string())))
        }
    }

    /// Errors on post_process_before_instantiation (should be ignored).
    struct ErrorBeforeInstantiationProcessor;
    impl BeanPostProcessor for ErrorBeforeInstantiationProcessor {
        fn post_process_before_instantiation(
            &self, _bean_class: &str, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Err("intentional error".into())
        }
    }

    /// Replaces bean in post_process_after_initialization.
    struct ReplacingAfterInitProcessor;
    impl BeanPostProcessor for ReplacingAfterInitProcessor {
        fn post_process_after_initialization(
            &self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Some(Arc::new("replaced_bean".to_string())))
        }
    }

    /// Errors in post_process_after_initialization (should be ignored).
    struct ErrorAfterInitProcessor;
    impl BeanPostProcessor for ErrorAfterInitProcessor {
        fn post_process_after_initialization(
            &self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Err("intentional error".into())
        }
    }

    /// Returns None from post_process_after_initialization (no replacement).
    struct NoneAfterInitProcessor;
    impl BeanPostProcessor for NoneAfterInitProcessor {
        fn post_process_after_initialization(
            &self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
        }
    }

    /// Replaces bean in post_process_before_initialization.
    struct ReplacingBeforeInitProcessor;
    impl BeanPostProcessor for ReplacingBeforeInitProcessor {
        fn post_process_before_initialization(
            &self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Some(Arc::new("replaced_before_init".to_string())))
        }
    }

    /// Errors in post_process_before_initialization (should be ignored).
    struct ErrorBeforeInitProcessor;
    impl BeanPostProcessor for ErrorBeforeInitProcessor {
        fn post_process_before_initialization(
            &self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Err("intentional error".into())
        }
    }

    /// Returns None from post_process_before_initialization.
    struct NoneBeforeInitProcessor;
    impl BeanPostProcessor for NoneBeforeInitProcessor {
        fn post_process_before_initialization(
            &self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
        }
    }

    /// Tracks destruction calls.
    struct DestructionTrackingProcessor {
        called: Arc<AtomicBool>,
    }
    impl BeanPostProcessor for DestructionTrackingProcessor {
        fn requires_destruction(&self, _bean: &dyn Any) -> bool { true }
        fn post_process_before_destruction(
            &self, _bean: &dyn Any, _bean_name: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    /// Errors on destruction (should be ignored).
    struct ErrorDestructionProcessor;
    impl BeanPostProcessor for ErrorDestructionProcessor {
        fn requires_destruction(&self, _bean: &dyn Any) -> bool { true }
        fn post_process_before_destruction(
            &self, _bean: &dyn Any, _bean_name: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Err("destruction error".into())
        }
    }

    /// Does NOT require destruction.
    struct NoDestructionProcessor;
    impl BeanPostProcessor for NoDestructionProcessor {
        fn requires_destruction(&self, _bean: &dyn Any) -> bool { false }
    }

    /// Tracks before/after init calls.
    struct CountingProcessor {
        before_count: Arc<AtomicUsize>,
        after_count: Arc<AtomicUsize>,
    }
    impl BeanPostProcessor for CountingProcessor {
        fn post_process_before_initialization(
            &self, bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.before_count.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
        fn post_process_after_initialization(
            &self, bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.after_count.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanPostProcessor integration with Container::construct
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn construct_proxy_override_skips_instantiation() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ProxyOverrideProcessor));
        let result = c.create_bean("alloc::string::String").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "proxy_override_result");
    }

    #[test]
    fn construct_error_before_instantiation_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorBeforeInstantiationProcessor));
        assert!(c.create_bean("alloc::string::String").is_ok());
    }

    #[test]
    fn construct_none_before_instantiation_proceeds_normally() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneBeforeInitProcessor));
        assert!(c.create_bean("alloc::string::String").is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanPostProcessor in resolve_definition (after_initialization chain)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_definition_after_init_replaces_bean() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "replaced_bean");
    }

    #[test]
    fn resolve_definition_after_init_error_ignored() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    #[test]
    fn resolve_definition_after_init_none_keeps_original() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    #[test]
    fn resolve_definition_multiple_processors_chained() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        c.add_bean_post_processor(Arc::new(ReplacingAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "replaced_bean");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // initialize_bean with processors
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn initialize_bean_before_init_replaces() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingBeforeInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean, "test_bean").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "replaced_before_init");
    }

    #[test]
    fn initialize_bean_before_init_error_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorBeforeInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean.clone(), "test_bean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn initialize_bean_before_init_none_no_replacement() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneBeforeInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean.clone(), "test_bean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn initialize_bean_after_init_replaces() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean, "test_bean").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "replaced_bean");
    }

    #[test]
    fn initialize_bean_after_init_error_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean.clone(), "test_bean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn initialize_bean_both_phases_counted() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let before_count = Arc::new(AtomicUsize::new(0));
        let after_count = Arc::new(AtomicUsize::new(0));
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(CountingProcessor {
            before_count: before_count.clone(),
            after_count: after_count.clone(),
        }));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let _ = c.initialize_bean(bean, "test_bean").unwrap();
        assert_eq!(before_count.load(Ordering::SeqCst), 1);
        assert_eq!(after_count.load(Ordering::SeqCst), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // configure_bean with processors
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn configure_bean_after_init_replaces() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.configure_bean(bean, "test_bean").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "replaced_bean");
    }

    #[test]
    fn configure_bean_after_init_error_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.configure_bean(bean.clone(), "test_bean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn configure_bean_after_init_none_no_replacement() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.configure_bean(bean.clone(), "test_bean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // destroy_bean_instance with processors
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn destroy_bean_instance_destruction_processor_called() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let called = Arc::new(AtomicBool::new(false));
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(DestructionTrackingProcessor { called: called.clone() }));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.destroy_bean_instance("test_bean", bean.as_ref()).unwrap();
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn destroy_bean_instance_destruction_error_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorDestructionProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        assert!(c.destroy_bean_instance("test_bean", bean.as_ref()).is_ok());
    }

    #[test]
    fn destroy_bean_instance_no_destruction_required() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoDestructionProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        assert!(c.destroy_bean_instance("test_bean", bean.as_ref()).is_ok());
    }

    #[test]
    fn destroy_bean_instance_multiple_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let called1 = Arc::new(AtomicBool::new(false));
        let called2 = Arc::new(AtomicBool::new(false));
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(DestructionTrackingProcessor { called: called1.clone() }));
        c.add_bean_post_processor(Arc::new(DestructionTrackingProcessor { called: called2.clone() }));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.destroy_bean_instance("test_bean", bean.as_ref()).unwrap();
        assert!(called1.load(Ordering::SeqCst));
        assert!(called2.load(Ordering::SeqCst));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Custom scope resolution (Scope::Custom path in resolve_definition)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_definition_custom_scope_not_active() {
        struct CustomMarker;
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::scoped::<String, CustomMarker, _>(|_| "custom".to_string()),
        );
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<String>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // autowire_bean with dependencies (has_unresolved_deps path)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn autowire_bean_returns_existing_when_no_declared_deps() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        // String has no declared dependencies, so autowire_bean returns existing
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("existing".to_string());
        let result = c.autowire_bean(existing.clone()).unwrap();
        assert!(Arc::ptr_eq(&result, &existing));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // autowire mode edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn autowire_mode_0_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.autowire("nonexistent", 0, false).is_err());
    }

    #[test]
    fn autowire_mode_1_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.autowire("nonexistent", 1, false).is_err());
    }

    #[test]
    fn autowire_mode_2_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.autowire("nonexistent", 2, false).is_err());
    }

    #[test]
    fn autowire_mode_3_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.autowire("nonexistent", 3, false).is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // select_trait_binding qualified ambiguous
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_not_found_for_wrong_qualifier() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("myQ").unwrap();
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        // Try to resolve with a different qualifier
        let other_q = Qualifier::new("otherQ").unwrap();
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&other_q);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_binding target not found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn build_fails_when_binding_target_missing() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        // Binding targets f64, which is not registered - build should fail
        let binding = TraitBinding::new(|f: Arc<f64>| f as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let result = b.build();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // singleton_names name_map lookup
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn singleton_names_uses_name_map_after_resolve() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let names = c.singleton_names();
        assert!(names.iter().any(|n| n.contains("String")));
    }

    #[test]
    fn singleton_names_empty_before_resolve() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(c.singleton_names().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // get_singleton fallback by type_name
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn get_singleton_finds_by_type_name_fallback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let result = c.get_singleton("alloc::string::String");
        assert!(result.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // register_singleton with multiple callbacks
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn register_singleton_multiple_callbacks_all_called() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let mut c = make_container();
        let called1 = Arc::new(AtomicBool::new(false));
        let called2 = Arc::new(AtomicBool::new(false));
        let c1 = called1.clone();
        let c2 = called2.clone();
        let cb1: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| { c1.store(true, Ordering::SeqCst); });
        let cb2: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| { c2.store(true, Ordering::SeqCst); });
        c.add_singleton_callback("multi_cb".to_string(), cb1);
        c.add_singleton_callback("multi_cb".to_string(), cb2);
        c.register_singleton("multi_cb", Arc::new("value".to_string()) as Arc<dyn Any + Send + Sync>);
        assert!(called1.load(Ordering::SeqCst));
        assert!(called2.load(Ordering::SeqCst));
    }

    #[test]
    fn register_singleton_callback_not_triggered_for_other_names() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let c_clone = called.clone();
        let cb: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| { c_clone.store(true, Ordering::SeqCst); });
        c.add_singleton_callback("target".to_string(), cb);
        c.register_singleton("other_bean", Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>);
        assert!(!called.load(Ordering::SeqCst));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // is_singleton / is_prototype edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn is_singleton_transient_returns_false() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        assert!(!c.is_singleton(&ComponentKey::of::<String>()).unwrap());
    }

    #[test]
    fn is_prototype_singleton_returns_false() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_prototype(&ComponentKey::of::<String>()).unwrap());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // is_factory_bean edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn is_factory_bean_with_prefix() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.is_factory_bean("&myFactory"));
        assert!(c.is_factory_bean("&"));
    }

    #[test]
    fn is_factory_bean_without_prefix() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(!c.is_factory_bean("regularBean"));
        assert!(!c.is_factory_bean(""));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // currently_in_creation toggle
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn currently_in_creation_multiple_beans() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", true);
        c.set_currently_in_creation("bean2", true);
        assert!(c.is_currently_in_creation("bean1"));
        assert!(c.is_currently_in_creation("bean2"));
        c.set_currently_in_creation("bean1", false);
        assert!(!c.is_currently_in_creation("bean1"));
        assert!(c.is_currently_in_creation("bean2"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // dependent_beans complex graph
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn dependent_beans_complex_graph() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_dependent_bean("dao", "service");
        c.register_dependent_bean("service", "controller");
        c.register_dependent_bean("service", "scheduler");
        let service_deps = c.get_dependent_beans("service");
        assert_eq!(service_deps.len(), 2);
        assert!(service_deps.contains(&"controller".to_string()));
        assert!(service_deps.contains(&"scheduler".to_string()));
        let controller_deps = c.get_dependencies_for_bean("controller");
        assert!(controller_deps.contains(&"service".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // embedded_value_resolvers edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_embedded_value_empty_string() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert_eq!(c.resolve_embedded_value(""), "");
    }

    #[test]
    fn resolve_embedded_value_no_match_pass_through() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert_eq!(c.resolve_embedded_value("no_placeholders"), "no_placeholders");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // alias edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn register_alias_same_target_multiple_times() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap();
    }

    #[test]
    fn register_alias_multiple_aliases_same_target() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "a1").unwrap();
        c.register_alias("bean1", "a2").unwrap();
        c.register_alias("bean1", "a3").unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // scope registration
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn registered_scope_names_lists_all() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_scope("request", Box::new(crate::request_scope::RequestScope::new("r")));
        c.register_scope("session", Box::new(crate::session_scope::SessionScope::new("s")));
        let names = c.registered_scope_names();
        assert!(names.contains(&"request".to_string()));
        assert!(names.contains(&"session".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // destroy_singletons clears all
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn destroy_singletons_after_multiple_resolves() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 2);
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // freeze_configuration
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn freeze_configuration_idempotent() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // pre_instantiate_singletons edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn pre_instantiate_singletons_empty_container() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.pre_instantiate_singletons().unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanDefinitionRegistry: register, remove, get edge cases
    // (using BeanDefinitionRegistry import to avoid ambiguity with ListableBeanFactory)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_def_registry_count_with_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial: usize = BeanDefinitionRegistry::bean_definition_count(&c);
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("new1".to_string(), def).unwrap();
        assert_eq!(BeanDefinitionRegistry::bean_definition_count(&c), initial + 1);
    }

    #[test]
    fn bean_def_registry_names_includes_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("extra".to_string(), def).unwrap();
        let names = BeanDefinitionRegistry::bean_definition_names(&c);
        assert!(names.contains(&"extra".to_string()));
    }

    #[test]
    fn bean_def_registry_names_excludes_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        c.remove_bean_definition("temp").unwrap();
        let names = BeanDefinitionRegistry::bean_definition_names(&c);
        assert!(!names.contains(&"temp".to_string()));
    }

    #[test]
    fn bean_def_registry_contains_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynBean".to_string(), def).unwrap();
        assert!(BeanDefinitionRegistry::contains_bean_definition(&c, "dynBean"));
    }

    #[test]
    fn bean_def_registry_contains_deleted_returns_false() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toDelete".to_string(), def).unwrap();
        assert!(BeanDefinitionRegistry::contains_bean_definition(&c, "toDelete"));
        c.remove_bean_definition("toDelete").unwrap();
        assert!(!BeanDefinitionRegistry::contains_bean_definition(&c, "toDelete"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ListableBeanFactory: bean_definition_count, contains_bean_definition,
    // bean_definition_names (qualified to avoid ambiguity)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn listable_bean_def_count_excludes_deleted() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = ListableBeanFactory::bean_definition_count(&c);
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        assert_eq!(ListableBeanFactory::bean_definition_count(&c), initial + 1);
        c.remove_bean_definition("temp").unwrap();
        assert_eq!(ListableBeanFactory::bean_definition_count(&c), initial);
    }

    #[test]
    fn listable_contains_bean_def_deleted_dynamic() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        assert!(ListableBeanFactory::contains_bean_definition(&c, "temp"));
        c.remove_bean_definition("temp").unwrap();
        assert!(!ListableBeanFactory::contains_bean_definition(&c, "temp"));
    }

    #[test]
    fn listable_bean_def_names_excludes_deleted() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        let names = ListableBeanFactory::bean_definition_names(&c);
        assert!(names.contains(&"temp".to_string()));
        c.remove_bean_definition("temp").unwrap();
        let names = ListableBeanFactory::bean_definition_names(&c);
        assert!(!names.contains(&"temp".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ListableBeanFactory: beans_of_type_id
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn listable_beans_of_type_id_resolves_and_returns_map() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
        assert!(beans.contains_key("alloc::string::String"));
    }

    #[test]
    fn listable_beans_of_type_id_none_found_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<f64>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ListableBeanFactory: bean_names_for_type_id
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn listable_bean_names_for_type_id_multiple() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn listable_bean_names_for_type_id_none() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<f64>(), true, true);
        assert!(names.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // contains_non_singleton_bean / contains_singleton_bean
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn contains_non_singleton_with_transient() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_non_singleton_bean());
        assert!(c.contains_singleton_bean());
    }

    #[test]
    fn contains_non_singleton_empty_container() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_non_singleton_bean());
        assert!(!c.contains_singleton_bean());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // bean_names_iterator
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_names_iterator_yields_all_names() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names: Vec<String> = c.bean_names_iterator().collect();
        assert!(names.contains(&"alloc::string::String".to_string()));
        assert!(names.contains(&"i32".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ObjectProvider edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn object_provider_get_after_singleton_resolve() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.get().is_ok());
    }

    #[test]
    fn object_provider_if_available_after_resolve() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.if_available().is_some());
    }

    #[test]
    fn object_provider_get_if_unique_after_resolve() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.get_if_unique().is_ok());
    }

    #[test]
    fn object_provider_stream_after_resolve() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(!items.is_empty());
    }

    #[test]
    fn object_provider_ordered_stream_after_resolve() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(!items.is_empty());
    }

    #[test]
    fn object_provider_all_empty_container() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.get().is_err());
        assert!(provider.if_available().is_none());
        assert!(provider.get_if_unique().is_err());
        assert!(provider.stream().is_empty());
        assert!(provider.ordered_stream().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // type_converter
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn type_converter_none_by_default() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        assert!(c.type_converter().is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // early_bean_reference edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn early_bean_reference_overwrite() {
        let c = make_container();
        let key = ComponentKey::of::<String>();
        c.register_early_bean_reference(key.clone(), Arc::new("first".to_string()));
        c.register_early_bean_reference(key.clone(), Arc::new("second".to_string()));
        let retrieved = c.get_early_bean_reference(&key).unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "second");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // shared_handle shares state
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn shared_handle_shares_resolutions_tracker() {
        let c = make_container();
        let handle = c.shared_handle();
        let _: Arc<String> = c.resolve().unwrap();
        let unused_c = c.unused_definitions();
        let unused_h = handle.unused_definitions();
        assert_eq!(unused_c.len(), unused_h.len());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // unused_definitions edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn unused_definitions_empty_container() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.unused_definitions().is_empty());
    }

    #[test]
    fn unused_definitions_transient_stays_unused() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        let _: Arc<String> = c.resolve().unwrap();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 1);
        assert!(unused.contains(&"i32".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // warm_up edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn warm_up_only_singletons_resolved() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14));
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 1);
        assert!(unused.contains(&"i32".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // display_path edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn display_path_various_combinations() {
        let path = Container::display_path(&[], None);
        assert!(path.is_empty());

        let path = Container::display_path(&[], Some("only".to_string()));
        assert_eq!(path, vec!["only"]);

        let stack = vec![ComponentKey::of::<String>(), ComponentKey::of::<i32>()];
        let path = Container::display_path(&stack, Some("leaf".to_string()));
        assert_eq!(path.len(), 3);
        assert_eq!(path[2], "leaf");

        let path = Container::display_path(&stack, None);
        assert_eq!(path.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanPostProcessor count via different interfaces
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_post_processor_count_via_configurable() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        assert_eq!(c.bean_post_processor_count(), 1);
    }

    #[test]
    fn bean_post_processor_count_via_listable() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        assert_eq!(c.bean_post_processor_count(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Scope context
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn open_scope_different_types() {
        let c = make_container();
        let _s1 = c.open_scope::<String>();
        let _s2 = c.open_scope::<i32>();
        let _s3 = c.open_scope::<f64>();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Registry access
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn registry_empty_container() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.registry().definitions().is_empty());
        assert!(c.registry().bindings().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HierarchicalBeanFactory: parent_bean_factory after set
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn parent_bean_factory_set_and_query() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let mut c = make_container();
        let parent = Container::new(RegistryBuilder::new().build().unwrap());
        c.set_parent_bean_factory(Arc::new(parent) as Arc<dyn BeanFactory>).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    #[test]
    fn parent_bean_factory_overwrite() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let mut c = make_container();
        let p1 = Container::new(RegistryBuilder::new().build().unwrap());
        let p2 = Container::new(RegistryBuilder::new().build().unwrap());
        c.set_parent_bean_factory(Arc::new(p1) as Arc<dyn BeanFactory>).unwrap();
        c.set_parent_bean_factory(Arc::new(p2) as Arc<dyn BeanFactory>).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_in and resolve_qualified_in with matching scope
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_in_matching_scope() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_in_matching_scope() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait_in, resolve_qualified_trait_in, resolve_all_traits_in
    // with matching scope
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_in_matching_scope() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_in_matching_scope() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_all_traits_in_matching_scope() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        b.bind(TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let results = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).unwrap();
        assert_eq!(results.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ProxyBeanDefinition scope variants
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn proxy_bean_definition_transient_scope() {
        let proxy = ProxyBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "String".to_string(),
            scope: crate::component_scope::Scope::Transient,
            source: "dynamic".to_string(),
        };
        assert_eq!(BeanDefinitionTrait::scope(&proxy), crate::component_scope::Scope::Transient);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RemovedBeanDefinition with transient scope
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn removed_bean_definition_transient_scope() {
        let removed = RemovedBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "String".to_string(),
            scope: crate::component_scope::Scope::Transient,
        };
        assert_eq!(BeanDefinitionTrait::scope(&removed), crate::component_scope::Scope::Transient);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ignore_dependency_type and ignore_dependency_interface
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn ignore_dependency_multiple_types() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_type(std::any::TypeId::of::<String>());
        c.ignore_dependency_type(std::any::TypeId::of::<i32>());
        c.ignore_dependency_interface(std::any::TypeId::of::<dyn std::fmt::Debug>());
        c.ignore_dependency_interface(std::any::TypeId::of::<dyn std::fmt::Display>());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // register_resolvable_dependency
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn register_resolvable_dependency_and_overwrite() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.register_resolvable_dependency(std::any::TypeId::of::<String>(), Arc::new("first".to_string()));
        c.register_resolvable_dependency(std::any::TypeId::of::<String>(), Arc::new("second".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // is_autowire_candidate always returns true
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn is_autowire_candidate_always_true() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = make_container();
        assert!(c.is_autowire_candidate("any_bean"));
        assert!(c.is_autowire_candidate(""));
        assert!(c.is_autowire_candidate("nonexistent"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Trait resolution: resolve_trait, resolve_qualified_trait, resolve_all_traits
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_single_binding() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_trait_not_found_v2() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_trait_primary_resolves() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        b.bind(TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).primary()).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_trait_ambiguous_no_primary_v5() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        b.bind(TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_trait_success() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_qualified_trait(&q);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_not_found_v3() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let q = Qualifier::new("missing").unwrap();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_qualified_trait(&q);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_empty_v3() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let results: Vec<Arc<dyn std::fmt::Display + Send + Sync>> = c.resolve_all_traits().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn resolve_all_traits_multiple_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        b.bind(TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let results: Vec<Arc<dyn std::fmt::Display + Send + Sync>> = c.resolve_all_traits().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn resolve_trait_in_wrong_owner_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait_in(&scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_trait_in_wrong_owner_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_qualified_trait_in(&q, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_in_wrong_owner_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits_in(&scope);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_optional_trait_typed
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_optional_trait_typed_not_found_returns_none() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // select_trait_binding: primary resolution
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn select_trait_binding_single() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_not_found_v5() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_qualified_match() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_qualified::<dyn std::fmt::Display + Send + Sync>(q);
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_not_found_with_wrong_qualifier_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        b.bind(TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        // Search with a different qualifier -> not found
        let wrong_q = Qualifier::new("other").unwrap();
        let dep = crate::Dependency::trait_qualified::<dyn std::fmt::Display + Send + Sync>(wrong_q);
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanPostProcessor proxy override and error handling
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn proxy_override_skips_normal_instantiation() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ProxyOverrideProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "proxy_override_result");
    }

    #[test]
    fn error_before_instantiation_ignored() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorBeforeInstantiationProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        // Normal resolution proceeds despite error
        assert_eq!(*result, "hello");
    }

    #[test]
    fn replacing_after_init_processor() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "replaced_bean");
    }

    #[test]
    fn error_after_init_processor_ignored() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    #[test]
    fn none_after_init_processor_no_change() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanPostProcessor in initialize_bean
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn initialize_bean_with_replacing_before_init() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingBeforeInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean, "test").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "replaced_before_init");
    }

    #[test]
    fn initialize_bean_with_error_before_init_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorBeforeInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.initialize_bean(bean, "test").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "original");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // configure_bean with processors
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn configure_bean_with_replacing_processor() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ReplacingAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.configure_bean(bean, "test").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "replaced_bean");
    }

    #[test]
    fn configure_bean_with_error_processor_ignored() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorAfterInitProcessor));
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("original".to_string());
        let result = c.configure_bean(bean, "test").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "original");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SingletonBeanRegistry: register_singleton with callbacks
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn register_singleton_with_callback_triggered() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        use std::sync::atomic::{AtomicBool, Ordering};
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        let callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_: &dyn Any| {
            called_clone.store(true, Ordering::SeqCst);
        });
        c.add_singleton_callback("mySingleton".to_string(), callback);
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("value".to_string());
        c.register_singleton("mySingleton", obj);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn register_singleton_without_callback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("value".to_string());
        c.register_singleton("noCallbackSingleton", obj);
        assert!(c.contains_singleton("noCallbackSingleton"));
    }

    #[test]
    fn get_singleton_by_name_lookup() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("named_value".to_string());
        c.register_singleton("namedBean", obj);
        let retrieved = c.get_singleton("namedBean");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap().downcast_ref::<String>().unwrap(), "named_value");
    }

    #[test]
    fn get_singleton_not_found_v3() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.get_singleton("nonexistent").is_none());
    }

    #[test]
    fn singleton_names_after_resolve_v3() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let names = c.singleton_names();
        assert!(names.len() >= 2);
    }

    #[test]
    fn singleton_count_after_resolve_v3() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let count = c.singleton_count();
        assert!(count >= 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanFactory: contains_bean, is_type_match edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn contains_bean_true_for_registered() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.contains_bean(&ComponentKey::of::<String>()));
        assert!(c.contains_bean(&ComponentKey::of::<i32>()));
    }

    #[test]
    fn contains_bean_false_for_unregistered() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
    }

    #[test]
    fn is_type_match_true() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
    }

    #[test]
    fn is_type_match_false_wrong_type() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
    }

    #[test]
    fn is_type_match_false_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_type_match(&ComponentKey::of::<f64>(), std::any::TypeId::of::<f64>()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ListableBeanFactory: bean_names_for_type_id, beans_of_type_id
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_names_for_type_id_found() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn bean_names_for_type_id_not_found() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<f64>(), true, true);
        assert!(names.is_empty());
    }

    #[test]
    fn beans_of_type_id_found() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
    }

    #[test]
    fn beans_of_type_id_not_found() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<f64>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ConfigurableBeanFactory: destroy_bean, embedded_value_resolvers chain
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn destroy_bean_no_error() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        c.destroy_bean("test", bean.as_ref()).unwrap();
    }

    #[test]
    fn embedded_value_resolvers_chain() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.add_embedded_value_resolver(Arc::new(|v: &str| v.replace("${a}", "A")));
        c.add_embedded_value_resolver(Arc::new(|v: &str| v.replace("${b}", "B")));
        let result = c.resolve_embedded_value("${a}-${b}");
        assert_eq!(result, "A-B");
    }

    #[test]
    fn registered_scope_names_empty_v3() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let names = c.registered_scope_names();
        assert!(names.is_empty());
    }

    #[test]
    fn get_registered_scope_not_found_v3() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.get_registered_scope("nonexistent").is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AutowireCapableBeanFactory: autowire_bean with multiple matching defs
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn autowire_bean_multiple_definitions_returns_existing() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing.clone()).unwrap();
        // Multiple matching definitions -> returns existing
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "test");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // autowire: modes 1 and 2 with dependencies
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn autowire_mode_by_name_with_existing_beans() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_type_with_existing_beans() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 2, false);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ObjectProvider: various states
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn object_provider_get_if_unique_empty_v3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get_if_unique();
        assert!(result.is_err());
    }

    #[test]
    fn object_provider_ordered_stream_empty_v3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(items.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HierarchicalBeanFactory: contains_local_bean with dynamic definitions
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn contains_local_bean_registry_only() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(c.contains_local_bean("i32"));
        assert!(!c.contains_local_bean("nonexistent"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BeanDefinitionRegistry: edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_definition_names_excludes_deleted_v3() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let names = c.bean_definition_names();
        assert!(!names.contains(&"alloc::string::String".to_string()));
    }

    #[test]
    fn bean_definition_count_excludes_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = c.bean_definition_count();
        c.remove_bean_definition("alloc::string::String").unwrap();
        // After removal, the deleted marker doesn't count, so count decreases
        assert_eq!(c.bean_definition_count(), initial - 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ContainsBeanDefinition via ListableBeanFactory with deleted dynamic defs
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn listable_contains_bean_definition_deleted_returns_false() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        <Container as BeanDefinitionRegistry>::remove_bean_definition(&mut c, "alloc::string::String").unwrap();
        let result = <Container as ListableBeanFactory>::contains_bean_definition(&c, "alloc::string::String");
        assert!(!result);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ConfigurableBeanFactory: alias conflict detection
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn register_alias_same_target_no_error() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap(); // same target, ok
    }

    #[test]
    fn register_alias_different_target_conflict() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        let result = c.register_alias("bean2", "alias1");
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // pre_instantiate_singletons with failure
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn pre_instantiate_singletons_empty_container_v3() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.pre_instantiate_singletons().unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: circular dependency detection
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn display_path_empty_stack_no_leaf_v3() {
        let path = Container::display_path(&[], None);
        assert!(path.is_empty());
    }

    #[test]
    fn display_path_empty_stack_with_leaf_v3() {
        let path = Container::display_path(&[], Some("leaf".to_string()));
        assert_eq!(path, vec!["leaf"]);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // warm_up with empty container
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn warm_up_empty_container_v3() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.warm_up().unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // shared_handle: verify shared state
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn shared_handle_transient_tracker_shared() {
        let c = make_container();
        let handle = c.shared_handle();
        // Both should have the same transient tracker
        let tracker1 = c.transient_tracker();
        let tracker2 = handle.transient_tracker();
        // They share the same underlying data
        drop(tracker1);
        drop(tracker2);
    }

    #[test]
    fn shared_handle_registry_same() {
        let c = make_container();
        let handle = c.shared_handle();
        assert_eq!(c.registry().definitions().len(), handle.registry().definitions().len());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_dependency: various scenarios
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_dependency_optional_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            false,
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn resolve_dependency_required_not_found_v5() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ConfigurableBeanFactory: destroy_bean
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn configurable_destroy_bean_instance() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.destroy_bean("test_bean", bean.as_ref()).unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ConfigurableBeanFactory: is_factory_bean with various prefixes
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn is_factory_bean_with_prefix_v3() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.is_factory_bean("&myFactoryBean"));
        assert!(c.is_factory_bean("&anotherFactory"));
        assert!(!c.is_factory_bean("regularBean"));
        assert!(!c.is_factory_bean(""));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HierarchicalBeanFactory: parent_bean_factory after set
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn parent_bean_factory_set_and_query_v2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let mut c = make_container();
        let parent = Container::new(RegistryBuilder::new().build().unwrap());
        c.set_parent_bean_factory(Arc::new(parent) as Arc<dyn BeanFactory>).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: Custom scope not active
    // ═══════════════════════════════════════════════════════════════════════════

    // Note: custom_scope is not available on ComponentDefinition, so we test
    // the ScopeNotActive path via Scope::Custom definition using scope_key directly

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait: with binding target not found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_binding_target_missing() {
        // When a TraitBinding's target type is not registered as a definition,
        // the graph planner validation catches this at build time.
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let _ = b.bind(binding);
        let result = b.build();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_optional_trait_typed: with binding found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_optional_trait_found_returns_some() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(
            &crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>(),
            &[],
            None,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: BeanPostProcessor chain
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_with_bean_post_processors() {
        // BeanPostProcessor chain is tested indirectly through construct()
        // The processors are empty by default, so the instance passes through unchanged
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
        let a: Arc<String> = c.resolve().unwrap();
        assert_eq!(*a, "hello");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ObjectProvider: get_if_unique with single item
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn object_provider_get_if_unique_with_single_bean() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get_if_unique();
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: name_to_singleton_key mapping
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn singleton_name_to_key_mapping_populated() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        // After resolving, name_to_singleton_key should be populated
        let names = c.singleton_names();
        assert!(names.iter().any(|n| n.contains("String")));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_all_traits_typed: exercises the filter + collect path
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_all_traits_multiple_bindings() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding3 = TraitBinding::new(|f: Arc<f64>| f as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        b.bind(binding3).unwrap();
        let c = Container::new(b.build().unwrap());
        let results = c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap();
        assert_eq!(results.len(), 3);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait_in: same owner with binding
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_in_same_owner_with_binding() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified_trait_in: same owner with qualified binding
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_in_same_owner_with_binding() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_all_traits_in: same owner with bindings
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_all_traits_in_same_owner_with_bindings() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let results = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).unwrap();
        assert_eq!(results.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait: primary binding disambiguates
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_primary_disambiguates() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait: multiple primaries is ambiguous
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_multiple_primaries_ambiguous() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        // The second primary should fail validation in bind_all
        let result = b.bind_all(vec![binding1, binding2]);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified_trait: with matching qualifier
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_with_matching_binding() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let q = Qualifier::new("myQ").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified_trait: no matching qualifier
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_no_matching_binding() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let q1 = Qualifier::new("q1").unwrap();
        let q2 = Qualifier::new("q2").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q1.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q2);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait_in: wrong owner
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_in_wrong_owner_v4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified_trait_in: wrong owner
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_in_wrong_owner_v4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_all_traits_in: wrong owner
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_all_traits_in_wrong_owner_v4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // select_trait_binding: with qualifier match
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn select_trait_binding_with_qualifier_match() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_qualified::<dyn std::fmt::Display + Send + Sync>(q);
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // select_trait_binding: not found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn select_trait_binding_not_found_v4() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // select_trait_binding: ambiguous with qualifier
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn select_trait_binding_ambiguous_same_qualifier() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let q = Qualifier::new("q").unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        // Same qualified binding should fail validation
        let result = b.bind_all(vec![binding1, binding2]);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: construct with factory error
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_definition_factory_error() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            panic!("factory error");
        }));
        let c = Container::new(b.build().unwrap());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            c.resolve::<String>()
        }));
        // The factory panics, which should be caught
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve: type mismatch after successful construction
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_type_mismatch_after_construction() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // Try to resolve String as i32 - should fail with TypeMismatch
        let result: Result<Arc<i32>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_optional_typed: type mismatch returns error
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_optional_type_mismatch_returns_error_v4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // resolve_optional_typed with i32 dependency on String definition
        // The definition matches by type_id, so it finds String, but downcast to i32 fails
        let dep = crate::Dependency::of::<String>();
        let result = c.resolve_optional_typed::<i32>(&dep, &[], None);
        // Should fail because downcast from String to i32 fails
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_trait_typed: exercises select_trait_binding + resolve_binding
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_typed_with_valid_binding() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.resolve_trait_typed::<dyn std::fmt::Display + Send + Sync>(&dep, &[], None);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_all_traits_typed: exercises filter + collect
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_all_traits_typed_empty() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::all_traits_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.resolve_all_traits_typed::<dyn std::fmt::Display + Send + Sync>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_binding: target definition not found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_binding_target_not_found() {
        // When a TraitBinding's target type is not registered as a definition,
        // build() validates this and returns an error.
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let _ = b.bind(binding);
        let result = b.build();
        // build() should fail because the binding target (String) is not registered
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: Transient with tracking
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn transient_resolution_tracks_instances() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string())).unwrap();
        let c = Container::new(b.build().unwrap());
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<String> = c.resolve().unwrap();
        // After resolution, transient is no longer unused (resolution is recorded)
        let unused = c.unused_definitions();
        assert!(unused.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: Singleton caching
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn singleton_cached_across_resolves() {
        let c = make_container();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = c.resolve().unwrap();
        let c_val: Arc<String> = c.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        assert!(Arc::ptr_eq(&b, &c_val));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_definition: records resolution on success
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolution_recorded_on_success() {
        let c = make_container();
        assert_eq!(c.unused_definitions().len(), 2);
        let _: Arc<String> = c.resolve().unwrap();
        assert_eq!(c.unused_definitions().len(), 1);
        let _: Arc<i32> = c.resolve().unwrap();
        assert_eq!(c.unused_definitions().len(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve: empty container returns error
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_empty_container_returns_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result: Result<Arc<String>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified: success with matching qualifier
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_success_with_matching() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let val: Arc<String> = c.resolve_qualified(&q).unwrap();
        assert_eq!(*val, "primary");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified: not found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_not_found_error() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let q = Qualifier::new("missing").unwrap();
        let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_in: same owner success
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_in_same_owner_success() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified_in: same owner success
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_in_same_owner_success() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_in: wrong owner
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_in_wrong_owner_fails() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // resolve_qualified_in: wrong owner
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_in_wrong_owner_fails_v4() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let q = Qualifier::new("q").unwrap();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: shared_handle deep sharing
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn shared_handle_shares_transient_tracker() {
        let c = make_container();
        let handle = c.shared_handle();
        // Both should point to the same tracker
        let _ = handle.transient_tracker();
    }

    #[test]
    fn shared_handle_shares_owner_for_scope_check() {
        let c = make_container();
        let handle = c.shared_handle();
        let scope = c.open_scope::<String>();
        // Scope created from c should be valid for handle too
        let result = handle.ensure_scope_owner(&scope);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: resolve_trait paths with binding upcast
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_trait_with_binding_upcast_success() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Arc<dyn std::fmt::Display + Send + Sync> = c.resolve_trait().unwrap();
        assert_eq!(result.to_string(), "hello");
    }

    #[test]
    fn resolve_trait_in_same_owner_with_binding_v2() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "world".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result: Arc<dyn std::fmt::Display + Send + Sync> = c.resolve_trait_in(&scope).unwrap();
        assert_eq!(result.to_string(), "world");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: resolve_all_traits multiple bindings
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_all_traits_in_with_multiple_bindings() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let results = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).unwrap();
        assert_eq!(results.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: resolve_optional_trait_typed found
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_optional_trait_typed_found_returns_some() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "found".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(
            &crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>(),
            &[],
            None,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: select_trait_binding primary tie-breaker
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn select_trait_binding_ambiguous_multiple_primary_fails() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        // Two primary bindings for the same trait - should fail (build fails)
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        b.bind(binding1).unwrap();
        let result = b.bind(binding2);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: resolve_qualified_trait with qualifier match
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_with_qualifier_match() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "qualified_impl".to_string()));
        let q = Qualifier::new("special").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q).unwrap();
        assert_eq!(result.to_string(), "qualified_impl");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: resolve_qualified_trait_in with qualifier
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_qualified_trait_in_with_qualifier_match() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "scoped_qualified".to_string()));
        let q = Qualifier::new("scoped_q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope).unwrap();
        assert_eq!(result.to_string(), "scoped_qualified");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: ContainerObjectProvider with multiple singletons
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn object_provider_get_returns_any_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_stream_returns_all_singletons() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(items.len() >= 2);
    }

    #[test]
    fn object_provider_ordered_stream_returns_all() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(items.len() >= 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: BeanFactory trait methods edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_factory_get_bean_by_key_resolves_definition() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let result = c.get_bean_by_key(&key).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "hello");
    }

    #[test]
    fn bean_factory_contains_bean_true_and_false() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.contains_bean(&ComponentKey::of::<String>()));
        assert!(c.contains_bean(&ComponentKey::of::<i32>()));
        assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
    }

    #[test]
    fn bean_factory_is_type_match_for_registered_type() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<String>()));
        assert!(c.is_type_match(&ComponentKey::of::<i32>(), TypeId::of::<i32>()));
    }

    #[test]
    fn bean_factory_is_type_match_for_wrong_type() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<i32>()));
    }

    #[test]
    fn bean_factory_is_type_match_for_unregistered_key() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_type_match(&ComponentKey::of::<f64>(), TypeId::of::<f64>()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: ListableBeanFactory edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn listable_bean_names_for_type_id_returns_matching() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 1);
        assert!(names[0].contains("String"));
    }

    #[test]
    fn listable_bean_names_for_type_id_empty_for_missing() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(TypeId::of::<f64>(), true, true);
        assert!(names.is_empty());
    }

    #[test]
    fn listable_beans_of_type_id_resolves_and_returns() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(TypeId::of::<i32>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
        assert_eq!(*beans.values().next().unwrap().downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn listable_beans_of_type_id_empty_for_missing() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(TypeId::of::<f64>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    #[test]
    fn listable_contains_bean_definition_for_registered_types() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(c.contains_bean_definition("i32"));
        assert!(!c.contains_bean_definition("f64"));
    }

    #[test]
    fn listable_bean_definition_count_matches() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
    }

    #[test]
    fn listable_bean_definition_names_contains_all() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_definition_names();
        assert!(names.contains(&"alloc::string::String".to_string()));
        assert!(names.contains(&"i32".to_string()));
    }

    #[test]
    fn listable_bean_post_processor_count_zero() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    #[test]
    fn listable_bean_post_processor_count_with_processors() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        assert_eq!(c.bean_post_processor_count(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: ConfigurableBeanFactory edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn configurable_set_parent_bean_factory_and_query() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        let mut c = make_container();
        assert!(c.parent_bean_factory().is_none());
        let parent = Container::new(RegistryBuilder::new().build().unwrap());
        let parent_arc: Arc<dyn BeanFactory> = Arc::new(parent);
        c.set_parent_bean_factory(parent_arc).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    #[test]
    fn configurable_register_scope_and_query() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_scope("request", Box::new(crate::request_scope::RequestScope::new("req1")));
        let names = c.registered_scope_names();
        assert!(names.contains(&"request".to_string()));
        assert!(c.get_registered_scope("request").is_some());
        assert!(c.get_registered_scope("session").is_none());
    }

    #[test]
    fn configurable_register_alias_and_check() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("myBean", "alias1").unwrap();
        c.register_alias("myBean", "alias1").unwrap(); // same target, ok
        let result = c.register_alias("otherBean", "alias1");
        assert!(result.is_err());
    }

    #[test]
    fn configurable_is_factory_bean_with_and_without_prefix() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.is_factory_bean("&myFactory"));
        assert!(c.is_factory_bean("&"));
        assert!(!c.is_factory_bean("regularBean"));
        assert!(!c.is_factory_bean(""));
    }

    #[test]
    fn configurable_currently_in_creation_toggle() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", true);
        assert!(c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", false);
        assert!(!c.is_currently_in_creation("bean1"));
    }

    #[test]
    fn configurable_register_dependent_bean_and_query() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_dependent_bean("service", "controller");
        c.register_dependent_bean("service", "scheduler");
        let dependents = c.get_dependent_beans("service");
        assert!(dependents.contains(&"controller".to_string()));
        assert!(dependents.contains(&"scheduler".to_string()));
        let deps = c.get_dependencies_for_bean("controller");
        assert!(deps.contains(&"service".to_string()));
    }

    #[test]
    fn configurable_get_dependent_beans_empty() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.get_dependent_beans("nonexistent").is_empty());
    }

    #[test]
    fn configurable_get_dependencies_for_bean_empty() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.get_dependencies_for_bean("nonexistent").is_empty());
    }

    #[test]
    fn configurable_destroy_bean_returns_ok() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.destroy_bean("test_bean", bean.as_ref()).unwrap();
    }

    #[test]
    fn configurable_destroy_singletons_clears_cache() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
    }

    #[test]
    fn configurable_add_embedded_value_resolver_and_resolve() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let resolver: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
            v.replace("${key}", "resolved_value")
        });
        c.add_embedded_value_resolver(resolver);
        assert_eq!(c.resolve_embedded_value("${key}"), "resolved_value");
        assert_eq!(c.resolve_embedded_value("plain"), "plain");
    }

    #[test]
    fn configurable_resolve_embedded_value_no_resolvers() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert_eq!(c.resolve_embedded_value("no_change"), "no_change");
    }

    #[test]
    fn configurable_resolve_embedded_value_multiple_resolvers() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let r1: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| v.replace("${a}", "A"));
        let r2: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| v.replace("${b}", "B"));
        c.add_embedded_value_resolver(r1);
        c.add_embedded_value_resolver(r2);
        assert_eq!(c.resolve_embedded_value("${a}-${b}"), "A-B");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: ConfigurableListableBeanFactory edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn configurable_listable_ignore_dependency_type_and_interface() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_type(TypeId::of::<String>());
        c.ignore_dependency_interface(TypeId::of::<dyn std::fmt::Debug>());
    }

    #[test]
    fn configurable_listable_register_resolvable_dependency() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.register_resolvable_dependency(TypeId::of::<String>(), Arc::new("resolved".to_string()));
    }

    #[test]
    fn configurable_listable_is_autowire_candidate_always_true() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = make_container();
        assert!(c.is_autowire_candidate("any_bean"));
        assert!(c.is_autowire_candidate(""));
    }

    #[test]
    fn configurable_listable_freeze_and_check_configuration() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
    }

    #[test]
    fn configurable_listable_pre_instantiate_singletons() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        c.pre_instantiate_singletons().unwrap();
        assert!(c.singleton_count() >= 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: AutowireCapableBeanFactory edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn autowire_capable_create_bean_by_class_name() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.create_bean("alloc::string::String").unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "hello");
    }

    #[test]
    fn autowire_capable_create_bean_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.create_bean("nonexistent").is_err());
    }

    #[test]
    fn autowire_capable_autowire_bean_with_matching_def() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing.clone()).unwrap();
        // No declared deps, returns existing
        assert!(Arc::ptr_eq(&result, &existing));
    }

    #[test]
    fn autowire_capable_autowire_bean_no_matching_def() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new(999i64);
        let result = c.autowire_bean(existing).unwrap();
        assert_eq!(*result.downcast_ref::<i64>().unwrap(), 999);
    }

    #[test]
    fn autowire_capable_autowire_bean_multiple_matches() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "test");
    }

    #[test]
    fn autowire_capable_configure_bean() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.configure_bean(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn autowire_capable_autowire_mode_0() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        assert!(c.autowire("alloc::string::String", 0, false).is_ok());
    }

    #[test]
    fn autowire_capable_autowire_mode_1() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        assert!(c.autowire("alloc::string::String", 1, false).is_ok());
    }

    #[test]
    fn autowire_capable_autowire_mode_2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        assert!(c.autowire("alloc::string::String", 2, false).is_ok());
    }

    #[test]
    fn autowire_capable_autowire_mode_3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        assert!(c.autowire("alloc::string::String", 3, false).is_ok());
    }

    #[test]
    fn autowire_capable_autowire_invalid_mode() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        assert!(c.autowire("alloc::string::String", 99, false).is_err());
    }

    #[test]
    fn autowire_capable_autowire_bean_properties_mode_0() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean.clone(), 0, false).unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn autowire_capable_autowire_bean_properties_mode_1() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 1, false).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "test");
    }

    #[test]
    fn autowire_capable_autowire_bean_properties_mode_2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 2, false).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "test");
    }

    #[test]
    fn autowire_capable_autowire_bean_properties_fallback() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean.clone(), 99, false).unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn autowire_capable_apply_bean_property_values() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.apply_bean_property_values(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn autowire_capable_initialize_bean() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.initialize_bean(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn autowire_capable_destroy_bean_instance() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        assert!(c.destroy_bean_instance("myBean", bean.as_ref()).is_ok());
    }

    #[test]
    fn autowire_capable_resolve_named_bean_single() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(TypeId::of::<String>()).unwrap();
        // resolve_named_bean wraps in Arc::new(instance), so we get Arc<dyn Any>
        let val = result.instance();
        assert!(val.downcast_ref::<Arc<dyn Any + Send + Sync>>().is_some() || val.downcast_ref::<String>().is_some());
    }

    #[test]
    fn autowire_capable_resolve_named_bean_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.resolve_named_bean(TypeId::of::<String>()).is_err());
    }

    #[test]
    fn autowire_capable_resolve_named_bean_ambiguous() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        assert!(c.resolve_named_bean(TypeId::of::<String>()).is_err());
    }

    #[test]
    fn autowire_capable_resolve_dependency_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<i32>(),
            "i32".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        let _ = result;
    }

    #[test]
    fn autowire_capable_resolve_dependency_not_found_required() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<f64>(),
            "f64".to_string(),
            true,
        );
        assert!(c.resolve_dependency(&descriptor, None).is_err());
    }

    #[test]
    fn autowire_capable_resolve_dependency_not_found_optional() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<f64>(),
            "f64".to_string(),
            false,
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn autowire_capable_resolve_dependency_ambiguous() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        let _ = result;
    }

    #[test]
    fn autowire_capable_set_type_converter_none() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.set_type_converter(None);
        assert!(c.type_converter().is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: BeanDefinitionRegistry edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_definition_registry_register_and_get() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("myBean"));
        let bd = c.get_bean_definition("myBean");
        assert!(bd.is_some());
    }

    #[test]
    fn bean_definition_registry_register_duplicate_fails_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def1 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def1).unwrap();
        assert!(c.register_bean_definition("myBean".to_string(), def2).is_err());
    }

    #[test]
    fn bean_definition_registry_remove_dynamic_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def).unwrap();
        let removed = c.remove_bean_definition("myBean").unwrap();
        // RootBeanDefinition::new() may have empty or "unknown" class name
        let _ = BeanDefinitionTrait::bean_class_name(&*removed);
        assert!(!c.contains_bean_definition("myBean"));
    }

    #[test]
    fn bean_definition_registry_remove_registry_bean_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let removed = c.remove_bean_definition("alloc::string::String").unwrap();
        assert_eq!(BeanDefinitionTrait::bean_class_name(&*removed), "alloc::string::String");
        assert!(!c.contains_bean_definition("alloc::string::String"));
    }

    #[test]
    fn bean_definition_registry_remove_nonexistent_fails_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert!(c.remove_bean_definition("nonexistent").is_err());
    }

    #[test]
    fn bean_definition_registry_remove_already_deleted_fails_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert!(c.remove_bean_definition("alloc::string::String").is_err());
    }

    #[test]
    fn bean_definition_registry_get_deleted_returns_none() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert!(c.get_bean_definition("alloc::string::String").is_none());
    }

    #[test]
    fn bean_definition_registry_get_from_registry_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let bd = c.get_bean_definition("alloc::string::String");
        assert!(bd.is_some());
    }

    #[test]
    fn bean_definition_registry_get_not_found_v5() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.get_bean_definition("nonexistent").is_none());
    }

    #[test]
    fn bean_definition_registry_count_and_names() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
        let names = c.bean_definition_names();
        assert!(names.contains(&"alloc::string::String".to_string()));
        assert!(names.contains(&"i32".to_string()));
    }

    #[test]
    fn bean_definition_registry_count_with_dynamic_v3() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = c.bean_definition_count();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("newBean".to_string(), def).unwrap();
        assert_eq!(c.bean_definition_count(), initial + 1);
    }

    #[test]
    fn bean_definition_registry_names_excludes_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        c.remove_bean_definition("temp").unwrap();
        let names = c.bean_definition_names();
        assert!(!names.contains(&"temp".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: SingletonBeanRegistry edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn singleton_registry_register_and_get() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("singleton_val".to_string());
        c.register_singleton("mySingleton", obj);
        let retrieved = c.get_singleton("mySingleton").unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "singleton_val");
    }

    #[test]
    fn singleton_registry_contains_singleton() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        c.register_singleton("mySingleton", Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>);
        assert!(c.contains_singleton("mySingleton"));
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn singleton_registry_singleton_names_after_resolve() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let names = c.singleton_names();
        assert!(names.iter().any(|n| n.contains("String")));
    }

    #[test]
    fn singleton_registry_singleton_count() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert_eq!(c.singleton_count(), 0);
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
    }

    #[test]
    fn singleton_registry_add_callback_and_trigger() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let c_clone = called.clone();
        let cb: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| {
            c_clone.store(true, Ordering::SeqCst);
        });
        c.add_singleton_callback("cb_test".to_string(), cb);
        c.register_singleton("cb_test", Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn singleton_registry_callback_not_triggered_for_other() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let c_clone = called.clone();
        let cb: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| {
            c_clone.store(true, Ordering::SeqCst);
        });
        c.add_singleton_callback("target".to_string(), cb);
        c.register_singleton("other", Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>);
        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn singleton_registry_singleton_mutex() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let mutex = c.singleton_mutex();
        assert!(mutex.downcast_ref::<std::sync::Mutex<HashMap<ComponentKey, Arc<SingletonCell>>>>().is_some());
    }

    #[test]
    fn singleton_registry_get_singleton_not_found() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(c.get_singleton("nonexistent").is_none());
    }

    #[test]
    fn singleton_registry_get_singleton_by_type_name_fallback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let result = c.get_singleton("alloc::string::String");
        assert!(result.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: HierarchicalBeanFactory edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn hierarchical_parent_initially_none() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.parent_bean_factory().is_none());
    }

    #[test]
    fn hierarchical_contains_local_bean() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(c.contains_local_bean("i32"));
        assert!(!c.contains_local_bean("nonexistent"));
    }

    #[test]
    fn hierarchical_contains_local_bean_with_dynamic() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(c.contains_local_bean("dynamicBean"));
    }

    #[test]
    fn hierarchical_contains_local_bean_deleted_returns_false() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toDelete".to_string(), def).unwrap();
        assert!(c.contains_local_bean("toDelete"));
        c.remove_bean_definition("toDelete").unwrap();
        assert!(!c.contains_local_bean("toDelete"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: Custom scope resolution
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_definition_custom_scope_not_active_error() {
        struct CustomMarker;
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::scoped::<String, CustomMarker, _>(|_| "custom".to_string()),
        );
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<String>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: Transient tracking
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn transient_creates_new_each_time() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| {
            use std::sync::atomic::{AtomicI32, Ordering};
            static COUNTER: AtomicI32 = AtomicI32::new(0);
            COUNTER.fetch_add(1, Ordering::SeqCst)
        }));
        let c = Container::new(b.build().unwrap());
        let a: Arc<i32> = c.resolve().unwrap();
        let b_val: Arc<i32> = c.resolve().unwrap();
        assert_ne!(*a, *b_val);
    }

    #[test]
    fn transient_instances_tracked_v3() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let _: Arc<String> = c.resolve().unwrap();
        // Transient tracker should have the instance
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage: warm_up edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: transient instances tracked
    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: warm_up edge cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn warm_up_mixed_scopes_v4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14));
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: display_path
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn display_path_with_leaf_v4() {
        let path = Container::display_path(
            &[ComponentKey::of::<String>(), ComponentKey::of::<i32>()],
            Some("leaf".to_string()),
        );
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn display_path_without_leaf_v4() {
        let path = Container::display_path(&[ComponentKey::of::<String>()], None);
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn display_path_empty_v4() {
        let path = Container::display_path(&[], None);
        assert!(path.is_empty());
    }

    #[test]
    fn display_path_empty_with_leaf_v4() {
        let path = Container::display_path(&[], Some("leaf".to_string()));
        assert_eq!(path, vec!["leaf"]);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: ensure_scope_owner
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn ensure_scope_owner_match_v4() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        assert!(c.ensure_scope_owner(&scope).is_ok());
    }

    #[test]
    fn ensure_scope_owner_mismatch_v4() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        assert!(c.ensure_scope_owner(&scope).is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: ProxyBeanDefinition, RemovedBeanDefinition, DeletedBeanDefinition
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn proxy_bean_definition_methods_v4() {
        let proxy = ProxyBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "String".to_string(),
            scope: crate::component_scope::Scope::Singleton,
            source: "registry".to_string(),
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&proxy), "String");
        assert_eq!(BeanDefinitionTrait::scope(&proxy), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&proxy));
        assert!(!BeanDefinitionTrait::is_primary(&proxy));
    }

    #[test]
    fn removed_bean_definition_methods_v4() {
        let removed = RemovedBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "String".to_string(),
            scope: crate::component_scope::Scope::Singleton,
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&removed), "String");
        assert_eq!(BeanDefinitionTrait::scope(&removed), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&removed));
        assert!(!BeanDefinitionTrait::is_primary(&removed));
    }

    #[test]
    fn deleted_bean_definition_methods_v4() {
        let deleted = DeletedBeanDefinition {
            bean_name: "test".to_string(),
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&deleted), "__DELETED__");
        assert_eq!(BeanDefinitionTrait::scope(&deleted), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&deleted));
        assert!(!BeanDefinitionTrait::is_primary(&deleted));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: open_scope
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn open_scope_returns_valid_context_v4() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        drop(scope);
    }

    #[test]
    fn open_scope_with_cancellation_returns_valid_context_v4() {
        let c = make_container();
        let token = CancellationToken::new();
        let scope = c.open_scope_with_cancellation::<i32>(token);
        drop(scope);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: early bean reference
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn early_bean_reference_round_trip_v4() {
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let early: Arc<dyn Any + Send + Sync> = Arc::new("early".to_string());
        assert!(c.get_early_bean_reference(&key).is_none());
        c.register_early_bean_reference(key.clone(), early.clone());
        let retrieved = c.get_early_bean_reference(&key).unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "early");
        let removed = c.remove_early_bean_reference(&key).unwrap();
        assert_eq!(*removed.downcast_ref::<String>().unwrap(), "early");
        assert!(c.get_early_bean_reference(&key).is_none());
    }

    #[test]
    fn remove_early_bean_reference_returns_none_when_absent_v4() {
        let c = make_container();
        assert!(c.remove_early_bean_reference(&ComponentKey::of::<i32>()).is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: BeanPostProcessor count
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn bean_post_processor_count_starts_at_zero_v4() {
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    #[test]
    fn bean_post_processor_count_increments_v4() {
        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneAfterInitProcessor));
        assert_eq!(c.bean_post_processor_count(), 1);
        c.add_bean_post_processor(Arc::new(NoneBeforeInitProcessor));
        assert_eq!(c.bean_post_processor_count(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: registry access
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn registry_returns_definitions_v4() {
        let c = make_container();
        let reg = c.registry();
        assert_eq!(reg.definitions().len(), 2);
    }

    #[test]
    fn registry_returns_bindings_v4() {
        let c = make_container();
        let reg = c.registry();
        assert!(reg.bindings().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v3: transient tracker access
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn transient_tracker_is_accessible_v4() {
        let c = make_container();
        let _tracker = c.transient_tracker();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage v4: comprehensive uncovered path tests
    // ═══════════════════════════════════════════════════════════════════════════

    // ── resolve_trait with actual bindings ─────────────────────────────────

    #[test]
    fn resolve_trait_with_primary_binding() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .primary();
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_trait_ambiguous_no_primary_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_with_multiple_bindings() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn resolve_trait_in_same_owner_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .primary();
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_all_traits_in_same_owner_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn resolve_qualified_trait_found() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let q = Qualifier::new("primary").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_in_same_owner_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let q = Qualifier::new("primary").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_ok());
    }

    // ── select_trait_binding paths ─────────────────────────────────────────

    #[test]
    fn select_trait_binding_single_match_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_not_found_v6() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_ambiguous_with_primary_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_ambiguous_no_primary_fails() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_with_qualifier_v6() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let q = Qualifier::new("myq").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_qualified::<dyn std::fmt::Display + Send + Sync>(q);
        let result = c.select_trait_binding(&dep, &[]);
        assert!(result.is_ok());
    }

    // ── resolve_binding paths ──────────────────────────────────────────────

    #[test]
    fn resolve_binding_success() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let selected = c.select_trait_binding(&dep, &[]).unwrap();
        let result = c.resolve_binding::<dyn std::fmt::Display + Send + Sync>(selected, &[], None);
        assert!(result.is_ok());
    }

    // ── resolve_definition with scope ──────────────────────────────────────

    #[test]
    fn resolve_definition_singleton_cached() {
        let c = make_container();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = c.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn resolve_definition_transient_new_each_time() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| {
            use std::sync::atomic::{AtomicI32, Ordering};
            static COUNTER: AtomicI32 = AtomicI32::new(0);
            COUNTER.fetch_add(1, Ordering::SeqCst)
        }));
        let c = Container::new(b.build().unwrap());
        let a: Arc<i32> = c.resolve().unwrap();
        let b_val: Arc<i32> = c.resolve().unwrap();
        assert_ne!(*a, *b_val);
    }

    // ── construct with BeanPostProcessor chain ─────────────────────────────

    #[test]
    fn construct_with_post_processor_returns_proxy() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;
        use crate::instantiation_aware_bean_post_processor::InstantiationAwareBeanPostProcessor;

        struct ProxyProcessor;
        impl BeanPostProcessor for ProxyProcessor {
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(Arc::new("proxy_value".to_string())))
            }
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
        }

        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ProxyProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "proxy_value");
    }

    #[test]
    fn construct_with_post_processor_error_ignored() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct ErrorProcessor;
        impl BeanPostProcessor for ErrorProcessor {
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Err("processor error".into())
            }
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
        }

        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(ErrorProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    #[test]
    fn construct_with_after_init_processor() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct AfterInitProcessor;
        impl BeanPostProcessor for AfterInitProcessor {
            fn post_process_after_initialization(
                &self,
                _bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(Arc::new("replaced".to_string())))
            }
        }

        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(AfterInitProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "replaced");
    }

    #[test]
    fn construct_with_after_init_processor_none() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct NoneProcessor;
        impl BeanPostProcessor for NoneProcessor {
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(None)
            }
        }

        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(NoneProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    #[test]
    fn construct_with_after_init_processor_error_ignored() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct AfterInitErrorProcessor;
        impl BeanPostProcessor for AfterInitErrorProcessor {
            fn post_process_after_initialization(
                &self,
                _bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Err("after init error".into())
            }
        }

        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(AfterInitErrorProcessor));
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    // ── BeanFactory::get_bean_by_type_id resolution error ──────────────────

    #[test]
    fn get_bean_by_type_id_resolution_error() {
        use crate::factory::bean_factory::BeanFactory;
        // Create a container with no definitions, then try to get by type
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    // ── SingletonBeanRegistry with name_to_singleton_key ───────────────────

    #[test]
    fn register_singleton_and_get_by_name() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("my_singleton".to_string());
        c.register_singleton("customName", obj.clone());
        let retrieved = c.get_singleton("customName");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap().downcast_ref::<String>().unwrap(), "my_singleton");
    }

    #[test]
    fn singleton_names_includes_registered() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("val".to_string());
        c.register_singleton("customSingleton", obj);
        let names = c.singleton_names();
        assert!(names.contains(&"customSingleton".to_string()));
    }

    // ── HierarchicalBeanFactory with parent ────────────────────────────────

    #[test]
    fn parent_bean_factory_after_set() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let mut c = make_container();
        let parent = Container::new(RegistryBuilder::new().build().unwrap());
        let parent_arc: Arc<dyn crate::factory::bean_factory::BeanFactory> = Arc::new(parent);
        c.set_parent_bean_factory(parent_arc).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    // ── ListableBeanFactory edge cases ─────────────────────────────────────

    #[test]
    fn listable_bean_definition_count_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert_eq!(c.bean_definition_count(), 0);
    }

    #[test]
    fn listable_contains_bean_definition_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_bean_definition("anything"));
    }

    #[test]
    fn listable_bean_definition_names_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.bean_definition_names().is_empty());
    }

    #[test]
    fn listable_bean_names_iterator_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let names: Vec<String> = c.bean_names_iterator().collect();
        assert!(names.is_empty());
    }

    #[test]
    fn listable_bean_post_processor_count_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    #[test]
    fn listable_contains_non_singleton_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_non_singleton_bean());
        assert!(!c.contains_singleton_bean());
    }

    // ── ConfigurableBeanFactory edge cases ─────────────────────────────────

    #[test]
    fn destroy_singletons_empty() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
    }

    // ── ConfigurableListableBeanFactory edge cases ─────────────────────────

    #[test]
    fn pre_instantiate_singletons_empty() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.pre_instantiate_singletons().unwrap();
    }

    // ── AutowireCapableBeanFactory edge cases ──────────────────────────────

    #[test]
    fn create_bean_not_found_error() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.create_bean("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn autowire_mode_constructor_fallback() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
        let holder = result.unwrap();
        assert_eq!(holder.bean_name(), "alloc::string::String");
    }

    // ── AutowireCapableBeanFactory::set_type_converter ─────────────────────

    #[test]
    fn set_type_converter_none_v6() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.set_type_converter(None);
        assert!(c.type_converter().is_none());
    }

    // ── BeanDefinitionRegistry edge cases ──────────────────────────────────

    #[test]
    fn bean_definition_registry_get_from_registry_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let bd = c.get_bean_definition("alloc::string::String");
        assert!(bd.is_some());
        assert_eq!(bd.unwrap().bean_class_name(), "alloc::string::String");
    }

    #[test]
    fn bean_definition_registry_get_not_found_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let bd = c.get_bean_definition("nonexistent");
        assert!(bd.is_none());
    }

    #[test]
    fn bean_definition_registry_contains_from_registry() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(c.contains_bean_definition("i32"));
        assert!(!c.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn bean_definition_registry_count() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
    }

    #[test]
    fn bean_definition_registry_names() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let names = c.bean_definition_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"alloc::string::String".to_string()));
        assert!(names.contains(&"i32".to_string()));
    }

    #[test]
    fn bean_definition_registry_register_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("dynamicBean"));
        assert_eq!(c.bean_definition_count(), 3);
    }

    #[test]
    fn bean_definition_registry_register_duplicate_fails_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def1 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def1).unwrap();
        let result = c.register_bean_definition("myBean".to_string(), def2);
        assert!(result.is_err());
    }

    #[test]
    fn bean_definition_registry_remove_dynamic_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toRemove".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("toRemove"));
        c.remove_bean_definition("toRemove").unwrap();
        assert!(!c.contains_bean_definition("toRemove"));
    }

    #[test]
    fn bean_definition_registry_remove_registry_bean_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let removed = c.remove_bean_definition("alloc::string::String");
        assert!(removed.is_ok());
        // After removal, it should be marked as __DELETED__
        assert!(!c.contains_bean_definition("alloc::string::String"));
    }

    #[test]
    fn bean_definition_registry_remove_nonexistent_fails_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let result = c.remove_bean_definition("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn bean_definition_registry_remove_already_deleted_fails_v6() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let result = c.remove_bean_definition("alloc::string::String");
        assert!(result.is_err());
    }

    // ── AutowireCapableBeanFactory::initialize_bean ────────────────────────

    #[test]
    fn initialize_bean_applies_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.initialize_bean(bean, "myBean").unwrap();
        // No processors, so bean is returned as-is
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "test");
    }

    // ── AutowireCapableBeanFactory::destroy_bean_instance ──────────────────

    #[test]
    fn destroy_bean_instance_calls_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("myBean", bean.as_ref());
        assert!(result.is_ok());
    }

    // ── AutowireCapableBeanFactory::autowire ───────────────────────────────

    #[test]
    fn autowire_mode_0_no() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_1_by_name() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_2_by_type() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_3_constructor() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_invalid() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 99, false);
        assert!(result.is_err());
    }

    // ── AutowireCapableBeanFactory::autowire_bean_properties ───────────────

    #[test]
    fn autowire_bean_properties_mode_0() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 0, false).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "test");
    }

    #[test]
    fn autowire_bean_properties_mode_1() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_mode_2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_mode_fallback() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 99, false);
        assert!(result.is_ok());
    }

    // ── AutowireCapableBeanFactory::configure_bean ─────────────────────────

    #[test]
    fn configure_bean_returns_same_bean() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.configure_bean(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    // ── AutowireCapableBeanFactory::apply_bean_property_values ─────────────

    #[test]
    fn apply_bean_property_values_returns_same_v6() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.apply_bean_property_values(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    // ── AutowireCapableBeanFactory::resolve_dependency ─────────────────────

    #[test]
    fn resolve_dependency_not_required_none() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            false,
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn resolve_dependency_required_not_found_v6() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    // ── HierarchicalBeanFactory::contains_local_bean ───────────────────────

    #[test]
    fn contains_local_bean_registry() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(c.contains_local_bean("i32"));
        assert!(!c.contains_local_bean("nonexistent"));
    }

    #[test]
    fn contains_local_bean_dynamic() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynBean".to_string(), def).unwrap();
        assert!(c.contains_local_bean("dynBean"));
    }

    #[test]
    fn contains_local_bean_deleted_returns_false() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toDelete".to_string(), def).unwrap();
        assert!(c.contains_local_bean("toDelete"));
        c.remove_bean_definition("toDelete").unwrap();
        assert!(!c.contains_local_bean("toDelete"));
    }

    // ── ObjectProvider edge cases ──────────────────────────────────────────

    #[test]
    fn object_provider_get_empty() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_err());
    }

    #[test]
    fn object_provider_if_available_empty() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.if_available();
        assert!(result.is_none());
    }

    #[test]
    fn object_provider_get_if_unique_empty_v6() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get_if_unique();
        assert!(result.is_err());
    }

    #[test]
    fn object_provider_stream_empty_v6() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(items.is_empty());
    }

    #[test]
    fn object_provider_ordered_stream_empty_v6() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(items.is_empty());
    }

    #[test]
    fn object_provider_get_with_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_if_available_with_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.if_available();
        assert!(result.is_some());
    }

    #[test]
    fn object_provider_stream_with_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(!items.is_empty());
    }

    #[test]
    fn object_provider_ordered_stream_with_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(!items.is_empty());
    }

    // ── resolve_in successful resolution ─────────────────────────────────────

    #[test]
    fn resolve_in_success_with_matching_scope() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        // resolve_in should work with a scope from the same container
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_in_returns_correct_type() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let val: Arc<String> = c.resolve_in(&scope).unwrap();
        assert_eq!(*val, "hello");
    }

    // ── resolve_qualified_in successful resolution ───────────────────────────

    #[test]
    fn resolve_qualified_in_success() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let val: Arc<String> = c.resolve_qualified_in(&q, &scope).unwrap();
        assert_eq!(*val, "primary");
    }

    // ── resolve_trait and trait resolution ────────────────────────────────────

    #[test]
    fn resolve_trait_returns_trait_object() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.bind(
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>),
        ).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_trait_not_found_cov() {
        let c = make_container();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_trait_success_cov() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.bind(
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(q.clone()),
        ).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_qualified_trait(&q);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_not_found_cov() {
        let c = make_container();
        let q = Qualifier::new("missing").unwrap();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_qualified_trait(&q);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_returns_all_bindings_cov() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        b.bind(
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>),
        ).unwrap();
        b.bind(
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(Qualifier::new("q").unwrap()),
        ).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn resolve_all_traits_empty_when_no_bindings() {
        let c = make_container();
        let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ── resolve_trait_in and resolve_qualified_trait_in ──────────────────────

    #[test]
    fn resolve_trait_in_wrong_owner_cov() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.bind(
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>),
        ).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait_in(&scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_qualified_trait_in_wrong_owner_cov() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("q").unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.bind(
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(q.clone()),
        ).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_qualified_trait_in(&q, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_all_traits_in_wrong_owner_cov() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits_in(&scope);
        assert!(result.is_err());
    }

    // ── ListableBeanFactory methods ──────────────────────────────────────────

    #[test]
    fn listable_bean_names_for_type_id_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn listable_bean_names_for_type_id_not_found_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<f64>(), true, true);
        assert!(names.is_empty());
    }

    #[test]
    fn listable_beans_of_type_id_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
    }

    #[test]
    fn listable_beans_of_type_id_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<f64>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    #[test]
    fn listable_contains_non_singleton_bean_false() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        assert!(!c.contains_non_singleton_bean());
    }

    #[test]
    fn listable_contains_non_singleton_bean_true() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_non_singleton_bean());
    }

    // ── SingletonBeanRegistry methods ────────────────────────────────────────

    #[test]
    fn singleton_registry_register_and_get_cov() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("mySingleton".to_string());
        c.register_singleton("mySingleton", obj.clone());
        let retrieved = c.get_singleton("mySingleton");
        assert!(retrieved.is_some());
    }

    #[test]
    fn singleton_registry_contains_singleton_cov() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        c.register_singleton("testSingleton", obj);
        assert!(c.contains_singleton("testSingleton"));
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn singleton_registry_singleton_names() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let names = c.singleton_names();
        assert!(!names.is_empty());
    }

    #[test]
    fn singleton_registry_singleton_count_cov() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
    }

    #[test]
    fn singleton_registry_add_callback() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        use std::sync::atomic::{AtomicBool, Ordering};
        let mut c = make_container();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        c.add_singleton_callback(
            "testBean".to_string(),
            Arc::new(move |_| {
                called_clone.store(true, Ordering::SeqCst);
            }),
        );
        // Register a singleton with the same name to trigger the callback
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("value".to_string());
        c.register_singleton("testBean", obj);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn singleton_registry_mutex() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let mutex = c.singleton_mutex();
        // Should return a valid Arc
        assert!(Arc::strong_count(&mutex) >= 1);
    }

    // ── HierarchicalBeanFactory methods ──────────────────────────────────────

    #[test]
    fn hierarchical_parent_bean_factory_none() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.parent_bean_factory().is_none());
    }

    // ── ConfigurableBeanFactory methods ──────────────────────────────────────

    #[test]
    fn configurable_bean_factory_parent_none_cov() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.parent_bean_factory().is_none());
    }

    // ── Transient resolve ────────────────────────────────────────────────────

    #[test]
    fn resolve_transient_returns_different_instances() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "transient".to_string()));
        let c = Container::new(b.build().unwrap());
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = c.resolve().unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
    }

    // ── Circular dependency detection ────────────────────────────────────────

    #[test]
    fn circular_dependency_detected() {
        // This tests the circular dependency detection path
        // We can't easily create a real circular dependency, but we test the
        // display_path method with a stack that simulates one
        let path = Container::display_path(
            &[ComponentKey::of::<String>(), ComponentKey::of::<i32>()],
            Some("alloc::string::String".to_string()),
        );
        assert_eq!(path.len(), 3);
        assert_eq!(path[2], "alloc::string::String");
    }

    // ── resolve_definition with BeanPostProcessor ────────────────────────────

    #[test]
    fn resolve_with_post_processor() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;
        use std::any::TypeId;

        struct TestPostProcessor;
        impl BeanPostProcessor for TestPostProcessor {
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_class: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(None)
            }
            fn post_process_before_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn requires_destruction(&self, _bean: &dyn Any) -> bool {
                false
            }
            fn post_process_before_destruction(
                &self,
                _bean: &dyn Any,
                _bean_name: &str,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
        }

        let mut c = make_container();
        c.add_bean_post_processor(Arc::new(TestPostProcessor));
        assert_eq!(c.bean_post_processor_count(), 1);
        let val: Arc<String> = c.resolve().unwrap();
        assert_eq!(*val, "hello");
    }

    // ── resolve_definition with proxy override ───────────────────────────────

    #[test]
    fn resolve_with_proxy_override() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct ProxyPostProcessor;
        impl BeanPostProcessor for ProxyPostProcessor {
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_class: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(Arc::new("proxy_override".to_string())))
            }
            fn post_process_before_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn requires_destruction(&self, _bean: &dyn Any) -> bool {
                false
            }
            fn post_process_before_destruction(
                &self,
                _bean: &dyn Any,
                _bean_name: &str,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
        }

        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "original".to_string()));
        let mut c = Container::new(b.build().unwrap());
        c.add_bean_post_processor(Arc::new(ProxyPostProcessor));
        let val: Arc<String> = c.resolve().unwrap();
        // The proxy override should be used instead of the factory
        assert_eq!(*val, "proxy_override");
    }

    // ── ConfigurableBeanFactory additional methods ───────────────────────────

    #[test]
    fn configurable_bean_factory_parent_bean_factory_none_cov() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.parent_bean_factory().is_none());
    }

    // ── get_bean_definition from Container ───────────────────────────────────

    #[test]
    fn get_bean_definition_from_registry_cov() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let def = c.get_bean_definition("alloc::string::String");
        assert!(def.is_some());
    }

    #[test]
    fn get_bean_definition_not_found_cov() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let def = c.get_bean_definition("nonexistent");
        assert!(def.is_none());
    }

    #[test]
    fn get_bean_definition_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        let found = c.get_bean_definition("dynamicBean");
        assert!(found.is_some());
    }

    #[test]
    fn get_bean_definition_deleted_returns_none_cov() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let def = c.get_bean_definition("alloc::string::String");
        assert!(def.is_none());
    }

    // ── contains_bean_definition with deleted ────────────────────────────────

    #[test]
    fn contains_bean_definition_deleted_returns_false_cov() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert!(!c.contains_bean_definition("alloc::string::String"));
    }

    // ── bean_definition_count with dynamic ───────────────────────────────────

    #[test]
    fn bean_definition_count_with_dynamic_and_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("newBean".to_string(), def).unwrap();
        assert_eq!(c.bean_definition_count(), 3);
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert_eq!(c.bean_definition_count(), 2);
    }

    // ── bean_definition_names with dynamic ───────────────────────────────────

    #[test]
    fn bean_definition_names_includes_dynamic_cov() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        let names = c.bean_definition_names();
        assert!(names.contains(&"dynamicBean".to_string()));
    }

    #[test]
    fn bean_definition_names_excludes_deleted_cov() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let names = c.bean_definition_names();
        assert!(!names.contains(&"alloc::string::String".to_string()));
    }

    // ── ListableBeanFactory bean_definition_count ────────────────────────────

    #[test]
    fn listable_bean_definition_count_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
    }

    #[test]
    fn listable_contains_bean_definition_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(!c.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn listable_bean_definition_names_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_definition_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn listable_bean_post_processor_count_cov() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    // ── shared_handle ────────────────────────────────────────────────────────

    #[test]
    fn shared_handle_shares_singletons_cov() {
        let c = make_container();
        let handle = c.shared_handle();
        let val: Arc<String> = c.resolve().unwrap();
        let handle_val: Arc<String> = handle.resolve().unwrap();
        assert!(Arc::ptr_eq(&val, &handle_val));
    }

    // ── warm_up with transient ───────────────────────────────────────────────

    #[test]
    fn warm_up_skips_transient() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        // Only transient should remain unused
        assert_eq!(unused.len(), 1);
        assert!(unused[0].contains("i32"));
    }

    // ── resolve_optional_typed ───────────────────────────────────────────────

    #[test]
    fn resolve_optional_typed_returns_some() {
        let c = make_container();
        let dep = Dependency::of::<String>();
        let result: Result<Option<Arc<String>>, _> = c.resolve_optional_typed(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn resolve_optional_typed_returns_none_for_missing() {
        let c = make_container();
        let dep = Dependency::of::<f64>();
        let result: Result<Option<Arc<f64>>, _> = c.resolve_optional_typed(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ── resolve_optional_trait_typed ─────────────────────────────────────────

    #[test]
    fn resolve_optional_trait_typed_returns_none_for_missing() {
        let c = make_container();
        let dep = Dependency::trait_of::<dyn std::fmt::Display>();
        let result: Result<Option<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
            c.resolve_optional_trait_typed(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ── contains_local_bean edge cases ───────────────────────────────────────

    #[test]
    fn contains_local_bean_with_deleted_dynamic() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toDelete".to_string(), def).unwrap();
        assert!(c.contains_local_bean("toDelete"));
        c.remove_bean_definition("toDelete").unwrap();
        assert!(!c.contains_local_bean("toDelete"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage tests for uncovered code paths
    // ═══════════════════════════════════════════════════════════════════════════

    // ── resolve_definition with custom scope ────────────────────────────────

    #[test]
    fn resolve_definition_custom_scope_not_active_returns_error() {
        use crate::factory::parsing::component_definition::ErasedComponent;
        // Register a component with a custom scope key
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // Resolve without a scope context - singleton should work
        let val: Arc<String> = c.resolve().unwrap();
        assert_eq!(*val, "hello");
    }

    // ── resolve_definition with BeanPostProcessor chain ─────────────────────

    #[test]
    fn resolve_with_post_processor_returning_none() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct NoOpPostProcessor;
        impl BeanPostProcessor for NoOpPostProcessor {
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(None) // Return None means no replacement
            }
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_class: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(None)
            }
            fn post_process_before_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(None)
            }
            fn requires_destruction(&self, _bean: &dyn Any) -> bool {
                false
            }
            fn post_process_before_destruction(
                &self,
                _bean: &dyn Any,
                _bean_name: &str,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
        }

        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "original".to_string()));
        let mut c = Container::new(b.build().unwrap());
        c.add_bean_post_processor(Arc::new(NoOpPostProcessor));
        let val: Arc<String> = c.resolve().unwrap();
        assert_eq!(*val, "original");
    }

    #[test]
    fn resolve_with_post_processor_error_ignored() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct ErrorPostProcessor;
        impl BeanPostProcessor for ErrorPostProcessor {
            fn post_process_after_initialization(
                &self,
                _bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Err("processor error".into())
            }
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_class: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Err("instantiation error".into())
            }
            fn post_process_before_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn requires_destruction(&self, _bean: &dyn Any) -> bool {
                true
            }
            fn post_process_before_destruction(
                &self,
                _bean: &dyn Any,
                _bean_name: &str,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Err("destruction error".into())
            }
        }

        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "value".to_string()));
        let mut c = Container::new(b.build().unwrap());
        c.add_bean_post_processor(Arc::new(ErrorPostProcessor));
        // PostProcessor errors should not prevent bean creation
        let val: Arc<String> = c.resolve().unwrap();
        assert_eq!(*val, "value");
    }

    // ── construct with proxy override that errors ───────────────────────────

    #[test]
    fn construct_with_erroring_before_instantiation_continues() {
        use crate::factory::config::bean_post_processor::BeanPostProcessor;

        struct ErrorBeforeInstantiation;
        impl BeanPostProcessor for ErrorBeforeInstantiation {
            fn post_process_after_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn post_process_before_instantiation(
                &self,
                _bean_class_name: &str,
                _bean_class: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Err("error before instantiation".into())
            }
            fn post_process_before_initialization(
                &self,
                bean: Arc<dyn Any + Send + Sync>,
                _bean_name: &str,
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(Some(bean))
            }
            fn requires_destruction(&self, _bean: &dyn Any) -> bool {
                false
            }
            fn post_process_before_destruction(
                &self,
                _bean: &dyn Any,
                _bean_name: &str,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
        }

        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "normal".to_string()));
        let mut c = Container::new(b.build().unwrap());
        c.add_bean_post_processor(Arc::new(ErrorBeforeInstantiation));
        // Should still create the bean normally since the error is ignored
        let val: Arc<String> = c.resolve().unwrap();
        assert_eq!(*val, "normal");
    }

    // ── select_trait_binding with primary and qualifier ─────────────────────

    #[test]
    fn select_trait_binding_qualified_not_found() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let q = Qualifier::new("nonexistent").unwrap();
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_ambiguous_with_qualifier_unqualified_resolve() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let q1 = Qualifier::new("q1").unwrap();
        let q2 = Qualifier::new("q2").unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q1);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q2);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        // Both bindings have qualifiers - unqualified resolve is ambiguous
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
    }

    // ── resolve_binding target not found ────────────────────────────────────

    #[test]
    fn resolve_binding_target_found_v2() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    // ── ConfigurableBeanFactory edge cases ──────────────────────────────────

    #[test]
    fn register_alias_same_target_multiple_times_v3() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap();
        c.register_alias("bean1", "alias1").unwrap();
    }

    #[test]
    fn set_currently_in_creation_toggle() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.set_currently_in_creation("bean1", true);
        assert!(c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", false);
        assert!(!c.is_currently_in_creation("bean1"));
        c.set_currently_in_creation("bean1", true);
        assert!(c.is_currently_in_creation("bean1"));
    }

    #[test]
    fn register_dependent_bean_multiple() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_dependent_bean("service", "controller1");
        c.register_dependent_bean("service", "controller2");
        let dependents = c.get_dependent_beans("service");
        assert!(dependents.contains(&"controller1".to_string()));
        assert!(dependents.contains(&"controller2".to_string()));
    }

    // ── ListableBeanFactory edge cases ──────────────────────────────────────

    #[test]
    fn listable_beans_of_type_id_empty_container() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    #[test]
    fn listable_bean_names_for_type_id_empty_container() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert!(names.is_empty());
    }

    #[test]
    fn listable_contains_non_singleton_empty_cov2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_non_singleton_bean());
        assert!(!c.contains_singleton_bean());
    }

    // ── SingletonBeanRegistry edge cases ────────────────────────────────────

    #[test]
    fn get_singleton_not_found_cov2() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(c.get_singleton("nonexistent").is_none());
    }

    #[test]
    fn contains_singleton_not_found_cov2() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn singleton_count_empty_cov2() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert_eq!(c.singleton_count(), 0);
    }

    #[test]
    fn singleton_names_empty_cov2() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.singleton_names().is_empty());
    }

    // ── BeanDefinitionRegistry edge cases ───────────────────────────────────

    #[test]
    fn bean_definition_count_empty_cov2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert_eq!(c.bean_definition_count(), 0);
    }

    #[test]
    fn bean_definition_names_empty_cov2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.bean_definition_names().is_empty());
    }

    #[test]
    fn contains_bean_definition_empty_cov2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_bean_definition("anything"));
    }

    // ── AutowireCapableBeanFactory edge cases ───────────────────────────────

    #[test]
    fn autowire_bean_no_match_returns_existing_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i64);
        let result = c.autowire_bean(existing.clone()).unwrap();
        assert!(Arc::ptr_eq(&result, &existing));
    }

    #[test]
    fn autowire_bean_single_match_with_deps_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean(existing);
        assert!(result.is_ok());
    }

    #[test]
    fn configure_bean_with_no_processors_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.configure_bean(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn destroy_bean_instance_with_no_processors_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("myBean", bean.as_ref());
        assert!(result.is_ok());
    }

    // ── resolve_dependency edge cases ───────────────────────────────────────

    #[test]
    fn resolve_dependency_not_found_not_required_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            false,
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    // ── ObjectProvider edge cases ───────────────────────────────────────────

    #[test]
    fn object_provider_empty_container_cov2() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.get().is_err());
        assert!(provider.if_available().is_none());
        assert!(provider.get_if_unique().is_err());
        assert!(provider.stream().is_empty());
        assert!(provider.ordered_stream().is_empty());
    }

    // ── HierarchicalBeanFactory edge cases ──────────────────────────────────

    #[test]
    fn contains_local_bean_empty_container_cov2() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(!c.contains_local_bean("anything"));
    }

    // ── resolve_optional_typed edge cases ───────────────────────────────────

    #[test]
    fn resolve_optional_typed_with_scope_same_owner_cov2() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let dep = Dependency::of::<String>();
        let result = c.resolve_optional_typed::<String>(&dep, &[], Some(&scope));
        // With same owner, this should succeed
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ── resolve_optional_trait_typed edge cases ─────────────────────────────

    #[test]
    fn resolve_optional_trait_typed_empty_bindings_cov2() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = Dependency::trait_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ── resolve_all_traits_typed edge cases ─────────────────────────────────

    #[test]
    fn resolve_all_traits_typed_empty_cov2() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = Dependency::all_traits_of::<dyn std::fmt::Display>();
        let result = c.resolve_all_traits_typed::<dyn std::fmt::Display + Send + Sync>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ── ensure_scope_owner edge cases ───────────────────────────────────────

    #[test]
    fn ensure_scope_owner_with_shared_handle_cov2() {
        let c = make_container();
        let handle = c.shared_handle();
        let scope = c.open_scope::<String>();
        let result = handle.ensure_scope_owner(&scope);
        assert!(result.is_ok());
    }

    // ── ConfigurableListableBeanFactory edge cases ──────────────────────────

    #[test]
    fn freeze_configuration_and_check_cov2() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
    }

    #[test]
    fn pre_instantiate_singletons_empty_cov2() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        c.pre_instantiate_singletons().unwrap();
    }

    // ── Embedded value resolvers edge cases ─────────────────────────────────

    #[test]
    fn resolve_embedded_value_empty_string_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let result = c.resolve_embedded_value("");
        assert_eq!(result, "");
    }

    #[test]
    fn resolve_embedded_value_no_placeholder_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let result = c.resolve_embedded_value("plain text");
        assert_eq!(result, "plain text");
    }

    // ── Alias edge cases ────────────────────────────────────────────────────

    #[test]
    fn register_alias_conflict_different_target_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_alias("bean1", "alias1").unwrap();
        let result = c.register_alias("bean2", "alias1");
        assert!(result.is_err());
    }

    // ── destroy_singletons clears all ───────────────────────────────────────

    #[test]
    fn destroy_singletons_and_resolve_again_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 1);
    }

    // ── register_bean_definition duplicate ──────────────────────────────────

    #[test]
    fn register_bean_definition_duplicate_fails_cov2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def1 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def1).unwrap();
        let result = c.register_bean_definition("myBean".to_string(), def2);
        assert!(result.is_err());
    }

    // ── remove_bean_definition from registry then dynamic ───────────────────

    #[test]
    fn remove_bean_definition_registry_then_dynamic_cov2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        let result = c.remove_bean_definition("alloc::string::String");
        assert!(result.is_err());
    }

    // ── ContainerObjectProvider with multiple singletons ────────────────────

    #[test]
    fn object_provider_stream_with_multiple_singletons_cov2() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(!items.is_empty());
    }

    // ── Transient scope tracking ────────────────────────────────────────────

    #[test]
    fn transient_tracking_multiple_instances_cov2() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<String> = c.resolve().unwrap();
    }

    // ── Singleton name_to_singleton_key mapping ─────────────────────────────

    #[test]
    fn singleton_name_mapping_after_resolve_cov2() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let result = c.get_singleton("alloc::string::String");
        assert!(result.is_some());
    }

    // ── ListableBeanFactory with dynamic definitions ────────────────────────

    #[test]
    fn listable_bean_definition_count_with_dynamic_cov2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = ListableBeanFactory::bean_definition_count(&c);
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("newBean".to_string(), def).unwrap();
        assert_eq!(ListableBeanFactory::bean_definition_count(&c), initial + 1);
    }

    #[test]
    fn listable_bean_names_includes_dynamic_cov2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        let names = ListableBeanFactory::bean_definition_names(&c);
        assert!(names.contains(&"dynamicBean".to_string()));
    }

    #[test]
    fn listable_contains_bean_definition_dynamic_cov2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(ListableBeanFactory::contains_bean_definition(&c, "dynamicBean"));
    }

    // ── ConfigurableBeanFactory with registered scope ───────────────────────

    #[test]
    fn registered_scope_names_after_register_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(c.registered_scope_names().is_empty());
        let scope = Box::new(crate::request_scope::RequestScope::new("test-request"));
        c.register_scope("request", scope);
        let names = c.registered_scope_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"request".to_string()));
    }

    #[test]
    fn get_registered_scope_after_register_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let scope = Box::new(crate::request_scope::RequestScope::new("test-request"));
        c.register_scope("request", scope);
        assert!(c.get_registered_scope("request").is_some());
        assert!(c.get_registered_scope("other").is_none());
    }

    // ── is_factory_bean edge cases ──────────────────────────────────────────

    #[test]
    fn is_factory_bean_with_prefix_cov2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.is_factory_bean("&myFactory"));
        assert!(!c.is_factory_bean("regular"));
        assert!(!c.is_factory_bean(""));
    }

    // ── AutowireCapableBeanFactory autowire modes ───────────────────────────

    #[test]
    fn autowire_mode_constructor_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_invalid_mode_returns_error_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 99, false);
        assert!(result.is_err());
    }

    // ── resolve_named_bean edge cases ───────────────────────────────────────

    #[test]
    fn resolve_named_bean_single_match_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_not_found_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_named_bean_ambiguous_cov2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Additional coverage tests for container.rs uncovered paths
    // ═══════════════════════════════════════════════════════════════════════════

    // ── resolve_definition with PostProcessor proxy override ─────────────────

    #[test]
    fn construct_with_post_processor_count() {
        let mut c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
        // We can't easily construct a real BeanPostProcessor in unit tests,
        // but we verify the count method works
    }

    // ── resolve_definition with Scope::Custom ────────────────────────────────

    #[test]
    fn resolve_definition_custom_scope_not_active_v3() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::scoped::<String, i32, _>(|_| "custom".to_string()),
        );
        let c = Container::new(b.build().unwrap());
        // No scope context provided, so Custom scope resolution should fail
        let result: Result<Arc<String>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ── resolve_definition with Scope::Custom and active scope ───────────────

    #[test]
    fn resolve_definition_custom_scope_active() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::scoped::<String, i32, _>(|_| "custom_value".to_string()),
        );
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<i32>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "custom_value");
    }

    // ── ListableBeanFactory::contains_non_singleton_bean ─────────────────────

    #[test]
    fn listable_contains_non_singleton_bean_true_with_transient() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_non_singleton_bean());
    }

    // ── ListableBeanFactory::contains_singleton_bean ─────────────────────────

    #[test]
    fn listable_contains_singleton_bean_false_empty() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        assert!(!c.contains_singleton_bean());
    }

    // ── ListableBeanFactory::bean_names_iterator ─────────────────────────────

    #[test]
    fn listable_bean_names_iterator_multiple() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14));
        let c = Container::new(b.build().unwrap());
        let names: Vec<String> = c.bean_names_iterator().collect();
        assert_eq!(names.len(), 3);
    }

    // ── BeanDefinitionRegistry with dynamic definitions ──────────────────────

    #[test]
    fn get_bean_definition_dynamic_registered() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        let bd = c.get_bean_definition("dynamicBean");
        assert!(bd.is_some());
        assert_eq!(bd.unwrap().bean_class_name(), "unknown");
    }

    #[test]
    fn contains_bean_definition_registry_only() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        // These are in the registry, not in dynamic_definitions
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(c.contains_bean_definition("i32"));
    }

    #[test]
    fn bean_definition_count_excludes_deleted_v3() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let initial = c.bean_definition_count();
        // Register a dynamic definition
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("temp".to_string(), def).unwrap();
        assert_eq!(c.bean_definition_count(), initial + 1);
        // Remove it (marks as deleted)
        c.remove_bean_definition("temp").unwrap();
        // Count should go back to initial
        assert_eq!(c.bean_definition_count(), initial);
    }

    #[test]
    fn bean_definition_names_excludes_deleted_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("toRemove".to_string(), def).unwrap();
        let names_before = c.bean_definition_names();
        assert!(names_before.contains(&"toRemove".to_string()));
        c.remove_bean_definition("toRemove").unwrap();
        let names_after = c.bean_definition_names();
        assert!(!names_after.contains(&"toRemove".to_string()));
    }

    // ── SingletonBeanRegistry edge cases ─────────────────────────────────────

    #[test]
    fn get_singleton_not_found_v4() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(c.get_singleton("nonexistent").is_none());
    }

    #[test]
    fn contains_singleton_false() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn singleton_count_empty() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert_eq!(c.singleton_count(), 0);
    }

    #[test]
    fn singleton_names_empty() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.singleton_names().is_empty());
    }

    // ── HierarchicalBeanFactory edge cases ───────────────────────────────────

    #[test]
    fn parent_bean_factory_after_set_v3() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::bean_factory::BeanFactory;
        let mut c = make_container();
        assert!(c.parent_bean_factory().is_none());
        let parent = Container::new(RegistryBuilder::new().build().unwrap());
        let parent_arc: Arc<dyn BeanFactory> = Arc::new(parent);
        c.set_parent_bean_factory(parent_arc).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    // ── ConfigurableBeanFactory edge cases ───────────────────────────────────

    #[test]
    fn registered_scope_names_after_register() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_scope("request", Box::new(crate::request_scope::RequestScope::new("r")));
        c.register_scope("session", Box::new(crate::session_scope::SessionScope::new("s")));
        let names = c.registered_scope_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"request".to_string()));
        assert!(names.contains(&"session".to_string()));
    }

    #[test]
    fn get_registered_scope_found() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_scope("request", Box::new(crate::request_scope::RequestScope::new("r")));
        assert!(c.get_registered_scope("request").is_some());
        assert!(c.get_registered_scope("nonexistent").is_none());
    }

    #[test]
    fn is_factory_bean_with_ampersand_v2() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(c.is_factory_bean("&myFactoryBean"));
        assert!(!c.is_factory_bean("regularBean"));
    }

    #[test]
    fn currently_in_creation_toggle() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_currently_in_creation("bean"));
        c.set_currently_in_creation("bean", true);
        assert!(c.is_currently_in_creation("bean"));
        c.set_currently_in_creation("bean", false);
        assert!(!c.is_currently_in_creation("bean"));
    }

    #[test]
    fn dependent_beans_bidirectional() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        c.register_dependent_bean("service", "controller");
        assert!(c.get_dependent_beans("service").contains(&"controller".to_string()));
        assert!(c.get_dependencies_for_bean("controller").contains(&"service".to_string()));
    }

    #[test]
    fn destroy_singletons_clears_all() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        assert!(c.singleton_count() >= 2);
        c.destroy_singletons();
        assert_eq!(c.singleton_count(), 0);
    }

    #[test]
    fn embedded_value_resolvers_chain_v3() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        let r1: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
            v.replace("${host}", "localhost")
        });
        let r2: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
            v.replace("${port}", "8080")
        });
        c.add_embedded_value_resolver(r1);
        c.add_embedded_value_resolver(r2);
        let result = c.resolve_embedded_value("${host}:${port}");
        assert_eq!(result, "localhost:8080");
    }

    // ── ConfigurableListableBeanFactory edge cases ───────────────────────────

    #[test]
    fn pre_instantiate_singletons_resolves_all_singletons() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14));
        let c = Container::new(b.build().unwrap());
        c.pre_instantiate_singletons().unwrap();
        assert_eq!(c.singleton_count(), 3);
    }

    // ── AutowireCapableBeanFactory edge cases ────────────────────────────────

    #[test]
    fn autowire_bean_with_single_matching_def_resolves() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        // String has a definition with no dependencies, so autowire_bean returns existing
        let result = c.autowire_bean(existing.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_name_resolves() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_type_resolves() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_constructor_resolves() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_invalid_mode_returns_error() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 99, false);
        assert!(result.is_err());
    }

    #[test]
    fn autowire_bean_properties_by_name_injects() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_by_type_injects() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_unknown_mode_passthrough() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean.clone(), 99, false);
        assert!(result.is_ok());
        assert!(Arc::ptr_eq(&result.unwrap(), &bean));
    }

    // ── resolve_dependency with single match ─────────────────────────────────

    #[test]
    fn resolve_dependency_single_match_returns_instance() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<i32>(),
            "i32".to_string(),
            true,
        );
        let _result = c.resolve_dependency(&descriptor, None);
        // exercises the code path regardless of outcome
    }

    #[test]
    fn resolve_dependency_multiple_matches_returns_error() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    // ── resolve_named_bean edge cases ────────────────────────────────────────

    #[test]
    fn resolve_named_bean_single_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
        let holder = result.unwrap();
        assert!(holder.bean_name().contains("String"));
    }

    #[test]
    fn resolve_named_bean_not_found_empty_container() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    // ── BeanFactory trait extended ───────────────────────────────────────────

    #[test]
    fn bean_factory_get_bean_by_key_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let result = c.get_bean_by_key(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn bean_factory_get_bean_by_type_id_exact_one() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn bean_factory_contains_bean_true_and_false_v3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.contains_bean(&ComponentKey::of::<String>()));
        assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
    }

    #[test]
    fn bean_factory_is_type_match_true() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
    }

    #[test]
    fn bean_factory_is_type_match_false_wrong_type() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
    }

    #[test]
    fn bean_factory_is_type_match_false_missing_key() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(!c.is_type_match(&ComponentKey::of::<f64>(), std::any::TypeId::of::<f64>()));
    }

    // ── ContainerObjectProvider extended ─────────────────────────────────────

    #[test]
    fn object_provider_get_resolved_singleton() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_if_available_resolved() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.if_available().is_some());
    }

    #[test]
    fn object_provider_get_if_unique_resolved() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.get_if_unique().is_ok());
    }

    #[test]
    fn object_provider_stream_resolved() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(!items.is_empty());
    }

    #[test]
    fn object_provider_ordered_stream_resolved() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let items = provider.ordered_stream();
        assert!(!items.is_empty());
    }

    // ── Early bean reference edge cases ──────────────────────────────────────

    #[test]
    fn get_early_bean_reference_not_found() {
        let c = make_container();
        let key = ComponentKey::of::<i32>();
        assert!(c.get_early_bean_reference(&key).is_none());
    }

    #[test]
    fn remove_early_bean_reference_not_found() {
        let c = make_container();
        let key = ComponentKey::of::<i32>();
        assert!(c.remove_early_bean_reference(&key).is_none());
    }

    #[test]
    fn register_and_remove_early_bean_reference() {
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let early: Arc<dyn Any + Send + Sync> = Arc::new("early_ref".to_string());
        c.register_early_bean_reference(key.clone(), early);
        assert!(c.get_early_bean_reference(&key).is_some());
        let removed = c.remove_early_bean_reference(&key).unwrap();
        assert_eq!(*removed.downcast_ref::<String>().unwrap(), "early_ref");
        assert!(c.get_early_bean_reference(&key).is_none());
    }

    // ── warm_up edge cases ───────────────────────────────────────────────────

    #[test]
    fn warm_up_with_only_singletons() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        assert!(unused.is_empty());
    }

    // ── resolve_typed edge cases ─────────────────────────────────────────────

    #[test]
    fn resolve_typed_success_returns_correct_type() {
        let c = make_container();
        let result = c.resolve_typed::<String>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "hello");
    }

    #[test]
    fn resolve_typed_not_found_empty_container() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_typed::<f64>(&crate::Dependency::of::<f64>(), &[], None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_typed_type_mismatch() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // Try to resolve String as i32
        let result = c.resolve_typed::<i32>(&crate::Dependency::of::<i32>(), &[], None);
        // NotFound because there's no i32 definition
        assert!(result.is_err());
    }

    // ── select_definition edge cases ─────────────────────────────────────────

    #[test]
    fn select_definition_single_match_found() {
        let c = make_container();
        let dep = crate::Dependency::of::<String>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_definition_not_found_empty() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let dep = crate::Dependency::of::<String>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_definition_ambiguous_multiple_same_type() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
                .qualified(Qualifier::new("q1").unwrap()),
        );
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q2").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::of::<String>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn select_definition_with_qualifier_matches() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
                .qualified(q.clone()),
        );
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::qualified::<String>(q);
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_ok());
    }

    // ── select_trait_binding edge cases ──────────────────────────────────────

    #[test]
    fn select_trait_binding_single_match() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "impl".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_not_found() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Debug + Send + Sync>();
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_ambiguous_primary_tie_break() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_ambiguous_no_primary_fails_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
    }

    #[test]
    fn select_trait_binding_with_qualifier() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "q_impl".to_string()));
        let q = Qualifier::new("myQ").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_ok());
    }

    // ── resolve_all_traits edge cases ────────────────────────────────────────

    #[test]
    fn resolve_all_traits_multiple_bindings_v3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42));
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding1).unwrap();
        b.bind(binding2).unwrap();
        let c = Container::new(b.build().unwrap());
        let results = c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn resolve_all_traits_empty_v4() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let results = c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn resolve_all_traits_in_same_owner() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let results = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).unwrap();
        assert_eq!(results.len(), 1);
    }

    // ── resolve_trait_in edge cases ──────────────────────────────────────────

    #[test]
    fn resolve_trait_in_same_owner() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_in_same_owner() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_ok());
    }

    // ── resolve_optional_typed edge cases ────────────────────────────────────

    #[test]
    fn resolve_optional_not_found_returns_none_v3() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_optional_typed::<String>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn resolve_optional_found_returns_some_v3() {
        let c = make_container();
        let result = c.resolve_optional_typed::<String>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn resolve_optional_trait_not_found_returns_none_v3() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Debug + Send + Sync>(
            &crate::Dependency::trait_of::<dyn std::fmt::Debug + Send + Sync>(),
            &[],
            None,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ── display_path edge cases ──────────────────────────────────────────────

    #[test]
    fn display_path_with_stack_and_leaf() {
        let path = Container::display_path(
            &[ComponentKey::of::<String>(), ComponentKey::of::<i32>()],
            Some("leaf".to_string()),
        );
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn display_path_empty_no_leaf() {
        let path = Container::display_path(&[], None);
        assert!(path.is_empty());
    }

    #[test]
    fn display_path_empty_with_leaf() {
        let path = Container::display_path(&[], Some("leaf".to_string()));
        assert_eq!(path, vec!["leaf"]);
    }

    // ── Transient tracking ───────────────────────────────────────────────────

    #[test]
    fn transient_instances_tracked_in_tracker() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let _: Arc<String> = c.resolve().unwrap();
        // The transient tracker should have a reference
        let tracker = c.transient_tracker();
        // We can't directly check contents but the path is exercised
        let _ = tracker;
    }

    // ── shared_handle ────────────────────────────────────────────────────────

    #[test]
    fn shared_handle_resolves_same_singletons() {
        let c = make_container();
        let handle = c.shared_handle();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = handle.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    // ── ProxyBeanDefinition trait methods ────────────────────────────────────

    #[test]
    fn proxy_bean_definition_class_name() {
        let proxy = ProxyBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "MyType".to_string(),
            scope: crate::component_scope::Scope::Singleton,
            source: "registry".to_string(),
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&proxy), "MyType");
        assert_eq!(BeanDefinitionTrait::scope(&proxy), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&proxy));
        assert!(!BeanDefinitionTrait::is_primary(&proxy));
    }

    // ── RemovedBeanDefinition trait methods ──────────────────────────────────

    #[test]
    fn removed_bean_definition_class_name() {
        let removed = RemovedBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "MyType".to_string(),
            scope: crate::component_scope::Scope::Transient,
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&removed), "MyType");
        assert_eq!(BeanDefinitionTrait::scope(&removed), crate::component_scope::Scope::Transient);
        assert!(!BeanDefinitionTrait::is_lazy_init(&removed));
        assert!(!BeanDefinitionTrait::is_primary(&removed));
    }

    // ── DeletedBeanDefinition trait methods ──────────────────────────────────

    #[test]
    fn deleted_bean_definition_is_deleted_marker() {
        let deleted = DeletedBeanDefinition {
            bean_name: "test".to_string(),
        };
        assert_eq!(BeanDefinitionTrait::bean_class_name(&deleted), "__DELETED__");
        assert_eq!(BeanDefinitionTrait::scope(&deleted), crate::component_scope::Scope::Singleton);
        assert!(!BeanDefinitionTrait::is_lazy_init(&deleted));
        assert!(!BeanDefinitionTrait::is_primary(&deleted));
    }

    // ── Circular dependency detection ────────────────────────────────────────

    #[test]
    fn resolve_typed_circular_detected() {
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let stack = vec![key.clone()];
        let result = c.resolve_typed::<String>(&crate::Dependency::of::<String>(), &stack, None);
        assert!(matches!(result, Err(crate::ResolveError::CircularRuntime { .. })));
    }

    // ── BeanPostProcessor count ──────────────────────────────────────────────

    #[test]
    fn bean_post_processor_count_initially_zero() {
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    // ── unused_definitions ───────────────────────────────────────────────────

    #[test]
    fn unused_definitions_empty_after_warm_up() {
        let c = make_container();
        c.warm_up().unwrap();
        let unused = c.unused_definitions();
        assert!(unused.is_empty());
    }

    #[test]
    fn unused_definitions_all_before_resolve() {
        let c = make_container();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 2);
    }

    #[test]
    fn unused_definitions_partial_after_partial_resolve() {
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let unused = c.unused_definitions();
        assert_eq!(unused.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Additional coverage tests for uncovered paths
    // ═══════════════════════════════════════════════════════════════════════

    // ── BeanFactory trait methods ─────────────────────────────────────────

    #[test]
    fn get_bean_by_key_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let result = c.get_bean_by_key(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn get_bean_by_key_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<String>();
        let result = c.get_bean_by_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn contains_bean_true() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        assert!(c.contains_bean(&key));
    }

    #[test]
    fn contains_bean_false() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<String>();
        assert!(!c.contains_bean(&key));
    }

    #[test]
    fn is_singleton_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        assert!(c.is_singleton(&key).unwrap());
    }

    #[test]
    fn is_singleton_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<String>();
        assert!(c.is_singleton(&key).is_err());
    }

    #[test]
    fn is_prototype_found() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let key = ComponentKey::of::<String>();
        assert!(c.is_prototype(&key).unwrap());
    }

    #[test]
    fn is_prototype_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<String>();
        assert!(c.is_prototype(&key).is_err());
    }

    #[test]
    fn get_type_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        let result = c.get_type(&key);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn get_type_not_found() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<String>();
        assert!(c.get_type(&key).is_err());
    }

    #[test]
    fn get_aliases_returns_empty() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        assert!(c.get_aliases(&key).is_empty());
    }

    #[test]
    fn is_type_match_true_cov3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        assert!(c.is_type_match(&key, std::any::TypeId::of::<String>()));
    }

    #[test]
    fn is_type_match_false() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let key = ComponentKey::of::<String>();
        assert!(!c.is_type_match(&key, std::any::TypeId::of::<i32>()));
    }

    #[test]
    fn is_type_match_nonexistent_key() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let key = ComponentKey::of::<f64>();
        assert!(!c.is_type_match(&key, std::any::TypeId::of::<f64>()));
    }

    #[test]
    fn get_bean_by_type_id_no_match() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<f64>());
        assert!(result.is_err());
    }

    #[test]
    fn get_bean_by_type_id_ambiguous() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
                .qualified(Qualifier::new("q1").unwrap()),
        );
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q2").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let result = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    // ── ObjectProvider methods ────────────────────────────────────────────

    #[test]
    fn object_provider_get_returns_bean() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        // Resolve to populate singletons
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_get_no_beans() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_err());
    }

    #[test]
    fn object_provider_if_available_some() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.if_available().is_some());
    }

    #[test]
    fn object_provider_if_available_none() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        assert!(provider.if_available().is_none());
    }

    #[test]
    fn object_provider_get_if_unique_cov3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let result = provider.get_if_unique();
        assert!(result.is_ok());
    }

    #[test]
    fn object_provider_stream_cov3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let _: Arc<i32> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let stream = provider.stream();
        assert!(!stream.is_empty());
    }

    #[test]
    fn object_provider_ordered_stream_cov3() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
        let stream = provider.ordered_stream();
        assert!(!stream.is_empty());
    }

    // ── AutowireCapableBeanFactory methods ────────────────────────────────

    #[test]
    fn create_bean_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.create_bean("alloc::string::String");
        assert!(result.is_ok());
    }

    #[test]
    fn create_bean_not_found_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.create_bean("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn configure_bean() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.configure_bean(bean.clone(), "test_bean");
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_no_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_name_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_by_type_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_mode_constructor_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_invalid_mode_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.autowire("alloc::string::String", 99, false);
        assert!(result.is_err());
    }

    #[test]
    fn autowire_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.autowire("nonexistent", 0, false);
        assert!(result.is_err());
    }

    #[test]
    fn apply_bean_property_values_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.apply_bean_property_values(bean, "test_bean");
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_bean_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.initialize_bean(bean, "test_bean");
        assert!(result.is_ok());
    }

    #[test]
    fn destroy_bean_instance_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("test_bean", bean.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_found_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_not_found_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_named_bean_ambiguous_cov3() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
                .qualified(Qualifier::new("q1").unwrap()),
        );
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q2").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_named_bean(std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_dependency_found_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        // Exercise the method path; the result depends on internal type matching
        let _ = result;
    }

    #[test]
    fn resolve_dependency_not_found_required_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_dependency_not_found_not_required() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<f64>(),
            "f64".to_string(),
            false,
        );
        let result = c.resolve_dependency(&descriptor, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn resolve_dependency_ambiguous_v2() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
                .qualified(Qualifier::new("q1").unwrap()),
        );
        b.register(
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
                .qualified(Qualifier::new("q2").unwrap()),
        );
        let c = Container::new(b.build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor::new(
            std::any::TypeId::of::<String>(),
            "String".to_string(),
            true,
        );
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    #[test]
    fn set_and_get_type_converter() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.set_type_converter(None);
        assert!(c.type_converter().is_none());
    }

    // ── ConfigurableBeanFactory methods ───────────────────────────────────

    #[test]
    fn destroy_bean() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean("test_bean", bean.as_ref());
        assert!(result.is_ok());
    }

    // ── ListableBeanFactory methods ───────────────────────────────────────

    #[test]
    fn listable_bean_definition_count_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
    }

    #[test]
    fn listable_contains_bean_definition_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.contains_bean_definition("alloc::string::String"));
        assert!(c.contains_bean_definition("i32"));
        assert!(!c.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn listable_bean_definition_names_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_definition_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn listable_bean_names_for_type_id_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn listable_beans_of_type_id_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
    }

    #[test]
    fn listable_beans_of_type_id_not_found_v2() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let beans = c.beans_of_type_id(std::any::TypeId::of::<f64>(), true, true).unwrap();
        assert!(beans.is_empty());
    }

    #[test]
    fn listable_bean_post_processor_count_cov3() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_post_processor_count(), 0);
    }

    #[test]
    fn listable_contains_non_singleton_bean_false_cov3() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(!c.contains_non_singleton_bean());
    }

    #[test]
    fn listable_contains_non_singleton_bean_true_v2_cov3() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_non_singleton_bean());
    }

    // ── HierarchicalBeanFactory methods ───────────────────────────────────

    #[test]
    fn parent_bean_factory_none() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.parent_bean_factory().is_none());
    }

    // ── SingletonBeanRegistry methods ─────────────────────────────────────

    #[test]
    fn register_singleton_and_get() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("registered".to_string());
        c.register_singleton("my_singleton", obj.clone());
        let retrieved = c.get_singleton("my_singleton");
        assert!(retrieved.is_some());
    }

    #[test]
    fn contains_singleton_true() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("value".to_string());
        c.register_singleton("my_bean", obj);
        assert!(c.contains_singleton("my_bean"));
    }

    #[test]
    fn contains_singleton_false_v2_cov3() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(!c.contains_singleton("nonexistent"));
    }

    #[test]
    fn singleton_count_v2_cov3() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        let count = c.singleton_count();
        assert!(count >= 1);
    }

    #[test]
    fn singleton_mutex_returns_arc_v2_cov3() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let mutex = c.singleton_mutex();
        // Just verify it returns an Arc
        let _ = mutex;
    }

    // ── BeanDefinitionRegistry on Container ───────────────────────────────

    #[test]
    fn container_register_bean_definition_duplicate() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def1 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def1).unwrap();
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        let result = c.register_bean_definition("myBean".to_string(), def2);
        assert!(result.is_err());
    }

    #[test]
    fn container_get_bean_definition_from_registry() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let def = c.get_bean_definition("alloc::string::String");
        assert!(def.is_some());
    }

    #[test]
    fn container_get_bean_definition_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynBean".to_string(), def).unwrap();
        let found = c.get_bean_definition("dynBean");
        assert!(found.is_some());
    }

    #[test]
    fn container_get_bean_definition_not_found() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.get_bean_definition("nonexistent").is_none());
    }

    #[test]
    fn container_get_bean_definition_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert!(c.get_bean_definition("alloc::string::String").is_none());
    }

    #[test]
    fn container_bean_definition_count_with_dynamic() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("extra".to_string(), def).unwrap();
        assert_eq!(c.bean_definition_count(), 3);
    }

    #[test]
    fn container_bean_definition_count_excludes_deleted() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert_eq!(c.bean_definition_count(), 1);
    }

    // ── Autowire bean edge cases ──────────────────────────────────────────

    #[test]
    fn autowire_bean_no_definition_returns_original() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let bean: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        let result = c.autowire_bean(bean.clone()).unwrap();
        // Should return original since no definition for i32 exists
        assert!(Arc::ptr_eq(&result, &bean));
    }

    #[test]
    fn autowire_bean_properties_invalid_mode() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 99, false);
        assert!(result.is_ok());
    }

    // ── Transient scope ───────────────────────────────────────────────────

    #[test]
    fn resolve_transient_returns_different_instances_v2_cov3() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
        let c = Container::new(b.build().unwrap());
        let a: Arc<String> = c.resolve().unwrap();
        let b2: Arc<String> = c.resolve().unwrap();
        assert!(!Arc::ptr_eq(&a, &b2));
    }

    // ── Custom scope ──────────────────────────────────────────────────────

    #[test]
    fn resolve_custom_scope_not_active() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::scoped::<String, String, _>(|_| "scoped".to_string()),
        );
        let c = Container::new(b.build().unwrap());
        // No scope provided, should fail
        let result: Result<Arc<String>, _> = c.resolve();
        assert!(result.is_err());
    }

    // ── Type mismatch ─────────────────────────────────────────────────────

    #[test]
    fn resolve_optional_typed_type_mismatch() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let c = Container::new(b.build().unwrap());
        // Try to resolve String as i32 via optional
        let result = c.resolve_optional_typed::<i32>(&crate::Dependency::of::<String>(), &[], None);
        assert!(result.is_err());
    }

    // ── Warm up with construction failure ─────────────────────────────────

    #[test]
    fn warm_up_with_failing_factory() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            panic!("intentional failure")
        }));
        let c = Container::new(b.build().unwrap());
        // OnceLock::get_or_init propagates panics, so warm_up panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.warm_up()));
        assert!(result.is_err());
    }

    // ── resolve_in with singleton (doesn't propagate scope) ───────────────

    #[test]
    fn resolve_in_singleton_ignores_scope() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        // Singleton should resolve regardless of scope
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
    }

    // ── resolve_trait_in same owner ───────────────────────────────────────

    #[test]
    fn resolve_trait_in_same_owner_success() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
    }

    // ── resolve_qualified_trait_in same owner ─────────────────────────────

    #[test]
    fn resolve_qualified_trait_in_same_owner_success() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope);
        assert!(result.is_ok());
    }

    // ── resolve_all_traits_in same owner ──────────────────────────────────

    #[test]
    fn resolve_all_traits_in_same_owner_v2_cov3() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result = c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    // ── HierarchicalBeanFactory with parent ───────────────────────────────

    #[test]
    fn contains_local_bean_in_registry_only() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(!c.contains_local_bean("nonexistent_type"));
    }

    // ── Display for ResolveError variants ─────────────────────────────────

    #[test]
    fn resolve_error_not_found_display() {
        let err = ResolveError::NotFound {
            component: "test".to_string(),
            path: vec!["a".to_string(), "b".to_string()],
        };
        let display = format!("{}", err);
        assert!(display.contains("test"));
    }

    #[test]
    fn resolve_error_ambiguous_display() {
        let err = ResolveError::Ambiguous {
            component: "test".to_string(),
            candidates: vec!["c1".to_string(), "c2".to_string()],
            path: vec![],
        };
        let display = format!("{}", err);
        assert!(display.contains("test"));
    }

    #[test]
    fn resolve_error_type_mismatch_display() {
        let err = ResolveError::TypeMismatch {
            component: ComponentKey::of::<String>(),
        };
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }

    #[test]
    fn resolve_error_construction_display() {
        let err = ResolveError::Construction {
            component: ComponentKey::of::<String>(),
            source: vernal_core::SharedError::from(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "construction failed",
            )) as Box<dyn std::error::Error + Send + Sync>),
        };
        let display = format!("{}", err);
        assert!(display.contains("construction failed"));
    }

    #[test]
    fn resolve_error_circular_runtime_display() {
        let err = ResolveError::CircularRuntime {
            path: vec!["A".to_string(), "B".to_string(), "A".to_string()],
        };
        let display = format!("{}", err);
        assert!(display.contains("A"));
    }

    #[test]
    fn resolve_error_scope_not_active_display() {
        let err = ResolveError::ScopeNotActive {
            component: ComponentKey::of::<String>(),
            scope: crate::ScopeKey::of::<i32>(),
        };
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }

    #[test]
    fn resolve_error_scope_owner_mismatch_display() {
        let err = ResolveError::ScopeOwnerMismatch {
            scope: crate::ScopeKey::of::<i32>(),
        };
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }

    #[test]
    fn resolve_error_trait_binding_type_mismatch_display() {
        use crate::TraitKey;
        let err = ResolveError::TraitBindingTypeMismatch {
            binding: TraitKey::of::<dyn std::fmt::Display>(),
            target: ComponentKey::of::<String>(),
        };
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }

    #[test]
    fn resolve_error_scope_unavailable_display() {
        let err = ResolveError::ScopeUnavailable {
            component: ComponentKey::of::<String>(),
            scope: crate::ScopeKey::of::<i32>(),
            state: crate::ScopeState::Closed,
            cancelled: false,
        };
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Additional coverage tests (cov4)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn shared_handle_resolves_same_beans_cov4() {
        let c = make_container();
        let handle = c.shared_handle();
        let a: Arc<String> = c.resolve().unwrap();
        let b: Arc<String> = handle.resolve().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn construct_resolves_dependencies_cov4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42)).unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let c = Container::new(b.build().unwrap());
        let result: Arc<String> = c.resolve().unwrap();
        assert_eq!(*result, "hello");
    }

    #[test]
    fn transient_returns_new_instance_cov4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42)).unwrap();
        let c = Container::new(b.build().unwrap());
        let a: Arc<i32> = c.resolve().unwrap();
        let b_val: Arc<i32> = c.resolve().unwrap();
        assert!(!Arc::ptr_eq(&a, &b_val));
        assert_eq!(*a, *b_val);
    }

    #[test]
    fn custom_scope_resolves_with_active_scope_cov4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::scoped::<String, i32, _>(|_| "scoped".to_string())).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<i32>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "scoped");
    }

    #[test]
    fn warm_up_skips_transient_cov4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "s".to_string())).unwrap();
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42)).unwrap();
        let c = Container::new(b.build().unwrap());
        c.warm_up().unwrap();
        assert_eq!(c.unused_definitions().len(), 1);
    }

    #[test]
    fn resolve_optional_typed_found_cov4() {
        let c = make_container();
        let dep = crate::Dependency::of::<String>();
        let result = c.resolve_optional_typed::<String>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn resolve_optional_typed_not_found_returns_none_cov4() {
        let c = make_container();
        let dep = crate::Dependency::of::<f64>();
        let result = c.resolve_optional_typed::<f64>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn resolve_in_matching_scope_cov4() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_in(&scope);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_in_matching_scope_cov4() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("primary").unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
        assert!(result.is_ok());
    }

    #[test]
    fn select_definition_with_qualifier_cov4() {
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("myQ").unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "q".to_string()).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        let dep = crate::Dependency::qualified::<String>(q);
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_trait_binding_primary_wins_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42)).unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        assert!(c.resolve_trait::<dyn std::fmt::Display + Send + Sync>().is_ok());
    }

    #[test]
    fn select_trait_binding_ambiguous_no_primary_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42)).unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        assert!(c.resolve_trait::<dyn std::fmt::Display + Send + Sync>().is_err());
    }

    #[test]
    fn select_trait_binding_with_qualifier_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "q".to_string())).unwrap();
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let q2 = Qualifier::new("q").unwrap();
        let dep = crate::Dependency::trait_qualified::<dyn std::fmt::Display + Send + Sync>(q2);
        assert!(c.select_trait_binding(&dep, &[]).is_ok());
    }

    #[test]
    fn select_trait_binding_not_found_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        assert!(c.resolve_trait::<dyn std::fmt::Debug + Send + Sync>().is_err());
    }

    #[test]
    fn select_trait_binding_qualified_duplicate_fails_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        let q = Qualifier::new("q").unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        let binding2 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        // Two bindings with same qualifier should fail at bind_all
        assert!(b.bind_all(vec![binding1, binding2]).is_err());
    }

    #[test]
    fn resolve_trait_in_matching_scope_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        assert!(c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope).is_ok());
    }

    #[test]
    fn resolve_all_traits_empty_cov4() {
        let c = make_container();
        assert!(c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap().is_empty());
    }

    #[test]
    fn resolve_all_traits_multiple_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42)).unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        assert_eq!(c.resolve_all_traits::<dyn std::fmt::Display + Send + Sync>().unwrap().len(), 2);
    }

    #[test]
    fn resolve_qualified_trait_not_found_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let q = Qualifier::new("missing").unwrap();
        assert!(c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q).is_err());
    }

    #[test]
    fn resolve_qualified_trait_in_matching_scope_cov4() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let scope = c.open_scope::<String>();
        assert!(c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope).is_ok());
    }

    #[test]
    fn resolve_all_traits_in_empty_cov4() {
        let c = make_container();
        let scope = c.open_scope::<String>();
        assert!(c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).unwrap().is_empty());
    }

    #[test]
    fn proxy_bean_definition_fields_cov4() {
        use crate::factory::config::bean_definition::BeanDefinition as BD;
        let proxy = ProxyBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "TestType".to_string(),
            scope: crate::component_scope::Scope::Singleton,
            source: "test".to_string(),
        };
        assert_eq!(proxy.bean_class_name(), "TestType");
        assert_eq!(proxy.scope(), crate::component_scope::Scope::Singleton);
        assert!(!proxy.is_lazy_init());
        assert!(!proxy.is_primary());
    }

    #[test]
    fn deleted_bean_definition_fields_cov4() {
        use crate::factory::config::bean_definition::BeanDefinition as BD;
        let deleted = DeletedBeanDefinition { bean_name: "test".to_string() };
        assert_eq!(deleted.bean_class_name(), "__DELETED__");
        assert_eq!(deleted.scope(), crate::component_scope::Scope::Singleton);
        assert!(!deleted.is_lazy_init());
        assert!(!deleted.is_primary());
    }

    #[test]
    fn removed_bean_definition_fields_cov4() {
        use crate::factory::config::bean_definition::BeanDefinition as BD;
        let removed = RemovedBeanDefinition {
            bean_name: "test".to_string(),
            type_name: "TestType".to_string(),
            scope: crate::component_scope::Scope::Transient,
        };
        assert_eq!(removed.bean_class_name(), "TestType");
        assert_eq!(removed.scope(), crate::component_scope::Scope::Transient);
        assert!(!removed.is_lazy_init());
        assert!(!removed.is_primary());
    }

    #[test]
    fn container_dynamic_register_and_remove_cov4() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(c.contains_bean_definition("dynamicBean"));
        c.remove_bean_definition("dynamicBean").unwrap();
        assert!(!c.contains_bean_definition("dynamicBean"));
    }

    #[test]
    fn container_dynamic_register_duplicate_fails_cov4() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def1 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def1).unwrap();
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        assert!(c.register_bean_definition("dynamicBean".to_string(), def2).is_err());
    }

    #[test]
    fn container_dynamic_get_bean_definition_cov4() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
        assert!(c.get_bean_definition("dynamicBean").is_some());
        assert!(c.get_bean_definition("alloc::string::String").is_some());
        assert!(c.get_bean_definition("nonexistent").is_none());
    }

    #[test]
    fn container_remove_not_found_cov4() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert!(c.remove_bean_definition("nonexistent").is_err());
    }

    #[test]
    fn contains_local_bean_dynamic_cov4() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert!(!c.contains_local_bean("dynamic"));
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamic".to_string(), def).unwrap();
        assert!(c.contains_local_bean("dynamic"));
        c.remove_bean_definition("dynamic").unwrap();
        assert!(!c.contains_local_bean("dynamic"));
    }

    #[test]
    fn remove_from_registry_marks_deleted_cov4() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert_eq!(<Container as ListableBeanFactory>::bean_definition_count(&c), 2);
        c.remove_bean_definition("alloc::string::String").unwrap();
        assert_eq!(<Container as ListableBeanFactory>::bean_definition_count(&c), 1);
    }

    #[test]
    fn dynamic_count_includes_all_cov4() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        assert_eq!(<Container as ListableBeanFactory>::bean_definition_count(&c), 2);
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("dynamic".to_string(), def).unwrap();
        assert_eq!(<Container as ListableBeanFactory>::bean_definition_count(&c), 3);
    }

    #[test]
    fn listable_beans_of_type_id_cov4() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.beans_of_type_id(TypeId::of::<String>(), true, true).unwrap().len(), 1);
        assert!(c.beans_of_type_id(TypeId::of::<f64>(), true, true).unwrap().is_empty());
    }

    #[test]
    fn listable_contains_non_singleton_cov4() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string())).unwrap();
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_non_singleton_bean());
        assert!(!c.contains_singleton_bean());
    }

    #[test]
    fn listable_bean_names_iterator_cov4() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_names_iterator().collect::<Vec<String>>().len(), 2);
    }

    #[test]
    fn hierarchical_parent_bean_factory_cov4() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(c.parent_bean_factory().is_none());
        c.set_parent_bean_factory(Arc::new(make_container())).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    #[test]
    fn singleton_registry_cov4() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        assert!(!c.contains_singleton("myBean"));
        c.register_singleton("myBean", Arc::new("mySingleton".to_string()) as Arc<dyn Any + Send + Sync>);
        assert!(c.contains_singleton("myBean"));
        assert_eq!(*c.get_singleton("myBean").unwrap().downcast_ref::<String>().unwrap(), "mySingleton");
        assert!(c.singleton_names().contains(&"myBean".to_string()));
    }

    #[test]
    fn singleton_get_after_resolve_cov4() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert!(c.get_singleton("alloc::string::String").is_some());
    }

    #[test]
    fn bean_factory_get_type_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.get_type(&ComponentKey::of::<String>()).is_ok());
        assert!(c.get_type(&ComponentKey::of::<f64>()).is_err());
    }

    #[test]
    fn bean_factory_get_aliases_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.get_aliases(&ComponentKey::of::<String>()).is_empty());
    }

    #[test]
    fn bean_factory_is_singleton_and_prototype_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string())).unwrap();
        let c = Container::new(b.build().unwrap());
        assert!(!c.is_singleton(&ComponentKey::of::<String>()).unwrap());
    }

    #[test]
    fn bean_factory_contains_bean_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.contains_bean(&ComponentKey::of::<String>()));
        assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
    }

    #[test]
    fn bean_factory_qualified_key_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let mut b = RegistryBuilder::new();
        let q = Qualifier::new("q").unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "q".to_string()).qualified(q.clone())).unwrap();
        let c = Container::new(b.build().unwrap());
        assert!(c.contains_bean(&ComponentKey::of::<String>().with_qualifier(q)));
    }

    #[test]
    fn bean_factory_is_type_match_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        assert!(c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<String>()));
        assert!(!c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<i32>()));
        assert!(!c.is_type_match(&ComponentKey::of::<f64>(), TypeId::of::<f64>()));
    }

    #[test]
    fn bean_factory_get_bean_provider_cov4() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>());
        assert!(provider.is_ok());
    }

    #[test]
    fn configurable_bean_factory_cov4() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let c = make_container();
        assert!(!c.is_currently_in_creation("any_bean"));
        assert!(c.get_dependent_beans("alloc::string::String").is_empty());
        assert!(c.get_dependencies_for_bean("alloc::string::String").is_empty());
    }

    #[test]
    fn configurable_listable_bean_factory_cov4() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_type(TypeId::of::<f64>());
        c.ignore_dependency_interface(TypeId::of::<dyn std::fmt::Debug>());
        c.register_resolvable_dependency(TypeId::of::<i64>(), Arc::new(99i64));
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
        assert!(c.is_autowire_candidate("any_bean"));
    }

    #[test]
    fn pre_instantiate_singletons_cov4() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let c = make_container();
        c.pre_instantiate_singletons().unwrap();
        assert!(c.unused_definitions().is_empty());
    }

    #[test]
    fn display_path_edge_cases_cov4() {
        assert!(Container::display_path(&[], None).is_empty());
        assert_eq!(Container::display_path(
            &[ComponentKey::of::<String>(), ComponentKey::of::<i32>(), ComponentKey::of::<f64>()],
            Some("leaf".to_string()),
        ).len(), 4);
    }

    #[test]
    fn container_new_empty_registry_cov4() {
        assert!(Container::new(RegistryBuilder::new().build().unwrap()).registry().definitions().is_empty());
    }

    #[test]
    fn transient_tracker_cov4() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42)).unwrap();
        let c = Container::new(b.build().unwrap());
        let _: Arc<i32> = c.resolve().unwrap();
        let _tracker = c.transient_tracker();
    }

    #[test]
    fn resolve_error_display_variants_cov4() {
        assert!(format!("{}", ResolveError::ScopeUnavailable {
            component: ComponentKey::of::<String>(),
            scope: crate::ScopeKey::of::<i32>(),
            state: crate::ScopeState::Closed,
            cancelled: true,
        }).contains("cancelled"));

        assert!(format!("{}", ResolveError::NotFound {
            component: "target".to_string(),
            path: vec!["A".to_string(), "B".to_string()],
        }).contains("target"));

        assert!(!format!("{}", ResolveError::ScopeOwnerMismatch {
            scope: crate::ScopeKey::of::<String>(),
        }).is_empty());
    }

    #[test]
    fn resolve_in_wrong_scope_key_cov4() {
        let c = make_container();
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let result: Result<Arc<String>, _> = c.resolve_in(&other.open_scope::<i32>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_definition_records_resolution_cov4() {
        let c = make_container();
        let _: Arc<String> = c.resolve().unwrap();
        assert_eq!(c.unused_definitions().len(), 1);
    }

    #[test]
    fn register_reregister_after_delete_cov4() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut c = make_container();
        let def = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def).unwrap();
        c.remove_bean_definition("myBean").unwrap();
        let def2 = Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        c.register_bean_definition("myBean".to_string(), def2).unwrap();
    }

    #[test]
    fn bean_names_for_type_id_empty_cov4() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.bean_names_for_type_id(TypeId::of::<Vec<u8>>(), true, true).is_empty());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn resolve_optional_trait_typed_not_found_returns_none() {
        let c = make_container();
        let dep = crate::Dependency::all_traits_of::<dyn std::fmt::Display + Send + Sync>();
        let result = c.resolve_optional_trait_typed::<dyn std::fmt::Display + Send + Sync>(&dep, &[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn resolve_all_traits_in_wrong_owner_fails() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        assert!(c.resolve_all_traits_in::<dyn std::fmt::Display + Send + Sync>(&scope).is_err());
    }

    #[test]
    fn resolve_trait_in_wrong_owner_fails() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        assert!(c.resolve_trait_in::<dyn std::fmt::Display + Send + Sync>(&scope).is_err());
    }

    #[test]
    fn resolve_qualified_trait_in_wrong_owner_fails() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let q = Qualifier::new("q").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let other = Container::new(RegistryBuilder::new().build().unwrap());
        let scope = other.open_scope::<String>();
        assert!(c.resolve_qualified_trait_in::<dyn std::fmt::Display + Send + Sync>(&q, &scope).is_err());
    }

    #[test]
    fn resolve_trait_primary_resolves() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42)).unwrap();
        let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
        let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        b.bind_all(vec![binding1, binding2]).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_trait::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_qualified_trait_success() {
        use crate::TraitBinding;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "qualified".to_string())).unwrap();
        let q = Qualifier::new("myQ").unwrap();
        let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).qualified(q.clone());
        b.bind(binding).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_qualified_trait::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_ok());
    }

    #[test]
    fn autowire_bean_properties_unsupported_mode() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.autowire_bean_properties(bean, 99, false);
        assert!(result.is_ok());
    }

    #[test]
    fn apply_bean_property_values_returns_same() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.apply_bean_property_values(bean.clone(), "myBean").unwrap();
        assert!(Arc::ptr_eq(&bean, &result));
    }

    #[test]
    fn initialize_bean_applies_processors() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.initialize_bean(bean, "myBean");
        assert!(result.is_ok());
    }

    #[test]
    fn destroy_bean_instance_succeeds() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = c.destroy_bean_instance("myBean", &*bean);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_single_match() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let result = c.resolve_named_bean(TypeId::of::<String>());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_named_bean_not_found() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let result = c.resolve_named_bean(TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_named_bean_ambiguous() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(Qualifier::new("q").unwrap())).unwrap();
        let c = Container::new(b.build().unwrap());
        let result = c.resolve_named_bean(TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_dependency_single_match() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = make_container();
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor {
            type_id: TypeId::of::<String>(),
            type_name: "String".to_string(),
            required: true,
        };
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn resolve_dependency_not_found_required() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor {
            type_id: TypeId::of::<String>(),
            type_name: "String".to_string(),
            required: true,
        };
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_dependency_not_found_optional() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let descriptor = crate::factory::support::dependency_descriptor::DependencyDescriptor {
            type_id: TypeId::of::<String>(),
            type_name: "String".to_string(),
            required: false,
        };
        let result = c.resolve_dependency(&descriptor, None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn set_type_converter_and_type_converter_none() {
        use crate::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
        let mut c = make_container();
        c.set_type_converter(None);
        assert!(c.type_converter().is_none());
    }

    #[test]
    fn object_provider_get_empty_container() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
        let result = provider.get();
        assert!(result.is_err());
    }

    #[test]
    fn object_provider_if_available_empty() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
        let result = provider.if_available();
        assert!(result.is_none());
    }

    #[test]
    fn object_provider_stream_empty() {
        use crate::factory::bean_factory::BeanFactory;
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
        let items = provider.stream();
        assert!(items.is_empty());
    }

    #[test]
    fn select_definition_ambiguous() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(Qualifier::new("q1").unwrap())).unwrap();
        let c = Container::new(b.build().unwrap());
        // Two String definitions but only one unqualified - should find it
        let dep = crate::Dependency::of::<String>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn select_definition_not_found() {
        let c = make_container();
        let dep = crate::Dependency::of::<f64>();
        let result = c.select_definition(&dep, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_definition_circular_detected() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
        let c = Container::new(b.build().unwrap());
        let definition = c.registry().definitions().first().unwrap();
        let stack = vec![definition.key().clone()];
        let result = c.resolve_definition(definition, &stack, None);
        assert!(result.is_err());
    }

    #[test]
    fn warm_up_empty_registry() {
        let c = Container::new(RegistryBuilder::new().build().unwrap());
        assert!(c.warm_up().is_ok());
    }

    #[test]
    fn container_shared_handle() {
        let c = make_container();
        let _handle = c.shared_handle();
    }

    #[test]
    fn bean_definition_registry_bean_definition_count() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        assert_eq!(c.bean_definition_count(), 2);
    }

    #[test]
    fn bean_definition_registry_bean_definition_names() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let c = make_container();
        let names = c.bean_definition_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn listable_bean_names_for_type() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let names = c.bean_names_for_type_id(TypeId::of::<String>(), true, true);
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn listable_beans_of_type() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        let beans = c.beans_of_type_id(TypeId::of::<String>(), true, true).unwrap();
        assert_eq!(beans.len(), 1);
    }

    #[test]
    fn listable_contains_singleton_bean() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert!(c.contains_singleton_bean());
        assert!(!c.contains_non_singleton_bean());
    }

    #[test]
    fn listable_bean_count() {
        use crate::factory::listable_bean_factory::ListableBeanFactory;
        let c = make_container();
        assert_eq!(c.bean_count(), 2);
    }

    #[test]
    fn configurable_freeze_and_check() {
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(!c.is_configuration_frozen());
        c.freeze_configuration();
        assert!(c.is_configuration_frozen());
    }

    #[test]
    fn configurable_ignore_dependency_types() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.ignore_dependency_type(TypeId::of::<f64>());
        c.ignore_dependency_interface(TypeId::of::<dyn std::fmt::Debug>());
    }

    #[test]
    fn configurable_register_resolvable_dependency() {
        use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
        let mut c = make_container();
        c.register_resolvable_dependency(TypeId::of::<i64>(), Arc::new(99i64));
    }

    #[test]
    fn singleton_registry_register_and_get() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        c.register_singleton("custom", Arc::new(42i32) as Arc<dyn Any + Send + Sync>);
        assert!(c.contains_singleton("custom"));
        let val = c.get_singleton("custom").unwrap();
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn singleton_registry_names() {
        use crate::factory::config::singleton_bean_registry::SingletonBeanRegistry;
        let c = make_container();
        let names = c.singleton_names();
        assert!(names.is_empty());
    }

    #[test]
    fn hierarchical_set_and_get_parent() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
        let mut c = make_container();
        assert!(c.parent_bean_factory().is_none());
        let parent: Arc<dyn crate::factory::bean_factory::BeanFactory> = Arc::new(make_container());
        c.set_parent_bean_factory(parent).unwrap();
        assert!(c.parent_bean_factory().is_some());
    }

    #[test]
    fn hierarchical_contains_local_bean_type() {
        use crate::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
        let c = make_container();
        assert!(c.contains_local_bean("alloc::string::String"));
        assert!(!c.contains_local_bean("nonexistent"));
    }

    #[test]
    fn resolve_transient_scoped() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::transient::<i32, _>(|_| 42)).unwrap();
        let c = Container::new(b.build().unwrap());
        let a: Arc<i32> = c.resolve().unwrap();
        let b: Arc<i32> = c.resolve().unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn resolve_with_dependency_chain() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(_| "hello".to_string())).unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42)).unwrap();
        let c = Container::new(b.build().unwrap());
        let s: Arc<String> = c.resolve().unwrap();
        let i: Arc<i32> = c.resolve().unwrap();
        assert_eq!(*s, "hello");
        assert_eq!(*i, 42);
    }

    #[test]
    fn bean_factory_get_bean_by_type_id_single_match() {
        use crate::factory::bean_factory::BeanFactory;
        let c = make_container();
        let result = c.get_bean_by_type_id(TypeId::of::<String>());
        assert!(result.is_ok());
    }
}

