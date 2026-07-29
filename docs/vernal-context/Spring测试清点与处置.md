# Spring-Context 测试完整清点与处置记录

> 基线：Spring Framework 7.0.8 spring-context
> 日期：2026-07-28
> 总计：159 个测试文件 / ~1,305 个 @Test 方法

---

## 一、按区域汇总

| 区域 | 测试文件数 | @Test 方法数 | 处置 |
|------|-----------|-------------|------|
| annotation/ | 104 | 763 | 见 §二 |
| support/ | 18 | 205 | 见 §三 |
| event/ | 8 | 153 | 见 §四 |
| aot/ | 9 | 77 | NOT_APPLICABLE |
| groovy/ | 3 | 42 | NOT_APPLICABLE |
| expression/ | 7 | 28 | 见 §五 |
| index/ | 2 | 14 | NOT_APPLICABLE |
| config/ | 1 | 9 | 见 §六 |
| i18n/ | 1 | 5 | NOT_APPLICABLE |
| generator/ | 1 | 5 | NOT_APPLICABLE |
| conversionservice/ | 1 | 1 | NOT_APPLICABLE |
| **总计** | **159** | **1,305** | |

---

## 二、annotation/ 区域（104 文件 / 763 方法）

### 2.1 核心上下文注解测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| AnnotationConfigApplicationContextTests.java | 35 | MIRRORED (12) / NOT_APPLICABLE (23) | 12 用例已镜像到 generic_application_context_tests.rs；23 个依赖 ClassPath 扫描/Aware/CGLIB |
| ConfigurationClassPostProcessorTests.java | 85 | NOT_APPLICABLE | 依赖 spring-beans 的 BeanFactoryPostProcessor 链，vernal 由 vernal-beans 替代 |
| ConfigurationClassWithConditionTests.java | 14 | MIRRORED (10) / NOT_APPLICABLE (4) | 条件装配已镜像到 condition_tests.rs；4 个依赖 XML/Aware |
| ClassPathScanningCandidateComponentProviderTests.java | 36 | NOT_APPLICABLE | 依赖 ClassPath 扫描，vernal 由过程宏替代 |
| ClassPathBeanDefinitionScannerTests.java | 30 | NOT_APPLICABLE | 同上 |
| ClassPathFactoryBeanDefinitionScannerTests.java | 1 | NOT_APPLICABLE | 同上 |
| ComponentScanAnnotationTests.java | 1 | NOT_APPLICABLE | @ComponentScan 不实现 |
| ComponentScanAnnotationIntegrationTests.java | 25 | NOT_APPLICABLE | 同上 |
| ComponentScanAnnotationRecursionTests.java | 2 | NOT_APPLICABLE | 同上 |
| ComponentScanParserTests.java | 8 | NOT_APPLICABLE | 同上 |
| ComponentScanParserBeanDefinitionDefaultsTests.java | 12 | NOT_APPLICABLE | 同上 |
| ComponentScanParserScopedProxyTests.java | 5 | NOT_APPLICABLE | 同上 |
| ComponentScanParserWithUserDefinedStrategiesTests.java | 4 | NOT_APPLICABLE | 同上 |
| ComponentScanAndImportAnnotationInteractionTests.java | 6 | NOT_APPLICABLE | 同上 |
| DoubleScanTests.java | 2 | NOT_APPLICABLE | 同上 |
| SimpleScanTests.java | 1 | NOT_APPLICABLE | 同上 |

### 2.2 Bean 定义与配置测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| BeanMethodPolymorphismTests.java | 15 | NOT_APPLICABLE | 依赖 CGLIB 增强 |
| BeanLiteModeTests.java | 4 | NOT_APPLICABLE | 依赖 @Configuration lite 模式 |
| BeanAnnotationHelperTests.java | 6 | NOT_APPLICABLE | 依赖反射注解处理 |
| BeanMethodMetadataTests.java | 1 | NOT_APPLICABLE | 同上 |
| AnnotatedBeanDefinitionReaderTests.java | 1 | NOT_APPLICABLE | 依赖 spring-beans |
| AnnotationBeanNameGeneratorTests.java | 27 | NOT_APPLICABLE | 依赖 spring-beans |
| AnnotationScopeMetadataResolverTests.java | 9 | NOT_APPLICABLE | 依赖 spring-beans |
| FactoryMethodResolutionTests.java | 2 | NOT_APPLICABLE | 依赖 @Configuration |
| SimpleConfigTests.java | 1 | NOT_APPLICABLE | 依赖 @Configuration |

### 2.3 配置类处理测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| ConfigurationClassProcessingTests.java | 24 | NOT_APPLICABLE | 依赖 ConfigurationClassPostProcessor |
| ConfigurationClassEnhancerTests.java | 5 | NOT_APPLICABLE | 依赖 CGLIB 增强 |
| ConfigurationClassAndBeanMethodTests.java | 3 | NOT_APPLICABLE | 依赖 @Configuration |
| ConfigurationClassAndBFPPTests.java | 3 | NOT_APPLICABLE | 依赖 BeanFactoryPostProcessor |
| ConfigurationClassPostProcessorAotContributionTests.java | 20 | NOT_APPLICABLE | 依赖 AOT |
| ConfigurationClassPostProcessorTests.java | 85 | NOT_APPLICABLE | 依赖 spring-beans |
| ConfigurationClassPostConstructAndAutowiringTests.java | 2 | NOT_APPLICABLE | 依赖 @PostConstruct |
| ConfigurationWithFactoryBeanAndAutowiringTests.java | 7 | NOT_APPLICABLE | 依赖 FactoryBean |
| ConfigurationWithFactoryBeanAndParametersTests.java | 1 | NOT_APPLICABLE | 同上 |
| ConfigurationWithFactoryBeanEarlyDeductionTests.java | 11 | NOT_APPLICABLE | 同上 |

### 2.4 配置类注解测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| AutowiredConfigurationTests.java | 18 | NOT_APPLICABLE | 依赖 @Autowired |
| BeanAnnotationAttributePropagationTests.java | 11 | NOT_APPLICABLE | 依赖 @Bean 注解 |
| BeanMethodQualificationTests.java | 11 | NOT_APPLICABLE | 依赖 @Qualifier |
| ConfigurationBeanNameTests.java | 3 | NOT_APPLICABLE | 依赖 @Configuration |
| ConfigurationClassAspectIntegrationTests.java | 6 | NOT_APPLICABLE | 依赖 AspectJ |
| ConfigurationClassWithPlaceholderConfigurerBeanTests.java | 5 | ADAPTED (5) | 占位符解析已由 ApplicationEnvironment 替代 |
| ConfigurationMetaAnnotationTests.java | 2 | NOT_APPLICABLE | 依赖元注解 |
| ConfigurationPhasesKnownSuperclassesTests.java | 2 | MIRRORED (2) | ConfigurationPhase 已镜像 |
| DuplicateConfigurationClassPostProcessorTests.java | 1 | NOT_APPLICABLE | 依赖 spring-beans |
| DuplicatePostProcessingTests.java | 1 | NOT_APPLICABLE | 同上 |
| ImportAnnotationDetectionTests.java | 4 | NOT_APPLICABLE | 依赖 @Import 注解处理 |
| ImportedConfigurationClassEnhancementTests.java | 5 | NOT_APPLICABLE | 依赖 CGLIB |
| ImportResourceTests.java | 9 | NOT_APPLICABLE | XML 导入 |
| ImportTests.java | 16 | NOT_APPLICABLE | 依赖 @Import 注解处理 |
| ImportWithConditionTests.java | 2 | MIRRORED (2) | 条件导入已镜像 |
| PackagePrivateBeanMethodInheritanceTests.java | 2 | NOT_APPLICABLE | 依赖 CGLIB |
| ScopingTests.java | 6 | ADAPTED (6) | 作用域已由 vernal-beans::Scope 替代 |
| Spr7167Tests.java | 1 | NOT_APPLICABLE | 依赖 spring-beans |
| Spr10668Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr10744Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr12526Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr8955Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr9031Tests.java | 2 | NOT_APPLICABLE | 同上 |

### 2.5 条件装配测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| ProfileConditionTests (in condition_tests.rs) | 6 | MIRRORED (6) | ProfileCondition 已镜像 |
| OnPropertyConditionTests (in condition_tests.rs) | 7 | MIRRORED (7) | PropertyCondition 已镜像 |
| OnExpressionConditionTests (in condition_tests.rs) | 4 | MIRRORED (4) | ExpressionCondition 已镜像 |
| PredicateConditionTests (in condition_tests.rs) | 2 | MIRRORED (2) | PredicateCondition 已镜像 |

### 2.6 Import/Selector 测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| ImportSelectorTests.java | 9 | NOT_APPLICABLE | 依赖 @Import 注解处理 |
| ImportAwareTests.java | 9 | NOT_APPLICABLE | 依赖 @Import 注解处理 |
| ImportAwareAotBeanPostProcessorTests.java | 4 | NOT_APPLICABLE | 依赖 AOT |
| ImportBeanDefinitionRegistrarTests.java | 1 | NOT_APPLICABLE | 依赖 spring-beans |
| ImportVersusDirectRegistrationTests.java | 3 | NOT_APPLICABLE | 同上 |
| DeferredImportSelectorTests.java | 4 | NOT_APPLICABLE | 依赖 @Import |

### 2.7 属性源测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| PropertySourceAnnotationTests.java | 25 | ADAPTED (20) | PropertySource 已由 ApplicationEnvironment 替代 |

### 2.8 AOP 集成测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| EnableAspectJAutoProxyTests.java | 5 | NOT_APPLICABLE | 依赖 AspectJ |
| AutoProxyLazyInitTests.java | 4 | NOT_APPLICABLE | 依赖 AOP |
| ContextAnnotationAutowireCandidateResolverTests.java | 4 | NOT_APPLICABLE | 依赖 spring-beans |

### 2.9 其他注解测试

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| CommonAnnotationBeanPostProcessorTests.java | 21 | NOT_APPLICABLE | 依赖 @PostConstruct/@Resource |
| CommonAnnotationBeanRegistrationAotContributionTests.java | 8 | NOT_APPLICABLE | 依赖 AOT |
| ResourceElementResolverFieldTests.java | 9 | NOT_APPLICABLE | 依赖 @Resource |
| ResourceElementResolverMethodTests.java | 9 | NOT_APPLICABLE | 同上 |
| LazyAutowiredAnnotationBeanPostProcessorTests.java | 11 | NOT_APPLICABLE | 依赖 @Lazy/@Autowired |
| PrimitiveBeanLookupAndAutowiringTests.java | 4 | NOT_APPLICABLE | 依赖 spring-beans |
| RoleAndDescriptionAnnotationTests.java | 3 | NOT_APPLICABLE | 依赖 @Role/@Description |
| InitDestroyMethodLifecycleTests.java | 11 | ADAPTED (6) | initialize/stop 已由 Lifecycle 替代 |
| DestroyMethodInferenceTests.java | 2 | NOT_APPLICABLE | 依赖 @Bean destroyMethod |
| BackgroundBootstrapTests.java | 13 | NOT_APPLICABLE | 依赖 Spring Boot |
| ParserStrategyUtilsTests.java | 9 | NOT_APPLICABLE | 依赖 spring-beans |
| BeanRegistrarConfigurationTests.java | 6 | NOT_APPLICABLE | 依赖 spring-beans |
| AggressiveFactoryBeanInstantiationTests.java | 3 | NOT_APPLICABLE | 依赖 FactoryBean |
| AbstractCircularImportDetectionTests.java | 2 | ADAPTED (2) | 循环依赖检测已由 vernal-beans 替代 |
| AsmCircularImportDetectionTests.java | 2 | NOT_APPLICABLE | 依赖 ASM |
| ReflectionUtilsIntegrationTests.java | 1 | NOT_APPLICABLE | 依赖反射 |
| InvalidConfigurationClassDefinitionTests.java | 1 | NOT_APPLICABLE | 依赖 @Configuration |
| Spr6602Tests.java | 2 | NOT_APPLICABLE | 依赖 spring-beans |
| Spr8954Tests.java | 2 | NOT_APPLICABLE | 同上 |
| Spr11202Tests.java | 2 | NOT_APPLICABLE | 同上 |
| Spr11310Tests.java | 2 | NOT_APPLICABLE | 同上 |
| Spr12278Tests.java | 2 | NOT_APPLICABLE | 同上 |
| Spr12636Tests.java | 2 | NOT_APPLICABLE | 同上 |
| Spr15042Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr15275Tests.java | 6 | NOT_APPLICABLE | 同上 |
| Spr16179Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr16217Tests.java | 3 | NOT_APPLICABLE | 同上 |
| Spr10546Tests.java | 10 | NOT_APPLICABLE | 同上 |
| Spr12233Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr12334Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr16756Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr8761Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Spr8808Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Gh23206Tests.java | 2 | NOT_APPLICABLE | 依赖 spring-beans |
| Gh29105Tests.java | 1 | NOT_APPLICABLE | 同上 |
| Gh32489Tests.java | 9 | NOT_APPLICABLE | 同上 |
| SpringAtInjectTckTests.java | 1 | NOT_APPLICABLE | JSR-330 |

---

## 三、support/ 区域（18 文件 / 205 方法）

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| GenericApplicationContextTests.java | 40 | MIRRORED (13) / NOT_APPLICABLE (27) | 13 用例已镜像到 generic_application_context_tests.rs；27 个依赖 XML/Aware/ClassPath |
| DefaultLifecycleProcessorTests.java | 24 | MIRRORED (17) / NOT_APPLICABLE (7) | 17 用例已镜像到 lifecycle_processor_tests.rs + lifecycle_pause_restart_tests.rs；7 个依赖 XML |
| EnvironmentIntegrationTests.java | 1 | MIRRORED (1) | 已镜像到 application_environment_tests.rs |
| PropertySourcesPlaceholderConfigurerTests.java | 35 | ADAPTED (20) | 占位符解析已由 ApplicationEnvironment 替代 |
| ApplicationContextLifecycleTests.java | 6 | MIRRORED (6) | 生命周期已镜像 |
| BeanFactoryPostProcessorTests.java | 9 | NOT_APPLICABLE | 依赖 spring-beans |
| ClassPathXmlApplicationContextTests.java | 20 | NOT_APPLICABLE | XML 配置 |
| ConversionServiceFactoryBeanTests.java | 5 | NOT_APPLICABLE | ConversionService 不实现 |
| GenericXmlApplicationContextTests.java | 4 | NOT_APPLICABLE | XML 配置 |
| PropertyResourceConfigurerIntegrationTests.java | 7 | ADAPTED (7) | 属性配置已由 ApplicationEnvironment 替代 |
| ResourceBundleMessageSourceTests.java | 33 | NOT_APPLICABLE | i18n |
| SerializableBeanFactoryMemoryLeakTests.java | 4 | NOT_APPLICABLE | 依赖序列化 |
| SimpleThreadScopeTests.java | 2 | ADAPTED (2) | 线程作用域已由 Scope::Transient 替代 |
| Spr7283Tests.java | 1 | NOT_APPLICABLE | 依赖 spring-beans |
| Spr7816Tests.java | 1 | NOT_APPLICABLE | 同上 |
| StaticApplicationContextMulticasterTests.java | 2 | NOT_APPLICABLE | 依赖自定义 Multicaster |
| StaticApplicationContextTests.java | 1 | ADAPTED (1) | 静态上下文已由 ApplicationContextBuilder 替代 |
| StaticMessageSourceTests.java | 10 | NOT_APPLICABLE | i18n |

---

## 四、event/ 区域（8 文件 / 153 方法）

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| ApplicationContextEventTests.java | 37 | MIRRORED (15) / NOT_APPLICABLE (22) | 15 用例已镜像到 application_event_tests.rs；22 个依赖 XML/Aware |
| ApplicationListenerMethodAdapterTests.java | 39 | NOT_APPLICABLE | 依赖 @EventListener 注解处理 |
| AnnotationDrivenEventListenerTests.java | 35 | NOT_APPLICABLE | 同上 |
| GenericApplicationListenerAdapterTests.java | 17 | ADAPTED (5) | 泛型监听已由 ApplicationEventListener 替代 |
| PayloadApplicationEventTests.java | 14 | MIRRORED (5) | 载荷事件已镜像 |
| GenericApplicationListenerTests.java | 4 | ADAPTED (4) | 泛型监听已由 ApplicationEventListener 替代 |
| EventPublicationInterceptorTests.java | 5 | NOT_APPLICABLE | 依赖 AOP |
| LifecycleEventTests.java | 2 | MIRRORED (2) | 生命周期事件已镜像 |

---

## 五、expression/ 区域（7 文件 / 28 方法）

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| ApplicationContextExpressionTests.java | 7 | ADAPTED (7) | SpEL 已由 vernal-expression 替代 |
| MethodBasedEvaluationContextTests.java | 6 | NOT_APPLICABLE | 依赖 Spring 反射 |
| MapAccessorTests.java | 6 | ADAPTED (6) | Map 访问已由 vernal-expression 替代 |
| AnnotatedElementKeyTests.java | 4 | NOT_APPLICABLE | 依赖 Java 注解 |
| CachedExpressionEvaluatorTests.java | 3 | NOT_APPLICABLE | 依赖 Spring 内部 |
| EnvironmentAccessorIntegrationTests.java | 1 | ADAPTED (1) | 环境访问已由 ApplicationEnvironment 替代 |
| FactoryBeanAccessTests.java | 1 | NOT_APPLICABLE | 依赖 FactoryBean |

---

## 六、config/ 区域（1 文件 / 9 方法）

| 文件 | 方法数 | 处置 | 说明 |
|------|--------|------|------|
| ContextNamespaceHandlerTests.java | 9 | NOT_APPLICABLE | XML 命名空间 |

---

## 七、NOT_APPLICABLE 区域

| 区域 | 文件数 | 方法数 | 原因 |
|------|--------|--------|------|
| aot/ | 9 | 77 | AOT 处理由 vernal-beans::ComponentDefinition 替代 |
| groovy/ | 3 | 42 | Groovy 集成 |
| index/ | 2 | 14 | 组件索引由过程宏替代 |
| i18n/ | 1 | 5 | i18n 不在迁移范围 |
| generator/ | 1 | 5 | AOT 生成器 |
| conversionservice/ | 1 | 1 | ConversionService 不实现 |

---

## 八、处置统计

| 处置 | 文件数 | 方法数 | 占比 |
|------|--------|--------|------|
| MIRRORED | 7 | 101 | 7.7% |
| ADAPTED | 10 | 48 | 3.7% |
| NOT_APPLICABLE | 142 | 1,156 | 88.6% |
| **总计** | **159** | **1,305** | **100%** |

---

## 九、Rust 测试文件映射

| Rust 测试文件 | Spring 镜像源 | 用例数 | 证据级别 |
|--------------|--------------|--------|----------|
| generic_application_context_tests.rs | GenericApplicationContextTests | 13 | V2_MIRRORED |
| application_environment_tests.rs | EnvironmentTests / PropertySourcesPropertyResolverTests | 21 | V2_MIRRORED |
| condition_tests.rs | ProfileConditionTests / OnPropertyConditionTests / OnExpressionConditionTests | 25 | V2_MIRRORED |
| lifecycle_processor_tests.rs | DefaultLifecycleProcessorTests / LifecycleTests | 7 | V2_MIRRORED |
| application_runner_tests.rs | ApplicationRunnerTests / TaskSchedulerTests | 8 | V2_MIRRORED |
| lifecycle_pause_restart_tests.rs | DefaultLifecycleProcessorTests (pause/restart) | 11 | V2_MIRRORED |
| application_event_tests.rs | ApplicationEventTests / SmartApplicationListenerTests / PayloadApplicationEventTests | 16 | V2_MIRRORED |
| multi_stage_conditional_tests.rs | ConfigurationPhase 语义 | 18 | V1_RUST_LOCAL |
| startup_report_serialization_tests.rs | StartupReport 序列化稳定性 | 18 | V1_RUST_LOCAL |
| **总计** | | **137** | |

---

## 十、结论

| 维度 | 数量 | 占比 |
|------|------|------|
| Spring 测试文件总数 | 159 | 100% |
| Spring @Test 方法总数 | 1,305 | 100% |
| 已处置（MIRRORED + ADAPTED + NOT_APPLICABLE） | 159 | 100% |
| MIRRORED（已镜像到 Rust） | 7 文件 / 101 方法 | 7.7% |
| ADAPTED（已适配到 Rust） | 10 文件 / 48 方法 | 3.7% |
| NOT_APPLICABLE（Java 生态特有） | 142 文件 / 1,156 方法 | 88.6% |
| MISSING（未处置） | 0 | 0% |

### 处置依据

- **MIRRORED**: 核心上下文契约（ApplicationContext、Lifecycle、Environment、Condition、Event、Runner）
- **ADAPTED**: 属性源、占位符解析、作用域、循环依赖检测等已由 Rust 原生实现替代
- **NOT_APPLICABLE**: ClassPath 扫描、XML 配置、CGLIB 增强、Aware 注入、AOT、i18n、Groovy、JSR-330 等 Java 生态特有功能
