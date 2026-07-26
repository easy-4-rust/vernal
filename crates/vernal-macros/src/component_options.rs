//! `Component` 派生宏的结构体级选项对象。
//!
//! 解析 `#[component(...)]` 属性中的所有结构体级选项，包括：
//! - 作用域（scope）
//! - AOP 接线（aop）
//! - 生命周期钩子（init / async_init / async_run / shutdown）
//! - 初始化排序（init_order）
//! - 配置绑定（config）
//! - Trait 注册（as_trait）
//!
//! 注意：组件发现已改为自动注册（所有 `#[derive(Component)]` 自动进入 linkme），
//! 不再需要 `discover` 属性。消费方通过 `ComponentScanModule` 按模块路径过滤。

use syn::{Attribute, LitInt, LitStr, Path, Type};

use crate::component_scope_option::ComponentScopeOption;

/// 保存组件派生宏的所有结构体级选项。
///
/// 结构体级选项与字段级注入选项分离，避免 `lib.rs` 或宏入口堆积解析状态。
pub(crate) struct ComponentOptions {
    /// 组件作用域（Singleton / Transient / Custom）
    scope: ComponentScopeOption,
    /// 是否启用 AOP 接线
    aop_enabled: bool,
    /// 同步初始化钩子方法名（对标 tx_di 的 `init`）
    init_hook: Option<LitStr>,
    /// 异步初始化钩子方法名（对标 tx_di 的 `app_async_init`）
    async_init_hook: Option<LitStr>,
    /// 异步后台运行钩子方法名（对标 tx_di 的 `app_async_run`）
    async_run_hook: Option<LitStr>,
    /// 关闭钩子方法名（对标 tx_di 的 `shutdown`）
    shutdown_hook: Option<LitStr>,
    /// 同层初始化排序值（越小越早，默认 i32::MAX）
    init_order: Option<LitInt>,
    /// 是否自动绑定 ConfigurationProperties（对标 tx_di 的 `conf`）
    config_mode: bool,
    /// 配置前缀（`#[component(config = "prefix")]`），None 时使用类型名 snake_case
    config_prefix: Option<LitStr>,
    /// 自动注册的 Trait 绑定（对标 tx_di 的 `as_trait`）
    as_trait: Option<Path>,
}

impl ComponentOptions {
    /// 解析全部结构体级 `#[component(...)]` 属性。
    ///
    /// 支持的属性：
    /// - `scope = "singleton"` / `scope = "transient"` / `scope = CustomType`
    /// - `aop` — 启用 AOP 接线
    /// - `init = "method_name"` — 同步初始化钩子
    /// - `async_init = "method_name"` — 异步初始化钩子
    /// - `async_run = "method_name"` — 异步后台运行钩子
    /// - `shutdown = "method_name"` — 关闭钩子
    /// - `init_order = N` — 同层初始化排序
    /// - `config` / `config = "prefix"` — 自动配置绑定
    /// - `as_trait = dyn Trait` — 自动 Trait 注册
    ///
    /// 注意：`discover` 属性已移除。所有 `#[derive(Component)]` 自动注册到
    /// linkme 分布式切片，消费方通过 `ComponentScanModule` 按模块路径过滤。
    pub(crate) fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut options = Self {
            scope: ComponentScopeOption::Singleton,
            aop_enabled: false,
            init_hook: None,
            async_init_hook: None,
            async_run_hook: None,
            shutdown_hook: None,
            init_order: None,
            config_mode: false,
            config_prefix: None,
            as_trait: None,
        };
        let mut scope_declared = false;

        for attribute in attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("component"))
        {
            attribute.parse_nested_meta(|metadata| {
                // ─── aop ───
                if metadata.path.is_ident("aop") {
                    if options.aop_enabled {
                        return Err(metadata.error("结构体 component 属性只能声明一次 aop"));
                    }
                    options.aop_enabled = true;
                    return Ok(());
                }

                // ─── scope ───
                if metadata.path.is_ident("scope") {
                    if scope_declared {
                        return Err(metadata.error("结构体 component 属性只能声明一次 scope"));
                    }
                    scope_declared = true;
                    let value = metadata.value()?;
                    if value.peek(LitStr) {
                        let value = value.parse::<LitStr>()?;
                        options.scope = match value.value().as_str() {
                            "singleton" => ComponentScopeOption::Singleton,
                            "transient" => ComponentScopeOption::Transient,
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "字符串 scope 只支持 \"singleton\" 或 \"transient\"；自定义作用域请直接填写标记类型",
                                ));
                            }
                        };
                        return Ok(());
                    }
                    options.scope =
                        ComponentScopeOption::Custom(Box::new(value.parse::<Type>()?));
                    return Ok(());
                }

                // ─── 生命周期钩子 ───
                if metadata.path.is_ident("init") {
                    if options.init_hook.is_some() {
                        return Err(metadata.error("init 钩子只能声明一次"));
                    }
                    options.init_hook = Some(metadata.value()?.parse::<LitStr>()?);
                    return Ok(());
                }
                if metadata.path.is_ident("async_init") {
                    if options.async_init_hook.is_some() {
                        return Err(metadata.error("async_init 钩子只能声明一次"));
                    }
                    options.async_init_hook = Some(metadata.value()?.parse::<LitStr>()?);
                    return Ok(());
                }
                if metadata.path.is_ident("async_run") {
                    if options.async_run_hook.is_some() {
                        return Err(metadata.error("async_run 钩子只能声明一次"));
                    }
                    options.async_run_hook = Some(metadata.value()?.parse::<LitStr>()?);
                    return Ok(());
                }
                if metadata.path.is_ident("shutdown") {
                    if options.shutdown_hook.is_some() {
                        return Err(metadata.error("shutdown 钩子只能声明一次"));
                    }
                    options.shutdown_hook = Some(metadata.value()?.parse::<LitStr>()?);
                    return Ok(());
                }

                // ─── init_order ───
                if metadata.path.is_ident("init_order") {
                    if options.init_order.is_some() {
                        return Err(metadata.error("init_order 只能声明一次"));
                    }
                    options.init_order = Some(metadata.value()?.parse::<LitInt>()?);
                    return Ok(());
                }

                // ─── config ───
                if metadata.path.is_ident("config") {
                    if options.config_mode {
                        return Err(metadata.error("config 只能声明一次"));
                    }
                    options.config_mode = true;
                    // 尝试解析 `config = "prefix"`；如果没有值则使用默认前缀
                    if metadata.value().is_ok() {
                        options.config_prefix = Some(metadata.value()?.parse::<LitStr>()?);
                    }
                    return Ok(());
                }

                // ─── as_trait ───
                if metadata.path.is_ident("as_trait") {
                    if options.as_trait.is_some() {
                        return Err(metadata.error("as_trait 只能声明一次"));
                    }
                    options.as_trait = Some(metadata.value()?.parse::<Path>()?);
                    return Ok(());
                }

                Err(metadata.error(
                    "结构体 component 属性只支持 scope、aop、init、async_init、async_run、shutdown、init_order、config、as_trait",
                ))
            })?;
        }

        // 校验：config 模式下不能同时指定 scope = transient
        if options.config_mode && matches!(options.scope, ComponentScopeOption::Transient) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "config 组件必须是 Singleton 作用域",
            ));
        }

        Ok(options)
    }

    /// 拆分已校验选项，供代码生成阶段取得所有权。
    pub(crate) fn into_parts(self) -> ComponentOptionsParts {
        ComponentOptionsParts {
            scope: self.scope,
            aop_enabled: self.aop_enabled,
            init_hook: self.init_hook,
            async_init_hook: self.async_init_hook,
            async_run_hook: self.async_run_hook,
            shutdown_hook: self.shutdown_hook,
            init_order: self.init_order,
            config_mode: self.config_mode,
            config_prefix: self.config_prefix,
            as_trait: self.as_trait,
        }
    }
}

/// 拆分后的组件选项，供代码生成阶段使用。
pub(crate) struct ComponentOptionsParts {
    pub scope: ComponentScopeOption,
    pub aop_enabled: bool,
    pub init_hook: Option<LitStr>,
    pub async_init_hook: Option<LitStr>,
    pub async_run_hook: Option<LitStr>,
    pub shutdown_hook: Option<LitStr>,
    pub init_order: Option<LitInt>,
    pub config_mode: bool,
    pub config_prefix: Option<LitStr>,
    pub as_trait: Option<Path>,
}
