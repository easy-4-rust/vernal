<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-aop 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 208 个 class/interface/enum/record；`package-info.java` 不计入 |
| 目录算法 | 去掉组织和模块根包，保留末 2 层包目录 |
| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |
| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |
| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |
| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |
| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |

本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。
<!-- current-migration-contract-end -->

## 汇总

| 指标 | 数量 |
|---|---:|
| Java 业务对象 | 208 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 5 |
| `MISSING` | 165 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 1 |
| `UNVERIFIED` | 37 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `target_source.rs`：`TargetSource`、`SingletonTargetSource`、`LazyTargetSource`、`TargetClassAware`
- 单文件多个公开对象位于 `method_matcher.rs`：`MethodMatcher`、`TrueMethodMatcher`、`StaticMethodMatcher`、`DynamicMethodMatcher`、`MethodMatcherFactory`
- 单文件多个公开对象位于 `class_filter.rs`：`ClassFilter`、`TrueClassFilter`、`FnClassFilter`、`ClassFilterFactory`
- 单文件多个公开对象位于 `local_invocation_result.rs`：`LocalInvocationValue`、`LocalInvocationResult`、`LocalInvocationFuture`、`LocalInvocationTarget`
- 单文件多个公开对象位于 `proxy_method_invocation.rs`：`ProxyMethodInvocation`、`SimpleProxyMethodInvocation`、`IntroductionAwareMethodMatcher`、`AspectJPrecedenceInformation`
- 单文件多个公开对象位于 `raw_target_access.rs`：`RawTargetAccess`、`Refreshable`、`PoolingConfig`、`ScopedObject`、`SpringProxy`、`ThreadLocalTargetSourceStats`、`AsyncUncaughtExceptionHandler`、`AopInfrastructureBean`、`AdvisedSupportListener`、`InstantiationModelAwarePointcutAdvisor`、`MetadataAwarePointcutAdvisor`
- 单文件多个公开对象位于 `invocation_result.rs`：`InvocationValue`、`InvocationResult`、`InvocationFuture`、`InvocationTarget`
- 单文件多个公开对象位于 `aspect_rs_adapter.rs`：`AspectRsAdapter`、`AroundAdapter`
- 单文件多个公开对象位于 `spring_proxy.rs`：`SpringProxy`、`DefaultSpringProxy`
- 单文件多个公开对象位于 `framework/advisor_chain_factory.rs`：`AdvisorChainFactory`、`DefaultAdvisorChainFactory`、`Advised`
- 单文件多个公开对象位于 `framework/aop_infrastructure_bean.rs`：`AopInfrastructureBean`、`DefaultAopInfrastructureBean`
- 单文件多个公开对象位于 `framework/aop_proxy.rs`：`AopProxy`、`AopProxyError`、`AopProxyFactory`、`FnAopProxy`
- 单文件多个公开对象位于 `framework/advised_support_listener.rs`：`AdvisedSupportListener`、`FnAdvisedSupportListener`
- 单文件多个公开对象位于 `intercept/joinpoint.rs`：`Joinpoint`、`MethodInvocation`、`InvocationChain`
- 单文件多个公开对象位于 `intercept/constructor_interceptor.rs`：`ConstructorInvocation`、`ConstructorInvocationError`、`ConstructorInterceptor`、`FnConstructorInterceptor`
- 单文件多个公开对象位于 `aspectj/aspect_instance_factory.rs`：`AspectInstanceFactory`、`AspectInstanceError`、`SingletonAspectInstanceFactory`、`LazyAspectInstanceFactory`、`MetadataAwareAspectInstanceFactory`
- 单文件多个公开对象位于 `aop/dynamic_introduction_advice.rs`：`DynamicIntroductionAdvice`、`IntroductionAdvice`、`IntroductionInterceptor`
- 单文件多个公开对象位于 `aop/after_advice.rs`：`AfterAdvice`、`AfterReturningAdvice`、`ThrowsAdvice`
- 单文件多个公开对象位于 `aop/advice.rs`：`Advice`、`AspectException`
- 单文件多个公开对象位于 `aop/before_advice.rs`：`BeforeAdvice`、`MethodBeforeAdvice`
- 单文件多个公开对象位于 `support/expression_pointcut.rs`：`ExpressionPointcut`、`StringExpressionPointcut`
- 单文件多个公开对象位于 `pointcut/dsl/pattern.rs`：`Visibility`、`NamePattern`、`ExecutionPattern`、`ModulePattern`
- 单文件多个公开对象位于 `pointcut/dsl/matcher.rs`：`FunctionDescriptor`、`PointcutMatcher`、`TagPattern`、`QualifierPattern`
- 单文件多个公开对象位于 `framework/autoproxy/target_source_creator.rs`：`TargetSourceCreator`、`FnTargetSourceCreator`
- 单文件多个公开对象位于 `framework/adapter/advisor_adapter.rs`：`AdvisorAdapter`、`AdvisorAdapterRegistry`、`DefaultAdvisorAdapterRegistry`

## 依赖复用边界

| Crate | 固定提交 | Cargo 证据 | 精确符号 |
|---|---|---|---|
| `aspect-core` | `89beaa9057b3f2b73093fc31d3219420e0bb6182` | `crates/vernal-aop/Cargo.toml` | aspect_core::{Aspect, AspectError, JoinPoint, Location, ProceedingJoinPoint}; aspect_core::pointcut::{Pointcut, FunctionInfo, Matcher, ExecutionPattern, ModulePattern, NamePattern, Visibility, parse_pointcut} |
| `aspect-std` | `89beaa9057b3f2b73093fc31d3219420e0bb6182` | `crates/vernal-aop/Cargo.toml` | aspect_std::{LoggingAspect, TimingAspect, CachingAspect, MetricsAspect, RateLimitAspect, CircuitBreakerAspect, AuthorizationAspect, AllowlistAspect, ValidationAspect} |

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.aopalliance.aop.Advice` | `org/aopalliance/aop/Advice.java` | `aop/advice.rs` | `aop/advice.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.aopalliance.aop.AspectException` | `org/aopalliance/aop/AspectException.java` | `aop/aspect_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.aopalliance.intercept.ConstructorInterceptor` | `org/aopalliance/intercept/ConstructorInterceptor.java` | `intercept/constructor_interceptor.rs` | `intercept/constructor_interceptor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.aopalliance.intercept.ConstructorInvocation` | `org/aopalliance/intercept/ConstructorInvocation.java` | `intercept/constructor_invocation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.aopalliance.intercept.Interceptor` | `org/aopalliance/intercept/Interceptor.java` | `intercept/interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.aopalliance.intercept.Invocation` | `org/aopalliance/intercept/Invocation.java` | `intercept/invocation.rs` | `intercept/invocation.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.aopalliance.intercept.Joinpoint` | `org/aopalliance/intercept/Joinpoint.java` | `intercept/joinpoint.rs` | `intercept/joinpoint.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.aopalliance.intercept.MethodInterceptor` | `org/aopalliance/intercept/MethodInterceptor.java` | `intercept/method_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.aopalliance.intercept.MethodInvocation` | `org/aopalliance/intercept/MethodInvocation.java` | `intercept/method_invocation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.Advisor` | `org/springframework/aop/Advisor.java` | `advisor.rs` | `advisor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.AfterAdvice` | `org/springframework/aop/AfterAdvice.java` | `after_advice.rs` | `aop/after_advice.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.aop.AfterReturningAdvice` | `org/springframework/aop/AfterReturningAdvice.java` | `after_returning_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.AopInvocationException` | `org/springframework/aop/AopInvocationException.java` | `aop_invocation_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.BeforeAdvice` | `org/springframework/aop/BeforeAdvice.java` | `before_advice.rs` | `aop/before_advice.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.aop.ClassFilter` | `org/springframework/aop/ClassFilter.java` | `class_filter.rs` | `class_filter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.DynamicIntroductionAdvice` | `org/springframework/aop/DynamicIntroductionAdvice.java` | `dynamic_introduction_advice.rs` | `aop/dynamic_introduction_advice.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.aop.IntroductionAdvisor` | `org/springframework/aop/IntroductionAdvisor.java` | `introduction_advisor.rs` | `introduction_advisor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.IntroductionAwareMethodMatcher` | `org/springframework/aop/IntroductionAwareMethodMatcher.java` | `introduction_aware_method_matcher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.IntroductionInfo` | `org/springframework/aop/IntroductionInfo.java` | `introduction_info.rs` | `introduction_info.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.IntroductionInterceptor` | `org/springframework/aop/IntroductionInterceptor.java` | `introduction_interceptor.rs` | `aop/introduction_interceptor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.aop.MethodBeforeAdvice` | `org/springframework/aop/MethodBeforeAdvice.java` | `method_before_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.MethodMatcher` | `org/springframework/aop/MethodMatcher.java` | `method_matcher.rs` | `method_matcher.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.Pointcut` | `org/springframework/aop/Pointcut.java` | `pointcut.rs` | `pointcut.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.PointcutAdvisor` | `org/springframework/aop/PointcutAdvisor.java` | `pointcut_advisor.rs` | `pointcut_advisor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.ProxyMethodInvocation` | `org/springframework/aop/ProxyMethodInvocation.java` | `proxy_method_invocation.rs` | `proxy_method_invocation.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.RawTargetAccess` | `org/springframework/aop/RawTargetAccess.java` | `raw_target_access.rs` | `raw_target_access.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.aop.SpringProxy` | `org/springframework/aop/SpringProxy.java` | `spring_proxy.rs` | `spring_proxy.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.TargetClassAware` | `org/springframework/aop/TargetClassAware.java` | `target_class_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.TargetSource` | `org/springframework/aop/TargetSource.java` | `target_source.rs` | `target_source.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.ThrowsAdvice` | `org/springframework/aop/ThrowsAdvice.java` | `throws_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.TrueClassFilter` | `org/springframework/aop/TrueClassFilter.java` | `true_class_filter.rs` | `true_class_filter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.TrueMethodMatcher` | `org/springframework/aop/TrueMethodMatcher.java` | `true_method_matcher.rs` | `true_method_matcher.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.TruePointcut` | `org/springframework/aop/TruePointcut.java` | `true_pointcut.rs` | `true_pointcut.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.aspectj.AbstractAspectJAdvice` | `org/springframework/aop/aspectj/AbstractAspectJAdvice.java` | `aspectj/abstract_aspect_j_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectInstanceFactory` | `org/springframework/aop/aspectj/AspectInstanceFactory.java` | `aspectj/aspect_instance_factory.rs` | `aspectj/aspect_instance_factory.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.aspectj.AspectJAdviceParameterNameDiscoverer` | `org/springframework/aop/aspectj/AspectJAdviceParameterNameDiscoverer.java` | `aspectj/aspect_j_advice_parameter_name_discoverer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJAfterAdvice` | `org/springframework/aop/aspectj/AspectJAfterAdvice.java` | `aspectj/aspect_j_after_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJAfterReturningAdvice` | `org/springframework/aop/aspectj/AspectJAfterReturningAdvice.java` | `aspectj/aspect_j_after_returning_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJAfterThrowingAdvice` | `org/springframework/aop/aspectj/AspectJAfterThrowingAdvice.java` | `aspectj/aspect_j_after_throwing_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJAopUtils` | `org/springframework/aop/aspectj/AspectJAopUtils.java` | `aspectj/aspect_j_aop_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJAroundAdvice` | `org/springframework/aop/aspectj/AspectJAroundAdvice.java` | `aspectj/aspect_j_around_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJExpressionPointcut` | `org/springframework/aop/aspectj/AspectJExpressionPointcut.java` | `aspectj/aspect_j_expression_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJExpressionPointcutAdvisor` | `org/springframework/aop/aspectj/AspectJExpressionPointcutAdvisor.java` | `aspectj/aspect_j_expression_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJMethodBeforeAdvice` | `org/springframework/aop/aspectj/AspectJMethodBeforeAdvice.java` | `aspectj/aspect_j_method_before_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJPointcutAdvisor` | `org/springframework/aop/aspectj/AspectJPointcutAdvisor.java` | `aspectj/aspect_j_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJPrecedenceInformation` | `org/springframework/aop/aspectj/AspectJPrecedenceInformation.java` | `aspectj/aspect_j_precedence_information.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJProxyUtils` | `org/springframework/aop/aspectj/AspectJProxyUtils.java` | `aspectj/aspect_j_proxy_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.AspectJWeaverMessageHandler` | `org/springframework/aop/aspectj/AspectJWeaverMessageHandler.java` | `aspectj/aspect_j_weaver_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.DeclareParentsAdvisor` | `org/springframework/aop/aspectj/DeclareParentsAdvisor.java` | `aspectj/declare_parents_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.InstantiationModelAwarePointcutAdvisor` | `org/springframework/aop/aspectj/InstantiationModelAwarePointcutAdvisor.java` | `aspectj/instantiation_model_aware_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.MethodInvocationProceedingJoinPoint` | `org/springframework/aop/aspectj/MethodInvocationProceedingJoinPoint.java` | `aspectj/method_invocation_proceeding_join_point.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.RuntimeTestWalker` | `org/springframework/aop/aspectj/RuntimeTestWalker.java` | `aspectj/runtime_test_walker.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.ShadowMatchUtils` | `org/springframework/aop/aspectj/ShadowMatchUtils.java` | `aspectj/shadow_match_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.SimpleAspectInstanceFactory` | `org/springframework/aop/aspectj/SimpleAspectInstanceFactory.java` | `aspectj/simple_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.SingletonAspectInstanceFactory` | `org/springframework/aop/aspectj/SingletonAspectInstanceFactory.java` | `aspectj/singleton_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.TypePatternClassFilter` | `org/springframework/aop/aspectj/TypePatternClassFilter.java` | `aspectj/type_pattern_class_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AbstractAspectJAdvisorFactory` | `org/springframework/aop/aspectj/annotation/AbstractAspectJAdvisorFactory.java` | `aspectj/annotation/abstract_aspect_j_advisor_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AnnotationAwareAspectJAutoProxyCreator` | `org/springframework/aop/aspectj/annotation/AnnotationAwareAspectJAutoProxyCreator.java` | `aspectj/annotation/annotation_aware_aspect_j_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AspectJAdvisorBeanRegistrationAotProcessor` | `org/springframework/aop/aspectj/annotation/AspectJAdvisorBeanRegistrationAotProcessor.java` | `aspectj/annotation/aspect_j_advisor_bean_registration_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AspectJAdvisorFactory` | `org/springframework/aop/aspectj/annotation/AspectJAdvisorFactory.java` | `aspectj/annotation/aspect_j_advisor_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AspectJBeanFactoryInitializationAotProcessor` | `org/springframework/aop/aspectj/annotation/AspectJBeanFactoryInitializationAotProcessor.java` | `aspectj/annotation/aspect_j_bean_factory_initialization_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AspectJProxyFactory` | `org/springframework/aop/aspectj/annotation/AspectJProxyFactory.java` | `aspectj/annotation/aspect_j_proxy_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.AspectMetadata` | `org/springframework/aop/aspectj/annotation/AspectMetadata.java` | `aspectj/annotation/aspect_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.BeanFactoryAspectInstanceFactory` | `org/springframework/aop/aspectj/annotation/BeanFactoryAspectInstanceFactory.java` | `aspectj/annotation/bean_factory_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.BeanFactoryAspectJAdvisorsBuilder` | `org/springframework/aop/aspectj/annotation/BeanFactoryAspectJAdvisorsBuilder.java` | `aspectj/annotation/bean_factory_aspect_j_advisors_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.InstantiationModelAwarePointcutAdvisorImpl` | `org/springframework/aop/aspectj/annotation/InstantiationModelAwarePointcutAdvisorImpl.java` | `aspectj/annotation/instantiation_model_aware_pointcut_advisor_impl.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.LazySingletonAspectInstanceFactoryDecorator` | `org/springframework/aop/aspectj/annotation/LazySingletonAspectInstanceFactoryDecorator.java` | `aspectj/annotation/lazy_singleton_aspect_instance_factory_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.MetadataAwareAspectInstanceFactory` | `org/springframework/aop/aspectj/annotation/MetadataAwareAspectInstanceFactory.java` | `aspectj/annotation/metadata_aware_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.NotAnAtAspectException` | `org/springframework/aop/aspectj/annotation/NotAnAtAspectException.java` | `aspectj/annotation/not_an_at_aspect_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.PrototypeAspectInstanceFactory` | `org/springframework/aop/aspectj/annotation/PrototypeAspectInstanceFactory.java` | `aspectj/annotation/prototype_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.ReflectiveAspectJAdvisorFactory` | `org/springframework/aop/aspectj/annotation/ReflectiveAspectJAdvisorFactory.java` | `aspectj/annotation/reflective_aspect_j_advisor_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.SimpleMetadataAwareAspectInstanceFactory` | `org/springframework/aop/aspectj/annotation/SimpleMetadataAwareAspectInstanceFactory.java` | `aspectj/annotation/simple_metadata_aware_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.annotation.SingletonMetadataAwareAspectInstanceFactory` | `org/springframework/aop/aspectj/annotation/SingletonMetadataAwareAspectInstanceFactory.java` | `aspectj/annotation/singleton_metadata_aware_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.autoproxy.AspectJAwareAdvisorAutoProxyCreator` | `org/springframework/aop/aspectj/autoproxy/AspectJAwareAdvisorAutoProxyCreator.java` | `aspectj/autoproxy/aspect_j_aware_advisor_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.aspectj.autoproxy.AspectJPrecedenceComparator` | `org/springframework/aop/aspectj/autoproxy/AspectJPrecedenceComparator.java` | `aspectj/autoproxy/aspect_j_precedence_comparator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AbstractInterceptorDrivenBeanDefinitionDecorator` | `org/springframework/aop/config/AbstractInterceptorDrivenBeanDefinitionDecorator.java` | `config/abstract_interceptor_driven_bean_definition_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AdviceEntry` | `org/springframework/aop/config/AdviceEntry.java` | `config/advice_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AdvisorComponentDefinition` | `org/springframework/aop/config/AdvisorComponentDefinition.java` | `config/advisor_component_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AdvisorEntry` | `org/springframework/aop/config/AdvisorEntry.java` | `config/advisor_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AopConfigUtils` | `org/springframework/aop/config/AopConfigUtils.java` | `config/aop_config_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AopNamespaceHandler` | `org/springframework/aop/config/AopNamespaceHandler.java` | `config/aop_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AopNamespaceUtils` | `org/springframework/aop/config/AopNamespaceUtils.java` | `config/aop_namespace_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AspectComponentDefinition` | `org/springframework/aop/config/AspectComponentDefinition.java` | `config/aspect_component_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AspectEntry` | `org/springframework/aop/config/AspectEntry.java` | `config/aspect_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.AspectJAutoProxyBeanDefinitionParser` | `org/springframework/aop/config/AspectJAutoProxyBeanDefinitionParser.java` | `config/aspect_j_auto_proxy_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.ConfigBeanDefinitionParser` | `org/springframework/aop/config/ConfigBeanDefinitionParser.java` | `config/config_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.MethodLocatingFactoryBean` | `org/springframework/aop/config/MethodLocatingFactoryBean.java` | `config/method_locating_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.PointcutComponentDefinition` | `org/springframework/aop/config/PointcutComponentDefinition.java` | `config/pointcut_component_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.PointcutEntry` | `org/springframework/aop/config/PointcutEntry.java` | `config/pointcut_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.ScopedProxyBeanDefinitionDecorator` | `org/springframework/aop/config/ScopedProxyBeanDefinitionDecorator.java` | `config/scoped_proxy_bean_definition_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.SimpleBeanFactoryAwareAspectInstanceFactory` | `org/springframework/aop/config/SimpleBeanFactoryAwareAspectInstanceFactory.java` | `config/simple_bean_factory_aware_aspect_instance_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.config.SpringConfiguredBeanDefinitionParser` | `org/springframework/aop/config/SpringConfiguredBeanDefinitionParser.java` | `config/spring_configured_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.AbstractAdvisingBeanPostProcessor` | `org/springframework/aop/framework/AbstractAdvisingBeanPostProcessor.java` | `framework/abstract_advising_bean_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.AbstractSingletonProxyFactoryBean` | `org/springframework/aop/framework/AbstractSingletonProxyFactoryBean.java` | `framework/abstract_singleton_proxy_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.Advised` | `org/springframework/aop/framework/Advised.java` | `framework/advised.rs` | `framework/advised.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AdvisedSupport` | `org/springframework/aop/framework/AdvisedSupport.java` | `framework/advised_support.rs` | `framework/advised_support.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AdvisedSupportListener` | `org/springframework/aop/framework/AdvisedSupportListener.java` | `framework/advised_support_listener.rs` | `framework/advised_support_listener.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AdvisorChainFactory` | `org/springframework/aop/framework/AdvisorChainFactory.java` | `framework/advisor_chain_factory.rs` | `framework/advisor_chain_factory.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AopConfigException` | `org/springframework/aop/framework/AopConfigException.java` | `framework/aop_config_exception.rs` | `framework/aop_config_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AopContext` | `org/springframework/aop/framework/AopContext.java` | `framework/aop_context.rs` | `framework/aop_context.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AopInfrastructureBean` | `org/springframework/aop/framework/AopInfrastructureBean.java` | `framework/aop_infrastructure_bean.rs` | `framework/aop_infrastructure_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AopProxy` | `org/springframework/aop/framework/AopProxy.java` | `framework/aop_proxy.rs` | `framework/aop_proxy.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.AopProxyFactory` | `org/springframework/aop/framework/AopProxyFactory.java` | `framework/aop_proxy_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.AopProxyUtils` | `org/springframework/aop/framework/AopProxyUtils.java` | `framework/aop_proxy_utils.rs` | `framework/aop_proxy_utils.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.CglibAopProxy` | `org/springframework/aop/framework/CglibAopProxy.java` | `framework/cglib_aop_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.CoroutinesUtils` | `org/springframework/aop/framework/CoroutinesUtils.java` | `framework/coroutines_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.DefaultAdvisorChainFactory` | `org/springframework/aop/framework/DefaultAdvisorChainFactory.java` | `framework/default_advisor_chain_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.DefaultAopProxyFactory` | `org/springframework/aop/framework/DefaultAopProxyFactory.java` | `framework/default_aop_proxy_factory.rs` | `framework/default_aop_proxy_factory.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.InterceptorAndDynamicMethodMatcher` | `org/springframework/aop/framework/InterceptorAndDynamicMethodMatcher.java` | `framework/interceptor_and_dynamic_method_matcher.rs` | `framework/interceptor_and_dynamic_method_matcher.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.JdkDynamicAopProxy` | `org/springframework/aop/framework/JdkDynamicAopProxy.java` | `framework/jdk_dynamic_aop_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.ObjenesisCglibAopProxy` | `org/springframework/aop/framework/ObjenesisCglibAopProxy.java` | `framework/objenesis_cglib_aop_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.ProxyConfig` | `org/springframework/aop/framework/ProxyConfig.java` | `framework/proxy_config.rs` | `framework/proxy_config.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.ProxyCreatorSupport` | `org/springframework/aop/framework/ProxyCreatorSupport.java` | `framework/proxy_creator_support.rs` | `framework/proxy_creator_support.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.ProxyFactory` | `org/springframework/aop/framework/ProxyFactory.java` | `framework/proxy_factory.rs` | `framework/proxy_factory.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.aop.framework.ProxyFactoryBean` | `org/springframework/aop/framework/ProxyFactoryBean.java` | `framework/proxy_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.ProxyProcessorSupport` | `org/springframework/aop/framework/ProxyProcessorSupport.java` | `framework/proxy_processor_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.ReflectiveMethodInvocation` | `org/springframework/aop/framework/ReflectiveMethodInvocation.java` | `framework/reflective_method_invocation.rs` | `framework/reflective_method_invocation.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.adapter.AdvisorAdapter` | `org/springframework/aop/framework/adapter/AdvisorAdapter.java` | `framework/adapter/advisor_adapter.rs` | `framework/adapter/advisor_adapter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.adapter.AdvisorAdapterRegistrationManager` | `org/springframework/aop/framework/adapter/AdvisorAdapterRegistrationManager.java` | `framework/adapter/advisor_adapter_registration_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.AdvisorAdapterRegistry` | `org/springframework/aop/framework/adapter/AdvisorAdapterRegistry.java` | `framework/adapter/advisor_adapter_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.AfterReturningAdviceAdapter` | `org/springframework/aop/framework/adapter/AfterReturningAdviceAdapter.java` | `framework/adapter/after_returning_advice_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.AfterReturningAdviceInterceptor` | `org/springframework/aop/framework/adapter/AfterReturningAdviceInterceptor.java` | `framework/adapter/after_returning_advice_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.DefaultAdvisorAdapterRegistry` | `org/springframework/aop/framework/adapter/DefaultAdvisorAdapterRegistry.java` | `framework/adapter/default_advisor_adapter_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.GlobalAdvisorAdapterRegistry` | `org/springframework/aop/framework/adapter/GlobalAdvisorAdapterRegistry.java` | `framework/adapter/global_advisor_adapter_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.MethodBeforeAdviceAdapter` | `org/springframework/aop/framework/adapter/MethodBeforeAdviceAdapter.java` | `framework/adapter/method_before_advice_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.MethodBeforeAdviceInterceptor` | `org/springframework/aop/framework/adapter/MethodBeforeAdviceInterceptor.java` | `framework/adapter/method_before_advice_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.ThrowsAdviceAdapter` | `org/springframework/aop/framework/adapter/ThrowsAdviceAdapter.java` | `framework/adapter/throws_advice_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.ThrowsAdviceInterceptor` | `org/springframework/aop/framework/adapter/ThrowsAdviceInterceptor.java` | `framework/adapter/throws_advice_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.adapter.UnknownAdviceTypeException` | `org/springframework/aop/framework/adapter/UnknownAdviceTypeException.java` | `framework/adapter/unknown_advice_type_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.AbstractAdvisorAutoProxyCreator` | `org/springframework/aop/framework/autoproxy/AbstractAdvisorAutoProxyCreator.java` | `framework/autoproxy/abstract_advisor_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.AbstractAutoProxyCreator` | `org/springframework/aop/framework/autoproxy/AbstractAutoProxyCreator.java` | `framework/autoproxy/abstract_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.AbstractBeanFactoryAwareAdvisingPostProcessor` | `org/springframework/aop/framework/autoproxy/AbstractBeanFactoryAwareAdvisingPostProcessor.java` | `framework/autoproxy/abstract_bean_factory_aware_advising_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.AutoProxyUtils` | `org/springframework/aop/framework/autoproxy/AutoProxyUtils.java` | `framework/autoproxy/auto_proxy_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.BeanFactoryAdvisorRetrievalHelper` | `org/springframework/aop/framework/autoproxy/BeanFactoryAdvisorRetrievalHelper.java` | `framework/autoproxy/bean_factory_advisor_retrieval_helper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.BeanNameAutoProxyCreator` | `org/springframework/aop/framework/autoproxy/BeanNameAutoProxyCreator.java` | `framework/autoproxy/bean_name_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.DefaultAdvisorAutoProxyCreator` | `org/springframework/aop/framework/autoproxy/DefaultAdvisorAutoProxyCreator.java` | `framework/autoproxy/default_advisor_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.InfrastructureAdvisorAutoProxyCreator` | `org/springframework/aop/framework/autoproxy/InfrastructureAdvisorAutoProxyCreator.java` | `framework/autoproxy/infrastructure_advisor_auto_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.ProxyCreationContext` | `org/springframework/aop/framework/autoproxy/ProxyCreationContext.java` | `framework/autoproxy/proxy_creation_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.TargetSourceCreator` | `org/springframework/aop/framework/autoproxy/TargetSourceCreator.java` | `framework/autoproxy/target_source_creator.rs` | `framework/autoproxy/target_source_creator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.framework.autoproxy.target.AbstractBeanFactoryBasedTargetSourceCreator` | `org/springframework/aop/framework/autoproxy/target/AbstractBeanFactoryBasedTargetSourceCreator.java` | `autoproxy/target/abstract_bean_factory_based_target_source_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.target.LazyInitTargetSourceCreator` | `org/springframework/aop/framework/autoproxy/target/LazyInitTargetSourceCreator.java` | `autoproxy/target/lazy_init_target_source_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.framework.autoproxy.target.QuickTargetSourceCreator` | `org/springframework/aop/framework/autoproxy/target/QuickTargetSourceCreator.java` | `autoproxy/target/quick_target_source_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.AbstractMonitoringInterceptor` | `org/springframework/aop/interceptor/AbstractMonitoringInterceptor.java` | `interceptor/abstract_monitoring_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.AbstractTraceInterceptor` | `org/springframework/aop/interceptor/AbstractTraceInterceptor.java` | `interceptor/abstract_trace_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.AsyncExecutionAspectSupport` | `org/springframework/aop/interceptor/AsyncExecutionAspectSupport.java` | `interceptor/async_execution_aspect_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.AsyncExecutionInterceptor` | `org/springframework/aop/interceptor/AsyncExecutionInterceptor.java` | `interceptor/async_execution_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.AsyncUncaughtExceptionHandler` | `org/springframework/aop/interceptor/AsyncUncaughtExceptionHandler.java` | `interceptor/async_uncaught_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.ConcurrencyThrottleInterceptor` | `org/springframework/aop/interceptor/ConcurrencyThrottleInterceptor.java` | `interceptor/concurrency_throttle_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.CustomizableTraceInterceptor` | `org/springframework/aop/interceptor/CustomizableTraceInterceptor.java` | `interceptor/customizable_trace_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.DebugInterceptor` | `org/springframework/aop/interceptor/DebugInterceptor.java` | `interceptor/debug_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.ExposeBeanNameAdvisors` | `org/springframework/aop/interceptor/ExposeBeanNameAdvisors.java` | `interceptor/expose_bean_name_advisors.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.ExposeInvocationInterceptor` | `org/springframework/aop/interceptor/ExposeInvocationInterceptor.java` | `interceptor/expose_invocation_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.PerformanceMonitorInterceptor` | `org/springframework/aop/interceptor/PerformanceMonitorInterceptor.java` | `interceptor/performance_monitor_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.SimpleAsyncUncaughtExceptionHandler` | `org/springframework/aop/interceptor/SimpleAsyncUncaughtExceptionHandler.java` | `interceptor/simple_async_uncaught_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.interceptor.SimpleTraceInterceptor` | `org/springframework/aop/interceptor/SimpleTraceInterceptor.java` | `interceptor/simple_trace_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.scope.DefaultScopedObject` | `org/springframework/aop/scope/DefaultScopedObject.java` | `scope/default_scoped_object.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.scope.ScopedObject` | `org/springframework/aop/scope/ScopedObject.java` | `scope/scoped_object.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.scope.ScopedProxyBeanRegistrationAotProcessor` | `org/springframework/aop/scope/ScopedProxyBeanRegistrationAotProcessor.java` | `scope/scoped_proxy_bean_registration_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.scope.ScopedProxyFactoryBean` | `org/springframework/aop/scope/ScopedProxyFactoryBean.java` | `scope/scoped_proxy_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.scope.ScopedProxyUtils` | `org/springframework/aop/scope/ScopedProxyUtils.java` | `scope/scoped_proxy_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.AbstractBeanFactoryPointcutAdvisor` | `org/springframework/aop/support/AbstractBeanFactoryPointcutAdvisor.java` | `support/abstract_bean_factory_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.AbstractExpressionPointcut` | `org/springframework/aop/support/AbstractExpressionPointcut.java` | `support/abstract_expression_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.AbstractGenericPointcutAdvisor` | `org/springframework/aop/support/AbstractGenericPointcutAdvisor.java` | `support/abstract_generic_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.AbstractPointcutAdvisor` | `org/springframework/aop/support/AbstractPointcutAdvisor.java` | `support/abstract_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.AbstractRegexpMethodPointcut` | `org/springframework/aop/support/AbstractRegexpMethodPointcut.java` | `support/abstract_regexp_method_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.AopUtils` | `org/springframework/aop/support/AopUtils.java` | `support/aop_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.ClassFilters` | `org/springframework/aop/support/ClassFilters.java` | `support/class_filters.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.ComposablePointcut` | `org/springframework/aop/support/ComposablePointcut.java` | `support/composable_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.ControlFlowPointcut` | `org/springframework/aop/support/ControlFlowPointcut.java` | `support/control_flow_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.DefaultBeanFactoryPointcutAdvisor` | `org/springframework/aop/support/DefaultBeanFactoryPointcutAdvisor.java` | `support/default_bean_factory_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.DefaultIntroductionAdvisor` | `org/springframework/aop/support/DefaultIntroductionAdvisor.java` | `support/default_introduction_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.DefaultPointcutAdvisor` | `org/springframework/aop/support/DefaultPointcutAdvisor.java` | `support/default_pointcut_advisor.rs` | `support/default_pointcut_advisor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.support.DelegatePerTargetObjectIntroductionInterceptor` | `org/springframework/aop/support/DelegatePerTargetObjectIntroductionInterceptor.java` | `support/delegate_per_target_object_introduction_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.DelegatingIntroductionInterceptor` | `org/springframework/aop/support/DelegatingIntroductionInterceptor.java` | `support/delegating_introduction_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.DynamicMethodMatcher` | `org/springframework/aop/support/DynamicMethodMatcher.java` | `support/dynamic_method_matcher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.DynamicMethodMatcherPointcut` | `org/springframework/aop/support/DynamicMethodMatcherPointcut.java` | `support/dynamic_method_matcher_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.ExpressionPointcut` | `org/springframework/aop/support/ExpressionPointcut.java` | `support/expression_pointcut.rs` | `support/expression_pointcut.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.aop.support.IntroductionInfoSupport` | `org/springframework/aop/support/IntroductionInfoSupport.java` | `support/introduction_info_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.JdkRegexpMethodPointcut` | `org/springframework/aop/support/JdkRegexpMethodPointcut.java` | `support/jdk_regexp_method_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.MethodMatchers` | `org/springframework/aop/support/MethodMatchers.java` | `support/method_matchers.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.NameMatchMethodPointcut` | `org/springframework/aop/support/NameMatchMethodPointcut.java` | `support/name_match_method_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.NameMatchMethodPointcutAdvisor` | `org/springframework/aop/support/NameMatchMethodPointcutAdvisor.java` | `support/name_match_method_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.Pointcuts` | `org/springframework/aop/support/Pointcuts.java` | `support/pointcuts.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.RegexpMethodPointcutAdvisor` | `org/springframework/aop/support/RegexpMethodPointcutAdvisor.java` | `support/regexp_method_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.RootClassFilter` | `org/springframework/aop/support/RootClassFilter.java` | `support/root_class_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.StaticMethodMatcher` | `org/springframework/aop/support/StaticMethodMatcher.java` | `support/static_method_matcher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.StaticMethodMatcherPointcut` | `org/springframework/aop/support/StaticMethodMatcherPointcut.java` | `support/static_method_matcher_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.StaticMethodMatcherPointcutAdvisor` | `org/springframework/aop/support/StaticMethodMatcherPointcutAdvisor.java` | `support/static_method_matcher_pointcut_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.annotation.AnnotationClassFilter` | `org/springframework/aop/support/annotation/AnnotationClassFilter.java` | `support/annotation/annotation_class_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.annotation.AnnotationMatchingPointcut` | `org/springframework/aop/support/annotation/AnnotationMatchingPointcut.java` | `support/annotation/annotation_matching_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.support.annotation.AnnotationMethodMatcher` | `org/springframework/aop/support/annotation/AnnotationMethodMatcher.java` | `support/annotation/annotation_method_matcher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.AbstractBeanFactoryBasedTargetSource` | `org/springframework/aop/target/AbstractBeanFactoryBasedTargetSource.java` | `target/abstract_bean_factory_based_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.AbstractLazyCreationTargetSource` | `org/springframework/aop/target/AbstractLazyCreationTargetSource.java` | `target/abstract_lazy_creation_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.AbstractPoolingTargetSource` | `org/springframework/aop/target/AbstractPoolingTargetSource.java` | `target/abstract_pooling_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.AbstractPrototypeBasedTargetSource` | `org/springframework/aop/target/AbstractPrototypeBasedTargetSource.java` | `target/abstract_prototype_based_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.CommonsPool2TargetSource` | `org/springframework/aop/target/CommonsPool2TargetSource.java` | `target/commons_pool2_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.EmptyTargetSource` | `org/springframework/aop/target/EmptyTargetSource.java` | `target/empty_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.HotSwappableTargetSource` | `org/springframework/aop/target/HotSwappableTargetSource.java` | `target/hot_swappable_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.LazyInitTargetSource` | `org/springframework/aop/target/LazyInitTargetSource.java` | `target/lazy_init_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.PoolingConfig` | `org/springframework/aop/target/PoolingConfig.java` | `target/pooling_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.PrototypeTargetSource` | `org/springframework/aop/target/PrototypeTargetSource.java` | `target/prototype_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.SimpleBeanTargetSource` | `org/springframework/aop/target/SimpleBeanTargetSource.java` | `target/simple_bean_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.SingletonTargetSource` | `org/springframework/aop/target/SingletonTargetSource.java` | `target/singleton_target_source.rs` | `singleton_target_source.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.aop.target.ThreadLocalTargetSource` | `org/springframework/aop/target/ThreadLocalTargetSource.java` | `target/thread_local_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.ThreadLocalTargetSourceStats` | `org/springframework/aop/target/ThreadLocalTargetSourceStats.java` | `target/thread_local_target_source_stats.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.dynamic.AbstractRefreshableTargetSource` | `org/springframework/aop/target/dynamic/AbstractRefreshableTargetSource.java` | `target/dynamic/abstract_refreshable_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.dynamic.BeanFactoryRefreshableTargetSource` | `org/springframework/aop/target/dynamic/BeanFactoryRefreshableTargetSource.java` | `target/dynamic/bean_factory_refreshable_target_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.aop.target.dynamic.Refreshable` | `org/springframework/aop/target/dynamic/Refreshable.java` | `target/dynamic/refreshable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
