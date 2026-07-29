//! 差分测试 - 多阶段条件装配（对标 Spring 7.0 ConfigurationCondition#getConfigurationPhase）
//!
//! Spring 7.0 引入了 `ConfigurationPhase` 枚举，允许条件在不同阶段评估：
//! - `PARSE_CONFIGURATION`：在解析 @Configuration 类时评估，条件不命中时整个类不加入依赖图
//! - `REGISTER_BEAN`：在注册普通 bean 时评估，条件不阻止 @Configuration 类加入依赖图
//!
//! vernal-context 通过 `ConditionalComponentModule::phase` 字段实现此语义。

use std::sync::Arc;

use vernal_context::{
    ApplicationEnvironment, ApplicationEnvironmentBuilder, ComponentCondition,
    ConditionalComponentModule, ConfigurationPhase,
};

// ════════════════════════════════════════════════════════════════════
// 测试用条件
// ════════════════════════════════════════════════════════════════════

/// 始终返回 true 的条件
struct AlwaysTrueCondition;

impl ComponentCondition for AlwaysTrueCondition {
    fn name(&self) -> &'static str {
        "always_true"
    }

    fn matches(
        &self,
        _environment: &ApplicationEnvironment,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
}

/// 始终返回 false 的条件
struct AlwaysFalseCondition;

impl ComponentCondition for AlwaysFalseCondition {
    fn name(&self) -> &'static str {
        "always_false"
    }

    fn matches(
        &self,
        _environment: &ApplicationEnvironment,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
}

// ════════════════════════════════════════════════════════════════════
// 测试: ConfigurationPhase 枚举语义
// ════════════════════════════════════════════════════════════════════

/// 对标 Spring ConfigurationPhase.PARSE_CONFIGURATION
#[test]
fn test_configuration_phase_parse_configuration() {
    let phase = ConfigurationPhase::ParseConfiguration;
    assert_eq!(phase.as_str(), "parse_configuration");
    assert_eq!(phase, ConfigurationPhase::ParseConfiguration);
    assert_ne!(phase, ConfigurationPhase::RegisterBean);
}

/// 对标 Spring ConfigurationPhase.REGISTER_BEAN
#[test]
fn test_configuration_phase_register_bean() {
    let phase = ConfigurationPhase::RegisterBean;
    assert_eq!(phase.as_str(), "register_bean");
    assert_eq!(phase, ConfigurationPhase::RegisterBean);
    assert_ne!(phase, ConfigurationPhase::ParseConfiguration);
}

/// 验证 ConfigurationPhase 的 Clone/Copy/Debug/Hash 特性
#[test]
fn test_configuration_phase_traits() {
    let phase = ConfigurationPhase::ParseConfiguration;
    let cloned = phase.clone();
    let copied = phase;
    assert_eq!(phase, cloned);
    assert_eq!(phase, copied);

    let debug = format!("{:?}", phase);
    assert!(debug.contains("ParseConfiguration"));

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    phase.hash(&mut hasher);
    let hash = hasher.finish();
    assert!(hash > 0);
}

/// 验证 ConfigurationPhase 的序列化
#[test]
fn test_configuration_phase_serde() {
    let phase = ConfigurationPhase::ParseConfiguration;
    let json = serde_json::to_string(&phase).unwrap();
    assert_eq!(json, "\"parse_configuration\"");

    let phase = ConfigurationPhase::RegisterBean;
    let json = serde_json::to_string(&phase).unwrap();
    assert_eq!(json, "\"register_bean\"");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ConditionalComponentModule 阶段语义
// ════════════════════════════════════════════════════════════════════

/// 对标 Spring: @Conditional 在 PARSE_CONFIGURATION 阶段评估
/// 条件不命中时整个模块不加入依赖图
#[test]
fn test_conditional_module_parse_configuration_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::ParseConfiguration,
        );

    assert_eq!(module.name(), "test-module");
    assert_eq!(module.phase(), ConfigurationPhase::ParseConfiguration);
}

/// 对标 Spring: @Conditional 在 REGISTER_BEAN 阶段评估
/// 条件不阻止 @Configuration 类加入依赖图
#[test]
fn test_conditional_module_register_bean_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::RegisterBean,
        );

    assert_eq!(module.name(), "test-module");
    assert_eq!(module.phase(), ConfigurationPhase::RegisterBean);
}

/// 验证默认阶段是 ParseConfiguration（与 Spring 一致）
#[test]
fn test_conditional_module_default_phase() {
    let module = ConditionalComponentModule::new("test-module", AlwaysTrueCondition);
    assert_eq!(module.phase(), ConfigurationPhase::ParseConfiguration);
}

/// 验证 with_phase 可以链式调用
#[test]
fn test_conditional_module_phase_chain() {
    let module = ConditionalComponentModule::new("test-module", AlwaysTrueCondition)
        .with_phase(ConfigurationPhase::ParseConfiguration)
        .with_phase(ConfigurationPhase::RegisterBean); // 最后一次调用生效

    assert_eq!(module.phase(), ConfigurationPhase::RegisterBean);
}

// ════════════════════════════════════════════════════════════════════
// 测试: 条件评估与阶段交互
// ════════════════════════════════════════════════════════════════════

/// 对标 Spring: 条件在 PARSE_CONFIGURATION 阶段命中
#[test]
fn test_conditional_matches_in_parse_configuration_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::ParseConfiguration,
        );

    let env = ApplicationEnvironmentBuilder::new().build();
    let result = module.matches(&env);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

/// 对标 Spring: 条件在 REGISTER_BEAN 阶段命中
#[test]
fn test_conditional_matches_in_register_bean_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::RegisterBean,
        );

    let env = ApplicationEnvironmentBuilder::new().build();
    let result = module.matches(&env);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

/// 对标 Spring: 条件在 PARSE_CONFIGURATION 阶段不命中
#[test]
fn test_conditional_not_matches_in_parse_configuration_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysFalseCondition).with_phase(
            ConfigurationPhase::ParseConfiguration,
        );

    let env = ApplicationEnvironmentBuilder::new().build();
    let result = module.matches(&env);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

/// 对标 Spring: 条件在 REGISTER_BEAN 阶段不命中
#[test]
fn test_conditional_not_matches_in_register_bean_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysFalseCondition).with_phase(
            ConfigurationPhase::RegisterBean,
        );

    let env = ApplicationEnvironmentBuilder::new().build();
    let result = module.matches(&env);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

/// 对标 Spring: 验证条件评估的幂等性
#[test]
fn test_conditional_evaluation_idempotent() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::ParseConfiguration,
        );

    let env = ApplicationEnvironmentBuilder::new().build();

    // 多次评估应该返回相同结果
    let result1 = module.matches(&env).unwrap();
    let result2 = module.matches(&env).unwrap();
    let result3 = module.matches(&env).unwrap();

    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
    assert!(result1);
}

/// 对标 Spring: 验证条件名称在不同阶段保持一致
#[test]
fn test_conditional_name_consistent_across_phases() {
    let module_parse =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::ParseConfiguration,
        );

    let module_register =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::RegisterBean,
        );

    assert_eq!(module_parse.name(), module_register.name());
}

/// 对标 Spring: 验证条件模块可以动态切换阶段
#[test]
fn test_conditional_module_phase_switch() {
    let mut module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition);

    // 初始阶段
    assert_eq!(module.phase(), ConfigurationPhase::ParseConfiguration);

    // 切换到 REGISTER_BEAN
    module = module.with_phase(ConfigurationPhase::RegisterBean);
    assert_eq!(module.phase(), ConfigurationPhase::RegisterBean);

    // 切换回 PARSE_CONFIGURATION
    module = module.with_phase(ConfigurationPhase::ParseConfiguration);
    assert_eq!(module.phase(), ConfigurationPhase::ParseConfiguration);
}

/// 对标 Spring: 验证条件评估快照
#[test]
fn test_conditional_snapshot_includes_phase() {
    let module =
        ConditionalComponentModule::new("test-module", AlwaysTrueCondition).with_phase(
            ConfigurationPhase::ParseConfiguration,
        );

    let snapshot = module.snapshot(true);
    // 快照应该包含模块名
    assert!(format!("{:?}", snapshot).contains("test-module"));
}

/// 对标 Spring: 验证条件模块的 shared 构造
#[test]
fn test_conditional_module_shared_constructor() {
    let module =
        ConditionalComponentModule::shared("test-module", Arc::new(AlwaysTrueCondition));

    assert_eq!(module.name(), "test-module");
    assert_eq!(module.phase(), ConfigurationPhase::ParseConfiguration);
}

/// 对标 Spring: 验证条件模块的 shared 构造与 with_phase 组合
#[test]
fn test_conditional_module_shared_with_phase() {
    let module =
        ConditionalComponentModule::shared("test-module", Arc::new(AlwaysTrueCondition))
            .with_phase(ConfigurationPhase::RegisterBean);

    assert_eq!(module.name(), "test-module");
    assert_eq!(module.phase(), ConfigurationPhase::RegisterBean);
}
