//! `Component` 派生宏的结构体级选项对象。

use syn::{Attribute, LitStr, Type};

use crate::component_scope_option::ComponentScopeOption;

/// 保存组件作用域、AOP 接线和可选链接期发现分组。
///
/// 结构体级选项与字段级注入选项分离，避免 `lib.rs` 或宏入口堆积解析状态。发现
/// 分组只生成静态定义入口，不改变 [`ComponentScopeOption`] 或组件构造语义。
pub(crate) struct ComponentOptions {
    scope: ComponentScopeOption,
    aop_enabled: bool,
    discovery_group: Option<LitStr>,
}

impl ComponentOptions {
    /// 解析全部结构体级 `#[component(...)]` 属性。
    ///
    /// 支持 `scope`、`aop` 和单个 `discover = "group"`。重复单值选项、空分组、
    /// 首尾空白或控制字符会在编译期返回明确诊断。
    pub(crate) fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut options = Self {
            scope: ComponentScopeOption::Singleton,
            aop_enabled: false,
            discovery_group: None,
        };
        let mut scope_declared = false;
        for attribute in attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("component"))
        {
            attribute.parse_nested_meta(|metadata| {
                if metadata.path.is_ident("aop") {
                    if options.aop_enabled {
                        return Err(metadata.error("结构体 component 属性只能声明一次 aop"));
                    }
                    options.aop_enabled = true;
                    return Ok(());
                }
                if metadata.path.is_ident("discover") {
                    if options.discovery_group.is_some() {
                        return Err(metadata.error(
                            "结构体 component 属性只能声明一个 discover 分组",
                        ));
                    }
                    let group = metadata.value()?.parse::<LitStr>()?;
                    Self::validate_group(&group)?;
                    options.discovery_group = Some(group);
                    return Ok(());
                }
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
                Err(metadata.error(
                    "结构体 component 属性只支持 scope、aop 或 discover",
                ))
            })?;
        }
        Ok(options)
    }

    /// 拆分已校验选项，供代码生成阶段取得所有权。
    pub(crate) fn into_parts(self) -> (ComponentScopeOption, bool, Option<LitStr>) {
        (self.scope, self.aop_enabled, self.discovery_group)
    }

    /// 校验发现分组可作为稳定、低基数的静态选择器。
    fn validate_group(group: &LitStr) -> syn::Result<()> {
        let text = group.value();
        if text.is_empty()
            || text
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(syn::Error::new_spanned(
                group,
                "discover 分组不能为空，也不能包含空白或控制字符",
            ));
        }
        Ok(())
    }
}
