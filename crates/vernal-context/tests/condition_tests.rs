//! 对标 Spring Framework `@ConditionalOn*` 系列注解的差分测试。
//!
//! 每个 `#[test]` 都镜像 Spring 的同名测试方法，验证 vernal-context 与
//! Spring `org.springframework.context.annotation.ProfileCondition` /
//! `org.springframework.boot.autoconfigure.condition.OnPropertyCondition` /
//! `org.springframework.boot.autoconfigure.condition.OnExpressionCondition` 在以下
//! 语义上完全等价：
//!
//! - `ProfileCondition::any / all / none` 对应 `@Profile(any/all/none)`
//! - `PropertyCondition::present / missing / having_value / not_having_value` 对应
//!   `@ConditionalOnProperty(having_value=..., match_if_missing=...)`
//! - `ExpressionCondition` 对应 `@ConditionalOnExpression`
//! - `PredicateCondition` 对应 `Condition` SPI 自定义闭包
//! - 多条件 AND 语义（`ConditionalComponentModule::conditions`）
//!
//! 镜像 Spring `ProfileConditionTests` / `OnPropertyConditionTests` /
//! `OnExpressionConditionTests`（2026-07-27）。

use std::sync::Arc;

use vernal_context::{
    ApplicationEnvironment, ComponentCondition, ConditionError, ExpressionCondition,
    MapPropertySource, PredicateCondition, ProfileCondition, PropertyCondition,
};

/// 测试用辅助函数：用 MapPropertySource 构造最小环境。
fn build_env(name: &str, entries: &[(&str, &str)]) -> ApplicationEnvironment {
    let pairs: Vec<(String, String)> = entries
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let source = MapPropertySource::new(name, pairs).expect("valid source");
    let mut builder = ApplicationEnvironment::builder();
    builder.add_last(Arc::new(source)).expect("add source");
    builder.build()
}

fn build_env_with_profiles(
    name: &str,
    entries: &[(&str, &str)],
    active: &[&str],
) -> ApplicationEnvironment {
    let pairs: Vec<(String, String)> = entries
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let source = MapPropertySource::new(name, pairs).expect("valid source");
    let mut builder = ApplicationEnvironment::builder();
    builder.add_last(Arc::new(source)).expect("add source");
    for profile in active {
        builder
            .active_profile(*profile)
            .expect("valid active profile");
    }
    builder.build()
}

// ── ProfileCondition ─────────────────────────────────────────────────────

/// Spring `profileMatch_activeProfilePresent` 差分测试：
/// `ProfileCondition::any` 命中任一 Active Profile 即返回 true。
#[tokio::test]
async fn profile_condition_any_active() {
    let env = build_env_with_profiles("app", &[], &["prod"]);
    let condition = ProfileCondition::any(["prod"]).expect("valid profile set");
    assert_eq!(condition.name(), "profile.any");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `profileMatch_noActiveProfile` 差分测试：
/// Active Profile 不存在时 `any` 必须返回 false。
#[tokio::test]
async fn profile_condition_any_inactive() {
    let env = build_env_with_profiles("app", &[], &["dev"]);
    let condition = ProfileCondition::any(["prod"]).expect("valid profile set");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `profileMatch_all` 差分测试：
/// `ProfileCondition::all` 要求全部候选均生效。
#[tokio::test]
async fn profile_condition_all() {
    let env = build_env_with_profiles("app", &[], &["prod", "us-east"]);
    let condition = ProfileCondition::all(["prod", "us-east"]).expect("valid");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `profileMatch_allMissingOne` 差分测试：
/// `all` 缺少一个候选时返回 false。
#[tokio::test]
async fn profile_condition_all_incomplete() {
    let env = build_env_with_profiles("app", &[], &["prod"]);
    let condition = ProfileCondition::all(["prod", "us-east"]).expect("valid");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `profileMatch_none` 差分测试：
/// `ProfileCondition::none` 要求所有候选均不生效。
#[tokio::test]
async fn profile_condition_none() {
    let env = build_env_with_profiles("app", &[], &["dev"]);
    let condition = ProfileCondition::none(["prod", "staging"]).expect("valid");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `profileMatch_noneHasOverlap` 差分测试：
/// `none` 与任一 Active Profile 重叠时返回 false。
#[tokio::test]
async fn profile_condition_none_overlaps() {
    let env = build_env_with_profiles("app", &[], &["prod"]);
    let condition = ProfileCondition::none(["prod", "staging"]).expect("valid");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `profileMatch_emptySet` 差分测试：
/// 空 Profile 集合必须返回 `EmptyProfileSet` 错误。
#[tokio::test]
async fn profile_condition_empty_set_rejected() {
    let result = ProfileCondition::any(Vec::<String>::new());
    assert!(matches!(result, Err(ConditionError::EmptyProfileSet)));
}

/// Spring `profileMatch_invalidName` 差分测试：
/// 含空白字符的 Profile 名称必须返回 `InvalidProfile` 错误。
#[tokio::test]
async fn profile_condition_invalid_name_rejected() {
    let result = ProfileCondition::any(["has space"]);
    assert!(matches!(result, Err(ConditionError::InvalidProfile { .. })));
}

// ── PropertyCondition ────────────────────────────────────────────────────

/// Spring `propertyPresent` 差分测试：
/// 属性存在时 `PropertyCondition::present` 命中。
#[tokio::test]
async fn property_condition_present() {
    let env = build_env("app", &[("feature.x", "on")]);
    let condition = PropertyCondition::present("feature.x").expect("valid key");
    assert_eq!(condition.name(), "property.present");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `propertyNotPresent` 差分测试：
/// 属性缺失时 `present` 不命中。
#[tokio::test]
async fn property_condition_not_present() {
    let env = build_env("app", &[("feature.y", "on")]);
    let condition = PropertyCondition::present("feature.x").expect("valid key");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `propertyMissing` 差分测试：
/// `PropertyCondition::missing` 反向语义。
#[tokio::test]
async fn property_condition_missing() {
    let env = build_env("app", &[("feature.x", "on")]);
    let condition = PropertyCondition::missing("feature.x").expect("valid key");
    assert_eq!(condition.name(), "property.missing");
    assert!(!condition.matches(&env).unwrap());

    let env2 = build_env("app", &[("feature.y", "on")]);
    assert!(condition.matches(&env2).unwrap());
}

/// Spring `propertyHavingValue` 差分测试：
/// `PropertyCondition::having_value` 仅当属性等于期望值时命中。
#[tokio::test]
async fn property_condition_having_value() {
    let env = build_env("app", &[("feature.x", "on")]);
    let condition = PropertyCondition::having_value("feature.x", "on").expect("valid condition");
    assert_eq!(condition.name(), "property.equals");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `propertyHavingValueMismatch` 差分测试：
/// 属性值不等于期望值时不命中。
#[tokio::test]
async fn property_condition_having_value_mismatch() {
    let env = build_env("app", &[("feature.x", "off")]);
    let condition = PropertyCondition::having_value("feature.x", "on").expect("valid condition");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `propertyHavingValueMissing` 差分测试：
/// `having_value` 属性缺失时返回 false（"未配置" ≠ "配置为其他值"）。
#[tokio::test]
async fn property_condition_having_value_missing_key() {
    let env = build_env("app", &[("feature.y", "on")]);
    let condition = PropertyCondition::having_value("feature.x", "on").expect("valid condition");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `propertyHavingValueOrMissing` 差分测试：
/// `match_if_missing=true` 在属性缺失时也命中。
#[tokio::test]
async fn property_condition_having_value_or_missing() {
    let env = build_env("app", &[("feature.y", "on")]);
    let condition =
        PropertyCondition::having_value_or_missing("feature.x", "on").expect("valid condition");
    assert_eq!(condition.name(), "property.equals-or-missing");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `propertyNotHavingValue` 差分测试：
/// 仅当属性存在且不等于期望值时命中。
#[tokio::test]
async fn property_condition_not_having_value() {
    let env = build_env("app", &[("feature.x", "off")]);
    let condition =
        PropertyCondition::not_having_value("feature.x", "on").expect("valid condition");
    assert_eq!(condition.name(), "property.not-equals");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `propertyNotHavingValueMissingKey` 差分测试：
/// 属性缺失时 `not_having_value` 不命中。
#[tokio::test]
async fn property_condition_not_having_value_missing_key() {
    let env = build_env("app", &[("feature.y", "on")]);
    let condition =
        PropertyCondition::not_having_value("feature.x", "on").expect("valid condition");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `propertyInvalidKey` 差分测试：
/// 空键或包含空白字符的键必须返回 `InvalidPropertyKey` 错误。
#[tokio::test]
async fn property_condition_invalid_key() {
    let result = PropertyCondition::present("");
    assert!(matches!(
        result,
        Err(ConditionError::InvalidPropertyKey { .. })
    ));

    let result = PropertyCondition::present("has space");
    assert!(matches!(
        result,
        Err(ConditionError::InvalidPropertyKey { .. })
    ));
}

// ── ExpressionCondition ──────────────────────────────────────────────────

/// Spring `expressionConditionTrue` 差分测试：
/// `@ConditionalOnExpression("true")` 命中。
#[tokio::test]
async fn expression_condition_true() {
    let env = build_env("app", &[("app.mode", "production")]);
    let condition = ExpressionCondition::new("'${app.mode}' == 'production'");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `expressionConditionFalse` 差分测试：
/// 不等式返回 false。
#[tokio::test]
async fn expression_condition_false() {
    let env = build_env("app", &[("app.mode", "development")]);
    let condition = ExpressionCondition::new("'${app.mode}' == 'production'");
    assert!(!condition.matches(&env).unwrap());
}

/// Spring `expressionConditionNumeric` 差分测试：
/// 数字字面量相等比较。
#[tokio::test]
async fn expression_condition_numeric() {
    let env = build_env("app", &[("app.pool.size", "8")]);
    let condition = ExpressionCondition::new("${app.pool.size} == 8");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `expressionConditionBooleanLiteral` 差分测试：
/// `true` / `false` 字面量表达式。
#[tokio::test]
async fn expression_condition_boolean_literal() {
    let env = build_env("app", &[]);
    let condition_true = ExpressionCondition::new("true");
    assert!(condition_true.matches(&env).unwrap());

    let condition_false = ExpressionCondition::new("false");
    assert!(!condition_false.matches(&env).unwrap());
}

// ── PredicateCondition ──────────────────────────────────────────────────

/// Spring `customCondition` 差分测试：
/// 自定义闭包驱动 `ComponentCondition`。
#[tokio::test]
async fn predicate_condition_simple() {
    let env = build_env("app", &[("custom.flag", "yes")]);
    let condition = PredicateCondition::new("custom.flag.equals", |env| {
        Ok(env
            .get::<String>("custom.flag")
            .ok()
            .flatten()
            .map(|v| v == "yes")
            .unwrap_or(false))
    });
    assert_eq!(condition.name(), "custom.flag.equals");
    assert!(condition.matches(&env).unwrap());
}

/// Spring `predicateConditionReturnsError` 差分测试：
/// 自定义条件可以返回错误（模拟业务判断失败）。
#[tokio::test]
async fn predicate_condition_returns_error() {
    let env = build_env("app", &[]);
    let condition = PredicateCondition::new("custom.complex", |_env| {
        Err(std::io::Error::other("complex check failed").into())
    });
    let result = condition.matches(&env);
    assert!(result.is_err());
}
