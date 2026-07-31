<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->

> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。
> [vernal-beans 自动审计](../migration-audit/vernal-beans.md)为准。

# vernal-beans 技术要求

## 1. 对标范围

- 结构与对象主线：Spring Framework `spring-beans`，固定提交
  `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。
- 当前审计对象：323 个 Java class/interface/enum/record；`package-info.java` 不计入。
- 目标：保留 Spring 对象边界和最后两层子包，转换为符合 Rust 习惯的文件、类型和方法名。
- 本文不以 `cargo test` 通过替代对象验收，也不允许用“合并实现”消除应有文件。

## 2. 目标目录

```text
crates/vernal-beans/src/
├── propertyeditors/
├── support/
└── factory/
    ├── annotation/
    ├── config/
    ├── support/
    └── xml/
```

当 Java 包层级更深时只保留最后两层。例如：

| Java 对象 | Rust 目标 |
|---|---|
| `factory/config/BeanDefinition.java` | `factory/config/bean_definition.rs` |
| `factory/xml/support/Foo.java` | `xml/support/foo.rs` |
| `propertyeditors/PatternEditor.java` | `propertyeditors/pattern_editor.rs` |

`lib.rs` 与 `mod.rs` 只能写模块说明、`mod` 声明和显式 `pub use`。

## 3. Bean 生命周期语义

CodeGraph 对固定 Spring 提交和当前 Vernal 工作树的调用链核对结果如下：

```mermaid
flowchart LR
    A["get_bean / doGetBean"] --> B["合并 BeanDefinition"]
    B --> C["create_bean / doCreateBean"]
    C --> D["构造器或工厂方法实例化"]
    D --> E["populate_bean：属性与依赖注入"]
    E --> F["BeanPostProcessor before"]
    F --> G["初始化回调"]
    G --> H["BeanPostProcessor after"]
    H --> I["注册销毁回调"]
    I --> J["作用域缓存并返回"]
```

迁移必须保留下列不变量：

1. 单例创建、早期引用和循环依赖处理的时序必须明确，不可用普通缓存替代。
2. 实例化、属性填充、初始化、后处理和销毁是不同阶段，错误传播需保留阶段信息。
3. `BeanFactoryPostProcessor` 操作元数据；`BeanPostProcessor` 操作实例，两者不得混用。
4. `FactoryBean` 的工厂对象与产品对象缓存语义必须分离。
5. 父子工厂、别名、作用域、依赖排序和销毁顺序需要语义测试。

## 4. Rust 实现约束

- 一个 Java 对象对应一个 `.rs` 文件；内部类型和 Builder 可与主对象同文件。
- 文件/目录、方法和参数使用 `snake_case`；类型使用 `PascalCase`。
- Java `nullable` 映射为 `Option<T>`，checked exception 映射为
  `thiserror` 错误枚举与 `Result`。
- 共享缓存按语义使用 `Arc<RwLock<HashMap<...>>>` 或 `DashMap`。
- 每个对象和 `pub` 方法必须有中文文档注释，并注明 Java FQN/方法来源。
- 生产代码禁止 wildcard import；禁止 `todo!()`、`unimplemented!()` 和空业务逻辑。

## 5. 当前事实

当前工作树的权威数量由审计器生成，不在本文手写。运行：

```bash
python3 scripts/audit_migration_docs.py --module vernal-beans --check
```

截至本次生成，323 个对象中只有 42 个满足严格 `IMPLEMENTED`；其余包含
`MISPLACED`、`MISSING`、`STUB` 和 `UNVERIFIED`。因此 `vernal-beans`
不得标记为“完成”或“完备”。具体对象见[对象级对照表](对象级对照表.md)。

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-beans 技术要求文档

> **对标**：`spring-beans`（IoC 容器内核）  
> **现状**：117 文件 / 13,644 行  
> **Rust 版本**：edition 2024 / rustc 1.88  
> **选型约定**：参见 [Spring 组件替换约定](../Spring-组件替换约定.md)  
> **日期**：2026-07-28

---

## 一、容器接口层级

### 1.1 BeanFactory

Spring `BeanFactory` 是 IoC 容器的顶层客户端视图。vernal-beans 用 `trait BeanFactory` 对标，
所有方法使用 `TypeId` 而非泛型参数以保持 `dyn BeanFactory` 可用。

**Spring API**

```java
public interface BeanFactory {
    Object getBean(String name) throws BeansException;
    <T> T getBean(Class<T> requiredType) throws BeansException;
    boolean containsBean(String name);
    boolean isSingleton(String name) throws NoSuchBeanDefinitionException;
    boolean isPrototype(String name) throws NoSuchBeanDefinitionException;
    Class<?> getType(String name) throws NoSuchBeanDefinitionException;
    String[] getAliases(String name);
    <T> ObjectProvider<T> getBeanProvider(Class<T> requiredType);
    boolean isTypeMatch(String name, Class<?> typeToMatch);
}
```

**Rust trait**

```rust
pub trait BeanFactory: Send + Sync + 'static {
    fn get_bean_by_key(&self, key: &ComponentKey)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn get_bean_by_type_id(&self, type_id: TypeId)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn contains_bean(&self, key: &ComponentKey) -> bool;
    fn is_singleton(&self, key: &ComponentKey) -> Result<bool, BoxError>;
    fn is_prototype(&self, key: &ComponentKey) -> Result<bool, BoxError>;
    fn get_type(&self, key: &ComponentKey) -> Result<Option<&'static str>, BoxError>;
    fn get_aliases(&self, key: &ComponentKey) -> Vec<ComponentKey>;
    fn get_bean_provider_by_type_id(&self, type_id: TypeId)
        -> Result<Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>, BoxError>;
    fn is_type_match(&self, key: &ComponentKey, type_id: TypeId) -> bool;
}
```

**约束表**

| 约束项 | 说明 |
|:---|:---|
| `Send + Sync + 'static` | 所有容器接口必须线程安全 |
| `TypeId` 替代泛型 | 保持 dyn-compatible，避免 `<T>` 泛型方法 |
| `ComponentKey` 替代 `String` | Rust 类型名 + 可选 `Qualifier`，不可变 |
| `Arc<dyn Any + Send + Sync>` | 实例返回值，对标 Java `Object` |
| `BoxError` | `Box<dyn Error + Send + Sync>`，对标 `BeansException` |

**待补齐**

- [ ] `getBean(String name, Object... args)` — 带构造参数的获取
- [ ] `getBean(String name, Class<T> requiredType)` — 按名称+类型获取

---

### 1.2 HierarchicalBeanFactory

**Spring API**

```java
public interface HierarchicalBeanFactory extends BeanFactory {
    BeanFactory getParentBeanFactory();
    boolean containsLocalBean(String name);
}
```

**Rust trait**

```rust
pub trait HierarchicalBeanFactory: BeanFactory {
    fn parent_bean_factory(&self) -> Option<&dyn BeanFactory>;
    fn contains_local_bean(&self, name: &str) -> bool;
}
```

**待补齐**

- [ ] Container 层实现 `HierarchicalBeanFactory`
- [ ] 父子容器 Bean 解析委托链

---

### 1.3 ListableBeanFactory

**Rust trait**

```rust
pub trait ListableBeanFactory: BeanFactory {
    fn contains_bean_definition(&self, bean_name: &str) -> bool;
    fn bean_definition_count(&self) -> usize;
    fn bean_definition_names(&self) -> Vec<String>;
    fn bean_names_for_type_id(&self, type_id: TypeId,
        include_non_singletons: bool, allow_eager_init: bool) -> Vec<String>;
    fn beans_of_type_id(&self, type_id: TypeId,
        include_non_singletons: bool, allow_eager_init: bool)
        -> Result<HashMap<String, Arc<dyn Any + Send + Sync>>, BoxError>;
    fn bean_post_processor_count(&self) -> usize;
    fn contains_non_singleton_bean(&self) -> bool;
    fn contains_singleton_bean(&self) -> bool;
    fn bean_names_iterator(&self) -> Box<dyn Iterator<Item = String> + '_>;
}
```

**待补齐**

- [ ] Container 层实现 `ListableBeanFactory`
- [ ] `bean_names_for_annotation_type` — 按注解类型列举（Spring 6.x 新增）

---

### 1.4 ConfigurableBeanFactory / ConfigurableListableBeanFactory

**Rust trait**

```rust
pub trait ConfigurableBeanFactory: HierarchicalBeanFactory {
    fn set_parent_bean_factory(&mut self, parent: Arc<dyn Any + Send + Sync>) -> Result<(), BoxError>;
    fn register_scope(&mut self, scope_name: &str, scope: Box<dyn BeanScope>);
    fn registered_scope_names(&self) -> Vec<String>;
    fn get_registered_scope(&self, scope_name: &str) -> Option<&dyn BeanScope>;
    fn add_bean_post_processor(&mut self, processor: Arc<dyn BeanPostProcessor>);
    fn bean_post_processor_count(&self) -> usize;
    fn register_alias(&mut self, bean_name: &str, alias: &str) -> Result<(), BoxError>;
    fn is_factory_bean(&self, name: &str) -> bool;
    fn set_currently_in_creation(&mut self, bean_name: &str, in_creation: bool);
    fn is_currently_in_creation(&self, bean_name: &str) -> bool;
    fn register_dependent_bean(&mut self, bean_name: &str, dependent_bean_name: &str);
    fn get_dependent_beans(&self, bean_name: &str) -> Vec<String>;
    fn get_dependencies_for_bean(&self, bean_name: &str) -> Vec<String>;
    fn destroy_bean(&self, bean_name: &str, bean_instance: &dyn Any) -> Result<(), BoxError>;
    fn destroy_singletons(&self);
    fn add_embedded_value_resolver(&mut self, resolver: Arc<dyn Fn(&str) -> String + Send + Sync>);
    fn resolve_embedded_value(&self, value: &str) -> String;
}

pub trait ConfigurableListableBeanFactory: ListableBeanFactory + ConfigurableBeanFactory {
    fn ignore_dependency_type(&mut self, type_id: TypeId);
    fn ignore_dependency_interface(&mut self, interface_id: TypeId);
    fn register_resolvable_dependency(&mut self, dependency_type: TypeId,
        autowired_value: Arc<dyn Any + Send + Sync>);
    fn is_autowire_candidate(&self, bean_name: &str) -> bool;
    fn freeze_configuration(&mut self);
    fn is_configuration_frozen(&self) -> bool;
    fn pre_instantiate_singletons(&self) -> Result<(), BoxError>;
}
```

**待补齐**

- [ ] Container 层完整实现 `ConfigurableBeanFactory`
- [ ] Container 层完整实现 `ConfigurableListableBeanFactory`
- [ ] `register_resolvable_dependency` 隐式依赖注册

---

## 二、Bean 定义

### 2.1 BeanDefinition

Spring `BeanDefinition` 描述一个 Bean 实例的全部元数据。vernal-beans 用
`trait BeanDefinition` 对标，每条 `RegistryBuilder::register()` 将类型转换为
不可变 `ComponentDefinition`。

**Rust trait**

```rust
pub trait BeanDefinition: Send + Sync + Any + fmt::Debug {
    fn bean_name(&self) -> &ComponentKey;
    fn bean_class_name(&self) -> &str;
    fn scope(&self) -> Scope;
    fn is_lazy_init(&self) -> bool;
    fn is_primary(&self) -> bool;
    fn is_fallback(&self) -> bool { false }              // Spring 6.2+
    fn is_autowire_candidate(&self) -> bool { true }
    fn role(&self) -> i32 { ROLE_APPLICATION }
    fn description(&self) -> Option<&str> { None }
    fn bean_class_name_internal(&self) -> Option<&str> { Some(self.bean_class_name()) }
    fn parent_name(&self) -> Option<&str> { None }
    fn factory_bean_name(&self) -> Option<&str> { None }
    fn factory_method_name(&self) -> Option<&str> { None }
    fn init_method_name(&self) -> Option<&str> { None }
    fn destroy_method_name(&self) -> Option<&str> { None }
    fn is_abstract(&self) -> bool { false }
    fn is_singleton(&self) -> bool { matches!(self.scope(), Scope::Singleton) }
    fn is_prototype(&self) -> bool { matches!(self.scope(), Scope::Transient) }
    fn resource_description(&self) -> Option<&str> { None }
    fn originating_bean_definition(&self) -> Option<&dyn BeanDefinition> { None }
}
```

**约束表**

| 约束项 | 说明 |
|:---|:---|
| `Send + Sync + Any + Debug` | 线程安全 + 类型擦除 + 调试输出 |
| `Scope` 枚举 | `Singleton` / `Transient` / `Custom(ScopeKey)` |
| 常量 | `SCOPE_SINGLETON`、`SCOPE_PROTOTYPE`、`ROLE_APPLICATION/0`、`ROLE_SUPPORT/1`、`ROLE_INFRASTRUCTURE/2` |

**待补齐**

- [ ] `ConstructorArgumentValues` 集成到 BeanDefinition
- [ ] `MutablePropertyValues` 集成到 BeanDefinition
- [ ] `getDependsOn()` — 显式依赖声明

---

### 2.2 BeanDefinitionRegistry

**Rust trait**

```rust
pub trait BeanDefinitionRegistry: Send + Sync + 'static {
    fn register_bean_definition(&mut self, bean_name: String,
        definition: Box<dyn BeanDefinition>) -> Result<(), BoxError>;
    fn remove_bean_definition(&mut self, bean_name: &str)
        -> Result<Box<dyn BeanDefinition>, BoxError>;
    fn get_bean_definition(&self, bean_name: &str) -> Option<&dyn BeanDefinition>;
    fn contains_bean_definition(&self, bean_name: &str) -> bool;
    fn bean_definition_count(&self) -> usize;
    fn bean_definition_names(&self) -> Vec<String>;
}
```

**待补齐**

- [ ] Container 层完整实现（当前有 ProxyBeanDefinition 泄漏问题）
- [ ] `registerAlias` / `removeAlias` / `getAliases` — 别名注册表

---

### 2.3 BeanDefinitionBuilder / GenericBeanDefinition / RootBeanDefinition

**相关文件**

- `bean_definition_builder.rs` — 建造式 API，对标 `BeanDefinitionBuilder`
- `generic_bean_definition.rs` — 通用 BeanDefinition，对标 `GenericBeanDefinition`
- `root_bean_definition.rs` — 根 BeanDefinition，对标 `RootBeanDefinition`
- `component_definition.rs` — vernal 原生不可变组件定义（工厂闭包 + 依赖声明）

**ComponentDefinition 建造式 API**

```rust
ComponentDefinition::singleton(|resolver| { ... })     // 单例
ComponentDefinition::transient(|resolver| { ... })     // 瞬时
ComponentDefinition::scoped::<T, MyScope>(|resolver| { ... }) // 自定义作用域
    .depends_on::<DepA>()                              // 声明依赖
    .depends_on_optional::<DepB>()                     // 可选依赖
    .depends_on_trait::<dyn Service>()                 // Trait Object 依赖
    .depends_on_qualified::<T>(Qualifier::new("x")?)   // 限定符依赖
    .qualified(Qualifier::new("primary")?)             // 自身限定符
    .with_init_order(100)                              // 初始化排序
```

**待补齐**

- [ ] `GenericBeanDefinition` 完整属性支持（constructor-arg、property values）
- [ ] `RootBeanDefinition` 合并逻辑（parent-child definition merge）

---

## 三、单例注册

### 3.1 SingletonBeanRegistry

**Rust trait**

```rust
pub trait SingletonBeanRegistry: Send + Sync + 'static {
    fn register_singleton(&self, bean_name: &str,
        singleton_object: Arc<dyn Any + Send + Sync>);
    fn get_singleton(&self, bean_name: &str) -> Option<Arc<dyn Any + Send + Sync>>;
    fn contains_singleton(&self, bean_name: &str) -> bool;
    fn singleton_names(&self) -> Vec<String>;
    fn singleton_count(&self) -> usize;
    fn add_singleton_callback(&mut self, bean_name: String,
        callback: Arc<dyn Fn(&dyn Any) + Send + Sync>);
    fn singleton_mutex(&self) -> Arc<dyn Any + Send + Sync>;
}
```

**Container 内部实现**

Container 使用 `OnceLock<Result<ErasedComponent, ResolveError>>` 保证单例并发只构造一次。
单例缓存按 Container 隔离（`HashMap<ComponentKey, Arc<SingletonCell>>`），不使用进程级全局变量。

**待补齐**

- [ ] Container 层实现 `SingletonBeanRegistry` trait
- [ ] `addSingletonCallback` 回调触发时机与 Spring 对齐
- [ ] 单例销毁顺序（逆拓扑序）

---

## 四、FactoryBean

### 4.1 FactoryBean / SmartFactoryBean

**Rust trait**

```rust
pub trait FactoryBean: Send + Sync + 'static {
    const OBJECT_TYPE_ATTRIBUTE: &'static str = "factoryBeanObjectType";
    fn get_object(&self) -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn get_object_type(&self) -> Option<TypeId>;
    fn is_singleton(&self) -> bool { true }
}

pub trait SmartFactoryBean: FactoryBean {
    fn is_eager_init(&self) -> bool;
}
```

**前缀约定**：`FACTORY_BEAN_PREFIX = "&"` — `getBean("&myFactory")` 返回 FactoryBean 本身。

**待补齐**

- [ ] Container 层自动识别 FactoryBean，`getBean` 时调用 `get_object`
- [ ] `&` 前缀解析逻辑
- [ ] `SmartFactoryBean.isEagerInit` 集成到容器启动流程

---

## 五、扩展点

### 5.1 BeanPostProcessor

**Rust trait**

```rust
pub trait BeanPostProcessor: Send + Sync + 'static {
    fn post_process_before_initialization(&self,
        bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, BoxError> {
        Ok(Some(bean))
    }

    fn post_process_after_initialization(&self,
        bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, BoxError> {
        Ok(Some(bean))
    }
}
```

Container 内部以 `Vec<Arc<dyn BeanPostProcessor>>` 持有 PostProcessor 链，
在 `resolve_definition` 成功后按注册顺序执行 `post_process_after_initialization`。

**待补齐**

- [ ] PostProcessor 排序（`Ordered` / `PriorityOrdered` trait）
- [ ] `post_process_before_initialization` 在 Container 中调用（当前只调用了 `after`）
- [ ] AOP 代理包装通过 PostProcessor 注入

---

### 5.2 BeanFactoryPostProcessor / BeanDefinitionRegistryPostProcessor

**Rust trait**

```rust
pub trait BeanFactoryPostProcessor: Send + Sync + 'static {
    fn post_process_bean_factory(&self,
        bean_factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), BoxError>;
}

pub trait BeanDefinitionRegistryPostProcessor: BeanFactoryPostProcessor {
    fn post_process_bean_definition_registry(&self,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), BoxError>;

    fn post_process_bean_factory(&self,
        _bean_factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), BoxError> { Ok(()) }
}
```

**执行顺序**：`BeanDefinitionRegistryPostProcessor` -> `BeanFactoryPostProcessor` -> Bean 实例化

**待补齐**

- [ ] Container 启动流程中调用 `BeanFactoryPostProcessor`
- [ ] `PropertyPlaceholderConfigurer` 实现
- [ ] `ConfigurationClassPostProcessor` 对标实现

---

### 5.3 InstantiationAwareBeanPostProcessor

**Rust trait**

```rust
pub trait InstantiationAwareBeanPostProcessor: BeanPostProcessor {
    fn post_process_before_instantiation(&self,
        _bean_type: &dyn Any, _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, BoxError> { Ok(None) }

    fn post_process_after_instantiation(&self,
        _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
    ) -> Result<bool, BoxError> { Ok(true) }

    fn post_process_properties(&self,
        _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
    ) -> Result<Option<Vec<(String, Arc<dyn Any + Send + Sync>)>>, BoxError> { Ok(None) }
}
```

**待补齐**

- [ ] Container 实例化流程中集成 `post_process_before_instantiation`
- [ ] `post_process_after_instantiation` 返回 false 时跳过属性注入
- [ ] `post_process_properties` 集成 `@Autowired` 处理

---

### 5.4 DestructionAwareBeanPostProcessor

```rust
pub trait DestructionAwareBeanPostProcessor: BeanPostProcessor {
    fn post_process_before_destruction(&self,
        bean: &dyn Any, bean_name: &str,
    ) -> Result<(), BoxError>;
    fn requires_destruction(&self, _bean: &dyn Any) -> bool { true }
}
```

**待补齐**

- [ ] Container 销毁流程中调用 `post_process_before_destruction`
- [ ] 销毁顺序：DestructionAwareBPP -> DisposableBean -> custom destroy-method

---

### 5.5 SmartInstantiationAwareBeanPostProcessor

```rust
pub trait SmartInstantiationAwareBeanPostProcessor:
    InstantiationAwareBeanPostProcessor {
    fn predict_bean_type(&self, _bean_type: &dyn Any, _bean_name: &str)
        -> Result<Option<Box<dyn Any + Send + Sync>>, BoxError> { Ok(None) }
    fn determine_candidate_constructors(&self, _bean_type: &dyn Any, _bean_name: &str)
        -> Result<Option<Vec<Box<dyn Any + Send + Sync>>>, BoxError> { Ok(None) }
    fn get_early_bean_reference(&self,
        bean: Arc<dyn Any + Send + Sync>, _bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, BoxError> { Ok(bean) }
}
```

**待补齐**

- [ ] 循环依赖解决：`get_early_bean_reference` 集成
- [ ] 构造器注入：`determine_candidate_constructors` 集成

---

## 六、生命周期与依赖注入

### 6.1 Aware 接口族

vernal-beans 采用 marker trait + 分离回调 trait 的设计，容器按 `is::<XxxAware>()` 检测后
调用对应的 `set_xxx` 方法。

```rust
pub trait Aware: Send + Sync + 'static {}

pub trait BeanNameAware: Aware {
    fn set_bean_name(&mut self, name: &str);
}

pub trait BeanFactoryAware: Aware {
    fn set_bean_factory(&mut self, bean_factory: Arc<dyn Any + Send + Sync>);
}
```

**执行顺序**：BeanNameAware -> BeanFactoryAware -> BPP.before -> InitializingBean -> init-method -> BPP.after

**待补齐**

- [ ] `EnvironmentAware` / `ApplicationContextAware` — 由 `vernal-context` 承接
- [ ] Container 创建流程中检测并回调 Aware 接口

---

### 6.2 InitializingBean / DisposableBean

**Rust trait**

```rust
pub trait InitializingBean: Aware {
    fn after_properties_set(&mut self) -> Result<(), BoxError>;
}

pub trait DisposableBean: Aware {
    fn destroy(&self) -> Result<(), BoxError> { Ok(()) }
}
```

**约束表**

| 约束项 | 说明 |
|:---|:---|
| 继承 `Aware` | 要求 `Send + Sync + 'static` |
| `&mut self` | `after_properties_set` 需要可变借用（可能修改内部状态） |
| `&self` | `destroy` 只读借用（清理外部资源） |
| `Err` 阻止创建 / 记录警告 | 与 Spring 异常语义对齐 |

**待补齐**

- [ ] Container 创建流程中调用 `after_properties_set`
- [ ] Container 销毁流程中调用 `destroy`
- [ ] 自定义 `init-method` / `destroy-method` 通过闭包实现

---

### 6.3 SmartInitializingSingleton

```rust
pub trait SmartInitializingSingleton: Send + Sync + 'static {
    fn after_singletons_instantiated(&self) -> Result<(), BoxError>;
}
```

**待补齐**

- [ ] Container `warm_up` 完成后调用所有 `SmartInitializingSingleton`

---

### 6.4 @Autowired / @Qualifier / @Value（过程宏替代）

Spring 通过注解实现依赖注入。Rust 无运行时注解，vernal-beans 用**过程宏 + trait 约束**替代。

| Spring 注解 | Rust 替代 | 说明 |
|:---|:---|:---|
| `@Autowired` | `Resolver::resolve::<T>()` | 工厂闭包中显式解析依赖 |
| `@Autowired(required=false)` | `Resolver::resolve_optional::<T>()` | 可选依赖 |
| `@Qualifier("name")` | `Resolver::resolve_qualified::<T>(&qualifier)` | 带限定符解析 |
| `@Value("${key}")` | `ConfigurableBeanFactory::resolve_embedded_value` | 占位符解析 |
| `@Inject` | `Resolver::resolve::<T>()` | JSR-330 语义等价 |
| `@Resource` | `Resolver::resolve_qualified::<T>(&qualifier)` | 按名称解析 |
| `@PostConstruct` | `InitializingBean::after_properties_set` | 初始化回调 |
| `@PreDestroy` | `DisposableBean::destroy` | 销毁回调 |
| `@Lazy` | `ComponentProvider` / `ObjectProvider` | 延迟解析 |

**vernal-beans 中的依赖声明方式**

```rust
ComponentDefinition::singleton(|resolver| {
    let dep_a = resolver.resolve::<ServiceA>()?;           // @Autowired
    let dep_b = resolver.resolve_optional::<ServiceB>()?;  // @Autowired(required=false)
    let dep_c = resolver.resolve_qualified::<ServiceC>(
        &Qualifier::new("primary").unwrap()
    )?;                                                     // @Qualifier("primary")
    MyService::new(dep_a, dep_b, dep_c)
})
```

**待补齐**

- [ ] `#[derive(Component)]` 过程宏自动生成工厂闭包
- [ ] `#[inject]` 属性宏标记注入点
- [ ] `@Value` 占位符解析集成到过程宏

---

### 6.5 AutowireCapableBeanFactory

**Rust trait（Container 已实现）**

```rust
pub trait AutowireCapableBeanFactory: BeanFactory {
    const AUTOWIRE_NO: i32 = 0;
    const AUTOWIRE_BY_NAME: i32 = 1;
    const AUTOWIRE_BY_TYPE: i32 = 2;
    const AUTOWIRE_CONSTRUCTOR: i32 = 3;

    fn create_bean(&self, bean_class_name: &str)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn autowire_bean(&self, existing_bean: Arc<dyn Any + Send + Sync>)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn configure_bean(&self, existing_bean: Arc<dyn Any + Send + Sync>, bean_name: &str)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn autowire(&self, bean_class_name: &str, autowire_mode: i32, dependency_check: bool)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn autowire_bean_properties(&self, existing_bean: Arc<dyn Any + Send + Sync>,
        autowire_mode: i32, dependency_check: bool)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn apply_bean_property_values(&self, existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn initialize_bean(&self, existing_bean: Arc<dyn Any + Send + Sync>, bean_name: &str)
        -> Result<Arc<dyn Any + Send + Sync>, BoxError>;
    fn destroy_bean_instance(&self, bean_name: &str, bean_instance: &dyn Any)
        -> Result<(), BoxError>;
    fn resolve_named_bean(&self, type_id: TypeId)
        -> Result<NamedBeanHolder<Arc<dyn Any + Send + Sync>>, BoxError>;
    fn resolve_dependency(&self, descriptor: &DependencyDescriptor,
        requesting_bean_name: Option<&str>)
        -> Result<Option<Arc<dyn Any + Send + Sync>>, BoxError>;
    fn set_type_converter(&mut self, converter: Option<Arc<dyn TypeConverter>>);
    fn type_converter(&self) -> Option<&dyn TypeConverter>;
}
```

Container 已实现全部 12 个方法。`create_bean` 按类名查找定义后调用 `resolve_definition`；
`autowire_bean` 查找匹配定义后通过工厂重新创建实例注入依赖；
`initialize_bean` 应用 `BeanPostProcessor` 链（before + after）。

**待补齐**

- [ ] `AUTOWIRE_BY_NAME` 按 ComponentKey 名称匹配注册表中的依赖
- [ ] `AUTOWIRE_BY_TYPE` 按 TypeId 匹配注册表中的依赖
- [ ] `set_type_converter` / `type_converter` 完整集成
- [ ] `apply_bean_property_values` 属性值注入（当前返回原实例）

---

### 6.6 DependencyDescriptor / TypeConverter / NamedBeanHolder

**DependencyDescriptor**

```rust
pub struct DependencyDescriptor {
    field_index: Option<usize>,     // 注入位置索引
    type_id: TypeId,                // 依赖类型
    type_name: &'static str,        // 类型名
    optional: bool,                 // 是否可选
    multiple: bool,                 // 是否批量（List/Array）
    qualifier: Option<String>,      // 限定符名称
    containing_bean_name: Option<String>, // 所属 Bean 名称
}
```

**TypeConverter**

```rust
pub trait TypeConverter: Send + Sync + 'static {
    fn convert_if_necessary(&self, property_name: Option<&str>,
        value: &dyn Any, target_type: TypeId)
        -> Result<Box<dyn Any + Send + Sync>, BoxError>;
    fn convert_value(&self, value: &dyn Any, target_type: TypeId)
        -> Result<Box<dyn Any + Send + Sync>, BoxError>;
}
```

**NamedBeanHolder**

```rust
pub struct NamedBeanHolder<T: Any + Send + Sync> {
    instance: Arc<T>,
    bean_name: String,
}
```

**待补齐**

- [ ] `TypeConverter` 实现注册到 Container
- [ ] `ConversionService` 集成到 `TypeConverter`
- [ ] PropertyEditor 体系完整集成（当前有 20+ editor 文件）

---

## 附录 A：Container 运行时结构

```rust
pub struct Container {
    registry: Registry,                                          // 不可变注册表
    singletons: Arc<Mutex<HashMap<ComponentKey, Arc<SingletonCell>>>>, // 单例缓存
    resolutions: Arc<ResolutionTracker>,                         // 解析追踪
    transient_tracker: Arc<TransientTracker>,                    // Transient 弱引用追踪
    bean_post_processors: Arc<Mutex<Vec<Arc<dyn BeanPostProcessor>>>>, // PostProcessor 链
    dynamic_definitions: Arc<Mutex<HashMap<String, Arc<dyn BeanDefinition>>>>, // 动态定义
    definition_cache: Arc<Mutex<HashMap<String, ProxyBeanDefinition>>>,        // 代理缓存
    owner: Arc<()>,                                              // Scope 归属标记
}
```

**已实现的 Spring trait**

- [x] `BeanFactory` — 9 个方法全部实现
- [x] `AutowireCapableBeanFactory` — 12 个方法全部实现
- [x] `BeanDefinitionRegistry` — 6 个方法全部实现（有 ProxyBeanDefinition 泄漏）

**未实现的 Spring trait**

- [ ] `HierarchicalBeanFactory`
- [ ] `ListableBeanFactory`
- [ ] `ConfigurableBeanFactory`
- [ ] `ConfigurableListableBeanFactory`
- [ ] `SingletonBeanRegistry`

---

## 附录 B：Scope 体系

**vernal-beans Scope 枚举**

```rust
pub enum Scope {
    Singleton,           // 每个容器惰性创建并缓存一个实例
    Transient,           // 每次解析都创建新实例
    Custom(ScopeKey),    // 在匹配类型身份的显式 ScopeContext 内缓存
}
```

**BeanScope 扩展接口**

```rust
pub trait BeanScope: Send + Sync + 'static {
    fn get(&self, name: &str,
        object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>)
        -> Result<Box<dyn Any + Send + Sync>, BoxError>;
    fn remove(&self, _name: &str)
        -> Result<Option<Box<dyn Any + Send + Sync>>, BoxError> { Ok(None) }
    fn register_destruction_callback(&self, _name: &str,
        _callback: Box<dyn FnOnce() + Send + Sync>) {}
    fn resolve_contextual_object(&self, _key: &str) -> Option<Box<dyn Any>> { None }
    fn conversation_id(&self) -> Option<&str> { None }
}
```

**内建 Scope 实现**：`RequestScope` / `SessionScope` / `ApplicationScope`

---

## 附录 C：ObjectProvider

```rust
pub trait ObjectProvider<T: ?Sized + Send + Sync>: Send + Sync {
    fn get(&self) -> Result<Arc<T>, BoxError>;
    fn if_available(&self) -> Option<Arc<T>>;
    fn get_if_unique(&self) -> Result<Arc<T>, BoxError>;
    fn stream(&self) -> Vec<Arc<T>>;
    fn ordered_stream(&self) -> Vec<Arc<T>>;
}
```

**待补齐**

- [ ] Container 返回的 `ObjectProvider` 行为不完整，需按 TypeId 查询实现
- [ ] `getIfUnique` 多候选时抛出 `NoUniqueBeanDefinitionException`

---

## 附录 D：完整 trait 继承层级图

```
BeanFactory
  +-- HierarchicalBeanFactory
  |     +-- ConfigurableBeanFactory
  |           +-- ConfigurableListableBeanFactory (also extends ListableBeanFactory)
  +-- ListableBeanFactory
  |     +-- ConfigurableListableBeanFactory
  +-- AutowireCapableBeanFactory

BeanDefinitionRegistry
  +-- BeanDefinitionRegistryPostProcessor (also extends BeanFactoryPostProcessor)

BeanFactoryPostProcessor
  +-- BeanDefinitionRegistryPostProcessor

BeanPostProcessor
  +-- InstantiationAwareBeanPostProcessor
  |     +-- SmartInstantiationAwareBeanPostProcessor
  +-- DestructionAwareBeanPostProcessor

Aware (marker)
  +-- BeanNameAware
  +-- BeanFactoryAware
  +-- InitializingBean
  +-- DisposableBean

FactoryBean
  +-- SmartFactoryBean

SingletonBeanRegistry (standalone)
ObjectProvider<T> (standalone)
BeanScope (standalone)
TypeConverter (standalone)
```

---

## 附录 E：选型与硬约束

**Cargo.toml 依赖**

```toml
serde, serde_json, tokio (rt/sync/time), tokio-util, vernal-core, vernal-expression
```

**约定文档声明的选型**：`dashmap` 6.1.0、`inventory` 0.3、`linkme` 0.3.37、`async-trait` 0.1

**硬约束**

- `#![forbid(unsafe_code)]` — 零 unsafe
- 所有 trait 必须 dyn-compatible（不使用泛型方法）
- edition 2024 / rustc 1.88
- `publish = false`

---

> **文档结束** — vernal-beans 对标 spring-beans 的技术要求与实现差距分析。  
> 所有 trait 定义均来自 `crates/vernal-beans/src/` 源码。  
> 选型约束参见 [Spring 组件替换约定](../Spring-组件替换约定.md)。
